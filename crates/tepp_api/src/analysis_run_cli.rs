//! Operator loopback CLI for scientific-acceptance analysis-run lifecycle.
//!
//! GAP-003A fifth slice: operators drive `POST /v1/analysis-runs`,
//! `POST /v1/analysis-runs/{run_id}/running`,
//! `POST /v1/analysis-runs/{run_id}/terminal`, and
//! `GET /v1/analysis-runs/{run_id}` without writing raw HTTP. Create and
//! running stay metric-free. Only a succeeded status whose request profile is
//! `scientific_acceptance_v1` may print `tepp.scientific_acceptance.v1`. This
//! module does not duplicate the library bind, the terminal-result DTO, the
//! GET listener, or the lifecycle POST listener. Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::analysis_run_status_http::encode_path_segment;
use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::scientific_acceptance_http::{
    SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    refuse_metrics_on_receipt,
};
use crate::wire::require_nonempty;
use crate::{
    ANALYSIS_RUN_ID_MAX_LEN, AnalysisRunLifecycleTransition, AnalysisRunLiveService,
    AnalysisRunRequest, AnalysisRunStatusState, ApiError, NARUON_LIVE_IO_TIMEOUT,
    NaruonLiveResponse,
};

/// Supported operator verbs for the loopback analysis-run CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunCliVerb {
    /// `POST /v1/analysis-runs` with a metric-free request body.
    Create,
    /// `GET /v1/analysis-runs/{run_id}`.
    Status,
    /// `POST /v1/analysis-runs/{run_id}/running`.
    Running,
    /// `POST /v1/analysis-runs/{run_id}/terminal`.
    Terminal,
}

impl AnalysisRunCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "create" => Ok(Self::Create),
            "status" => Ok(Self::Status),
            "running" => Ok(Self::Running),
            "terminal" => Ok(Self::Terminal),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Status => "status",
            Self::Running => "running",
            Self::Terminal => "terminal",
        }
    }

    const fn requires_stdin_body(self) -> bool {
        matches!(self, Self::Create | Self::Terminal)
    }
}

/// One operator CLI invocation against a loopback analysis-run listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Request-bound idempotency key.
    pub idempotency_key: String,
    /// Server-assigned run identity, required except for `create`.
    pub run_id: Option<String>,
    /// JSON body. Empty for `status`; constructed for empty `running`.
    pub body: String,
}

impl AnalysisRunCliInvocation {
    /// Parse argv plus stdin body into a validated loopback CLI invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags,
    /// a non-loopback host, an unpublished consumer, credential-shaped flags,
    /// metric keys on create/running, or a verb/body mismatch.
    pub fn from_args<I, S>(args: I, body: impl Into<String>) -> Result<Self, ApiError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let tokens: Vec<String> = args
            .into_iter()
            .map(|token| token.as_ref().to_owned())
            .collect();
        let (verb_token, rest) = tokens.split_first().ok_or(ApiError::InvalidWirePayload)?;
        let verb = AnalysisRunCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or empty identities.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] for empty or unpublished fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.idempotency_key)?;
        match self.verb {
            AnalysisRunCliVerb::Create => {
                if self.run_id.is_some() || self.body.is_empty() {
                    return Err(ApiError::InvalidWirePayload);
                }
                refuse_metrics_on_receipt(&self.body)?;
                let request = AnalysisRunRequest::from_json(&self.body)?;
                if request.idempotency_key != self.idempotency_key {
                    return Err(ApiError::InvalidWirePayload);
                }
            }
            AnalysisRunCliVerb::Status => {
                require_status_run_id(self.run_id.as_deref())?;
                if !self.body.is_empty() {
                    return Err(ApiError::InvalidWirePayload);
                }
            }
            AnalysisRunCliVerb::Running => {
                let run_id = require_status_run_id(self.run_id.as_deref())?;
                refuse_metrics_on_receipt(&self.body)?;
                let transition = AnalysisRunLifecycleTransition::from_json(&self.body)?;
                if transition.run_state != AnalysisRunStatusState::Running
                    || transition.run_id != run_id
                    || transition.idempotency_key != self.idempotency_key
                {
                    return Err(ApiError::InvalidWirePayload);
                }
            }
            AnalysisRunCliVerb::Terminal => {
                let run_id = require_status_run_id(self.run_id.as_deref())?;
                let transition = AnalysisRunLifecycleTransition::from_json(&self.body)?;
                if !matches!(
                    transition.run_state,
                    AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
                ) || transition.run_id != run_id
                    || transition.idempotency_key != self.idempotency_key
                {
                    return Err(ApiError::InvalidWirePayload);
                }
            }
        }
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    consumer: Option<String>,
    idempotency_key: Option<String>,
    run_id: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        consumer: None,
        idempotency_key: None,
        run_id: None,
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        if !flag.starts_with("--") {
            return Err(ApiError::InvalidWirePayload);
        }
        let name = &flag[2..];
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        let slot = match name {
            "host" => &mut flags.host,
            "consumer" => &mut flags.consumer,
            "idempotency-key" => &mut flags.idempotency_key,
            "run-id" => &mut flags.run_id,
            _ => return Err(ApiError::InvalidWirePayload),
        };
        if slot.is_some() || index + 1 >= rest.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        let value = rest[index + 1].as_str();
        require_nonempty(value)?;
        *slot = Some(value.to_owned());
        index += 2;
    }
    Ok(flags)
}

