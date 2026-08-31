//! Operator loopback CLI for `LineageWeave` project-history POST.
//!
//! Operator-visible client of `POST /v1/project-histories` on
//! `AnalysisRunLiveService` / `tepp-loopback` (ADR 0021 / ADR 0011). Operators
//! run `tepp-project-history query` to mint
//! `lineageweave_project_history_exchange` onto spawned `tepp-loopback` TCP.
//! Stdout is a metric-free `temporal_association_only` projection.
//! `tepp.scientific_acceptance.v1` never appears. The CLI does not infer
//! causality. Naruon is refused on this LineageWeave-owned adapter.
//! `NaruonLiveService` stays POST-only for analysis-run and export. This
//! module does not duplicate temporal-context CLI, export CLIs, analysis-run
//! CLIs, GET-by-id, Leiden, or GAP-010 Figma/export. Persistence remains
//! GAP-003B.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT, NARUON_LIVE_HEADER_COUNT_LIMIT,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, PROJECT_HISTORY_CONTRACT_VERSION,
    PROJECT_HISTORY_PATH, ProjectHistoryHttpExchange, ProjectHistoryProjection,
    ProjectHistoryRequest, lineageweave_project_history_exchange,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const MAXIMUM_HTTP_RESPONSE_BYTES: usize =
    NARUON_LIVE_HEADER_BYTE_LIMIT + 4 + DEFAULT_PROJECT_HISTORY_BYTE_LIMIT;

/// Supported operator verbs for the loopback project-history CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectHistoryCliVerb {
    /// `POST /v1/project-histories`.
    Query,
}

impl ProjectHistoryCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "query" => Ok(Self::Query),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
        }
    }
}

/// One operator CLI invocation against a loopback project-history listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectHistoryCliInvocation {
    /// CLI verb to execute.
    pub verb: ProjectHistoryCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed project-history exchange.
    pub origin: String,
    /// Published modular consumer. Project-history admits `lineageweave` only.
    pub consumer: String,
    /// Validated cutoff-safe project-history request.
    pub request: ProjectHistoryRequest,
}

impl ProjectHistoryCliInvocation {
    /// Parse argv plus stdin JSON into a validated loopback query invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing flags, a
    /// non-loopback host, a non-`https` origin, an unpublished or naruon
    /// consumer, credential-shaped flags, metric keys, or an invalid body.
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
        let verb = ProjectHistoryCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        let body = body.into();
        refuse_scientific_acceptance(&body)?;
        refuse_metrics_on_project_history_cli_payload(&body)?;
        let request = ProjectHistoryRequest::from_json(&body)?;
        let invocation = Self {
            verb,
            host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
            origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
            consumer: flags
                .consumer
                .unwrap_or_else(|| LINEAGEWEAVE_CONSUMER_CODE.to_owned()),
            request,
        };
        invocation.validate()?;
        Ok(invocation)
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile origin.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] when the origin is not `https` or the
    /// consumer is not `lineageweave`.
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
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
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

