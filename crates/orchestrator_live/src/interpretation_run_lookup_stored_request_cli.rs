//! Operator loopback CLI for contextual-orchestrator lookup stored-request GET.
//!
//! GAP-003A unique slice: operators run
//! `tepp-interpretation-run-lookup-request get` to mint
//! `contextual_orchestrator_interpretation_run_lookup_stored_request_exchange`
//! onto spawned `tepp-orchestrator-loopback` TCP. Stdout is the stored create
//! request with `scientific_authority=false`. `tepp.scientific_acceptance.v1`
//! never appears. The CLI does not infer causality or call a model provider.
//! Naruon and `LineageWeave` are refused. `NaruonLiveService` stays POST-only.
//! This module does not duplicate lookup stored-request GET (#469), lookup
//! GET/CLI (#467/#468), stored-request GET/CLI (#453/#454), GET-by-id (#438),
//! retrieval CLI (#439), collection GET/CLI (#433/#436), create CLI (#425),
//! export lookup (#466), analysis-run lookup (#380/#401), or cancel lineages
//! (closed). Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::http::{header_is_credential, map_io_error};
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_lookup_http::INTERPRETATION_RUN_LOOKUP_PREFIX;
use crate::interpretation_run_lookup_stored_request_http::{
    InterpretationRunLookupStoredRequestHttpExchange,
    contextual_orchestrator_interpretation_run_lookup_stored_request_exchange,
    interpretation_run_lookup_stored_request_path_id,
};
use crate::interpretation_run_stored_request_http::refuse_metrics_on_interpretation_run_stored_request_payload;
use crate::request::{
    DEFAULT_INTERPRETATION_BYTE_LIMIT, InterpretationRunRequest, require_nonempty,
};
use crate::{
    LIVE_HEADER_BYTE_LIMIT, LIVE_HEADER_COUNT_LIMIT, OrchestratorLiveError,
    OrchestratorLiveResponse, OrchestratorLiveService,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const CLI_IO_TIMEOUT: Duration = Duration::from_secs(2);
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_INTERPRETATION_BYTE_LIMIT;

/// Supported operator verbs for lookup stored-request GET.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationRunLookupStoredRequestCliVerb {
    /// `GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request`.
    Get,
}

impl InterpretationRunLookupStoredRequestCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, OrchestratorLiveError> {
        match token {
            "get" => Ok(Self::Get),
            _ => Err(OrchestratorLiveError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
        }
    }
}

/// One operator CLI invocation against a loopback lookup stored-request listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunLookupStoredRequestCliInvocation {
    /// CLI verb to execute.
    pub verb: InterpretationRunLookupStoredRequestCliVerb,
    /// Loopback `host:port` of `tepp-orchestrator-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed exchange.
    pub origin: String,
    /// Published modular consumer. Lookup stored-request GET admits
    /// `contextual-orchestrator` only.
    pub consumer: String,
    /// Server-assigned opaque interpretation-run identity.
    pub interpretation_run_id: String,
    /// JSON body. GET requires empty.
    pub body: String,
}