fn assemble_invocation(
    verb: AnalysisRunCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunCliInvocation, ApiError> {
    let host = flags.host.ok_or(ApiError::InvalidWirePayload)?;
    let consumer = flags
        .consumer
        .unwrap_or_else(|| crate::NARUON_CONSUMER_CODE.to_owned());
    let invocation = match verb {
        AnalysisRunCliVerb::Create => {
            let request = AnalysisRunRequest::from_json(&body)?;
            if let Some(run_id) = flags.run_id.as_deref() {
                require_nonempty(run_id)?;
                return Err(ApiError::InvalidWirePayload);
            }
            if let Some(key) = flags.idempotency_key.as_deref()
                && key != request.idempotency_key
            {
                return Err(ApiError::InvalidWirePayload);
            }
            AnalysisRunCliInvocation {
                verb,
                host,
                consumer,
                idempotency_key: request.idempotency_key,
                run_id: None,
                body,
            }
        }
        AnalysisRunCliVerb::Status => AnalysisRunCliInvocation {
            verb,
            host,
            consumer,
            idempotency_key: flags.idempotency_key.ok_or(ApiError::InvalidWirePayload)?,
            run_id: Some(flags.run_id.ok_or(ApiError::InvalidWirePayload)?),
            body,
        },
        AnalysisRunCliVerb::Running => {
            let run_id = flags.run_id.ok_or(ApiError::InvalidWirePayload)?;
            let idempotency_key = flags.idempotency_key.ok_or(ApiError::InvalidWirePayload)?;
            let body = if body.is_empty() {
                AnalysisRunLifecycleTransition::running(run_id.clone(), idempotency_key.clone())?
                    .to_json()?
            } else {
                body
            };
            AnalysisRunCliInvocation {
                verb,
                host,
                consumer,
                idempotency_key,
                run_id: Some(run_id),
                body,
            }
        }
        AnalysisRunCliVerb::Terminal => {
            let transition = AnalysisRunLifecycleTransition::from_json(&body)?;
            if let Some(run_id) = flags.run_id.as_deref()
                && run_id != transition.run_id
            {
                return Err(ApiError::InvalidWirePayload);
            }
            if let Some(key) = flags.idempotency_key.as_deref()
                && key != transition.idempotency_key
            {
                return Err(ApiError::InvalidWirePayload);
            }
            AnalysisRunCliInvocation {
                verb,
                host,
                consumer,
                idempotency_key: transition.idempotency_key.clone(),
                run_id: Some(transition.run_id.clone()),
                body,
            }
        }
    };
    invocation.validate()?;
    Ok(invocation)
}

fn require_loopback_host(host: &str) -> Result<SocketAddr, ApiError> {
    let addr: SocketAddr = host.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

fn require_status_run_id(run_id: Option<&str>) -> Result<&str, ApiError> {
    let run_id = run_id.ok_or(ApiError::InvalidWirePayload)?;
    require_nonempty(run_id)?;
    if run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    Ok(run_id)
}

/// Compose one HTTP/1.1 request for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunCliInvocation::validate`].
pub fn compose_analysis_run_cli_http(
    invocation: &AnalysisRunCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let (method, path, body) = match invocation.verb {
        AnalysisRunCliVerb::Create => (
            "POST",
            NARUON_ANALYSIS_RUN_PATH.to_owned(),
            invocation.body.as_str(),
        ),
        AnalysisRunCliVerb::Status => (
            "GET",
            format!(
                "{NARUON_ANALYSIS_RUN_PATH}/{}",
                encode_path_segment(
                    invocation
                        .run_id
                        .as_deref()
                        .ok_or(ApiError::InvalidWirePayload)?
                )
            ),
            invocation.body.as_str(),
        ),
        AnalysisRunCliVerb::Running => (
            "POST",
            format!(
                "{NARUON_ANALYSIS_RUN_PATH}/{}/running",
                encode_path_segment(
                    invocation
                        .run_id
                        .as_deref()
                        .ok_or(ApiError::InvalidWirePayload)?
                )
            ),
            invocation.body.as_str(),
        ),
        AnalysisRunCliVerb::Terminal => (
            "POST",
            format!(
                "{NARUON_ANALYSIS_RUN_PATH}/{}/terminal",
                encode_path_segment(
                    invocation
                        .run_id
                        .as_deref()
                        .ok_or(ApiError::InvalidWirePayload)?
                )
            ),
            invocation.body.as_str(),
        ),
    };
    Ok(format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        invocation.host,
        invocation.consumer,
        invocation.idempotency_key,
        body.len()
    ))
}

/// Dispatch one CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_cli(
    invocation: &AnalysisRunCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_cli_http(invocation)?;
    let mut stream = TcpStream::connect(addr).map_err(|error| map_io_error(&error))?;
    stream
        .set_read_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .set_write_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| map_io_error(&error))?;
    stream.flush().map_err(|error| map_io_error(&error))?;
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|error| map_io_error(&error))?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so scientific acceptance prints only on succeeded
/// `scientific_acceptance_v1` statuses.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// a failed or non-terminal body carries `tepp.scientific_acceptance.v1`, or a
/// scientific-acceptance profile is missing its artifact.
pub fn render_analysis_run_cli_stdout(
    invocation: &AnalysisRunCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    if !(200..300).contains(&response.status_code) {
        refuse_scientific_acceptance_schema(&response.body)?;
        return Ok(response.body.clone());
    }
    match invocation.verb {
        AnalysisRunCliVerb::Create | AnalysisRunCliVerb::Running => {
            refuse_metrics_on_receipt(&response.body)?;
            refuse_scientific_acceptance_schema(&response.body)?;
            Ok(response.body.clone())
        }
        AnalysisRunCliVerb::Status | AnalysisRunCliVerb::Terminal => {
            render_status_stdout(&response.body)
        }
    }
}

