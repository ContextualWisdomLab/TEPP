//! Operator loopback CLI for contextual-orchestrator interpretation-run collection GET.
//!
//! GAP-003A unique slice: operators run `tepp-interpretation-runs list` to mint
//! `contextual_orchestrator_interpretation_run_collection_exchange` onto spawned
//! `tepp-orchestrator-loopback` TCP. Stdout is a metric-free collection page
//! with `claim_status` `hypothetical` and `scientific_authority` false.
//! `tepp.scientific_acceptance.v1` never appears. The CLI does not infer
//! causality or call a model provider. Naruon and `LineageWeave` are refused
//! on this orchestrator-owned adapter. `NaruonLiveService` stays POST-only.
//! This module does not duplicate interpretation-run CLI (#425), collection
//! GET (#433), project-history collection CLI (#428), GET-by-id (#429),
//! retrieval CLI (#431), analysis-run collection CLI (#371), Leiden, or
//! GAP-010 Figma/export. Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::http::{header_is_credential, map_io_error};
use crate::interpretation_run_cli::CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE;
use crate::interpretation_run_collection_http::{
    parse_interpretation_run_collection_page_cursor,
    parse_interpretation_run_collection_page_limit,
    refuse_metrics_on_interpretation_run_collection_payload,
};
use crate::request::{
    require_nonempty, DEFAULT_INTERPRETATION_BYTE_LIMIT, HYPOTHETICAL_CLAIM_STATUS,
    INTERPRETATION_RUN_PATH,
};
use crate::{
    contextual_orchestrator_interpretation_run_collection_exchange, InterpretationRunCollection,
    InterpretationRunCollectionHttpExchange, OrchestratorLiveError, OrchestratorLiveResponse,
    OrchestratorLiveService, LIVE_HEADER_BYTE_LIMIT, LIVE_HEADER_COUNT_LIMIT,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const CLI_IO_TIMEOUT: Duration = Duration::from_secs(2);
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_INTERPRETATION_BYTE_LIMIT;

/// Supported operator verbs for the loopback interpretation-run collection CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationRunCollectionCliVerb {
    /// `GET /v1/interpretation-runs`.
    List,
}

