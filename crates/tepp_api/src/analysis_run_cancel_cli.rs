//! Operator loopback CLI for analysis-run cancel POST.
//!
//! GAP-003A ninth slice: operators run `tepp-analysis-runs cancel` to withdraw
//! accepted or running runs without writing raw HTTP. Stdout stays metric-free.
//! `tepp.scientific_acceptance.v1` never appears. This module does not duplicate
//! GET-by-id (#359), lifecycle POST (#360), cancel HTTP (#361),
//! scientific-acceptance CLI (#362), collection GET (#368), collection CLI list
//! (#371), or consumer-parity cancel (#373). Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::analysis_run_cancel_http::{encode_path_segment, refuse_metrics_on_cancel_payload};
use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::wire::require_nonempty;
use crate::{
    ANALYSIS_RUN_CANCEL_ID_MAX_LEN, AnalysisRunCancelRequest, AnalysisRunLiveService,
    AnalysisRunStatus, AnalysisRunStatusState, ApiError, NARUON_LIVE_IO_TIMEOUT,
    NaruonLiveResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback cancel CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunCancelCliVerb {
    /// `POST /v1/analysis-runs/{run_id}/cancel`.
    Cancel,
}

impl AnalysisRunCancelCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "cancel" => Ok(Self::Cancel),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancel => "cancel",
        }
    }
}

/// One operator CLI invocation against a loopback cancel POST listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunCancelCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunCancelCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Exact request idempotency key of the accepted run.
    pub idempotency_key: String,
    /// Optional typed cancel JSON. Empty POST is admitted.
    pub body: String,
}

