//! Operator loopback CLI for contextual-orchestrator interpretation-run cancel.
//!
//! GAP-003A unique slice: operators run `tepp-interpretation-run-cancel cancel`
//! to mint `contextual_orchestrator_interpretation_run_cancel_exchange` onto
//! spawned `tepp-orchestrator-loopback` TCP. Stdout is one metric-free
//! cancelled identity with `claim_status=hypothetical`,
//! `scientific_authority=false`, and `cancelled=true`.
//! `tepp.scientific_acceptance.v1` never appears. The CLI does not infer
//! causality or call a model provider. Naruon and `LineageWeave` are refused.
//! `NaruonLiveService` stays POST-only. This module does not duplicate
//! interpretation-run CLI (#425), collection GET (#433), collection CLI
//! (#436), GET-by-id HTTP (#438), retrieval CLI (#439), cancel HTTP (#440),
//! analysis-run cancel CLI (#378), Leiden, or GAP-010 Figma/export.
//! Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::http::{header_is_credential, map_io_error};
use crate::interpretation_run_cancel_http::{
    contextual_orchestrator_interpretation_run_cancel_exchange, interpretation_run_cancel_path_id,
    refuse_metrics_on_interpretation_run_cancel_payload, InterpretationRunCancelHttpExchange,
    InterpretationRunCancelled,
};
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::request::{
    require_nonempty, DEFAULT_INTERPRETATION_BYTE_LIMIT, HYPOTHETICAL_CLAIM_STATUS,
};
use crate::{
    OrchestratorLiveError, OrchestratorLiveResponse, OrchestratorLiveService,
    LIVE_HEADER_BYTE_LIMIT, LIVE_HEADER_COUNT_LIMIT,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const CLI_IO_TIMEOUT: Duration = Duration::from_secs(2);
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_INTERPRETATION_BYTE_LIMIT;

/// Supported operator verbs for the loopback interpretation-run cancel CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationRunCancelCliVerb {
    /// `POST /v1/interpretation-runs/{idempotency_key}/cancel`.
    Cancel,
}

