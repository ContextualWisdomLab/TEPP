//! Operator loopback CLI for analysis-run status GET.
//!
//! GAP-003A operator-visible client of `GET /v1/analysis-runs/{run_id}` (ADR
//! 0027 / live #359, consumer-parity #383). Operators run
//! `tepp-analysis-runs status` to inspect one accepted, running, succeeded, or
//! failed run without writing raw HTTP. Accepted, running, and failed stdout
//! stays metric-free. `tepp.scientific_acceptance.v1` appears only on a
//! succeeded GET whose request profile is `scientific_acceptance_v1`. This
//! module does not duplicate GET-by-id HTTP, lifecycle POST, cancel HTTP,
//! scientific-acceptance CLI, collection GET/CLI, cancel CLI, or create CLI.
//! Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::analysis_run_status_http::{ANALYSIS_RUN_ID_MAX_LEN, encode_path_segment};
use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::scientific_acceptance_http::{
    SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA, refuse_metrics_on_receipt,
};
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, AnalysisRunStatus, ApiError, NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse,
};

/// Supported operator verbs for the loopback status CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunStatusCliVerb {
    /// `GET /v1/analysis-runs/{run_id}`.
    Status,
}

impl AnalysisRunStatusCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "status" => Ok(Self::Status),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
        }
    }
}

/// One operator CLI invocation against a loopback status GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunStatusCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunStatusCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Exact request idempotency key of the accepted run.
    pub idempotency_key: String,
    /// JSON body. Status GET requires empty.
    pub body: String,
}

impl AnalysisRunStatusCliInvocation {
    /// Parse argv plus stdin body into a validated loopback status invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, an unpublished consumer, credential-shaped flags,
    /// hostile identities, or a nonempty body.
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
        let verb = AnalysisRunStatusCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile status body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, oversized, or nonempty-body fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_scientific_acceptance_on_non_succeeded(&self.body)?;
        refuse_metrics_on_receipt(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    consumer: Option<String>,
    run_id: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        consumer: None,
        run_id: None,
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
            "run-id" => &mut flags.run_id,
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
    verb: AnalysisRunStatusCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunStatusCliInvocation, ApiError> {
    let invocation = AnalysisRunStatusCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| crate::NARUON_CONSUMER_CODE.to_owned()),
        run_id: flags.run_id.ok_or(ApiError::InvalidWirePayload)?,
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

/// Compose one HTTP/1.1 status GET for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunStatusCliInvocation::validate`].
pub fn compose_analysis_run_status_cli_http(
    invocation: &AnalysisRunStatusCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let encoded_run_id = encode_path_segment(&invocation.run_id);
    let path = format!("{NARUON_ANALYSIS_RUN_PATH}/{encoded_run_id}");
    Ok(format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: 0\r\n\r\n",
        invocation.host, invocation.consumer, invocation.idempotency_key
    ))
}

/// Dispatch one status CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_status_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunStatusCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_status_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one status CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_status_cli(
    invocation: &AnalysisRunStatusCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_status_cli_http(invocation)?;
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

/// Filter CLI stdout so non-succeeded status never prints scientific acceptance.
///
/// Succeeded GET with profile `scientific_acceptance_v1` may print
/// `tepp.scientific_acceptance.v1`. Accepted, running, and failed stdout stay
/// metric-free.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a non-succeeded receipt
/// carries metric keys or the scientific-acceptance schema, or when identities
/// do not match the invocation.
pub fn render_analysis_run_status_cli_stdout(
    invocation: &AnalysisRunStatusCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    if !(200..300).contains(&response.status_code) {
        refuse_scientific_acceptance_on_non_succeeded(&response.body)?;
        refuse_metrics_on_receipt(&response.body)?;
        return Ok(response.body.clone());
    }
    if response.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA) {
        if !response.body.contains("\"run_state\":\"succeeded\"") {
            return Err(ApiError::InvalidWirePayload);
        }
        return Ok(response.body.clone());
    }
    refuse_metrics_on_receipt(&response.body)?;
    let status = AnalysisRunStatus::from_json(&response.body)?;
    if status.run_id != invocation.run_id || status.idempotency_key != invocation.idempotency_key {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_on_non_succeeded(&response.body)?;
    status.to_json()
}

fn refuse_scientific_acceptance_on_non_succeeded(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA)
        && !body.contains("\"run_state\":\"succeeded\"")
    {
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

/// Read stdin leftover bytes on a non-terminal; status GET refuses a body.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_status_cli_stdin(
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
        AnalysisRunStatusCliInvocation, AnalysisRunStatusCliVerb,
        compose_analysis_run_status_cli_http, dispatch_analysis_run_status_cli,
        execute_analysis_run_status_cli, parse_http_response, read_analysis_run_status_cli_stdin,
        render_analysis_run_status_cli_stdout, static_reason,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, ANALYSIS_RUN_ID_MAX_LEN, AnalysisResultSummary,
        AnalysisRunAccepted, AnalysisRunLiveService, AnalysisRunRequest, AnalysisRunStatus,
        AnalysisRunStatusState, AnalysisRunTerminalResult, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_CONSUMER_CODE, NaruonLiveResponse, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
        SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    };
    use sha2::{Digest, Sha256};