impl AnalysisRunCancelCliInvocation {
    /// Parse argv plus stdin body into a validated loopback cancel invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, an unpublished consumer, credential-shaped flags,
    /// hostile identities, metric bodies, or a typed body that does not match
    /// the path identity and idempotency key.
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
        let verb = AnalysisRunCancelCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile cancel body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, oversized, or metric-bearing fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_CANCEL_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_cancel_payload(&self.body)?;
        if !self.body.is_empty() {
            let request = AnalysisRunCancelRequest::from_json(&self.body)?;
            if request.run_id != self.run_id || request.idempotency_key != self.idempotency_key {
                return Err(ApiError::InvalidWirePayload);
            }
        }
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
    verb: AnalysisRunCancelCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunCancelCliInvocation, ApiError> {
    let invocation = AnalysisRunCancelCliInvocation {
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

/// Compose one HTTP/1.1 cancel POST for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunCancelCliInvocation::validate`].
pub fn compose_analysis_run_cancel_cli_http(
    invocation: &AnalysisRunCancelCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let encoded_run_id = encode_path_segment(&invocation.run_id);
    let path = format!("{NARUON_ANALYSIS_RUN_PATH}/{encoded_run_id}/cancel");
    Ok(format!(
        "POST {path} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{}",
        invocation.host,
        invocation.consumer,
        invocation.idempotency_key,
        invocation.body.len(),
        invocation.body
    ))
}

/// Dispatch one cancel CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_cancel_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunCancelCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_cancel_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one cancel CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_cancel_cli(
    invocation: &AnalysisRunCancelCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_cancel_cli_http(invocation)?;
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

/// Filter CLI stdout so cancel receipts never print scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a non-cancelled success body.
pub fn render_analysis_run_cancel_cli_stdout(
    invocation: &AnalysisRunCancelCliInvocation,
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
    let status = AnalysisRunStatus::from_json(&response.body)?;
    if status.run_state != AnalysisRunStatusState::Cancelled
        || status.run_id != invocation.run_id
        || status.idempotency_key != invocation.idempotency_key
        || status.terminal_result.is_some()
    {
        return Err(ApiError::InvalidWirePayload);
    }
    status.to_json()
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

/// Read stdin leftover bytes on a non-terminal; empty cancel POST is admitted.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_cancel_cli_stdin(
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
        AnalysisRunCancelCliInvocation, AnalysisRunCancelCliVerb, SCIENTIFIC_ACCEPTANCE_SCHEMA,
        compose_analysis_run_cancel_cli_http, dispatch_analysis_run_cancel_cli,
        execute_analysis_run_cancel_cli, parse_http_response, read_analysis_run_cancel_cli_stdin,
        render_analysis_run_cancel_cli_stdout, static_reason,
    };
    use crate::{
        ANALYSIS_RUN_CANCEL_CONTRACT_VERSION, ANALYSIS_RUN_CANCEL_ID_MAX_LEN,
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunCancelRequest,
        AnalysisRunLiveService, AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState,
        ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
        NaruonLiveResponse,
    };

    fn request(idempotency_key: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "cli-cancel-tenant".into(),
            snapshot_id: "cli-cancel-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn create_http(run: &AnalysisRunRequest, consumer: &str, host: &str) -> String {
        let body = run.to_json().expect("json");
        format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    }

    fn cancel_invocation(run_id: &str, idempotency_key: &str) -> AnalysisRunCancelCliInvocation {
        AnalysisRunCancelCliInvocation::from_args(
            [
                "cancel",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                run_id,
                "--idempotency-key",
                idempotency_key,
            ],
            "",
        )
        .expect("cancel")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunCancelCliVerb::parse("cancel").expect("cancel"),
            AnalysisRunCancelCliVerb::Cancel
        );
        assert_eq!(AnalysisRunCancelCliVerb::Cancel.as_str(), "cancel");
        assert_eq!(
            AnalysisRunCancelCliVerb::parse("CANCEL"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelCliVerb::parse("list"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelCliVerb::parse("status"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCancelCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_and_credential_flags() {
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(["cancel"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(["cancel", "--host", "127.0.0.1:18081"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "extra"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
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
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1",
                    "--host",
                    "127.0.0.1:9"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    &"a".repeat(ANALYSIS_RUN_CANCEL_ID_MAX_LEN + 1),
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                r#"{"rmse":1.0}"#
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#)
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mismatched = AnalysisRunCancelRequest::new("other-run", "idem-1")
            .expect("request")
            .to_json()
            .expect("json");
        assert_eq!(
            AnalysisRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                mismatched
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn cancel_assembles_default_consumer_and_optional_typed_body() {
        let cancel = cancel_invocation("tepp-run-1", "idem-1");
        assert_eq!(cancel.verb, AnalysisRunCancelCliVerb::Cancel);
        assert_eq!(cancel.consumer, NARUON_CONSUMER_CODE);
        assert!(cancel.body.is_empty());
        let http = compose_analysis_run_cancel_cli_http(&cancel).expect("http");
        assert!(http.starts_with("POST /v1/analysis-runs/tepp-run-1/cancel HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(http.contains("idempotency-key: idem-1"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("copilot"));
        assert!(!http.contains("tepp-page-cursor"));

        let typed = AnalysisRunCancelRequest::new("tepp-run-1", "idem-1")
            .expect("typed")
            .to_json()
            .expect("json");
        let with_body = AnalysisRunCancelCliInvocation::from_args(
            [
                "cancel",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1",
            ],
            typed.clone(),
        )
        .expect("typed body");
        assert_eq!(with_body.consumer, LINEAGEWEAVE_CONSUMER_CODE);
        let typed_http = compose_analysis_run_cancel_cli_http(&with_body).expect("typed http");
        assert!(typed_http.contains("tepp-consumer: lineageweave"));
        assert!(typed_http.contains(&format!("content-length: {}", typed.len())));
        assert!(typed_http.contains(&typed));
        assert_eq!(
            ANALYSIS_RUN_CANCEL_CONTRACT_VERSION,
            AnalysisRunCancelRequest::from_json(&typed)
                .expect("decode")
                .contract_version
        );

        let encoded = cancel_invocation("run/../../etc", "key");
        let encoded_http = compose_analysis_run_cancel_cli_http(&encoded).expect("encoded");
        assert!(
            encoded_http.contains("POST /v1/analysis-runs/run%2F..%2F..%2Fetc/cancel HTTP/1.1")
        );
    }

    #[test]
    fn dispatch_cancels_accepted_runs_without_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let first = request("cli-cancel-idem-1");
        let created = service.handle_http_request(&create_http(
            &first,
            NARUON_CONSUMER_CODE,
            "127.0.0.1:18081",
        ));
        assert_eq!(created.status_code, 202);
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let invocation = cancel_invocation(&accepted.run_id, &first.idempotency_key);
        let cancelled =
            dispatch_analysis_run_cancel_cli(&mut service, &invocation).expect("cancel");
        assert_eq!(cancelled.status_code, 200);
        let stdout =
            render_analysis_run_cancel_cli_stdout(&invocation, &cancelled).expect("stdout");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("\"terminal_result\":{"));
        let status = AnalysisRunStatus::from_json(&stdout).expect("status");
        assert_eq!(status.run_id, accepted.run_id);
        assert_eq!(status.run_state, AnalysisRunStatusState::Cancelled);
        assert!(status.terminal_result.is_none());

        let replay = dispatch_analysis_run_cancel_cli(&mut service, &invocation).expect("replay");
        assert_eq!(replay.status_code, 200);
        let replay_stdout =
            render_analysis_run_cancel_cli_stdout(&invocation, &replay).expect("replay stdout");
        let replay_status = AnalysisRunStatus::from_json(&replay_stdout).expect("replay status");
        assert_eq!(replay_status.run_state, AnalysisRunStatusState::Cancelled);

        let other = AnalysisRunCancelCliInvocation::from_args(
            [
                "cancel",
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
        .expect("other consumer");
        let isolated = dispatch_analysis_run_cancel_cli(&mut service, &other).expect("isolated");
        assert_eq!(isolated.status_code, 400);
        let isolated_stdout =
            render_analysis_run_cancel_cli_stdout(&other, &isolated).expect("isolated stdout");
        assert!(isolated_stdout.contains("invalid_wire_payload"));
        assert!(!isolated_stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
    }

    #[test]
    fn render_refuses_metrics_scientific_acceptance_and_empty_bodies() {
        let cancel = cancel_invocation("tepp-run-1", "idem-1");
        assert_eq!(
            render_analysis_run_cancel_cli_stdout(
                &cancel,
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
            render_analysis_run_cancel_cli_stdout(
                &cancel,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"cancelled","idempotency_key":"idem-1","rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cancel_cli_stdout(
                &cancel,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        r#"{{"contract_version":1,"run_id":"tepp-run-1","run_state":"cancelled","idempotency_key":"idem-1","schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_cancel_cli_stdout(
                &cancel,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-1","terminal_result":null}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_analysis_run_cancel_cli_stdout(
            &cancel,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        assert_eq!(
            render_analysis_run_cancel_cli_stdout(
                &cancel,
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
            render_analysis_run_cancel_cli_stdout(
                &cancel,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: r#"{"rmse":0.1}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let cancelled_ok = render_analysis_run_cancel_cli_stdout(
            &cancel,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"cancelled","idempotency_key":"idem-1","terminal_result":null}"#.into(),
            },
        )
        .expect("cancelled");
        assert!(cancelled_ok.contains("\"run_state\":\"cancelled\""));
        assert!(!cancelled_ok.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let created = service.handle_http_request(&create_http(
            &request("cli-cancel-tcp"),
            NARUON_CONSUMER_CODE,
            &addr.to_string(),
        ));
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = cancel_invocation(&accepted.run_id, "cli-cancel-tcp");
        invocation.host = addr.to_string();
        let response = execute_analysis_run_cancel_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_analysis_run_cancel_cli(&invocation).unwrap_err(),
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
        let empty = read_analysis_run_cancel_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped = read_analysis_run_cancel_cli_stdin(false, std::io::Cursor::new(b"leftover"))
            .expect("piped");
        assert_eq!(piped, "leftover");
        let piped_empty =
            read_analysis_run_cancel_cli_stdin(false, std::io::Cursor::new(b"")).expect("empty");
        assert!(piped_empty.is_empty());
    }
}
