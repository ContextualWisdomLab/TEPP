//! Operator loopback CLI for analysis-run retry-lineage GET.
//!
//! GAP-003A retry-lineage CLI slice: operators run `tepp-retry-lineage retries`
//! to mint a typed naruon/`LineageWeave` retry-lineage GET onto spawned
//! `tepp-loopback` TCP. Stdout is a metric-free `200 OK` inspect
//! (`"retries": []` when the parent was never retried).
//! `tepp.scientific_acceptance.v1` never appears. Persistence remains
//! GAP-003B. This module does not add GET to `NaruonLiveService` beyond the
//! Naruon-only compatibility inspect already owned by ADR 0045.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN, AnalysisRunLiveService, AnalysisRunRetryLineage,
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NARUON_LIVE_IO_TIMEOUT,
    NaruonHttpExchange, NaruonLiveResponse, lineageweave_analysis_run_retry_lineage_exchange,
    naruon_analysis_run_retry_lineage_exchange, refuse_metrics_on_retry_lineage_payload,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback retry-lineage CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunRetryLineageCliVerb {
    /// `GET /v1/analysis-runs/{run_id}/retries`.
    Retries,
}

impl AnalysisRunRetryLineageCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "retries" => Ok(Self::Retries),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retries => "retries",
        }
    }
}

/// One operator CLI invocation against a loopback retry-lineage GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunRetryLineageCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunRetryLineageCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed retry-lineage exchange.
    pub origin: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Opaque server-assigned parent run identity to inspect.
    pub run_id: String,
    /// JSON body. Retry-lineage GET requires empty.
    pub body: String,
}

impl AnalysisRunRetryLineageCliInvocation {
    /// Parse argv plus stdin body into a validated loopback retry-lineage invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished consumer,
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
        let verb = AnalysisRunRetryLineageCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile inspect body.
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
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.run_id)?;
        if self.run_id.len() > ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_retry_lineage_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    run_id: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        run_id: None,
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
            "run-id" => &mut flags.run_id,
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
    verb: AnalysisRunRetryLineageCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunRetryLineageCliInvocation, ApiError> {
    let invocation = AnalysisRunRetryLineageCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
        run_id: flags.run_id.ok_or(ApiError::InvalidWirePayload)?,
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

fn retry_lineage_exchange(
    invocation: &AnalysisRunRetryLineageCliInvocation,
) -> Result<NaruonHttpExchange, ApiError> {
    if invocation.consumer == LINEAGEWEAVE_CONSUMER_CODE {
        lineageweave_analysis_run_retry_lineage_exchange(&invocation.origin, &invocation.run_id)
    } else if invocation.consumer == NARUON_CONSUMER_CODE {
        naruon_analysis_run_retry_lineage_exchange(&invocation.origin, &invocation.run_id)
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

/// Render a typed retry-lineage exchange as HTTP/1.1 for a bound loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/retries`.
pub fn loopback_http1_from_retry_lineage_exchange(
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
    if path.rsplit('/').next() != Some("retries") {
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

/// Compose one HTTP/1.1 retry-lineage GET from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunRetryLineageCliInvocation::validate`].
pub fn compose_analysis_run_retry_lineage_cli_http(
    invocation: &AnalysisRunRetryLineageCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = retry_lineage_exchange(invocation)?;
    loopback_http1_from_retry_lineage_exchange(&exchange, &invocation.host)
}

/// Dispatch one retry-lineage CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_retry_lineage_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunRetryLineageCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_retry_lineage_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one retry-lineage CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_retry_lineage_cli(
    invocation: &AnalysisRunRetryLineageCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_retry_lineage_cli_http(invocation)?;
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

/// Filter CLI stdout so retry-lineage inspect never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a metric-free
/// retry-lineage inspect for the requested `run_id`.
pub fn render_analysis_run_retry_lineage_cli_stdout(
    invocation: &AnalysisRunRetryLineageCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_retry_lineage_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let inspected = AnalysisRunRetryLineage::from_json(&response.body)?;
    if inspected.run_id != invocation.run_id {
        return Err(ApiError::InvalidWirePayload);
    }
    inspected.to_json()
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

/// Read stdin leftover bytes on a non-terminal; retry-lineage GET requires empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_retry_lineage_cli_stdin(
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
