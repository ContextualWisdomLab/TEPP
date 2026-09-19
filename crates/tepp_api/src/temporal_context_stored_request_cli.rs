//! Operator loopback CLI for `LineageWeave` temporal-context stored-request GET.
//!
//! GAP-003A unique slice: operators run `tepp-temporal-context-request get` to
//! mint `lineageweave_temporal_context_stored_request_exchange` onto spawned
//! `tepp-loopback` TCP. Stdout is the stored `TemporalContextRequest`. Event
//! labels and actor lists belong to the original create request and are
//! admitted. `tepp.scientific_acceptance.v1` never appears. RMSE and causal
//! scores fail closed. The CLI does not infer causality. Naruon is refused on
//! this `LineageWeave`-owned adapter. `NaruonLiveService` stays POST-only.
//! Dedicated binary so it does not collide with `tepp-temporal-context-get`
//! (#452) or `tepp-temporal-context` (#414). This module does not duplicate
//! stored-request GET (#463), GET-by-id HTTP (#451), retrieval CLI (#452),
//! temporal-context CLI (#414), collection GET/CLI (closed #449/#450),
//! project-history stored-request CLI (#456), interpretation-run stored-request
//! CLI (#454), export stored-request CLI (#459), cancel lineages, Leiden, or
//! GAP-010 Figma/export. Persistence remains GAP-003B.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::temporal_context_retrieval_http::validate_temporal_context_registry_identity;
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, ErrorEnvelope,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse, TemporalContextRequest,
    lineageweave_temporal_context_stored_request_exchange,
    refuse_metrics_on_temporal_context_stored_request_payload,
    temporal_context_stored_request_path_id,
};

const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    NARUON_LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

/// Supported operator verbs for the loopback temporal-context stored-request CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalContextStoredRequestCliVerb {
    /// `GET /v1/temporal-context/{idempotency_key}/request`.
    Get,
}

impl TemporalContextStoredRequestCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "get" => Ok(Self::Get),
            _ => Err(ApiError::InvalidWirePayload),
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

/// One operator CLI invocation against a loopback stored-request listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalContextStoredRequestCliInvocation {
    /// CLI verb to execute.
    pub verb: TemporalContextStoredRequestCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed stored-request exchange.
    pub origin: String,
    /// Published modular consumer. Stored-request GET admits `lineageweave` only.
    pub consumer: String,
    /// Opaque idempotency key that minted the stored create request.
    pub idempotency_key: String,
    /// JSON body. Stored-request GET requires empty.
    pub body: String,
}

