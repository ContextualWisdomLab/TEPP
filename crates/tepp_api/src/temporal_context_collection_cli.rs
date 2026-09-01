//! Operator loopback CLI for `LineageWeave` temporal-context collection GET.
//!
//! GAP-003A unique slice: operators run `tepp-temporal-contexts list` to mint
//! `lineageweave_temporal_context_collection_exchange` onto spawned
//! `tepp-loopback` TCP. Stdout is a metric-free
//! `temporal_association_only` collection page. `tepp.scientific_acceptance.v1`
//! never appears. The CLI does not infer causality. Naruon is refused on this
//! `LineageWeave`-owned adapter. `NaruonLiveService` stays POST-only. Dedicated
//! binary so it does not collide with `tepp-temporal-context` (#414). This
//! module does not duplicate temporal-context collection GET (#449),
//! temporal-context CLI (#414), project-history collection CLI (#428), export
//! collection CLI (#444), interpretation-run collection CLI (#436), Leiden, or
//! GAP-010 Figma/export. Persistence remains GAP-003B.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::temporal_context_collection_http::{
    parse_temporal_context_collection_page_cursor, parse_temporal_context_collection_page_limit,
    refuse_metrics_on_temporal_context_collection_payload,
};
use crate::wire::require_nonempty;
use crate::{
    lineageweave_temporal_context_collection_exchange, AnalysisRunLiveService, ApiError,
    DEFAULT_PROJECT_HISTORY_BYTE_LIMIT, LINEAGEWEAVE_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT,
    NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse,
    TEMPORAL_CONTEXT_PATH, TemporalContextCollection,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    NARUON_LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

/// Supported operator verbs for the loopback temporal-context collection CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalContextCollectionCliVerb {
    /// `GET /v1/temporal-context`.
    List,
}

impl TemporalContextCollectionCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "list" => Ok(Self::List),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
        }
    }
}

/// One operator CLI invocation against a loopback collection GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalContextCollectionCliInvocation {
    /// CLI verb to execute.
    pub verb: TemporalContextCollectionCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed collection exchange.
    pub origin: String,
    /// Published modular consumer. Collection GET admits `lineageweave` only.
    pub consumer: String,
    /// Optional exclusive page cursor (`tepp-page-cursor`).
    pub page_cursor: Option<String>,
    /// Optional page limit (`tepp-page-limit`).
    pub page_limit: Option<String>,
    /// JSON body. Collection GET requires empty.
    pub body: String,
}

