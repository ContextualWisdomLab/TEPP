//! Operator loopback CLI for naruon export idempotency-key lookup GET.
//!
//! Operators run `tepp-export-lookup lookup` to mint
//! `naruon_export_idempotency_lookup_exchange` onto spawned `tepp-loopback`
//! TCP. Stdout is the metric-free `ExportIdempotencyLookup`. Accepted
//! idempotency keys remain opaque data: slash-containing keys are percent-
//! encoded by the HTTP contract and the literal `by-idempotency` value remains
//! addressable after the route prefix. `tepp.scientific_acceptance.v1` never
//! appears. `LineageWeave` is refused and `NaruonLiveService` stays POST-only.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::export_idempotency_lookup_http::export_idempotency_lookup_path_key;
use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    naruon_export_idempotency_lookup_exchange, refuse_metrics_on_export_idempotency_lookup_payload,
    AnalysisRunLiveService, ApiError, ErrorEnvelope, ExportIdempotencyLookup, NaruonHttpExchange,
    NaruonLiveResponse, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN,
    NARUON_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT,
};

const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    NARUON_LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

/// Supported operator verbs for the loopback export idempotency-lookup CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportIdempotencyLookupCliVerb {
    /// `GET /v1/exports/by-idempotency/{idempotency_key}`.
    Lookup,
}

impl ExportIdempotencyLookupCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "lookup" => Ok(Self::Lookup),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lookup => "lookup",
        }
    }
}

/// One operator CLI invocation against a loopback export lookup listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportIdempotencyLookupCliInvocation {
    /// CLI verb to execute.
    pub verb: ExportIdempotencyLookupCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed lookup exchange.
    pub origin: String,
    /// Published modular consumer. Lookup GET admits `naruon` only.
    pub consumer: String,
    /// Exact request idempotency key to resolve.
    pub idempotency_key: String,
    /// JSON body. Lookup GET requires empty.
    pub body: String,
}