impl InterpretationRunCollectionCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, OrchestratorLiveError> {
        match token {
            "list" => Ok(Self::List),
            _ => Err(OrchestratorLiveError::InvalidWirePayload),
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
pub struct InterpretationRunCollectionCliInvocation {
    /// CLI verb to execute.
    pub verb: InterpretationRunCollectionCliVerb,
    /// Loopback `host:port` of `tepp-orchestrator-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed collection exchange.
    pub origin: String,
    /// Published modular consumer. Collection GET admits
    /// `contextual-orchestrator` only.
    pub consumer: String,
    /// Optional exclusive page cursor (`tepp-page-cursor`).
    pub page_cursor: Option<String>,
    /// Optional page limit (`tepp-page-limit`).
    pub page_limit: Option<String>,
    /// JSON body. Collection GET requires empty.
    pub body: String,
}

impl InterpretationRunCollectionCliInvocation {
    /// Parse argv plus stdin body into a validated loopback collection invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished consumer,
    /// credential-shaped flags, hostile pagination, or a nonempty body.
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
        let verb = InterpretationRunCollectionCliVerb::parse(verb_token)?;
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
    /// nonempty-body, or out-of-bounds fields.
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
        if !self.body.is_empty() {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        refuse_scientific_acceptance(&self.body)?;
        refuse_metrics_on_interpretation_run_collection_payload(&self.body)?;
        parse_interpretation_run_collection_page_limit(self.page_limit.as_deref())?;
        parse_interpretation_run_collection_page_cursor(self.page_cursor.as_deref())?;
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

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, OrchestratorLiveError> {
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
            "page-cursor" => &mut flags.page_cursor,
            "page-limit" => &mut flags.page_limit,
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
    verb: InterpretationRunCollectionCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<InterpretationRunCollectionCliInvocation, OrchestratorLiveError> {
    let invocation = InterpretationRunCollectionCliInvocation {
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
        page_cursor: flags.page_cursor,
        page_limit: flags.page_limit,
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

/// Render a typed collection GET exchange as HTTP/1.1 for a loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
/// host or a credential-bearing header, and
/// [`OrchestratorLiveError::InvalidWirePayload`] when the exchange is not a
/// GET `/v1/interpretation-runs` with an empty body.
pub fn loopback_http1_from_interpretation_run_collection_exchange(
    exchange: &InterpretationRunCollectionHttpExchange,
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
    if path != INTERPRETATION_RUN_PATH {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(OrchestratorLiveError::AuthorizationDenied);
        }
        if name.eq_ignore_ascii_case("idempotency-key") {
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

/// Compose one HTTP/1.1 collection GET from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`InterpretationRunCollectionCliInvocation::validate`].
pub fn compose_interpretation_run_collection_cli_http(
    invocation: &InterpretationRunCollectionCliInvocation,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    let exchange = contextual_orchestrator_interpretation_run_collection_exchange(
        &invocation.origin,
        invocation.page_cursor.as_deref(),
        invocation.page_limit.as_deref(),
    )?;
    loopback_http1_from_interpretation_run_collection_exchange(&exchange, &invocation.host)
}

/// Dispatch one collection CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_interpretation_run_collection_cli(
    service: &mut OrchestratorLiveService,
    invocation: &InterpretationRunCollectionCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let request = compose_interpretation_run_collection_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one collection CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_interpretation_run_collection_cli(
    invocation: &InterpretationRunCollectionCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_interpretation_run_collection_cli_http(invocation)?;
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

/// Filter CLI stdout so collection pages never print scientific acceptance.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a receipt carries
/// metric keys, evidence, causal scores, or
/// `tepp.scientific_acceptance.v1`, or when rows violate the exclusive cursor
/// / sort / next-cursor contract.
pub fn render_interpretation_run_collection_cli_stdout(
    invocation: &InterpretationRunCollectionCliInvocation,
    response: &OrchestratorLiveResponse,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_interpretation_run_collection_payload(&response.body)?;
    if response.status_code != 200 {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let collection = InterpretationRunCollection::from_json(&response.body)?;
    let limit = parse_interpretation_run_collection_page_limit(invocation.page_limit.as_deref())?;
    if collection.items.len() > limit {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let cursor =
        parse_interpretation_run_collection_page_cursor(invocation.page_cursor.as_deref())?;
    for item in &collection.items {
        if item.claim_status != HYPOTHETICAL_CLAIM_STATUS || item.scientific_authority {
            return Err(OrchestratorLiveError::ScientificAuthorityRefused);
        }
    }
    for index in 1..collection.items.len() {
        if collection.items[index - 1].idempotency_key >= collection.items[index].idempotency_key {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
    }
    if let Some(cursor) = cursor {
        for row in &collection.items {
            if row.idempotency_key <= cursor {
                return Err(OrchestratorLiveError::InvalidWirePayload);
            }
        }
    }
    if let Some(next_cursor) = &collection.next_cursor {
        match collection.items.last() {
            Some(row) if row.idempotency_key == *next_cursor => {}
            Some(_) | None => return Err(OrchestratorLiveError::InvalidWirePayload),
        }
    }
    collection.to_json()
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

/// Read stdin leftover bytes on a non-terminal; collection GET admits empty.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when stdin cannot be
/// read and [`OrchestratorLiveError::LimitExceeded`] when leftover stdin
/// exceeds the interpretation-run wire limit.
pub fn read_interpretation_run_collection_cli_stdin(
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
        compose_interpretation_run_collection_cli_http, dispatch_interpretation_run_collection_cli,
        execute_interpretation_run_collection_cli,
        loopback_http1_from_interpretation_run_collection_exchange, parse_http_response,
        read_interpretation_run_collection_cli_stdin,
        render_interpretation_run_collection_cli_stdout, static_reason,
        InterpretationRunCollectionCliInvocation, InterpretationRunCollectionCliVerb,
        CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_SCHEMA,
    };
    use crate::{
        compose_interpretation_run_cli_http,
        contextual_orchestrator_interpretation_run_collection_exchange,
        InterpretationRunCollection, InterpretationRunCollectionHttpExchange,
        InterpretationRunRequest, OrchestrationMode, OrchestratorLiveError,
        OrchestratorLiveResponse, OrchestratorLiveService, HYPOTHETICAL_CLAIM_STATUS,
        INTERPRETATION_RUN_COLLECTION_MAX_LIMIT, INTERPRETATION_RUN_CONTRACT_VERSION,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn query_body(idem: &str) -> String {
        InterpretationRunRequest::new(
            INTERPRETATION_RUN_CONTRACT_VERSION,
            idem,
            "orch-tenant-demo",
            "tepp-snapshot-demo-001",
            "2026-08-01T00:00:00Z",
            OrchestrationMode::Direct,
            2048,
            vec!["span-001".into()],
            false,
        )
        .expect("request")
        .to_json()
        .expect("json")
    }

    fn create_http(idem: &str) -> String {
        let invocation = crate::InterpretationRunCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                ORIGIN,
                "--consumer",
                CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
            ],
            query_body(idem),
        )
        .expect("create");
        compose_interpretation_run_cli_http(&invocation).expect("post")
    }

    fn list_args() -> [&'static str; 7] {
        [
            "list",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
        ]
    }

    fn list_invocation() -> InterpretationRunCollectionCliInvocation {
        InterpretationRunCollectionCliInvocation::from_args(list_args(), "").expect("list")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            InterpretationRunCollectionCliVerb::parse("list").expect("list"),
            InterpretationRunCollectionCliVerb::List
        );
        assert_eq!(InterpretationRunCollectionCliVerb::List.as_str(), "list");
        assert_eq!(
            InterpretationRunCollectionCliVerb::parse("LIST"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            InterpretationRunCollectionCliVerb::parse("create"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            InterpretationRunCollectionCliVerb::parse("get"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_public_bind_localhost_http_and_credentials() {
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(Vec::<String>::new(), "")
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                ["list", "--host", "8.8.8.8:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                ["list", "--host", "0.0.0.0:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                ["list", "--host", "localhost:18082", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    "http://tepp.example.test"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
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
    fn from_args_refuses_unpublished_consumers_body_and_hostile_pagination() {
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "lineageweave"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(list_args(), "{}").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(list_args(), r#"{"rmse":1.0}"#)
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--page-limit",
                    "0"
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--page-limit",
                    &(INTERPRETATION_RUN_COLLECTION_MAX_LIMIT + 1).to_string()
                ],
                ""
            )
            .unwrap_err(),
            OrchestratorLiveError::LimitExceeded
        );
    }

    #[test]
    fn list_assembles_get_without_credentials_or_idempotency() {
        let list = InterpretationRunCollectionCliInvocation::from_args(
            ["list", "--host", "127.0.0.1:18082", "--origin", ORIGIN],
            "",
        )
        .expect("default consumer");
        assert_eq!(list.verb, InterpretationRunCollectionCliVerb::List);
        assert_eq!(list.consumer, CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE);
        let http = compose_interpretation_run_collection_cli_http(&list).expect("http");
        assert!(http.starts_with("GET /v1/interpretation-runs HTTP/1.1"));
        assert!(http.contains("tepp-consumer: contextual-orchestrator"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key"));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("/analysis-runs"));
        assert!(!http.contains("/v1/project-histories"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));

        let paged = InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                ORIGIN,
                "--page-cursor",
                "idem-a",
                "--page-limit",
                "8",
            ],
            "",
        )
        .expect("paged");
        let paged_http = compose_interpretation_run_collection_cli_http(&paged).expect("paged");
        assert!(paged_http.contains("tepp-page-cursor: idem-a"));
        assert!(paged_http.contains("tepp-page-limit: 8"));
    }

    #[test]
    fn loopback_http1_refuses_post_nonempty_and_foreign_paths() {
        let exchange =
            contextual_orchestrator_interpretation_run_collection_exchange(ORIGIN, None, None)
                .expect("exchange");
        loopback_http1_from_interpretation_run_collection_exchange(&exchange, "127.0.0.1:18082")
            .expect("ok");
        let mut posted = exchange.clone();
        posted.method = "POST";
        assert_eq!(
            loopback_http1_from_interpretation_run_collection_exchange(&posted, "127.0.0.1:18082")
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let mut nonempty = exchange.clone();
        nonempty.body = "{}".into();
        assert_eq!(
            loopback_http1_from_interpretation_run_collection_exchange(
                &nonempty,
                "127.0.0.1:18082"
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let foreign = InterpretationRunCollectionHttpExchange {
            method: "GET",
            target_url: "https://tepp.example.test/v1/analysis-runs".into(),
            headers: exchange.headers.clone(),
            body: String::new(),
        };
        assert_eq!(
            loopback_http1_from_interpretation_run_collection_exchange(&foreign, "127.0.0.1:18082")
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let mut credential = exchange;
        credential
            .headers
            .push(("authorization".into(), "Bearer secret".into()));
        assert_eq!(
            loopback_http1_from_interpretation_run_collection_exchange(
                &credential,
                "127.0.0.1:18082"
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
    }

    #[test]
    fn dispatch_lists_hypothetical_identities_without_metrics() {
        let mut service = OrchestratorLiveService::new();
        assert_eq!(
            service
                .handle_http_request(&create_http("idem-a"))
                .status_code,
            202
        );
        assert_eq!(
            service
                .handle_http_request(&create_http("idem-b"))
                .status_code,
            202
        );
        let listed = dispatch_interpretation_run_collection_cli(&mut service, &list_invocation())
            .expect("list");
        assert_eq!(listed.status_code, 200, "{}", listed.body);
        let stdout = render_interpretation_run_collection_cli_stdout(&list_invocation(), &listed)
            .expect("out");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("evidence_span_ids"));
        assert!(!stdout.contains("causal_score"));
        assert!(!stdout.contains("findings"));
        let page = InterpretationRunCollection::from_json(&stdout).expect("page");
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].idempotency_key, "idem-a");
        assert_eq!(page.items[0].claim_status, HYPOTHETICAL_CLAIM_STATUS);
        assert!(!page.items[0].scientific_authority);
        assert_eq!(page.items[1].idempotency_key, "idem-b");

        let paged = InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                ORIGIN,
                "--page-limit",
                "1",
            ],
            "",
        )
        .expect("limit 1");
        assert_eq!(
            render_interpretation_run_collection_cli_stdout(&paged, &listed),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        let first =
            dispatch_interpretation_run_collection_cli(&mut service, &paged).expect("page 1");
        let first_json =
            render_interpretation_run_collection_cli_stdout(&paged, &first).expect("page 1 out");
        let first_page = InterpretationRunCollection::from_json(&first_json).expect("first");
        assert_eq!(first_page.items.len(), 1);
        let cursor = first_page.next_cursor.expect("cursor");
        let second_inv = InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                ORIGIN,
                "--page-cursor",
                cursor.as_str(),
                "--page-limit",
                "1",
            ],
            "",
        )
        .expect("page 2");
        let second =
            dispatch_interpretation_run_collection_cli(&mut service, &second_inv).expect("page 2");
        let second_json = render_interpretation_run_collection_cli_stdout(&second_inv, &second)
            .expect("page 2 out");
        let second_page = InterpretationRunCollection::from_json(&second_json).expect("second");
        assert_eq!(second_page.items.len(), 1);
        assert_ne!(
            first_page.items[0].idempotency_key,
            second_page.items[0].idempotency_key
        );
    }

    #[test]
    fn render_refuses_metrics_schema_and_empty_bodies() {
        let list = list_invocation();
        assert_eq!(
            render_interpretation_run_collection_cli_stdout(
                &list,
                &OrchestratorLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            render_interpretation_run_collection_cli_stdout(
                &list,
                &OrchestratorLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"items":[],"rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            render_interpretation_run_collection_cli_stdout(
                &list,
                &OrchestratorLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let empty = render_interpretation_run_collection_cli_stdout(
            &list,
            &OrchestratorLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"items":[]}"#.into(),
            },
        )
        .expect("empty");
        assert!(empty.contains("\"items\":[]"));
    }

    #[test]
    fn execute_over_tcp_and_stdin_reader() {
        let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = list_invocation();
        invocation.host = addr.to_string();
        let response = execute_interpretation_run_collection_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200, "{}", response.body);
        let stdout =
            render_interpretation_run_collection_cli_stdout(&invocation, &response).expect("out");
        let page = InterpretationRunCollection::from_json(&stdout).expect("empty page");
        assert!(page.items.is_empty());
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_interpretation_run_collection_cli(&invocation).unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );

        let parsed =
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}").expect("parse");
        assert_eq!(parsed.status_code, 200);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(
            static_reason(500).unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );

        let empty =
            read_interpretation_run_collection_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped = read_interpretation_run_collection_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("pipe");
        assert!(piped.is_empty());
        let leftover =
            read_interpretation_run_collection_cli_stdin(false, std::io::Cursor::new(b"leftover"))
                .expect("leftover");
        assert_eq!(leftover, "leftover");
    }
}
