//! Operator loopback CLI for naruon export collection GET.
//!
//! GAP-003A unique slice: operators run `tepp-export-list list` to mint
//! `naruon_export_collection_exchange` onto spawned `tepp-loopback` TCP.
//! Stdout is one metric-free collection page of authorized identities.
//! `tepp.scientific_acceptance.v1` never appears. `LineageWeave` is refused.
//! `NaruonLiveService` stays POST-only. This module does not duplicate export
//! collection GET (#443), export-retrieval CLI (#417), export retrieval GET
//! (#411), export-authorize CLI (#410), interpretation-run collection CLI
//! (#436), Leiden, or GAP-010 Figma/export. Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    naruon_export_collection_exchange, parse_export_collection_page_cursor,
    parse_export_collection_page_limit, refuse_metrics_on_export_retrieval_payload,
    AnalysisRunLiveService, ApiError, ExportCollection, NARUON_CONSUMER_CODE,
    NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback export-collection CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportCollectionCliVerb {
    /// `GET /v1/exports`.
    List,
}

impl ExportCollectionCliVerb {
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

/// One operator CLI invocation against a loopback export-collection GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportCollectionCliInvocation {
    /// CLI verb to execute.
    pub verb: ExportCollectionCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed collection exchange.
    pub origin: String,
    /// Published modular consumer. Collection GET is naruon-only.
    pub consumer: String,
    /// Optional page size for `tepp-page-limit`.
    pub page_limit: Option<String>,
    /// Optional exclusive `export_id` cursor for `tepp-page-cursor`.
    pub page_cursor: Option<String>,
    /// JSON body. Collection GET requires empty.
    pub body: String,
}

impl ExportCollectionCliInvocation {
    /// Parse argv plus stdin body into a validated loopback collection invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, a non-naruon consumer,
    /// credential-shaped flags, hostile pagination, or a nonempty body.
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
        let verb = ExportCollectionCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, nonempty-body, or hostile pagination fields.
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
        parse_export_collection_page_limit(self.page_limit.as_deref())?;
        parse_export_collection_page_cursor(self.page_cursor.as_deref())?;
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_export_retrieval_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    page_limit: Option<String>,
    page_cursor: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        page_limit: None,
        page_cursor: None,
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
            "page-limit" => &mut flags.page_limit,
            "page-cursor" => &mut flags.page_cursor,
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
    verb: ExportCollectionCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<ExportCollectionCliInvocation, ApiError> {
    let invocation = ExportCollectionCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
        page_limit: flags.page_limit,
        page_cursor: flags.page_cursor,
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

/// Render a typed export-collection exchange as HTTP/1.1 for a loopback listener.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/exports` with an empty body.
pub fn loopback_http1_from_export_collection_exchange(
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
    if !crate::is_export_collection_path(path) {
        return Err(ApiError::InvalidWirePayload);
    }
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        if name.eq_ignore_ascii_case("idempotency-key") {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    let mut request = String::new();
    write!(
        request,
        "{} {path} HTTP/1.1\r\nHost: {host}\r\n",
        exchange.method
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    for (name, value) in &exchange.headers {
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        write!(request, "{name}: {value}\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    }
    write!(request, "content-length: 0\r\n\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 collection GET from the typed naruon exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ExportCollectionCliInvocation::validate`].
pub fn compose_export_collection_cli_http(
    invocation: &ExportCollectionCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let mut exchange = naruon_export_collection_exchange(&invocation.origin)?;
    if let Some(limit) = invocation.page_limit.as_deref() {
        exchange
            .headers
            .push(("tepp-page-limit".into(), limit.to_owned()));
    }
    if let Some(cursor) = invocation.page_cursor.as_deref() {
        exchange
            .headers
            .push(("tepp-page-cursor".into(), cursor.to_owned()));
    }
    loopback_http1_from_export_collection_exchange(&exchange, &invocation.host)
}

/// Dispatch one collection CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_export_collection_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ExportCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_export_collection_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one collection CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_export_collection_cli(
    invocation: &ExportCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_export_collection_cli_http(invocation)?;
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

/// Filter CLI stdout so collection never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a metric-free
/// collection page.
pub fn render_export_collection_cli_stdout(
    invocation: &ExportCollectionCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_export_retrieval_payload(&response.body)?;
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let parsed: ExportCollection =
        serde_json::from_str(&response.body).map_err(|_| ApiError::InvalidWirePayload)?;
    let collection = ExportCollection::new(parsed.items, parsed.next_cursor)?;
    collection.to_json()
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
    let reason_phrase = match code {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        _ => return Err(ApiError::InvalidWirePayload),
    };
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

/// Read stdin leftover bytes on a non-terminal; collection GET admits empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_export_collection_cli_stdin(
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
mod tests {
    use super::{
        compose_export_collection_cli_http, loopback_http1_from_export_collection_exchange,
        read_export_collection_cli_stdin, ExportCollectionCliInvocation, ExportCollectionCliVerb,
    };
    use crate::{
        naruon_export_collection_exchange, ApiError, NARUON_CONSUMER_CODE, NaruonHttpExchange,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn list_args() -> [&'static str; 7] {
        [
            "list",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            NARUON_CONSUMER_CODE,
        ]
    }

    #[test]
    fn from_args_mints_list_and_refuses_fail_closed_inputs() {
        assert_eq!(
            ExportCollectionCliVerb::parse("list").expect("list"),
            ExportCollectionCliVerb::List
        );
        assert_eq!(ExportCollectionCliVerb::List.as_str(), "list");
        assert_eq!(
            ExportCollectionCliVerb::parse("get"),
            Err(ApiError::InvalidWirePayload)
        );
        let list = ExportCollectionCliInvocation::from_args(list_args(), "").expect("list");
        assert_eq!(list.verb, ExportCollectionCliVerb::List);
        let http = compose_export_collection_cli_http(&list).expect("http");
        assert!(http.starts_with("GET /v1/exports HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key:"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            ExportCollectionCliInvocation::from_args(
                ["list", "--host", "8.8.8.8:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ExportCollectionCliInvocation::from_args(
                ["list", "--host", "localhost:18081", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportCollectionCliInvocation::from_args(
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
            ExportCollectionCliInvocation::from_args(
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
    fn from_args_refuses_unpublished_body_export_id_and_non_get() {
        assert_eq!(
            ExportCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "lineageweave"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportCollectionCliInvocation::from_args(list_args(), "{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--export-id",
                    "export-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let leftover = read_export_collection_cli_stdin(false, std::io::Cursor::new(b"leftover"))
            .expect("leftover");
        assert_eq!(leftover, "leftover");
        assert!(read_export_collection_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty());
        let exchange = naruon_export_collection_exchange(ORIGIN).expect("exchange");
        let posted = NaruonHttpExchange {
            method: "POST",
            target_url: exchange.target_url,
            headers: exchange.headers,
            body: exchange.body,
        };
        assert_eq!(
            loopback_http1_from_export_collection_exchange(&posted, "127.0.0.1:18081").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }
}