fn render_status_stdout(body: &str) -> Result<String, ApiError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| ApiError::InvalidWirePayload)?;
    let object = value.as_object().ok_or(ApiError::InvalidWirePayload)?;
    let run_state = object
        .get("run_state")
        .and_then(serde_json::Value::as_str)
        .ok_or(ApiError::InvalidWirePayload)?;
    match run_state {
        "accepted" | "running" | "failed" => {
            refuse_metrics_on_receipt(body)?;
            refuse_scientific_acceptance_schema(body)?;
            Ok(body.to_owned())
        }
        "succeeded" => render_succeeded_stdout(object, body),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

fn render_succeeded_stdout(
    object: &serde_json::Map<String, serde_json::Value>,
    body: &str,
) -> Result<String, ApiError> {
    let terminal = object
        .get("terminal_result")
        .and_then(serde_json::Value::as_object)
        .ok_or(ApiError::InvalidWirePayload)?;
    let profile = terminal
        .get("output_profile")
        .and_then(serde_json::Value::as_str)
        .ok_or(ApiError::InvalidWirePayload)?;
    match (
        profile == SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
        terminal.get("scientific_acceptance"),
    ) {
        (true, Some(artifact)) => {
            let schema = artifact
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                .ok_or(ApiError::InvalidWirePayload)?;
            if schema == SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA {
                Ok(body.to_owned())
            } else {
                Err(ApiError::InvalidWirePayload)
            }
        }
        (false, None) => {
            refuse_metrics_on_receipt(body)?;
            Ok(body.to_owned())
        }
        _ => Err(ApiError::InvalidWirePayload),
    }
}

fn refuse_scientific_acceptance_schema(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn parse_http_response(bytes: &[u8]) -> Result<NaruonLiveResponse, ApiError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ApiError::InvalidWirePayload)?;
    let (header_block, body) = text
        .split_once("\r\n\r\n")
        .ok_or(ApiError::InvalidWirePayload)?;
    let mut lines = header_block.split("\r\n");
    let status_line = lines.next().ok_or(ApiError::InvalidWirePayload)?;
    let mut parts = status_line.split(' ');
    if parts.next() != Some("HTTP/1.1") {
        return Err(ApiError::InvalidWirePayload);
    }
    let code = parts
        .next()
        .ok_or(ApiError::InvalidWirePayload)?
        .parse::<u16>()
        .map_err(|_| ApiError::InvalidWirePayload)?;
    let reason_phrase = static_reason(code)?;
    let mut content_length = None;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(ApiError::InvalidWirePayload)?;
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(ApiError::InvalidWirePayload);
            }
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| ApiError::InvalidWirePayload)?,
            );
        }
    }
    let declared = content_length.ok_or(ApiError::InvalidWirePayload)?;
    if declared != body.len() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(NaruonLiveResponse {
        status_code: code,
        reason_phrase,
        body: body.to_owned(),
    })
}