impl InterpretationRunCancelCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, OrchestratorLiveError> {
        match token {
            "cancel" => Ok(Self::Cancel),
            _ => Err(OrchestratorLiveError::InvalidWirePayload),
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

/// One operator CLI invocation against a loopback cancel listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunCancelCliInvocation {
    /// CLI verb to execute.
    pub verb: InterpretationRunCancelCliVerb,
    /// Loopback `host:port` of `tepp-orchestrator-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed cancel exchange.
    pub origin: String,
    /// Published modular consumer. Cancel admits `contextual-orchestrator` only.
    pub consumer: String,
    /// Opaque idempotency key that minted the stored identity.
    pub idempotency_key: String,
    /// JSON body. Cancel POST requires empty.
    pub body: String,
}

impl InterpretationRunCancelCliInvocation {
    /// Parse argv plus stdin body into a validated loopback cancel invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished consumer,
    /// credential-shaped flags, a hostile identity, or a nonempty body.
    pub fn from_args<I, S>(args: I, body: impl Into<String>) -> Result<Self, OrchestratorLiveError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let tokens: Vec<String> = args
            .into_iter()
            .map(|token| token.as_ref().to_owned())
            .collect();
        let (verb_token, rest) = tokens
            .split_first()
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
        let verb = InterpretationRunCancelCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile POST body.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
    /// host and [`OrchestratorLiveError::InvalidWirePayload`] or
    /// [`OrchestratorLiveError::LimitExceeded`] for empty, unpublished,
    /// nonempty-body, or oversized fields.
    pub fn validate(&self) -> Result<(), OrchestratorLiveError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        if !self.origin.starts_with("https://") {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        require_nonempty(&self.consumer)?;
        if self.consumer != CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        require_nonempty(&self.idempotency_key)?;
        if self.idempotency_key.contains('/') || self.idempotency_key.contains('\0') {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if !self.body.is_empty() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        refuse_scientific_acceptance(&self.body)?;
        refuse_metrics_on_interpretation_run_cancel_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, OrchestratorLiveError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        idempotency_key: None,
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        if !flag.starts_with("--") {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        let name = &flag[2..];
        if header_is_credential(name) {
            return Err(OrchestratorLiveError::AuthorizationDenied);
        }
        let slot = match name {
            "host" => &mut flags.host,
            "origin" => &mut flags.origin,
            "consumer" => &mut flags.consumer,
            "idempotency-key" => &mut flags.idempotency_key,
            _ => return Err(OrchestratorLiveError::InvalidWirePayload),
        };
        if slot.is_some() || index + 1 >= rest.len() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        let value = rest[index + 1].as_str();
        require_nonempty(value)?;
        *slot = Some(value.to_owned());
        index += 2;
    }
    Ok(flags)
}

fn assemble_invocation(
    verb: InterpretationRunCancelCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<InterpretationRunCancelCliInvocation, OrchestratorLiveError> {
    let invocation = InterpretationRunCancelCliInvocation {
        verb,
        host: flags
            .host
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?,
        origin: flags
            .origin
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE.to_owned()),
        idempotency_key: flags
            .idempotency_key
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?,
        body,
    };
    invocation.validate()?;
    Ok(invocation)
}

fn require_loopback_host(host: &str) -> Result<SocketAddr, OrchestratorLiveError> {
    let addr: SocketAddr = host
        .parse()
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(OrchestratorLiveError::AuthorizationDenied)
    }
}

/// Render a typed cancel POST exchange as HTTP/1.1 for a loopback listener.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
/// host or a credential-bearing header, and
/// [`OrchestratorLiveError::InvalidWirePayload`] when the exchange is not a
/// POST `/v1/interpretation-runs/{idempotency_key}/cancel` with an empty body.
pub fn loopback_http1_from_interpretation_run_cancel_exchange(
    exchange: &InterpretationRunCancelHttpExchange,
    loopback_host: &str,
) -> Result<String, OrchestratorLiveError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "POST" {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if !exchange.body.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let rest = exchange
        .target_url
        .strip_prefix("https://")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let path = rest
        .find('/')
        .map(|index| &rest[index..])
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let _idempotency_key = interpretation_run_cancel_path_id(path)?;
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(OrchestratorLiveError::AuthorizationDenied);
        }
        if name.eq_ignore_ascii_case("idempotency-key")
            || name.eq_ignore_ascii_case("tepp-page-limit")
            || name.eq_ignore_ascii_case("tepp-page-cursor")
        {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
    }
    let mut request = String::new();
    write!(
        request,
        "{} {path} HTTP/1.1\r\nHost: {host}\r\n",
        exchange.method
    )
    .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    for (name, value) in &exchange.headers {
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        write!(request, "{name}: {value}\r\n")
            .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    }
    write!(request, "content-length: 0\r\n\r\n")
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 cancel POST from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`InterpretationRunCancelCliInvocation::validate`].
pub fn compose_interpretation_run_cancel_cli_http(
    invocation: &InterpretationRunCancelCliInvocation,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    let exchange = contextual_orchestrator_interpretation_run_cancel_exchange(
        &invocation.origin,
        &invocation.idempotency_key,
    )?;
    loopback_http1_from_interpretation_run_cancel_exchange(&exchange, &invocation.host)
}

/// Dispatch one cancel CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_interpretation_run_cancel_cli(
    service: &mut OrchestratorLiveService,
    invocation: &InterpretationRunCancelCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let request = compose_interpretation_run_cancel_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one cancel CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_interpretation_run_cancel_cli(
    invocation: &InterpretationRunCancelCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_interpretation_run_cancel_cli_http(invocation)?;
    let mut stream = TcpStream::connect(addr).map_err(|error| map_io_error(&error))?;
    stream
        .set_read_timeout(Some(CLI_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .set_write_timeout(Some(CLI_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| map_io_error(&error))?;
    stream.flush().map_err(|error| map_io_error(&error))?;
    let bytes = read_bounded(&mut stream, MAXIMUM_HTTP_RESPONSE_BYTES)?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so cancel never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a receipt carries
/// metric keys, evidence, causal scores, or
/// `tepp.scientific_acceptance.v1`, or when the identity does not match.
pub fn render_interpretation_run_cancel_cli_stdout(
    invocation: &InterpretationRunCancelCliInvocation,
    response: &OrchestratorLiveResponse,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_interpretation_run_cancel_payload(&response.body)?;
    if response.status_code != 200 {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let parsed: InterpretationRunCancelled = serde_json::from_str(&response.body)
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if !parsed.cancelled {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let item = InterpretationRunCancelled::new(
        parsed.interpretation_run_id,
        parsed.idempotency_key,
        parsed.orchestration_mode,
        parsed.claim_status,
        parsed.scientific_authority,
    )?;
    if item.idempotency_key != invocation.idempotency_key
        || item.claim_status != HYPOTHETICAL_CLAIM_STATUS
        || item.scientific_authority
        || !item.cancelled
    {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    item.to_json()
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), OrchestratorLiveError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(OrchestratorLiveError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn parse_http_response(bytes: &[u8]) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let text = std::str::from_utf8(bytes).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    let (header_block, body) = text
        .split_once("\r\n\r\n")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if header_block.len() > LIVE_HEADER_BYTE_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let mut lines = header_block.split("\r\n");
    let status_line = lines
        .next()
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let mut parts = status_line.split(' ');
    if parts.next() != Some("HTTP/1.1") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let code = parts
        .next()
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?
        .parse::<u16>()
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    let reason_phrase = static_reason(code)?;
    let mut content_length = None;
    for (index, line) in lines.enumerate() {
        if index >= LIVE_HEADER_COUNT_LIMIT {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(OrchestratorLiveError::InvalidWirePayload);
            }
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?,
            );
        }
    }
    let declared = content_length.ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if declared > DEFAULT_INTERPRETATION_BYTE_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    if declared != body.len() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(OrchestratorLiveResponse {
        status_code: code,
        reason_phrase,
        body: body.to_owned(),
    })
}

fn static_reason(code: u16) -> Result<&'static str, OrchestratorLiveError> {
    match code {
        200 => Ok("OK"),
        202 => Ok("Accepted"),
        400 => Ok("Bad Request"),
        403 => Ok("Forbidden"),
        413 => Ok("Payload Too Large"),
        422 => Ok("Unprocessable Entity"),
        _ => Err(OrchestratorLiveError::InvalidWirePayload),
    }
}

/// Read stdin leftover bytes on a non-terminal; cancel POST admits empty.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when stdin cannot be
/// read and [`OrchestratorLiveError::LimitExceeded`] when leftover stdin
/// exceeds the interpretation-run wire limit.
pub fn read_interpretation_run_cancel_cli_stdin(
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, OrchestratorLiveError> {
    if stdin_is_terminal {
        Ok(String::new())
    } else {
        let bytes = read_bounded(&mut stdin, DEFAULT_INTERPRETATION_BYTE_LIMIT)?;
        String::from_utf8(bytes).map_err(|_| OrchestratorLiveError::InvalidWirePayload)
    }
}

fn read_bounded(
    reader: &mut impl Read,
    maximum_bytes: usize,
) -> Result<Vec<u8>, OrchestratorLiveError> {
    let mut bytes = Vec::new();
    reader
        .take((maximum_bytes + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| map_io_error(&error))?;
    if bytes.len() > maximum_bytes {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        compose_interpretation_run_cancel_cli_http,
        loopback_http1_from_interpretation_run_cancel_exchange,
        read_interpretation_run_cancel_cli_stdin, InterpretationRunCancelCliInvocation,
        InterpretationRunCancelCliVerb,
    };
    use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
    use crate::{
        contextual_orchestrator_interpretation_run_cancel_exchange, OrchestratorLiveError,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn cancel_args() -> [&'static str; 9] {
        [
            "cancel",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
            "--idempotency-key",
            "idem-a",
        ]
    }

    #[test]
    fn from_args_mints_cancel_and_refuses_fail_closed_inputs() {
        assert_eq!(
            InterpretationRunCancelCliVerb::parse("cancel").expect("cancel"),
            InterpretationRunCancelCliVerb::Cancel
        );
        assert_eq!(InterpretationRunCancelCliVerb::Cancel.as_str(), "cancel");
        assert_eq!(
            InterpretationRunCancelCliVerb::parse("get"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        let cancel =
            InterpretationRunCancelCliInvocation::from_args(cancel_args(), "").expect("cancel");
        assert_eq!(cancel.verb, InterpretationRunCancelCliVerb::Cancel);
        let http = compose_interpretation_run_cancel_cli_http(&cancel).expect("http");
        assert!(http.starts_with("POST /v1/interpretation-runs/idem-a/cancel HTTP/1.1"));
        assert!(http.contains("tepp-consumer: contextual-orchestrator"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key:"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "8.8.8.8:80",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "localhost:18082",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    "http://tepp.example.test",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "idem-a",
                    "--authorization",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_unpublished_body_pagination_and_non_post() {
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(cancel_args(), "{}").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "idem-a",
                    "--page-limit",
                    "1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let leftover =
            read_interpretation_run_cancel_cli_stdin(false, std::io::Cursor::new(b"leftover"))
                .expect("leftover");
        assert_eq!(leftover, "leftover");
        assert!(
            read_interpretation_run_cancel_cli_stdin(true, std::io::empty())
                .expect("tty")
                .is_empty()
        );
        let exchange = contextual_orchestrator_interpretation_run_cancel_exchange(ORIGIN, "idem-a")
            .expect("exchange");
        let mut gotten = exchange.clone();
        gotten.method = "GET";
        assert_eq!(
            loopback_http1_from_interpretation_run_cancel_exchange(&gotten, "127.0.0.1:18082")
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
    }
}