impl InterpretationRunLookupStoredRequestCliInvocation {
    /// Parse argv plus stdin body into a validated loopback invocation.
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
        let verb = InterpretationRunLookupStoredRequestCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
    /// host and [`OrchestratorLiveError::InvalidWirePayload`] or
    /// [`OrchestratorLiveError::LimitExceeded`] for empty, unpublished,
    /// nonempty-body, reserved-prefix, or oversized fields.
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
        require_nonempty(&self.interpretation_run_id)?;
        if self.interpretation_run_id == INTERPRETATION_RUN_LOOKUP_PREFIX {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if self.interpretation_run_id.contains('/') || self.interpretation_run_id.contains('\0') {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        if !self.body.is_empty() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        refuse_scientific_acceptance(&self.body)?;
        refuse_metrics_on_interpretation_run_stored_request_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    interpretation_run_id: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, OrchestratorLiveError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        interpretation_run_id: None,
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
            "interpretation-run-id" => &mut flags.interpretation_run_id,
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
    verb: InterpretationRunLookupStoredRequestCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<InterpretationRunLookupStoredRequestCliInvocation, OrchestratorLiveError> {
    let invocation = InterpretationRunLookupStoredRequestCliInvocation {
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
        interpretation_run_id: flags
            .interpretation_run_id
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

/// Render a typed lookup stored-request GET as HTTP/1.1 for a loopback listener.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
/// host or a credential-bearing header, and
/// [`OrchestratorLiveError::InvalidWirePayload`] when the exchange is not a
/// GET `/v1/interpretation-runs/by-run-id/{id}/request` with an empty body.
pub fn loopback_http1_from_interpretation_run_lookup_stored_request_exchange(
    exchange: &InterpretationRunLookupStoredRequestHttpExchange,
    loopback_host: &str,
) -> Result<String, OrchestratorLiveError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "GET" {
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
    let _interpretation_run_id = interpretation_run_lookup_stored_request_path_id(path)?;
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

/// Compose one HTTP/1.1 lookup stored-request GET from the typed exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`InterpretationRunLookupStoredRequestCliInvocation::validate`].
pub fn compose_interpretation_run_lookup_stored_request_cli_http(
    invocation: &InterpretationRunLookupStoredRequestCliInvocation,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    let exchange = contextual_orchestrator_interpretation_run_lookup_stored_request_exchange(
        &invocation.origin,
        &invocation.interpretation_run_id,
    )?;
    loopback_http1_from_interpretation_run_lookup_stored_request_exchange(
        &exchange,
        &invocation.host,
    )
}

/// Dispatch one lookup stored-request CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_interpretation_run_lookup_stored_request_cli(
    service: &mut OrchestratorLiveService,
    invocation: &InterpretationRunLookupStoredRequestCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let request = compose_interpretation_run_lookup_stored_request_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one lookup stored-request CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_interpretation_run_lookup_stored_request_cli(
    invocation: &InterpretationRunLookupStoredRequestCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_interpretation_run_lookup_stored_request_cli_http(invocation)?;
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

/// Filter CLI stdout so lookup stored-request GET never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a receipt carries
/// metric keys, evidence, causal scores, or
/// `tepp.scientific_acceptance.v1`.
pub fn render_interpretation_run_lookup_stored_request_cli_stdout(
    invocation: &InterpretationRunLookupStoredRequestCliInvocation,
    response: &OrchestratorLiveResponse,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_interpretation_run_stored_request_payload(&response.body)?;
    if response.status_code != 200 {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let stored = InterpretationRunRequest::from_json(&response.body)?;
    stored.to_json()
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

/// Read stdin leftover bytes on a non-terminal; GET admits empty.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when stdin cannot be
/// read and [`OrchestratorLiveError::LimitExceeded`] when leftover stdin
/// exceeds the interpretation-run wire limit.
pub fn read_interpretation_run_lookup_stored_request_cli_stdin(
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
        InterpretationRunLookupStoredRequestCliInvocation,
        InterpretationRunLookupStoredRequestCliVerb,
        compose_interpretation_run_lookup_stored_request_cli_http,
        loopback_http1_from_interpretation_run_lookup_stored_request_exchange,
        read_interpretation_run_lookup_stored_request_cli_stdin,
    };
    use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
    use crate::{
        OrchestratorLiveError,
        contextual_orchestrator_interpretation_run_lookup_stored_request_exchange,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn get_args() -> [&'static str; 9] {
        [
            "get",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
            "--interpretation-run-id",
            "orch-run-1",
        ]
    }

    #[test]
    fn from_args_mints_get_and_refuses_fail_closed_inputs() {
        assert_eq!(
            InterpretationRunLookupStoredRequestCliVerb::parse("get").expect("get"),
            InterpretationRunLookupStoredRequestCliVerb::Get
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliVerb::Get.as_str(),
            "get"
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliVerb::parse("lookup"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        let get = InterpretationRunLookupStoredRequestCliInvocation::from_args(get_args(), "")
            .expect("get");
        assert_eq!(get.verb, InterpretationRunLookupStoredRequestCliVerb::Get);
        let http = compose_interpretation_run_lookup_stored_request_cli_http(&get).expect("http");
        assert!(
            http.starts_with("GET /v1/interpretation-runs/by-run-id/orch-run-1/request HTTP/1.1")
        );
        assert!(http.contains("tepp-consumer: contextual-orchestrator"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key:"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "8.8.8.8:80",
                    "--origin",
                    ORIGIN,
                    "--interpretation-run-id",
                    "orch-run-1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "localhost:18082",
                    "--origin",
                    ORIGIN,
                    "--interpretation-run-id",
                    "orch-run-1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    "http://tepp.example.test",
                    "--interpretation-run-id",
                    "orch-run-1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--interpretation-run-id",
                    "orch-run-1",
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
    fn from_args_refuses_unpublished_body_and_reserved_prefix() {
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon",
                    "--interpretation-run-id",
                    "orch-run-1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "lineageweave",
                    "--interpretation-run-id",
                    "orch-run-1"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(get_args(), "{}")
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunLookupStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--interpretation-run-id",
                    "by-run-id"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let leftover = read_interpretation_run_lookup_stored_request_cli_stdin(
            false,
            std::io::Cursor::new(b"leftover"),
        )
        .expect("leftover");
        assert_eq!(leftover, "leftover");
        assert!(
            read_interpretation_run_lookup_stored_request_cli_stdin(true, std::io::empty())
                .expect("tty")
                .is_empty()
        );
        let exchange = contextual_orchestrator_interpretation_run_lookup_stored_request_exchange(
            ORIGIN,
            "orch-run-1",
        )
        .expect("exchange");
        let mut posted = exchange.clone();
        posted.method = "POST";
        assert_eq!(
            loopback_http1_from_interpretation_run_lookup_stored_request_exchange(
                &posted,
                "127.0.0.1:18082"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }
}
