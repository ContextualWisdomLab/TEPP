//! Operator loopback CLI for `LineageWeave` project-history collection GET.
//!
//! GAP-003A unique slice: operators run `tepp-project-histories list` to mint
//! `lineageweave_project_history_collection_exchange` onto spawned
//! `tepp-loopback` TCP. Stdout is a metric-free
//! `temporal_association_only` collection page. `tepp.scientific_acceptance.v1`
//! never appears. The CLI does not infer causality. Naruon is refused on this
//! LineageWeave-owned adapter. `NaruonLiveService` stays POST-only. This
//! module does not duplicate project-history POST CLI (#420), collection GET
//! (#424), temporal-context CLI (#414), export CLIs, analysis-run collection
//! CLI (#371), GET-by-id, Leiden, or GAP-010 Figma/export. Persistence remains
//! GAP-003B.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::project_history_collection_http::{
    parse_project_history_collection_page_cursor, parse_project_history_collection_page_limit,
    refuse_metrics_on_project_history_collection_payload,
};
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse, PROJECT_HISTORY_PATH,
    ProjectHistoryCollection, lineageweave_project_history_collection_exchange,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    NARUON_LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

/// Supported operator verbs for the loopback project-history collection CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectHistoryCollectionCliVerb {
    /// `GET /v1/project-histories`.
    List,
}