impl ExportIdempotencyLookupCliInvocation {
    /// Parse argv plus stdin body into a validated loopback lookup invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished or `LineageWeave`
    /// consumer, credential-shaped flags, an invalid key, or a nonempty body.
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
        let verb = ExportIdempotencyLookupCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, `LineageWeave`, nonempty-body, NUL-containing, or
    /// oversized fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        if !self.origin.starts_with("https://") {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.consumer)?;
        if self.consumer != NARUON_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.idempotency_key)?;
        if self.idempotency_key.contains('\0') {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.idempotency_key.len() > EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_export_idempotency_lookup_payload(&self.body)?;
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
    verb: ExportIdempotencyLookupCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<ExportIdempotencyLookupCliInvocation, ApiError> {
    let invocation = ExportIdempotencyLookupCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
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

/// Render a typed lookup GET exchange as HTTP/1.1 for a loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed. GET-by-id,
/// collection, stored-request extra-segments, and pagination headers fail
/// closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/exports/by-idempotency/{key}` with an empty body.
pub fn loopback_http1_from_export_idempotency_lookup_exchange(
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
    let _key = export_idempotency_lookup_path_key(path)?;
    let mut seen = HashSet::with_capacity(exchange.headers.len());
    let mut has_content_type = false;
    let mut has_consumer = false;
    let mut has_contract = false;
    for (name, value) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
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
                value == NARUON_CONSUMER_CODE
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

/// Compose one HTTP/1.1 lookup GET from the typed naruon exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ExportIdempotencyLookupCliInvocation::validate`].
pub fn compose_export_idempotency_lookup_cli_http(
    invocation: &ExportIdempotencyLookupCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange =
        naruon_export_idempotency_lookup_exchange(&invocation.origin, &invocation.idempotency_key)?;
    loopback_http1_from_export_idempotency_lookup_exchange(&exchange, &invocation.host)
}

/// Dispatch one lookup CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_export_idempotency_lookup_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ExportIdempotencyLookupCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_export_idempotency_lookup_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one lookup CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_export_idempotency_lookup_cli(
    invocation: &ExportIdempotencyLookupCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_export_idempotency_lookup_cli_http(invocation)?;
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

/// Filter CLI stdout so lookup GET never prints scientific acceptance.
///
/// RMSE, bias, coverage, SE-gate, tenant, principal, source-text, and
/// causal-score keys fail closed. Success stdout is only the metric-free
/// identity projection.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a body carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not an
/// `ExportIdempotencyLookup`.
pub fn render_export_idempotency_lookup_cli_stdout(
    invocation: &ExportIdempotencyLookupCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_metrics_on_export_idempotency_lookup_payload(&response.body)?;
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
    let lookup = ExportIdempotencyLookup::from_json(&response.body)?;
    lookup.to_json()
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

/// Read stdin leftover bytes on a non-terminal; lookup GET admits empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read and
/// [`ApiError::LimitExceeded`] when leftover stdin exceeds the live wire
/// limit.
pub fn read_export_idempotency_lookup_cli_stdin(
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
mod branch_coverage_tests {
    use std::io::{self, Cursor, Read};

    use super::{
        loopback_http1_from_export_idempotency_lookup_exchange, parse_http_response,
        read_export_idempotency_lookup_cli_stdin, valid_http_field_name,
        ExportIdempotencyLookupCliInvocation, ExportIdempotencyLookupCliVerb,
    };
    use crate::{
        naruon_export_idempotency_lookup_exchange, ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT,
        NARUON_CONSUMER_CODE,
    };

    fn invocation() -> ExportIdempotencyLookupCliInvocation {
        ExportIdempotencyLookupCliInvocation {
            verb: ExportIdempotencyLookupCliVerb::Lookup,
            host: "127.0.0.1:18081".into(),
            origin: "https://tepp.example.test".into(),
            consumer: NARUON_CONSUMER_CODE.into(),
            idempotency_key: "idem-1".into(),
            body: String::new(),
        }
    }

    #[test]
    fn invocation_and_flag_error_arms_are_covered() {
        let mut value = invocation();
        value.origin = "http://tepp.example.test".into();
        assert_eq!(value.validate(), Err(ApiError::InvalidWirePayload));
        value = invocation();
        value.consumer = "lineageweave".into();
        assert_eq!(value.validate(), Err(ApiError::InvalidWirePayload));
        value = invocation();
        value.body = "{}".into();
        assert_eq!(value.validate(), Err(ApiError::InvalidWirePayload));
        value = invocation();
        value.idempotency_key = "idem\nother".into();
        assert_eq!(value.validate(), Err(ApiError::InvalidWirePayload));
        value = invocation();
        value.origin = "https://bad/path".into();
        assert!(super::compose_export_idempotency_lookup_cli_http(&value).is_err());

        for args in [
            vec!["lookup", "host"],
            vec!["lookup", "--host"],
            vec!["lookup", "--host", "a", "--host", "b"],
            vec!["lookup", "--host", ""],
        ] {
            assert!(ExportIdempotencyLookupCliInvocation::from_args(args, "").is_err());
        }
    }

    #[test]
    fn exchange_header_and_target_error_arms_are_covered() {
        let origin = "https://tepp.example.test";
        let base = naruon_export_idempotency_lookup_exchange(origin, "idem-1").expect("exchange");
        let mut cases = Vec::new();
        let mut value = base.clone();
        value.body = "{}".into();
        cases.push(value);
        let mut value = base.clone();
        value.target_url = "http://tepp.example.test/v1/exports/by-idempotency/idem-1".into();
        cases.push(value);
        let mut value = base.clone();
        value.target_url = "https://tepp.example.test".into();
        cases.push(value);
        for (name, header_value) in [("bad name", "x"), ("x-good", "bad\nvalue")] {
            let mut value = base.clone();
            value.headers.push((name.into(), header_value.into()));
            cases.push(value);
        }
        let mut value = base.clone();
        value
            .headers
            .push(("content-type".into(), "application/json".into()));
        cases.push(value);
        for index in 0..base.headers.len() {
            let mut value = base.clone();
            value.headers.remove(index);
            cases.push(value);
        }
        for value in cases {
            assert!(loopback_http1_from_export_idempotency_lookup_exchange(
                &value,
                "127.0.0.1:18081"
            )
            .is_err());
        }
    }

    #[test]
    fn response_parser_and_reader_error_arms_are_covered() {
        use std::fmt::Write as _;

        let oversized_header = "x".repeat(crate::NARUON_LIVE_HEADER_BYTE_LIMIT + 1);
        let mut many_headers = String::new();
        for index in 0..=crate::NARUON_LIVE_HEADER_COUNT_LIMIT {
            write!(many_headers, "x-{index}: b\r\n").expect("string write");
        }
        let cases = [
            vec![0xff],
            b"HTTP/1.1 200 OK".to_vec(),
            format!("{oversized_header}\r\n\r\n").into_bytes(),
            b"HTTP/1.0 200 OK\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 nope\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 999 Unknown\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 Bad\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nbad\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nbad name: x\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nx-good: bad\x01value\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nx-good: a\r\nx-good: b\r\ncontent-length: 0\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\ncontent-length: x\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\ncontent-length: 1\r\n\r\n".to_vec(),
            format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n",
                DEFAULT_PROJECT_HISTORY_BYTE_LIMIT + 1
            )
            .into_bytes(),
            format!("HTTP/1.1 200 OK\r\n{many_headers}content-length: 0\r\n\r\n").into_bytes(),
        ];
        for bytes in cases {
            assert!(parse_http_response(&bytes).is_err());
        }
        for (code, reason) in [
            (202, "Accepted"),
            (400, "Bad Request"),
            (403, "Forbidden"),
            (413, "Payload Too Large"),
            (422, "Unprocessable Entity"),
        ] {
            let response = format!("HTTP/1.1 {code} {reason}\r\ncontent-length: 0\r\n\r\n");
            assert_eq!(
                parse_http_response(response.as_bytes())
                    .expect("response")
                    .status_code,
                code
            );
        }
        assert!(read_export_idempotency_lookup_cli_stdin(false, Cursor::new([0xff])).is_err());
        assert!(read_export_idempotency_lookup_cli_stdin(
            false,
            Cursor::new(vec![b'a'; DEFAULT_PROJECT_HISTORY_BYTE_LIMIT + 1]),
        )
        .is_err());
        assert!(read_export_idempotency_lookup_cli_stdin(false, FailingReader).is_err());
        assert!(!valid_http_field_name(""));
        assert!(!valid_http_field_name("bad name"));
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("redacted"))
        }
    }
}