    fn request(idempotency_key: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "cli-status-tenant".into(),
            snapshot_id: "cli-status-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn create_http(run: &AnalysisRunRequest, consumer: &str, host: &str) -> String {
        let body = run.to_json().expect("json");
        format!(
            "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    }

    fn status_invocation(run_id: &str, idempotency_key: &str) -> AnalysisRunStatusCliInvocation {
        AnalysisRunStatusCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                run_id,
                "--idempotency-key",
                idempotency_key,
            ],
            "",
        )
        .expect("status")
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

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunStatusCliVerb::parse("status").expect("status"),
            AnalysisRunStatusCliVerb::Status
        );
        assert_eq!(AnalysisRunStatusCliVerb::Status.as_str(), "status");
        assert_eq!(
            AnalysisRunStatusCliVerb::parse("STATUS"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStatusCliVerb::parse("list"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStatusCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunStatusCliVerb::parse("cancel"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_and_credential_flags() {
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(["status"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(["status", "--host", "127.0.0.1:18081"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "8.8.8.8:80",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "not-a-socket",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--authorization",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--pretty"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                "{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--page-limit",
                    "1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--consumer",
                    "other"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    &"a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1),
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunStatusCliInvocation::from_args(
                [
                    "status",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--github-token",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn status_assembles_default_consumer_and_encoded_path() {
        let status = status_invocation("tepp-run-1", "idem-1");
        assert_eq!(status.verb, AnalysisRunStatusCliVerb::Status);
        assert_eq!(status.consumer, NARUON_CONSUMER_CODE);
        let http = compose_analysis_run_status_cli_http(&status).expect("http");
        assert!(http.starts_with("GET /v1/analysis-runs/tepp-run-1 HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(http.contains("idempotency-key: idem-1"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("copilot"));
        assert!(!http.contains("tepp-page-cursor"));
        assert!(!http.contains("/cancel"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

        let encoded = status_invocation("run/../../etc", "key");
        let encoded_http = compose_analysis_run_status_cli_http(&encoded).expect("encoded");
        assert!(encoded_http.contains("GET /v1/analysis-runs/run%2F..%2F..%2Fetc HTTP/1.1"));

        let lw = AnalysisRunStatusCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1",
            ],
            "",
        )
        .expect("lw");
        let lw_http = compose_analysis_run_status_cli_http(&lw).expect("lw http");
        assert!(lw_http.contains("tepp-consumer: lineageweave"));
    }

    #[test]
    fn dispatch_reads_accepted_running_and_succeeded_without_leaking_on_receipts() {
        let mut service = AnalysisRunLiveService::new();
        let first = request("cli-status-idem-1");
        let created = service.handle_http_request(&create_http(
            &first,
            NARUON_CONSUMER_CODE,
            "127.0.0.1:18081",
        ));
        assert_eq!(created.status_code, 202);
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let invocation = status_invocation(&accepted.run_id, &first.idempotency_key);
        let got = dispatch_analysis_run_status_cli(&mut service, &invocation).expect("status");
        assert_eq!(got.status_code, 200);
        let stdout = render_analysis_run_status_cli_stdout(&invocation, &got).expect("stdout");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        assert!(!stdout.contains("rmse"));
        let status = AnalysisRunStatus::from_json(&stdout).expect("status");
        assert_eq!(status.run_id, accepted.run_id);
        assert_eq!(status.run_state, AnalysisRunStatusState::Accepted);

        let running = AnalysisRunStatus::running(&accepted).expect("running");
        service
            .record_loopback_status(&accepted.run_id, running, None)
            .expect("running");
        let got_running =
            dispatch_analysis_run_status_cli(&mut service, &invocation).expect("running");
        let running_stdout =
            render_analysis_run_status_cli_stdout(&invocation, &got_running).expect("running out");
        assert!(running_stdout.contains("\"running\""));
        assert!(!running_stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

        let other = AnalysisRunStatusCliInvocation::from_args(
            [
                "status",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--run-id",
                accepted.run_id.as_str(),
                "--idempotency-key",
                first.idempotency_key.as_str(),
            ],
            "",
        )
        .expect("other");
        let isolated = dispatch_analysis_run_status_cli(&mut service, &other).expect("isolated");
        assert_eq!(isolated.status_code, 400);
        let isolated_stdout =
            render_analysis_run_status_cli_stdout(&other, &isolated).expect("isolated stdout");
        assert!(!isolated_stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

        let mut scientific = request("cli-status-sa");
        scientific.output_profile = SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into();
        let created_sa = service.handle_http_request(&create_http(
            &scientific,
            NARUON_CONSUMER_CODE,
            "127.0.0.1:18081",
        ));
        let accepted_sa = AnalysisRunAccepted::from_json(&created_sa.body).expect("sa accepted");
        let artifact = format!(
            r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}","binding_sha256":"{}","run_id":"{}"}}"#,
            "ab".repeat(32),
            accepted_sa.run_id
        );
        let digest = sha256_hex(artifact.as_bytes());
        let terminal = AnalysisRunTerminalResult::succeeded(
            &scientific,
            &accepted_sa,
            "artifact-cli-1",
            digest,
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            "2026-08-02T03:04:05Z",
            AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated")
                .expect("summary"),
        )
        .expect("terminal");
        let succeeded =
            AnalysisRunStatus::terminal(&scientific, &accepted_sa, terminal).expect("succeeded");
        service
            .record_loopback_status(&accepted_sa.run_id, succeeded, Some(artifact))
            .expect("recorded");
        let sa_invocation = status_invocation(&accepted_sa.run_id, "cli-status-sa");
        let got_sa = dispatch_analysis_run_status_cli(&mut service, &sa_invocation).expect("sa");
        assert_eq!(got_sa.status_code, 200);
        let sa_stdout =
            render_analysis_run_status_cli_stdout(&sa_invocation, &got_sa).expect("sa stdout");
        assert!(sa_stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        assert!(sa_stdout.contains("\"run_state\":\"succeeded\""));
    }

    #[test]
    fn render_refuses_metrics_schema_on_non_succeeded_and_empty_bodies() {
        let status = status_invocation("tepp-run-1", "idem-1");
        assert_eq!(
            render_analysis_run_status_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_status_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","terminal_result":null,"rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_status_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        r#"{{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}"}}"#
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_analysis_run_status_cli_stdout(
            &status,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        assert_eq!(
            render_analysis_run_status_cli_stdout(
                &status,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}"}}"#),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let accepted_ok = render_analysis_run_status_cli_stdout(
            &status,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","terminal_result":null}"#.into(),
            },
        )
        .expect("accepted");
        assert!(accepted_ok.contains("\"run_state\":\"accepted\""));
        assert!(!accepted_ok.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        let succeeded_ok = render_analysis_run_status_cli_stdout(
            &status,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: format!(
                    r#"{{"contract_version":1,"run_id":"tepp-run-1","run_state":"succeeded","idempotency_key":"idem-1","schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}"}}"#
                ),
            },
        )
        .expect("succeeded");
        assert!(succeeded_ok.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let created = service.handle_http_request(&create_http(
            &request("cli-status-tcp"),
            NARUON_CONSUMER_CODE,
            &addr.to_string(),
        ));
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = status_invocation(&accepted.run_id, "cli-status-tcp");
        invocation.host = addr.to_string();
        let response = execute_analysis_run_status_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_analysis_run_status_cli(&invocation).unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let parsed =
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}").expect("parse");
        assert_eq!(parsed.status_code, 200);
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
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: x\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\nhost: 127.0.0.1\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn stdin_reader_skips_terminal_and_reads_otherwise() {
        let empty = read_analysis_run_status_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped = read_analysis_run_status_cli_stdin(false, std::io::Cursor::new(b"leftover"))
            .expect("piped");
        assert_eq!(piped, "leftover");
        let piped_empty =
            read_analysis_run_status_cli_stdin(false, std::io::Cursor::new(b"")).expect("empty");
        assert!(piped_empty.is_empty());
    }
}