impl ProjectHistoryCollectionCliVerb {
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
pub struct ProjectHistoryCollectionCliInvocation {
    /// CLI verb to execute.
    pub verb: ProjectHistoryCollectionCliVerb,
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

impl ProjectHistoryCollectionCliInvocation {
    /// Parse argv plus stdin body into a validated loopback collection invocation.
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
        let verb = ProjectHistoryCollectionCliVerb::parse(verb_token)?;
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
        refuse_metrics_on_project_history_collection_payload(&self.body)?;
        parse_project_history_collection_page_limit(self.page_limit.as_deref())?;
        parse_project_history_collection_page_cursor(self.page_cursor.as_deref())?;
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
    verb: ProjectHistoryCollectionCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<ProjectHistoryCollectionCliInvocation, ApiError> {
    let invocation = ProjectHistoryCollectionCliInvocation {
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
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a GET `/v1/project-histories` with an empty body.
pub fn loopback_http1_from_project_history_collection_exchange(
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
    if path != PROJECT_HISTORY_PATH {
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
            "tepp-page-cursor" => parse_project_history_collection_page_cursor(Some(value)).is_ok(),
            "tepp-page-limit" => parse_project_history_collection_page_limit(Some(value)).is_ok(),
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
/// [`ProjectHistoryCollectionCliInvocation::validate`].
pub fn compose_project_history_collection_cli_http(
    invocation: &ProjectHistoryCollectionCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lineageweave_project_history_collection_exchange(
        &invocation.origin,
        invocation.page_cursor.as_deref(),
        invocation.page_limit.as_deref(),
    )?;
    loopback_http1_from_project_history_collection_exchange(&exchange, &invocation.host)
}

/// Dispatch one collection CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_project_history_collection_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ProjectHistoryCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_project_history_collection_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one collection CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_project_history_collection_cli(
    invocation: &ProjectHistoryCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_project_history_collection_cli_http(invocation)?;
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
/// evidence, causal scores, or `tepp.scientific_acceptance.v1`.
pub fn render_project_history_collection_cli_stdout(
    invocation: &ProjectHistoryCollectionCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_project_history_collection_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let collection = ProjectHistoryCollection::from_json(&response.body)?;
    collection.to_json()
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), ApiError> {
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

/// Read stdin leftover bytes on a non-terminal; collection GET refuses a body.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read and
/// [`ApiError::LimitExceeded`] when leftover stdin exceeds the project-history
/// wire limit.
pub fn read_project_history_collection_cli_stdin(
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
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        ProjectHistoryCollectionCliInvocation, ProjectHistoryCollectionCliVerb,
        SCIENTIFIC_ACCEPTANCE_SCHEMA, compose_project_history_collection_cli_http,
        dispatch_project_history_collection_cli, execute_project_history_collection_cli,
        loopback_http1_from_project_history_collection_exchange, parse_http_response,
        read_project_history_collection_cli_stdin, render_project_history_collection_cli_stdout,
        static_reason,
    };
    use crate::{
        AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
        NaruonHttpExchange, NaruonLiveResponse, PROJECT_HISTORY_COLLECTION_MAX_LIMIT,
        PROJECT_HISTORY_CONTRACT_VERSION, PROJECT_HISTORY_PATH, ProjectHistoryCollection,
        ProjectHistoryEvent, ProjectHistoryRequest,
        lineageweave_project_history_collection_exchange,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn sample_request(idempotency_key: &str, project_key: &str) -> ProjectHistoryRequest {
        ProjectHistoryRequest {
            contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "history-cli-tenant".into(),
            project_key: project_key.into(),
            project_name: "Project".into(),
            knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
            focus_event_id: "focus".into(),
            events: vec![ProjectHistoryEvent {
                event_id: "focus".into(),
                event_type_code: "voc_received".into(),
                event_title: "VOC".into(),
                occurred_at: "2026-08-19T09:00:00Z".into(),
                available_at: "2026-08-19T10:00:00Z".into(),
                source_post_id: "post".into(),
                evidence_text: "explicit evidence".into(),
                actor_ids: Vec::new(),
            }],
        }
    }

    fn project_history_post(request: &ProjectHistoryRequest) -> String {
        let body = request.to_json().expect("history json");
        format!(
            "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            request.idempotency_key,
            body.len()
        )
    }

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

    fn list_invocation() -> ProjectHistoryCollectionCliInvocation {
        ProjectHistoryCollectionCliInvocation::from_args(list_args(), "").expect("list")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            ProjectHistoryCollectionCliVerb::parse("list").expect("list"),
            ProjectHistoryCollectionCliVerb::List
        );
        assert_eq!(ProjectHistoryCollectionCliVerb::List.as_str(), "list");
        assert_eq!(
            ProjectHistoryCollectionCliVerb::parse("LIST"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionCliVerb::parse("query"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionCliVerb::parse("get"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCollectionCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_naruon_and_credential_flags() {
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(["list"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                ["list", "--host", "8.8.8.8:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                ["list", "--host", "0.0.0.0:80", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                ["list", "--host", "localhost:18081", "--origin", ORIGIN],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
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
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    NARUON_CONSUMER_CODE
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
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
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--pretty"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--run-id",
                    "tepp-run-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(list_args(), "{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(list_args(), r#"{"rmse":1.0}"#)
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--page-limit",
                    "0"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--page-limit",
                    &(PROJECT_HISTORY_COLLECTION_MAX_LIMIT + 1).to_string()
                ],
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            ProjectHistoryCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--page-cursor",
                    ""
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn list_assembles_default_consumer_and_optional_page_headers() {
        let list = ProjectHistoryCollectionCliInvocation::from_args(
            ["list", "--host", "127.0.0.1:18081", "--origin", ORIGIN],
            "",
        )
        .expect("default consumer");
        assert_eq!(list.verb, ProjectHistoryCollectionCliVerb::List);
        assert_eq!(list.consumer, LINEAGEWEAVE_CONSUMER_CODE);
        assert!(list.page_cursor.is_none());
        assert!(list.page_limit.is_none());
        let http = compose_project_history_collection_cli_http(&list).expect("http");
        assert!(http.starts_with("GET /v1/project-histories HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(!http.contains("idempotency-key"));
        assert!(!http.contains("tepp-page-cursor"));
        assert!(!http.contains("tepp-page-limit"));
        assert!(!http.contains("authorization"));
        assert!(http.contains("content-length: 0"));

        let paged = ProjectHistoryCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--page-cursor",
                "idem-1",
                "--page-limit",
                "8",
            ],
            "",
        )
        .expect("paged");
        let paged_http = compose_project_history_collection_cli_http(&paged).expect("paged http");
        assert!(paged_http.contains("tepp-page-cursor: idem-1"));
        assert!(paged_http.contains("tepp-page-limit: 8"));
        assert!(paged_http.contains("tepp-consumer: lineageweave"));
    }

    #[test]
    fn loopback_http1_refuses_post_naruon_and_nonempty_bodies() {
        let exchange =
            lineageweave_project_history_collection_exchange(ORIGIN, None, None).expect("exchange");
        let http =
            loopback_http1_from_project_history_collection_exchange(&exchange, "127.0.0.1:18081")
                .expect("http");
        assert!(http.starts_with("GET /v1/project-histories HTTP/1.1"));

        let mut posted = exchange.clone();
        posted.method = "POST";
        assert_eq!(
            loopback_http1_from_project_history_collection_exchange(&posted, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let mut nonempty = exchange.clone();
        nonempty.body = "{}".into();
        assert_eq!(
            loopback_http1_from_project_history_collection_exchange(&nonempty, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let mut naruon = exchange.clone();
        naruon.headers = vec![
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), NARUON_CONSUMER_CODE.into()),
            ("tepp-contract-version".into(), "1".into()),
        ];
        assert_eq!(
            loopback_http1_from_project_history_collection_exchange(&naruon, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let mut credential = exchange;
        credential
            .headers
            .push(("authorization".into(), "Bearer secret".into()));
        assert_eq!(
            loopback_http1_from_project_history_collection_exchange(&credential, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::AuthorizationDenied
        );

        assert_eq!(
            loopback_http1_from_project_history_collection_exchange(
                &NaruonHttpExchange {
                    method: "GET",
                    target_url: "https://tepp.example.test/v1/analysis-runs".into(),
                    headers: vec![
                        ("content-type".into(), "application/json".into()),
                        ("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()),
                        ("tepp-contract-version".into(), "1".into()),
                    ],
                    body: String::new(),
                },
                "127.0.0.1:18081"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn dispatch_lists_accepted_projections_without_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let first = sample_request("idem-a", "project-a");
        let second = sample_request("idem-b", "project-b");
        assert_eq!(
            service
                .handle_http_request(&project_history_post(&first))
                .status_code,
            200
        );
        assert_eq!(
            service
                .handle_http_request(&project_history_post(&second))
                .status_code,
            200
        );

        let listed = dispatch_project_history_collection_cli(&mut service, &list_invocation())
            .expect("list");
        assert_eq!(listed.status_code, 200);
        let stdout = render_project_history_collection_cli_stdout(&list_invocation(), &listed)
            .expect("stdout");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("evidence_text"));
        assert!(!stdout.contains("findings"));
        assert!(!stdout.contains("causal_score"));
        let page = ProjectHistoryCollection::from_json(&stdout).expect("page");
        assert_eq!(page.histories.len(), 2);
        assert_eq!(page.histories[0].idempotency_key, "idem-a");
        assert_eq!(page.histories[0].project_key, "project-a");
        assert_eq!(
            page.histories[0].inference_status,
            crate::PROJECT_HISTORY_COLLECTION_INFERENCE_STATUS
        );
        assert_eq!(page.histories[1].project_key, "project-b");

        let paged = ProjectHistoryCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--page-limit",
                "1",
            ],
            "",
        )
        .expect("limit 1");
        let first_page =
            dispatch_project_history_collection_cli(&mut service, &paged).expect("page 1");
        let first_json = render_project_history_collection_cli_stdout(&paged, &first_page)
            .expect("page 1 stdout");
        let first_collection = ProjectHistoryCollection::from_json(&first_json).expect("first");
        assert_eq!(first_collection.histories.len(), 1);
        let cursor = first_collection.next_cursor.expect("cursor");
        let second_page_invocation = ProjectHistoryCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
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
        let second_page =
            dispatch_project_history_collection_cli(&mut service, &second_page_invocation)
                .expect("page 2");
        let second_json =
            render_project_history_collection_cli_stdout(&second_page_invocation, &second_page)
                .expect("page 2 stdout");
        let second_collection = ProjectHistoryCollection::from_json(&second_json).expect("second");
        assert_eq!(second_collection.histories.len(), 1);
        assert_ne!(
            first_collection.histories[0].idempotency_key,
            second_collection.histories[0].idempotency_key
        );
    }

    #[test]
    fn render_refuses_metrics_scientific_acceptance_and_empty_bodies() {
        let list = list_invocation();
        assert_eq!(
            render_project_history_collection_cli_stdout(
                &list,
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
            render_project_history_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"histories":[],"rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_project_history_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        r#"{{"contract_version":1,"histories":[],"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_project_history_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        let empty_ok = render_project_history_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"histories":[]}"#.into(),
            },
        )
        .expect("empty");
        assert!(empty_ok.contains("\"histories\":[]"));
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = list_invocation();
        invocation.host = addr.to_string();
        let response = execute_project_history_collection_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        let stdout =
            render_project_history_collection_cli_stdout(&invocation, &response).expect("stdout");
        let page = ProjectHistoryCollection::from_json(&stdout).expect("empty page");
        assert!(page.histories.is_empty());
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_project_history_collection_cli(&invocation).unwrap_err(),
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
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(static_reason(400).expect("400"), "Bad Request");
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn stdin_reader_skips_terminal_and_reads_otherwise() {
        let empty = read_project_history_collection_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped = read_project_history_collection_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("empty pipe");
        assert!(piped.is_empty());
        let leftover =
            read_project_history_collection_cli_stdin(false, std::io::Cursor::new(b"leftover"))
                .expect("piped");
        assert_eq!(leftover, "leftover");
    }
}