fn require_loopback_host(host: &str) -> Result<SocketAddr, ApiError> {
    let addr: SocketAddr = host.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

/// Render a typed project-history exchange as HTTP/1.1 for a loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a POST `/v1/project-histories`.
pub fn loopback_http1_from_project_history_exchange(
    exchange: &ProjectHistoryHttpExchange,
    loopback_host: &str,
) -> Result<String, ApiError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "POST" {
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
    let body = ProjectHistoryRequest::from_json(&exchange.body)?;
    let mut seen = HashSet::with_capacity(exchange.headers.len());
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
            "content-type" => value == "application/json",
            "tepp-consumer" => value == LINEAGEWEAVE_CONSUMER_CODE,
            "tepp-contract-version" => value == &PROJECT_HISTORY_CONTRACT_VERSION.to_string(),
            "idempotency-key" => value == &body.idempotency_key,
            _ => false,
        };
        if !valid {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    if seen.len() != 4 {
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
    write!(
        request,
        "content-length: {}\r\n\r\n{}",
        exchange.body.len(),
        exchange.body
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 project-history POST from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ProjectHistoryCliInvocation::validate`].
pub fn compose_project_history_cli_http(
    invocation: &ProjectHistoryCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lineageweave_project_history_exchange(&invocation.origin, &invocation.request)?;
    loopback_http1_from_project_history_exchange(&exchange, &invocation.host)
}

/// Dispatch one project-history CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_project_history_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ProjectHistoryCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_project_history_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one project-history CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_project_history_cli(
    invocation: &ProjectHistoryCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_project_history_cli_http(invocation)?;
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

/// Filter CLI stdout so project-history never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when the body is empty, carries
/// metric or causal-score keys, or a success body is not a
/// `temporal_association_only` projection for the requested project.
pub fn render_project_history_cli_stdout(
    invocation: &ProjectHistoryCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_project_history_cli_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Err(ApiError::InvalidWirePayload);
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let projection = ProjectHistoryProjection::from_json(&response.body)?;
    if projection.project_key != invocation.request.project_key
        || projection.focus_event_id != invocation.request.focus_event_id
    {
        return Err(ApiError::InvalidWirePayload);
    }
    projection.to_json()
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

/// Refuse project-history JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted so missing stdin can fail later as invalid
/// wire. Non-object JSON fails closed.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a forbidden metric or causal
/// key is present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_project_history_cli_payload(payload: &str) -> Result<(), ApiError> {
    const FORBIDDEN: [&str; 13] = [
        "rmse",
        "rmse_standard_error",
        "mean_bias",
        "bias_standard_error",
        "interval_coverage",
        "coverage_wilson_lower",
        "coverage_wilson_upper",
        "temporal_order_accuracy",
        "se_gate_accepted",
        "scientific_acceptance",
        "causal_score",
        "causality",
        "terminal_result",
    ];
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(ApiError::InvalidWirePayload);
    }
    if contains_forbidden(&value, &FORBIDDEN) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

fn contains_forbidden(value: &serde_json::Value, forbidden: &[&str]) -> bool {
    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, nested)| {
            forbidden.contains(&key.as_str()) || contains_forbidden(nested, forbidden)
        }),
        serde_json::Value::Array(values) => values
            .iter()
            .any(|nested| contains_forbidden(nested, forbidden)),
        _ => false,
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

/// Read stdin leftover bytes on a non-terminal; query requires JSON.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_project_history_cli_stdin(
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
    use std::fmt::Write as _;

    use super::{
        ProjectHistoryCliInvocation, ProjectHistoryCliVerb, SCIENTIFIC_ACCEPTANCE_SCHEMA,
        compose_project_history_cli_http, dispatch_project_history_cli,
        execute_project_history_cli, loopback_http1_from_project_history_exchange,
        parse_http_response, read_project_history_cli_stdin, render_project_history_cli_stdout,
        static_reason,
    };
    use crate::{
        AnalysisRunLiveService, ApiError, DEFAULT_PROJECT_HISTORY_BYTE_LIMIT,
        LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NARUON_LIVE_HEADER_BYTE_LIMIT,
        NARUON_LIVE_HEADER_COUNT_LIMIT, NaruonLiveResponse, NaruonLiveService,
        PROJECT_HISTORY_CONTRACT_VERSION, ProjectHistoryEvent, ProjectHistoryHttpExchange,
        ProjectHistoryProjection, ProjectHistoryRequest, lineageweave_project_history_exchange,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn sample_event() -> ProjectHistoryEvent {
        ProjectHistoryEvent {
            event_id: "event-voc".into(),
            event_type_code: "voc_received".into(),
            event_title: "VOC received".into(),
            occurred_at: "2026-07-30T09:00:00Z".into(),
            available_at: "2026-07-30T09:00:00Z".into(),
            source_post_id: "post-voc".into(),
            evidence_text: "evidence for VOC received".into(),
            actor_ids: vec!["person-3".into()],
        }
    }

    fn query_body() -> String {
        ProjectHistoryRequest {
            contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
            idempotency_key: "lineageweave-project-cli-1".into(),
            tenant_workspace_id: "tenant-demo".into(),
            project_key: "project-acme".into(),
            project_name: "Acme renewal".into(),
            knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
            focus_event_id: "event-voc".into(),
            events: vec![
                ProjectHistoryEvent {
                    event_id: "event-award".into(),
                    event_type_code: "contract_awarded".into(),
                    event_title: "Contract award".into(),
                    occurred_at: "2022-03-11T09:00:00Z".into(),
                    available_at: "2022-03-11T09:00:00Z".into(),
                    source_post_id: "post-award".into(),
                    evidence_text: "evidence for Contract award".into(),
                    actor_ids: vec!["person-1".into()],
                },
                sample_event(),
            ],
        }
        .to_json()
        .expect("json")
    }

    fn query_args() -> [&'static str; 7] {
        [
            "query",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ]
    }

    #[test]
    fn verbs_and_from_args_fail_closed() {
        assert_eq!(
            ProjectHistoryCliVerb::parse("query").expect("verb"),
            ProjectHistoryCliVerb::Query
        );
        assert_eq!(ProjectHistoryCliVerb::Query.as_str(), "query");
        assert_eq!(
            ProjectHistoryCliVerb::parse("QUERY"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCliVerb::parse("authorize"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "8.8.8.8:80",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    LINEAGEWEAVE_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        for args in [
            vec!["query", "host", "value"],
            vec!["query", "--unknown", "value"],
            vec!["query", "--host"],
            vec!["query", "--host", "127.0.0.1:1", "--host", "127.0.0.1:2"],
        ] {
            assert_eq!(
                ProjectHistoryCliInvocation::from_args(args, query_body()).unwrap_err(),
                ApiError::InvalidWirePayload
            );
        }
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    "http://tepp.example.test",
                    "--consumer",
                    LINEAGEWEAVE_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "localhost:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    LINEAGEWEAVE_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--authorization",
                    "secret"
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_naruon_metrics_and_empty_body() {
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    NARUON_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "unpublished"
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(query_args(), r#"{"rmse":1.0}"#).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(query_args(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let injected =
            query_body().replace("lineageweave-project-cli-1", r"safe\r\nx-api-key: secret");
        assert_eq!(
            ProjectHistoryCliInvocation::from_args(query_args(), injected).unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn compose_is_typed_https_post_without_credentials() {
        let invocation =
            ProjectHistoryCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let http = compose_project_history_cli_http(&invocation).expect("http");
        assert!(http.starts_with("POST /v1/project-histories HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(http.contains("idempotency-key: lineageweave-project-cli-1"));
        assert!(!http.to_ascii_lowercase().contains("authorization"));
        assert!(!http.contains("/analysis-runs"));
        assert!(!http.contains("/v1/exports"));
        assert!(!http.contains("/v1/temporal-context"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!http.contains("rmse"));
    }

    #[test]
    fn dispatch_returns_association_only_and_naruon_live_stays_post_only() {
        let mut service = AnalysisRunLiveService::new();
        let invocation =
            ProjectHistoryCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let got = dispatch_project_history_cli(&mut service, &invocation).expect("dispatch");
        assert_eq!(got.status_code, 200, "{}", got.body);
        let stdout = render_project_history_cli_stdout(&invocation, &got).expect("stdout");
        let projection = ProjectHistoryProjection::from_json(&stdout).expect("projection");
        assert_eq!(projection.project_key, "project-acme");
        assert_eq!(projection.focus_event_id, "event-voc");
        assert_eq!(projection.inference_status, "temporal_association_only");
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("causal_score"));

        let http = compose_project_history_cli_http(&invocation).expect("http");
        let mut naruon = NaruonLiveService::new();
        assert_eq!(naruon.handle_http_request(&http).status_code, 400);
    }

    #[test]
    fn loopback_http1_refuses_non_post_and_wrong_path() {
        let invocation =
            ProjectHistoryCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let mut exchange =
            lineageweave_project_history_exchange(&invocation.origin, &invocation.request)
                .expect("exchange");
        exchange.method = "GET";
        assert_eq!(
            loopback_http1_from_project_history_exchange(&exchange, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let collection = ProjectHistoryHttpExchange {
            method: "POST",
            target_url: "https://tepp.example.test/v1/temporal-context".into(),
            headers: exchange.headers.clone(),
            body: exchange.body.clone(),
        };
        assert_eq!(
            loopback_http1_from_project_history_exchange(&collection, &invocation.host)
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let fresh = lineageweave_project_history_exchange(&invocation.origin, &invocation.request)
            .expect("exchange");
        let mut injected = fresh.clone();
        injected.headers[3].1 = "safe\r\nx-api-key: secret".into();
        assert_eq!(
            loopback_http1_from_project_history_exchange(&injected, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut extra = fresh;
        extra.headers.push(("x-extra".into(), "value".into()));
        assert_eq!(
            loopback_http1_from_project_history_exchange(&extra, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut duplicate =
            lineageweave_project_history_exchange(&invocation.origin, &invocation.request)
                .expect("exchange");
        duplicate
            .headers
            .push(("Content-Type".into(), "application/json".into()));
        assert_eq!(
            loopback_http1_from_project_history_exchange(&duplicate, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut missing =
            lineageweave_project_history_exchange(&invocation.origin, &invocation.request)
                .expect("exchange");
        missing.headers.pop();
        assert_eq!(
            loopback_http1_from_project_history_exchange(&missing, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut credential = missing.clone();
        credential
            .headers
            .push(("authorization".into(), "secret".into()));
        assert_eq!(
            loopback_http1_from_project_history_exchange(&credential, &invocation.host)
                .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        let mut malformed = missing;
        malformed.headers.push(("bad name".into(), "value".into()));
        assert_eq!(
            loopback_http1_from_project_history_exchange(&malformed, &invocation.host).unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn render_refuses_metrics_schema_and_identity_mismatch() {
        let invocation =
            ProjectHistoryCliInvocation::from_args(query_args(), query_body()).expect("inv");
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut mismatched =
            crate::project_history_projection(&invocation.request).expect("projection");
        mismatched.project_key = "other-project".into();
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: mismatched.to_json().expect("projection json"),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let valid = crate::project_history_projection(&invocation.request)
            .expect("projection")
            .to_json()
            .expect("projection json");
        let mut wrong_focus = invocation.clone();
        wrong_focus.request.focus_event_id = "event-award".into();
        assert_eq!(
            render_project_history_cli_stdout(
                &wrong_focus,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: valid,
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: r#"{"error":"pending"}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            super::refuse_metrics_on_project_history_cli_payload("[]").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: r#"{"error":"invalid"}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"inference_status":"temporal_association_only","rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_project_history_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn execute_over_tcp_and_stdin_reader() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation =
            ProjectHistoryCliInvocation::from_args(query_args(), query_body()).expect("inv");
        invocation.host = addr.to_string();
        let response = execute_project_history_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200, "{}", response.body);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_project_history_cli(&invocation).unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let parsed =
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}").expect("parse");
        assert_eq!(parsed.status_code, 200);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        for response in [
            "HTTP/1.0 200 OK\r\ncontent-length: 2\r\n\r\n{}".to_owned(),
            "HTTP/1.1 200 Wrong\r\ncontent-length: 2\r\n\r\n{}".to_owned(),
            "HTTP/1.1 200 OK\r\ncontent-length: 1\r\n\r\n{}".to_owned(),
        ] {
            assert_eq!(
                parse_http_response(response.as_bytes()).unwrap_err(),
                ApiError::InvalidWirePayload
            );
        }
        assert_eq!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\ncontent-length: 2\r\n\r\n{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        for response in [
            b"HTTP/1.1 200 OK\r\n: value\r\ncontent-length: 2\r\n\r\n{}".as_slice(),
            b"HTTP/1.1 200 OK\r\nx-value: bad\0value\r\ncontent-length: 2\r\n\r\n{}".as_slice(),
            b"HTTP/1.1 200 OK\r\nx-value: one\r\nX-Value: two\r\ncontent-length: 2\r\n\r\n{}"
                .as_slice(),
        ] {
            assert_eq!(
                parse_http_response(response).unwrap_err(),
                ApiError::InvalidWirePayload
            );
        }
        assert_eq!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\nx-value:\tvalue\r\ncontent-length: 2\r\n\r\n{}"
            )
            .expect("tab header")
            .status_code,
            200
        );
        let oversized = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n",
            DEFAULT_PROJECT_HISTORY_BYTE_LIMIT + 1
        );
        assert_eq!(
            parse_http_response(oversized.as_bytes()).unwrap_err(),
            ApiError::LimitExceeded
        );
        let long_header = format!(
            "HTTP/1.1 200 OK\r\nx-long: {}\r\ncontent-length: 2\r\n\r\n{{}}",
            "x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT)
        );
        assert_eq!(
            parse_http_response(long_header.as_bytes()).unwrap_err(),
            ApiError::LimitExceeded
        );
        let mut many_headers = String::new();
        for index in 0..NARUON_LIVE_HEADER_COUNT_LIMIT {
            write!(many_headers, "x-{index}: value\r\n").expect("header");
        }
        let crowded = format!("HTTP/1.1 200 OK\r\n{many_headers}content-length: 2\r\n\r\n{{}}");
        assert_eq!(
            parse_http_response(crowded.as_bytes()).unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        for (code, reason) in [
            (202, "Accepted"),
            (400, "Bad Request"),
            (403, "Forbidden"),
            (413, "Payload Too Large"),
            (422, "Unprocessable Entity"),
        ] {
            assert_eq!(static_reason(code).expect("known status"), reason);
        }
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let empty = read_project_history_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped =
            read_project_history_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("piped");
        assert_eq!(piped, "{}");
        let exact = vec![b'x'; DEFAULT_PROJECT_HISTORY_BYTE_LIMIT];
        assert_eq!(
            read_project_history_cli_stdin(false, std::io::Cursor::new(exact))
                .expect("bounded stdin")
                .len(),
            DEFAULT_PROJECT_HISTORY_BYTE_LIMIT
        );
        let excess = vec![b'x'; DEFAULT_PROJECT_HISTORY_BYTE_LIMIT + 1];
        assert_eq!(
            read_project_history_cli_stdin(false, std::io::Cursor::new(excess)).unwrap_err(),
            ApiError::LimitExceeded
        );
    }
}