fn static_reason(code: u16) -> Result<&'static str, ApiError> {
    match code {
        200 => Ok("OK"),
        202 => Ok("Accepted"),
        400 => Ok("Bad Request"),
        403 => Ok("Forbidden"),
        413 => Ok("Payload Too Large"),
        422 => Ok("Unprocessable Entity"),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

/// Read stdin when the verb requires a JSON body; refuse leftover piped bytes
/// on `status` and `running` when stdin is not a terminal.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_cli_stdin(
    verb: AnalysisRunCliVerb,
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, ApiError> {
    if verb.requires_stdin_body() || !stdin_is_terminal {
        let mut body = String::new();
        stdin
            .read_to_string(&mut body)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        Ok(body)
    } else {
        Ok(String::new())
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        AnalysisRunCliInvocation, AnalysisRunCliVerb, compose_analysis_run_cli_http,
        dispatch_analysis_run_cli, execute_analysis_run_cli, parse_http_response,
        read_analysis_run_cli_stdin, render_analysis_run_cli_stdout, static_reason,
    };
    use crate::scientific_acceptance_http::{
        SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, ANALYSIS_RUN_ID_MAX_LEN, AnalysisResultSummary,
        AnalysisRunAccepted, AnalysisRunLifecycleTransition, AnalysisRunLiveService,
        AnalysisRunRequest, AnalysisRunTerminalResult, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_CONSUMER_CODE, NaruonLiveResponse, receipt_json_carries_scientific_metrics,
    };
    use sha2::{Digest, Sha256};

    fn request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "cli-idem-1".into(),
            tenant_workspace_id: "cli-tenant-1".into(),
            snapshot_id: "cli-snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "validation_cpu_f64_v1".into(),
            output_profile: SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into(),
        }
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = Sha256::digest(bytes);
        let mut encoded = String::with_capacity(64);
        for byte in digest {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        encoded
    }

    fn create_invocation() -> AnalysisRunCliInvocation {
        AnalysisRunCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                NARUON_CONSUMER_CODE,
            ],
            request().to_json().expect("request"),
        )
        .expect("create")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunCliVerb::parse("create").expect("create"),
            AnalysisRunCliVerb::Create
        );
        assert_eq!(
            AnalysisRunCliVerb::parse("status").expect("status"),
            AnalysisRunCliVerb::Status
        );
        assert_eq!(
            AnalysisRunCliVerb::parse("running").expect("running"),
            AnalysisRunCliVerb::Running
        );
        assert_eq!(
            AnalysisRunCliVerb::parse("terminal").expect("terminal"),
            AnalysisRunCliVerb::Terminal
        );
        assert_eq!(
            AnalysisRunCliVerb::parse("CREATE"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(AnalysisRunCliVerb::Create.as_str(), "create");
        assert_eq!(AnalysisRunCliVerb::Status.as_str(), "status");
        assert_eq!(AnalysisRunCliVerb::Running.as_str(), "running");
        assert_eq!(AnalysisRunCliVerb::Terminal.as_str(), "terminal");
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_and_credential_flags() {
        assert_eq!(
            AnalysisRunCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(["create"], request().to_json().expect("json"))
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "8.8.8.8:80"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "not-a-socket"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--authorization",
                    "secret"
                ],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081", "--pretty"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081", "extra"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn create_status_running_and_terminal_assemble() {
        let create = create_invocation();
        assert_eq!(create.verb, AnalysisRunCliVerb::Create);
        assert!(create.run_id.is_none());
        let http = compose_analysis_run_cli_http(&create).expect("http");
        assert!(http.starts_with("POST /v1/analysis-runs HTTP/1.1"));

        let status = AnalysisRunCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                "run-1",
                "--idempotency-key",
                "cli-idem-1",
            ],
            "",
        )
        .expect("status");
        assert_eq!(status.consumer, NARUON_CONSUMER_CODE);
        let status_http = compose_analysis_run_cli_http(&status).expect("status http");
        assert!(status_http.starts_with("GET /v1/analysis-runs/run-1 HTTP/1.1"));

        let running = AnalysisRunCliInvocation::from_args(
            [
                "running",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--run-id",
                "run-1",
                "--idempotency-key",
                "cli-idem-1",
            ],
            "",
        )
        .expect("running");
        assert_eq!(running.consumer, LINEAGEWEAVE_CONSUMER_CODE);
        assert!(
            compose_analysis_run_cli_http(&running)
                .expect("running http")
                .contains("/running")
        );

        let terminal_body = AnalysisRunLifecycleTransition::terminal(
            "run-1",
            "cli-idem-1",
            AnalysisRunTerminalResult::failed(
                &request(),
                &AnalysisRunAccepted::new("run-1", "accepted", "cli-idem-1").expect("accepted"),
                "2026-08-02T03:04:05Z",
                "estimation_failed",
            )
            .expect("failed"),
            None,
        )
        .expect("transition")
        .to_json()
        .expect("json");
        let terminal = AnalysisRunCliInvocation::from_args(
            ["terminal", "--host", "127.0.0.1:18081"],
            terminal_body,
        )
        .expect("terminal");
        assert_eq!(terminal.run_id.as_deref(), Some("run-1"));
        assert!(
            compose_analysis_run_cli_http(&terminal)
                .expect("terminal http")
                .contains("/terminal")
        );
    }

    #[test]
    fn create_and_lifecycle_bodies_fail_closed() {
        assert_eq!(
            AnalysisRunCliInvocation::from_args(["create", "--host", "127.0.0.1:18081"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut metric_json = request().to_json().expect("json");
        metric_json.pop();
        metric_json.push_str(",\"rmse\":0.1}");
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081"],
                metric_json
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081", "--run-id", "run-1"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "other"
                ],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081", "--consumer", "other"],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "run-1",
                    "--idempotency-key",
                    "cli-idem-1"
                ],
                "{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "cli-idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "running",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "cli-idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let oversized = "r".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1);
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    oversized.as_str(),
                    "--idempotency-key",
                    "cli-idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(["terminal", "--host", "127.0.0.1:18081"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--host",
                    "127.0.0.1:9"
                ],
                request().to_json().expect("json")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn running_body_must_stay_running_and_terminal_ids_must_match() {
        let running = AnalysisRunLifecycleTransition::running("run-1", "cli-idem-1")
            .expect("running")
            .to_json()
            .expect("json");
        AnalysisRunCliInvocation::from_args(
            [
                "running",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                "run-1",
                "--idempotency-key",
                "cli-idem-1",
            ],
            running.clone(),
        )
        .expect("matching running");
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "running",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "run-2",
                    "--idempotency-key",
                    "cli-idem-1",
                ],
                running,
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let terminal_body = AnalysisRunLifecycleTransition::terminal(
            "run-1",
            "cli-idem-1",
            AnalysisRunTerminalResult::failed(
                &request(),
                &AnalysisRunAccepted::new("run-1", "accepted", "cli-idem-1").expect("accepted"),
                "2026-08-02T03:04:05Z",
                "estimation_failed",
            )
            .expect("failed"),
            None,
        )
        .expect("transition")
        .to_json()
        .expect("json");
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                ["terminal", "--host", "127.0.0.1:18081", "--run-id", "run-2"],
                terminal_body.clone()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "terminal",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "other"
                ],
                terminal_body
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCliInvocation::from_args(
                [
                    "running",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "run-1",
                    "--idempotency-key",
                    "cli-idem-1",
                ],
                AnalysisRunLifecycleTransition::terminal(
                    "run-1",
                    "cli-idem-1",
                    AnalysisRunTerminalResult::failed(
                        &request(),
                        &AnalysisRunAccepted::new("run-1", "accepted", "cli-idem-1")
                            .expect("accepted"),
                        "2026-08-02T03:04:05Z",
                        "estimation_failed",
                    )
                    .expect("failed"),
                    None,
                )
                .expect("transition")
                .to_json()
                .expect("json"),
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn dispatch_create_running_terminal_then_status_prints_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let create = create_invocation();
        let accepted = dispatch_analysis_run_cli(&mut service, &create).expect("create");
        assert_eq!(accepted.status_code, 202);
        let stdout = render_analysis_run_cli_stdout(&create, &accepted).expect("create stdout");
        assert!(!receipt_json_carries_scientific_metrics(&stdout));
        let accepted_dto = AnalysisRunAccepted::from_json(&stdout).expect("accepted");

        let running = AnalysisRunCliInvocation::from_args(
            [
                "running",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                accepted_dto.run_id.as_str(),
                "--idempotency-key",
                "cli-idem-1",
            ],
            "",
        )
        .expect("running");
        let running_response =
            dispatch_analysis_run_cli(&mut service, &running).expect("running dispatch");
        assert_eq!(running_response.status_code, 200);
        let running_stdout =
            render_analysis_run_cli_stdout(&running, &running_response).expect("running stdout");
        assert!(running_stdout.contains("\"running\""));
        assert!(!running_stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

        let artifact = format!(
            r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}","binding_sha256":"{}","run_id":"{}"}}"#,
            "ab".repeat(32),
            accepted_dto.run_id
        );
        let digest = sha256_hex(artifact.as_bytes());
        let terminal = AnalysisRunTerminalResult::succeeded(
            &request(),
            &accepted_dto,
            "artifact-cli-1",
            digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            "2026-08-02T03:04:05Z",
            AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated")
                .expect("summary"),
        )
        .expect("terminal");
        let transition = AnalysisRunLifecycleTransition::terminal(
            accepted_dto.run_id.clone(),
            "cli-idem-1",
            terminal,
            Some(artifact),
        )
        .expect("transition");
        let terminal_invocation = AnalysisRunCliInvocation::from_args(
            ["terminal", "--host", "127.0.0.1:18081"],
            transition.to_json().expect("json"),
        )
        .expect("terminal invocation");
        let terminal_response =
            dispatch_analysis_run_cli(&mut service, &terminal_invocation).expect("terminal");
        assert_eq!(terminal_response.status_code, 200);
        let terminal_stdout =
            render_analysis_run_cli_stdout(&terminal_invocation, &terminal_response)
                .expect("terminal stdout");
        assert!(terminal_stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

        let status = AnalysisRunCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                accepted_dto.run_id.as_str(),
                "--idempotency-key",
                "cli-idem-1",
            ],
            "",
        )
        .expect("status");
        let status_response = dispatch_analysis_run_cli(&mut service, &status).expect("status");
        let status_stdout =
            render_analysis_run_cli_stdout(&status, &status_response).expect("status stdout");
        assert_eq!(status_stdout, terminal_stdout);
    }

    #[test]
    fn render_refuses_metrics_failed_artifact_and_unknown_state() {
        let create = create_invocation();
        assert_eq!(
            render_analysis_run_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: "{\"run_state\":\"accepted\",\"rmse\":1.0}".into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_analysis_run_cli_stdout(
            &create,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: "{\"error_code\":\"invalid_wire_payload\"}".into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        assert_eq!(
            render_analysis_run_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: format!("{{\"schema_version\":\"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}\"}}"),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let status = AnalysisRunCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                "run-1",
                "--idempotency-key",
                "cli-idem-1",
            ],
            "",
        )
        .expect("status");
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        "{{\"run_state\":\"accepted\",\"schema_version\":\"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}\"}}"
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        "{{\"run_state\":\"failed\",\"schema_version\":\"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}\"}}"
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: "{\"run_state\":\"queued\"}".into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: "not-json".into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let failed_ok = render_analysis_run_cli_stdout(
            &status,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: "{\"run_state\":\"failed\",\"run_id\":\"run-1\"}".into(),
            },
        )
        .expect("failed");
        assert!(failed_ok.contains("\"failed\""));
        let other_profile = render_analysis_run_cli_stdout(
            &status,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: "{\"run_state\":\"succeeded\",\"terminal_result\":{\"output_profile\":\"other_v1\"}}"
                    .into(),
            },
        )
        .expect("other");
        assert!(other_profile.contains("other_v1"));
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        "{{\"run_state\":\"succeeded\",\"terminal_result\":{{\"output_profile\":\"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}\"}}}}"
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        "{{\"run_state\":\"succeeded\",\"terminal_result\":{{\"output_profile\":\"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}\",\"scientific_acceptance\":{{\"schema_version\":\"other\"}}}}}}"
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: "{\"run_state\":\"succeeded\",\"terminal_result\":{\"output_profile\":\"other_v1\",\"scientific_acceptance\":{}}}"
                        .into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = create_invocation();
        invocation.host = addr.to_string();
        let response = execute_analysis_run_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 202);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_analysis_run_cli(&invocation).unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let parsed = parse_http_response(b"HTTP/1.1 202 Accepted\r\ncontent-length: 2\r\n\r\n{}")
            .expect("parse");
        assert_eq!(parsed.status_code, 202);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.0 200 OK\r\ncontent-length: 2\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 299 Mystery\r\ncontent-length: 2\r\n\r\n{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\ncontent-length: 2\r\n\r\n{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 9\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\nbad-header\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(&[0xff, 0xfe]).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(static_reason(400).expect("400"), "Bad Request");
        assert_eq!(static_reason(403).expect("403"), "Forbidden");
        assert_eq!(static_reason(413).expect("413"), "Payload Too Large");
        assert_eq!(static_reason(422).expect("422"), "Unprocessable Entity");
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1\r\ncontent-length: 0\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 abc OK\r\ncontent-length: 0\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: x\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\nhost: 127.0.0.1\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn stdin_reader_skips_terminal_status_and_reads_otherwise() {
        let empty = read_analysis_run_cli_stdin(AnalysisRunCliVerb::Status, true, std::io::empty())
            .expect("tty status");
        assert!(empty.is_empty());
        let piped = read_analysis_run_cli_stdin(
            AnalysisRunCliVerb::Status,
            false,
            std::io::Cursor::new(b"leftover"),
        )
        .expect("piped");
        assert_eq!(piped, "leftover");
        let create = read_analysis_run_cli_stdin(
            AnalysisRunCliVerb::Create,
            true,
            std::io::Cursor::new(b"{\"ok\":true}"),
        )
        .expect("create");
        assert_eq!(create, "{\"ok\":true}");
        let terminal = read_analysis_run_cli_stdin(
            AnalysisRunCliVerb::Terminal,
            false,
            std::io::Cursor::new(b"{}"),
        )
        .expect("terminal");
        assert_eq!(terminal, "{}");
        let running = read_analysis_run_cli_stdin(
            AnalysisRunCliVerb::Running,
            true,
            std::io::Cursor::new(b"ignored"),
        )
        .expect("running tty");
        assert!(running.is_empty());
    }
}