impl TemporalContextStoredRequestCliInvocation {
    /// Parse argv plus stdin body into a validated loopback stored-request invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished or naruon
    /// consumer, credential-shaped flags, a hostile identity, or a nonempty
    /// body.
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
        let verb = TemporalContextStoredRequestCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, naruon, nonempty-body, or hostile identities.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        if !self.origin.starts_with("https://") {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.consumer)?;
        if self.consumer != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        validate_temporal_context_registry_identity(&self.idempotency_key)?;
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_temporal_context_stored_request_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
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
            return Err(ApiError::InvalidWirePayload);
        }
        let name = &flag[2..];
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        let slot = match name {
            "host" => &mut flags.host,
            "origin" => &mut flags.origin,
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
    verb: TemporalContextStoredRequestCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<TemporalContextStoredRequestCliInvocation, ApiError> {
    let invocation = TemporalContextStoredRequestCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| LINEAGEWEAVE_CONSUMER_CODE.to_owned()),
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

/// Render a typed stored-request GET exchange as HTTP/1.1 for a loopback listener.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/temporal-context/{idempotency_key}/request` with
/// an empty body.
pub fn loopback_http1_from_temporal_context_stored_request_exchange(
    exchange: &NaruonHttpExchange,
    loopback_host: &str,
) -> Result<String, ApiError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "GET" {
        return Err(ApiError::InvalidWirePayload);
    }
    if !exchange.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    let rest = exchange
        .target_url
        .strip_prefix("https://")
        .ok_or(ApiError::InvalidWirePayload)?;
    let path = rest
        .find('/')
        .map(|index| &rest[index..])
        .ok_or(ApiError::InvalidWirePayload)?;
    let _idempotency_key = temporal_context_stored_request_path_id(path)?;
    let mut seen = HashSet::with_capacity(exchange.headers.len());
    let mut has_content_type = false;
    let mut has_consumer = false;
    let mut has_contract = false;
    for (name, value) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        if name.eq_ignore_ascii_case("idempotency-key") {
            return Err(ApiError::InvalidWirePayload);
        }
        if !valid_http_field_name(name)
            || value.chars().any(char::is_control)
            || !seen.insert(name.to_ascii_lowercase())
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let valid = match name.to_ascii_lowercase().as_str() {
            "content-type" => {
                has_content_type = true;
                value == "application/json"
            }
            "tepp-consumer" => {
                has_consumer = true;
                value == LINEAGEWEAVE_CONSUMER_CODE
            }
            "tepp-contract-version" => {
                has_contract = true;
                value == "1"
            }
            _ => false,
        };
        if !valid {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    if !has_content_type || !has_consumer || !has_contract {
        return Err(ApiError::InvalidWirePayload);
    }
    let mut request = String::new();
    write!(
        request,
        "{} {path} HTTP/1.1\r\nHost: {host}\r\n",
        exchange.method
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    for (name, value) in &exchange.headers {
        write!(request, "{name}: {value}\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    }
    write!(request, "content-length: 0\r\n\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 stored-request GET from the typed `LineageWeave` exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`TemporalContextStoredRequestCliInvocation::validate`].
pub fn compose_temporal_context_stored_request_cli_http(
    invocation: &TemporalContextStoredRequestCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lineageweave_temporal_context_stored_request_exchange(
        &invocation.origin,
        &invocation.idempotency_key,
    )?;
    loopback_http1_from_temporal_context_stored_request_exchange(&exchange, &invocation.host)
}

/// Dispatch one stored-request CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_temporal_context_stored_request_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &TemporalContextStoredRequestCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_temporal_context_stored_request_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one stored-request CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_temporal_context_stored_request_cli(
    invocation: &TemporalContextStoredRequestCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_temporal_context_stored_request_cli_http(invocation)?;
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
    let bytes = read_bounded(&mut stream, MAXIMUM_HTTP_RESPONSE_BYTES)?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so stored-request GET never prints scientific acceptance.
///
/// Event labels and actor lists belong to the stored create request and are
/// admitted. RMSE, bias, coverage, SE-gate, and causal-score keys fail closed.
/// `TemporalContextRequest` has no idempotency-key field; identity is the path.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a body carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a stored
/// [`TemporalContextRequest`].
pub fn render_temporal_context_stored_request_cli_stdout(
    invocation: &TemporalContextStoredRequestCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_metrics_on_temporal_context_stored_request_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        let expected_code = match response.status_code {
            400 => "invalid_wire_payload",
            403 => "authorization_denied",
            413 => "limit_exceeded",
            422 => "unsupported_contract_version",
            _ => return Err(ApiError::InvalidWirePayload),
        };
        let envelope: ErrorEnvelope =
            serde_json::from_str(&response.body).map_err(|_| ApiError::InvalidWirePayload)?;
        if envelope.error_code() != expected_code {
            return Err(ApiError::InvalidWirePayload);
        }
        return envelope.to_json();
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let stored = TemporalContextRequest::from_json(&response.body)?;
    stored.to_json()
}

fn parse_http_response(bytes: &[u8]) -> Result<NaruonLiveResponse, ApiError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ApiError::InvalidWirePayload)?;
    let (header_block, body) = text
        .split_once("\r\n\r\n")
        .ok_or(ApiError::InvalidWirePayload)?;
    if header_block.len() > NARUON_LIVE_HEADER_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    let mut lines = header_block.split("\r\n");
    let status_line = lines.next().ok_or(ApiError::InvalidWirePayload)?;
    let (version, status) = status_line
        .split_once(' ')
        .ok_or(ApiError::InvalidWirePayload)?;
    if version != "HTTP/1.1" {
        return Err(ApiError::InvalidWirePayload);
    }
    let (code, reason) = status.split_once(' ').ok_or(ApiError::InvalidWirePayload)?;
    let code = code
        .parse::<u16>()
        .map_err(|_| ApiError::InvalidWirePayload)?;
    let reason_phrase = static_reason(code)?;
    if reason != reason_phrase {
        return Err(ApiError::InvalidWirePayload);
    }
    let mut content_length = None;
    let mut seen = HashSet::new();
    for (index, line) in lines.enumerate() {
        if index >= NARUON_LIVE_HEADER_COUNT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let (name, value) = line.split_once(':').ok_or(ApiError::InvalidWirePayload)?;
        if !valid_http_field_name(name)
            || value
                .chars()
                .any(|character| character.is_control() && character != '\t')
            || !seen.insert(name.to_ascii_lowercase())
            || name.eq_ignore_ascii_case("transfer-encoding")
        {
            return Err(ApiError::InvalidWirePayload);
        }
        if name.eq_ignore_ascii_case("content-length") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| ApiError::InvalidWirePayload)?,
            );
        }
    }
    let declared = content_length.ok_or(ApiError::InvalidWirePayload)?;
    if declared > DEFAULT_PROJECT_HISTORY_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
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

/// Read stdin leftover bytes on a non-terminal; stored-request GET admits empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read and
/// [`ApiError::LimitExceeded`] when leftover stdin exceeds the wire limit.
pub fn read_temporal_context_stored_request_cli_stdin(
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, ApiError> {
    if stdin_is_terminal {
        Ok(String::new())
    } else {
        let bytes = read_bounded(&mut stdin, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT)?;
        String::from_utf8(bytes).map_err(|_| ApiError::InvalidWirePayload)
    }
}

fn read_bounded(reader: &mut impl Read, maximum_bytes: usize) -> Result<Vec<u8>, ApiError> {
    let mut bytes = Vec::new();
    reader
        .take((maximum_bytes + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| map_io_error(&error))?;
    if bytes.len() > maximum_bytes {
        return Err(ApiError::LimitExceeded);
    }
    Ok(bytes)
}

fn valid_http_field_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

#[cfg(test)]
mod tests {
    use super::{
        TemporalContextStoredRequestCliInvocation, TemporalContextStoredRequestCliVerb,
        compose_temporal_context_stored_request_cli_http,
        loopback_http1_from_temporal_context_stored_request_exchange,
        read_temporal_context_stored_request_cli_stdin,
    };
    use crate::{
        ApiError, LINEAGEWEAVE_CONSUMER_CODE, NaruonHttpExchange,
        lineageweave_temporal_context_stored_request_exchange,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn get_args() -> [&'static str; 9] {
        [
            "get",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
            "--idempotency-key",
            "idem-a",
        ]
    }

    #[test]
    fn from_args_mints_get_and_refuses_fail_closed_hosts() {
        assert_eq!(
            TemporalContextStoredRequestCliVerb::parse("get").expect("get"),
            TemporalContextStoredRequestCliVerb::Get
        );
        assert_eq!(TemporalContextStoredRequestCliVerb::Get.as_str(), "get");
        assert_eq!(
            TemporalContextStoredRequestCliVerb::parse("list"),
            Err(ApiError::InvalidWirePayload)
        );
        let get =
            TemporalContextStoredRequestCliInvocation::from_args(get_args(), "").expect("get");
        let http = compose_temporal_context_stored_request_cli_http(&get).expect("http");
        assert!(http.starts_with("GET /v1/temporal-context/idem-a/request HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key:"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(
                [
                    "get",
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
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "localhost:18081",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--authorization",
                    "secret",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_naruon_body_slash_and_non_get() {
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18081",
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
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(get_args(), "{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextStoredRequestCliInvocation::from_args(
                [
                    "get",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--idempotency-key",
                    "a/b"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert!(
            read_temporal_context_stored_request_cli_stdin(true, std::io::empty())
                .expect("tty")
                .is_empty()
        );
        let exchange =
            lineageweave_temporal_context_stored_request_exchange(ORIGIN, "idem-a").expect("ex");
        let posted = NaruonHttpExchange {
            method: "POST",
            target_url: exchange.target_url,
            headers: exchange.headers,
            body: exchange.body,
        };
        assert_eq!(
            loopback_http1_from_temporal_context_stored_request_exchange(
                &posted,
                "127.0.0.1:18081"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }
}
