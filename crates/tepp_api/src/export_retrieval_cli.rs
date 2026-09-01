//! Operator loopback CLI for purpose-bound export retrieval GET.
//!
//! GAP-003A unique slice: operators run `tepp-export-get get` to mint a typed
//! naruon export-retrieval GET onto spawned `tepp-loopback` TCP. Stdout is a
//! metric-free `200 OK` identity (`export_id`, `artifact_id`, `decision_code`,
//! `purpose`, `idempotency_key`). `tepp.scientific_acceptance.v1` never
//! appears. `LineageWeave` is refused. `NaruonLiveService` stays POST-only.
//! Persistence remains GAP-003B. This module does not duplicate POST
//! authorize CLI (`tepp-exports`), GET-by-id, wait CLI, Leiden, or GAP-010
//! Figma/export.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_EXPORT_PATH, header_is_credential};
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, ApiError, EXPORT_RETRIEVAL_ID_MAX_LEN, ExportRetrieval,
    NARUON_CONSUMER_CODE, NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse,
    naruon_export_retrieval_exchange, refuse_metrics_on_export_retrieval_payload,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback export-retrieval CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportRetrievalCliVerb {
    /// `GET /v1/exports/{export_id}`.
    Get,
}

impl ExportRetrievalCliVerb {
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

/// One operator CLI invocation against a loopback export-retrieval GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRetrievalCliInvocation {
    /// CLI verb to execute.
    pub verb: ExportRetrievalCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed export-retrieval exchange.
    pub origin: String,
    /// Published modular consumer. Export retrieval is naruon-only.
    pub consumer: String,
    /// Opaque server-assigned export identity to retrieve.
    pub export_id: String,
    /// JSON body. Export-retrieval GET requires empty.
    pub body: String,
}

impl ExportRetrievalCliInvocation {
    /// Parse argv plus stdin body into a validated loopback retrieval invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, a non-naruon consumer,
    /// credential-shaped flags, hostile identities, or a nonempty body.
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
        let verb = ExportRetrievalCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile GET body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, oversized, nonempty-body, or metric-bearing fields.
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
        require_nonempty(&self.export_id)?;
        if self.export_id.len() > EXPORT_RETRIEVAL_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
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
    export_id: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        export_id: None,
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
            "export-id" => &mut flags.export_id,
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
    verb: ExportRetrievalCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<ExportRetrievalCliInvocation, ApiError> {
    let invocation = ExportRetrievalCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
        export_id: flags.export_id.ok_or(ApiError::InvalidWirePayload)?,
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

fn export_retrieval_exchange(
    invocation: &ExportRetrievalCliInvocation,
) -> Result<NaruonHttpExchange, ApiError> {
    if invocation.consumer == NARUON_CONSUMER_CODE {
        naruon_export_retrieval_exchange(&invocation.origin, &invocation.export_id)
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

/// Render a typed export-retrieval exchange as HTTP/1.1 for a bound loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/exports/{export_id}`.
pub fn loopback_http1_from_export_retrieval_exchange(
    exchange: &NaruonHttpExchange,
    loopback_host: &str,
) -> Result<String, ApiError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "GET" {
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
    let remainder = path
        .strip_prefix(NARUON_EXPORT_PATH)
        .ok_or(ApiError::InvalidWirePayload)?;
    let encoded = remainder
        .strip_prefix('/')
        .ok_or(ApiError::InvalidWirePayload)?;
    if encoded.is_empty() || encoded.contains('/') {
        return Err(ApiError::InvalidWirePayload);
    }
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
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
    write!(
        request,
        "content-length: {}\r\n\r\n{}",
        exchange.body.len(),
        exchange.body
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 export-retrieval GET from the typed naruon exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ExportRetrievalCliInvocation::validate`].
pub fn compose_export_retrieval_cli_http(
    invocation: &ExportRetrievalCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = export_retrieval_exchange(invocation)?;
    loopback_http1_from_export_retrieval_exchange(&exchange, &invocation.host)
}

/// Dispatch one export-retrieval CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_export_retrieval_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ExportRetrievalCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_export_retrieval_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one export-retrieval CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_export_retrieval_cli(
    invocation: &ExportRetrievalCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_export_retrieval_cli_http(invocation)?;
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

/// Filter CLI stdout so export retrieval never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a metric-free
/// export-retrieval identity for the requested `export_id`.
pub fn render_export_retrieval_cli_stdout(
    invocation: &ExportRetrievalCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_export_retrieval_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let retrieved = ExportRetrieval::from_json(&response.body)?;
    if retrieved.export_id != invocation.export_id {
        return Err(ApiError::InvalidWirePayload);
    }
    retrieved.to_json()
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

/// Read stdin leftover bytes on a non-terminal; export-retrieval GET requires empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_export_retrieval_cli_stdin(
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
