//! Operator loopback CLI for analysis-run create POST.
//!
//! GAP-003A tenth slice: operators run `tepp-analysis-runs create` to submit
//! metric-free analysis runs without writing raw HTTP. Stdout stays metric-free
//! `202 Accepted`. `tepp.scientific_acceptance.v1` never appears. This module
//! does not duplicate GET-by-id (#359), lifecycle POST (#360), cancel HTTP
//! (#361), scientific-acceptance CLI (#362), collection GET (#368), collection
//! CLI list (#371), cancel CLI (#378), retry HTTP (#369), stored-request GET
//! (#377), or consumer-parity cancel (#373). Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::analysis_run_cancel_http::refuse_metrics_on_cancel_payload;
use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunAccepted, AnalysisRunLiveService, AnalysisRunRequest, ApiError,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback create CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunCreateCliVerb {
    /// `POST /v1/analysis-runs`.
    Create,
}

impl AnalysisRunCreateCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "create" => Ok(Self::Create),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
        }
    }
}

/// One operator CLI invocation against a loopback create POST listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunCreateCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunCreateCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Exact request idempotency key of the create body.
    pub idempotency_key: String,
    /// Typed metric-free `AnalysisRunRequest` JSON. Empty POST is refused.
    pub body: String,
}

impl AnalysisRunCreateCliInvocation {
    /// Parse argv plus stdin body into a validated loopback create invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, an unpublished consumer, credential-shaped flags,
    /// empty or metric bodies, or a typed body whose idempotency key does not
    /// match the flag.
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
        let verb = AnalysisRunCreateCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile create body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] for empty, unpublished, metric-bearing,
    /// or mismatched fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.body)?;
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_cancel_payload(&self.body)?;
        let request = AnalysisRunRequest::from_json(&self.body)?;
        if request.idempotency_key != self.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    consumer: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        consumer: None,
        idempotency_key: None,
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
    verb: AnalysisRunCreateCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunCreateCliInvocation, ApiError> {
    let invocation = AnalysisRunCreateCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| crate::NARUON_CONSUMER_CODE.to_owned()),
        idempotency_key: flags.idempotency_key.ok_or(ApiError::InvalidWirePayload)?,
        body,
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

/// Compose one HTTP/1.1 create POST for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunCreateCliInvocation::validate`].
pub fn compose_analysis_run_create_cli_http(
    invocation: &AnalysisRunCreateCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    Ok(format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{}",
        invocation.host,
        invocation.consumer,
        invocation.idempotency_key,
        invocation.body.len(),
        invocation.body
    ))
}

/// Dispatch one create CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_create_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunCreateCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_create_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one create CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_create_cli(
    invocation: &AnalysisRunCreateCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_create_cli_http(invocation)?;
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

/// Filter CLI stdout so create receipts never print scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a non-accepted success body.
pub fn render_analysis_run_create_cli_stdout(
    invocation: &AnalysisRunCreateCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_cancel_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    let accepted = AnalysisRunAccepted::from_json(&response.body)?;
    if accepted.run_state != "accepted" || accepted.idempotency_key != invocation.idempotency_key {
        return Err(ApiError::InvalidWirePayload);
    }
    accepted.to_json()
}