impl TemporalContextCollectionCliInvocation {
    /// Parse argv plus stdin body into a validated loopback collection invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished or naruon
    /// consumer, credential-shaped flags, hostile pagination, or a nonempty
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
        let verb = TemporalContextCollectionCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, naruon, nonempty-body, or out-of-bounds fields.
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
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_scientific_acceptance(&self.body)?;
        refuse_metrics_on_temporal_context_collection_payload(&self.body)?;
        refuse_event_pii(&self.body)?;
        parse_temporal_context_collection_page_limit(self.page_limit.as_deref())?;
        parse_temporal_context_collection_page_cursor(self.page_cursor.as_deref())?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    page_cursor: Option<String>,
    page_limit: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        page_cursor: None,
        page_limit: None,
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
            "page-cursor" => &mut flags.page_cursor,
            "page-limit" => &mut flags.page_limit,
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
    verb: TemporalContextCollectionCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<TemporalContextCollectionCliInvocation, ApiError> {
    let invocation = TemporalContextCollectionCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| LINEAGEWEAVE_CONSUMER_CODE.to_owned()),
        page_cursor: flags.page_cursor,
        page_limit: flags.page_limit,
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

/// Render a typed collection GET exchange as HTTP/1.1 for a loopback listener.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/temporal-context` with an empty body.
pub fn loopback_http1_from_temporal_context_collection_exchange(
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
    if path != TEMPORAL_CONTEXT_PATH {
        return Err(ApiError::InvalidWirePayload);
    }
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
            "tepp-page-cursor" => parse_temporal_context_collection_page_cursor(Some(value)).is_ok(),
            "tepp-page-limit" => parse_temporal_context_collection_page_limit(Some(value)).is_ok(),
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

/// Compose one HTTP/1.1 collection GET from the typed `LineageWeave` exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`TemporalContextCollectionCliInvocation::validate`].
pub fn compose_temporal_context_collection_cli_http(
    invocation: &TemporalContextCollectionCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lineageweave_temporal_context_collection_exchange(
        &invocation.origin,
        invocation.page_cursor.as_deref(),
        invocation.page_limit.as_deref(),
    )?;
    loopback_http1_from_temporal_context_collection_exchange(&exchange, &invocation.host)
}

/// Dispatch one collection CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_temporal_context_collection_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &TemporalContextCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_temporal_context_collection_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one collection CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_temporal_context_collection_cli(
    invocation: &TemporalContextCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_temporal_context_collection_cli_http(invocation)?;
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

/// Filter CLI stdout so collection pages never print scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// event labels, actor lists, or `tepp.scientific_acceptance.v1`.
pub fn render_temporal_context_collection_cli_stdout(
    invocation: &TemporalContextCollectionCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_temporal_context_collection_payload(&response.body)?;
    refuse_event_pii(&response.body)?;
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let collection = TemporalContextCollection::from_json(&response.body)?;
    let limit = parse_temporal_context_collection_page_limit(invocation.page_limit.as_deref())?;
    if collection.contexts.len() > limit {
        return Err(ApiError::InvalidWirePayload);
    }
    let cursor = parse_temporal_context_collection_page_cursor(invocation.page_cursor.as_deref())?;
    for index in 1..collection.contexts.len() {
        if collection.contexts[index - 1].idempotency_key
            >= collection.contexts[index].idempotency_key
        {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    if let Some(cursor) = cursor {
        for row in &collection.contexts {
            if row.idempotency_key <= cursor {
                return Err(ApiError::InvalidWirePayload);
            }
        }
    }
    if let Some(next_cursor) = &collection.next_cursor {
        match collection.contexts.last() {
            Some(row) if row.idempotency_key == *next_cursor => {}
            Some(_) | None => return Err(ApiError::InvalidWirePayload),
        }
    }
    collection.to_json()
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn refuse_event_pii(body: &str) -> Result<(), ApiError> {
    if body.contains("event_label")
        || body.contains("actor_references")
        || body.contains("timeline_events")
        || body.contains("evidence_text")
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
    let reason_phrase = match code {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        _ => return Err(ApiError::InvalidWirePayload),
    };
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

/// Read stdin leftover bytes on a non-terminal; collection GET admits empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read and
/// [`ApiError::LimitExceeded`] when leftover stdin exceeds the wire limit.
pub fn read_temporal_context_collection_cli_stdin(
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
        compose_temporal_context_collection_cli_http,
        loopback_http1_from_temporal_context_collection_exchange,
        read_temporal_context_collection_cli_stdin, TemporalContextCollectionCliInvocation,
        TemporalContextCollectionCliVerb,
    };
    use crate::lineageweave_temporal_context_collection_exchange;
    use crate::{ApiError, LINEAGEWEAVE_CONSUMER_CODE, NaruonHttpExchange};

    const ORIGIN: &str = "https://tepp.example.test";

    fn list_args() -> [&'static str; 7] {
        [
            "list",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ]
    }

    #[test]
    fn from_args_mints_list_and_refuses_fail_closed_inputs() {
        assert_eq!(
            TemporalContextCollectionCliVerb::parse("list").expect("list"),
            TemporalContextCollectionCliVerb::List
        );
        assert_eq!(TemporalContextCollectionCliVerb::List.as_str(), "list");
        assert_eq!(
            TemporalContextCollectionCliVerb::parse("cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        let list = TemporalContextCollectionCliInvocation::from_args(list_args(), "").expect("list");
        let http = compose_temporal_context_collection_cli_http(&list).expect("http");
        assert!(http.starts_with("GET /v1/temporal-context HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(
                ["list", "--host", "8.8.8.8:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(
                ["list", "--host", "localhost:18081", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    "http://tepp.example.test"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--authorization",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_naruon_body_and_non_get() {
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCollectionCliInvocation::from_args(list_args(), "{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let leftover =
            read_temporal_context_collection_cli_stdin(false, std::io::Cursor::new(b"leftover"))
                .expect("leftover");
        assert_eq!(leftover, "leftover");
        assert!(
            read_temporal_context_collection_cli_stdin(true, std::io::empty())
                .expect("tty")
                .is_empty()
        );
        let exchange =
            lineageweave_temporal_context_collection_exchange(ORIGIN, None, None).expect("ex");
        let posted = NaruonHttpExchange {
            method: "POST",
            target_url: exchange.target_url,
            headers: exchange.headers,
            body: exchange.body,
        };
        assert_eq!(
            loopback_http1_from_temporal_context_collection_exchange(&posted, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }
}