fn refuse_scientific_acceptance_schema(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
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

/// Read stdin leftover bytes on a non-terminal; empty create POST is refused.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_create_cli_stdin(
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, ApiError> {
    if stdin_is_terminal {
        Ok(String::new())
    } else {
        let mut body = String::new();
        stdin
            .read_to_string(&mut body)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        Ok(body)
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        AnalysisRunCreateCliInvocation, AnalysisRunCreateCliVerb, SCIENTIFIC_ACCEPTANCE_SCHEMA,
        compose_analysis_run_create_cli_http, dispatch_analysis_run_create_cli,
        execute_analysis_run_create_cli, parse_http_response, read_analysis_run_create_cli_stdin,
        render_analysis_run_create_cli_stdout, static_reason,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLiveService,
        AnalysisRunRequest, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
        NaruonLiveResponse,
    };

    fn request(idempotency_key: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "cli-create-tenant".into(),
            snapshot_id: "cli-create-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn request_json(idempotency_key: &str) -> String {
        request(idempotency_key).to_json().expect("json")
    }

    fn create_invocation(idempotency_key: &str) -> AnalysisRunCreateCliInvocation {
        AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--idempotency-key",
                idempotency_key,
            ],
            request_json(idempotency_key),
        )
        .expect("create")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunCreateCliVerb::parse("create").expect("create"),
            AnalysisRunCreateCliVerb::Create
        );
        assert_eq!(AnalysisRunCreateCliVerb::Create.as_str(), "create");
        assert_eq!(
            AnalysisRunCreateCliVerb::parse("CREATE"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCreateCliVerb::parse("list"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCreateCliVerb::parse("cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCreateCliVerb::parse("status"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_and_credential_flags() {
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(["create"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(["create", "--host", "127.0.0.1:18081"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "8.8.8.8:80",
                    "--idempotency-key",
                    "idem-1"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "not-a-socket",
                    "--idempotency-key",
                    "idem-1"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--authorization",
                    "secret"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--pretty"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "extra"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--run-id",
                    "tepp-run-1"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--page-limit",
                    "1"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--consumer",
                    "other"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--host",
                    "127.0.0.1:9"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1"
                ],
                r#"{"rmse":1.0}"#
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1"
                ],
                format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#)
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1"
                ],
                request_json("other-key")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                ["create", "--host", "127.0.0.1:18081", "--idempotency-key"],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCreateCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18081",
                    "--idempotency-key",
                    "idem-1",
                    "--github-token",
                    "secret"
                ],
                request_json("idem-1")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn create_assembles_default_consumer_and_typed_body() {
        let create = create_invocation("idem-1");
        assert_eq!(create.verb, AnalysisRunCreateCliVerb::Create);
        assert_eq!(create.consumer, NARUON_CONSUMER_CODE);
        assert!(!create.body.is_empty());
        let http = compose_analysis_run_create_cli_http(&create).expect("http");
        assert!(http.starts_with("POST /v1/analysis-runs HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(http.contains("idempotency-key: idem-1"));
        assert!(http.contains(&format!("content-length: {}", create.body.len())));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("copilot"));
        assert!(!http.contains("tepp-page-cursor"));
        assert!(!http.contains("/cancel"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));

        let with_consumer = AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--idempotency-key",
                "idem-lw",
            ],
            request_json("idem-lw"),
        )
        .expect("lineageweave");
        assert_eq!(with_consumer.consumer, LINEAGEWEAVE_CONSUMER_CODE);
        let lw_http = compose_analysis_run_create_cli_http(&with_consumer).expect("lw http");
        assert!(lw_http.contains("tepp-consumer: lineageweave"));
        assert!(lw_http.contains("idempotency-key: idem-lw"));
    }

    #[test]
    fn dispatch_creates_accepted_runs_without_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let invocation = create_invocation("cli-create-idem-1");
        let created = dispatch_analysis_run_create_cli(&mut service, &invocation).expect("create");
        assert_eq!(created.status_code, 202);
        let stdout = render_analysis_run_create_cli_stdout(&invocation, &created).expect("stdout");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("terminal_result"));
        let accepted = AnalysisRunAccepted::from_json(&stdout).expect("accepted");
        assert_eq!(accepted.run_state, "accepted");
        assert_eq!(accepted.idempotency_key, "cli-create-idem-1");
        assert!(!accepted.run_id.is_empty());

        let replay = dispatch_analysis_run_create_cli(&mut service, &invocation).expect("replay");
        assert_eq!(replay.status_code, 202);
        let replay_stdout =
            render_analysis_run_create_cli_stdout(&invocation, &replay).expect("replay stdout");
        let replay_accepted = AnalysisRunAccepted::from_json(&replay_stdout).expect("replay");
        assert_eq!(replay_accepted.run_id, accepted.run_id);

        let other = AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--idempotency-key",
                "cli-create-idem-1",
            ],
            request_json("cli-create-idem-1"),
        )
        .expect("other consumer");
        let isolated = dispatch_analysis_run_create_cli(&mut service, &other).expect("isolated");
        assert_eq!(isolated.status_code, 202);
        let isolated_stdout =
            render_analysis_run_create_cli_stdout(&other, &isolated).expect("isolated stdout");
        let isolated_accepted = AnalysisRunAccepted::from_json(&isolated_stdout).expect("isolated");
        assert_ne!(isolated_accepted.run_id, accepted.run_id);
        assert!(!isolated_stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));

        let conflict_body = {
            let mut conflict = request("cli-create-idem-1");
            conflict.snapshot_id = "other-snapshot".into();
            conflict.to_json().expect("conflict json")
        };
        let conflict = AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--idempotency-key",
                "cli-create-idem-1",
            ],
            conflict_body,
        )
        .expect("conflict invocation");
        let refused = dispatch_analysis_run_create_cli(&mut service, &conflict).expect("conflict");
        assert_eq!(refused.status_code, 400);
        let refused_stdout =
            render_analysis_run_create_cli_stdout(&conflict, &refused).expect("conflict stdout");
        assert!(refused_stdout.contains("invalid_wire_payload") || !refused_stdout.is_empty());
        assert!(!refused_stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
    }

    #[test]
    fn render_refuses_metrics_scientific_acceptance_and_empty_bodies() {
        let create = create_invocation("idem-1");
        assert_eq!(
            render_analysis_run_create_cli_stdout(
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
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: format!(
                        r#"{{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"cancelled","idempotency_key":"idem-1"}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"other"}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_analysis_run_create_cli_stdout(
            &create,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        assert_eq!(
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_create_cli_stdout(
                &create,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: r#"{"rmse":0.1}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let accepted_ok = render_analysis_run_create_cli_stdout(
            &create,
            &NaruonLiveResponse {
                status_code: 202,
                reason_phrase: "Accepted",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1"}"#.into(),
            },
        )
        .expect("accepted");
        assert!(accepted_ok.contains("\"run_state\":\"accepted\""));
        assert!(!accepted_ok.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = create_invocation("cli-create-tcp");
        invocation.host = addr.to_string();
        let response = execute_analysis_run_create_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 202);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_analysis_run_create_cli(&invocation).unwrap_err(),
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
            parse_http_response(b"HTTP/1.0 202 Accepted\r\ncontent-length: 2\r\n\r\n{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 299 Mystery\r\ncontent-length: 2\r\n\r\n{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(
                b"HTTP/1.1 202 Accepted\r\ncontent-length: 2\r\ncontent-length: 2\r\n\r\n{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 202 Accepted\r\ncontent-length: 9\r\n\r\n{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 202 Accepted\r\nbad-header\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(&[0xff, 0xfe]).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(static_reason(202).expect("202"), "Accepted");
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
            parse_http_response(b"HTTP/1.1 202 Accepted\r\ncontent-length: x\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 202 Accepted\r\nhost: 127.0.0.1\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn stdin_reader_skips_terminal_and_reads_otherwise() {
        let empty = read_analysis_run_create_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped = read_analysis_run_create_cli_stdin(false, std::io::Cursor::new(b"leftover"))
            .expect("piped");
        assert_eq!(piped, "leftover");
        let piped_empty =
            read_analysis_run_create_cli_stdin(false, std::io::Cursor::new(b"")).expect("empty");
        assert!(piped_empty.is_empty());
    }
}
