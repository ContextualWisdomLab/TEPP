//! Operator loopback CLI for `LineageWeave` temporal-context POST.
//!
//! Operator-visible client of `POST /v1/temporal-context` on
//! `AnalysisRunLiveService` / `tepp-loopback` (ADR 0002 / ADR 0011). Operators
//! run `tepp-temporal-context query` without writing raw HTTP. Bodies stay
//! cutoff-safe and metric-free. `tepp.scientific_acceptance.v1` never appears.
//! The CLI does not infer causality. This module does not duplicate
//! analysis-run CLIs, export CLI, GET-by-id, Leiden, or GAP-010 Figma/export.
//! Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_LIVE_IO_TIMEOUT,
    NaruonLiveResponse, TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY, TEMPORAL_CONTEXT_PATH,
    TemporalContextRequest, TemporalContextResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const FORBIDDEN_CONTEXT_KEYS: [&str; 12] = [
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
];

/// Supported operator verbs for the loopback temporal-context CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalContextCliVerb {
    /// `POST /v1/temporal-context`.
    Query,
}

impl TemporalContextCliVerb {
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

/// One operator CLI invocation against a loopback temporal-context listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalContextCliInvocation {
    /// CLI verb to execute.
    pub verb: TemporalContextCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Validated cutoff-safe temporal-context request.
    pub request: TemporalContextRequest,
}

impl TemporalContextCliInvocation {
    /// Parse argv plus stdin JSON into a validated loopback query invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing flags, a
    /// non-loopback host, credential-shaped flags, unpublished consumers,
    /// metric keys, or an invalid temporal-context body.
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
        let verb = TemporalContextCliVerb::parse(verb_token)?;
        let host = parse_host(rest)?;
        let body = body.into();
        refuse_scientific_acceptance(&body)?;
        refuse_metrics(&body)?;
        let request = TemporalContextRequest::from_json(&body)?;
        let invocation = Self {
            verb,
            host,
            request,
        };
        invocation.validate()?;
        Ok(invocation)
    }

    /// Reject a non-loopback host or a non-`LineageWeave` consumer.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] when the consumer is not `LineageWeave`.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        if self.request.consumer_code != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn parse_host(rest: &[String]) -> Result<String, ApiError> {
    let mut host = None;
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
        if name != "host" {
            return Err(ApiError::InvalidWirePayload);
        }
        if host.is_some() || index + 1 >= rest.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        let value = rest[index + 1].as_str();
        require_nonempty(value)?;
        host = Some(value.to_owned());
        index += 2;
    }
    host.ok_or(ApiError::InvalidWirePayload)
}

fn require_loopback_host(host: &str) -> Result<SocketAddr, ApiError> {
    let addr: SocketAddr = host.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

/// Compose one HTTP/1.1 temporal-context POST for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`TemporalContextCliInvocation::validate`].
pub fn compose_temporal_context_cli_http(
    invocation: &TemporalContextCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let body = invocation.request.to_json()?;
    refuse_scientific_acceptance(&body)?;
    refuse_metrics(&body)?;
    Ok(format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: {}\r\n\r\n{body}",
        invocation.host,
        body.len()
    ))
}

/// Dispatch one temporal-context CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_temporal_context_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &TemporalContextCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_temporal_context_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one temporal-context CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_temporal_context_cli(
    invocation: &TemporalContextCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_temporal_context_cli_http(invocation)?;
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

/// Filter CLI stdout so temporal context never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when the body is empty, carries
/// metric or causal-score keys, or is not a cutoff-safe association response.
pub fn render_temporal_context_cli_stdout(
    invocation: &TemporalContextCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics(&response.body)?;
    if (200..300).contains(&response.status_code) {
        let parsed = TemporalContextResponse::from_json(&response.body)?;
        if parsed.claim_boundary != TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    Ok(response.body.clone())
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn refuse_metrics(payload: &str) -> Result<(), ApiError> {
    if payload.trim().is_empty() {
        return Ok(());
    }
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| ApiError::InvalidWirePayload)?;
    let Some(object) = value.as_object() else {
        return Err(ApiError::InvalidWirePayload);
    };
    if FORBIDDEN_CONTEXT_KEYS
        .iter()
        .any(|key| object.contains_key(*key))
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
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
    let reason_phrase = static_reason(code)?;
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
pub fn read_temporal_context_cli_stdin(
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
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        SCIENTIFIC_ACCEPTANCE_SCHEMA, TemporalContextCliInvocation, TemporalContextCliVerb,
        compose_temporal_context_cli_http, dispatch_temporal_context_cli,
        execute_temporal_context_cli, parse_http_response, read_temporal_context_cli_stdin,
        render_temporal_context_cli_stdout, static_reason,
    };
    use crate::{
        AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NaruonLiveResponse,
        TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY, TemporalContextEvent, TemporalContextRequest,
    };

    fn query_body() -> String {
        TemporalContextRequest {
            contract_version: 1,
            consumer_code: LINEAGEWEAVE_CONSUMER_CODE.into(),
            knowledge_cutoff: "2026-08-20T00:00:00Z".into(),
            subject_post_id: None,
            events: vec![TemporalContextEvent {
                event_id: "event-1".into(),
                source_post_id: "post-1".into(),
                event_type_code: "order_awarded".into(),
                event_label: "Order awarded".into(),
                event_time: "2026-08-01T09:00:00Z".into(),
                available_time: "2026-08-01T10:00:00Z".into(),
                project_reference: None,
                actor_references: vec!["actor-1".into()],
            }],
        }
        .to_json()
        .expect("json")
    }

    fn query_args() -> [&'static str; 3] {
        ["query", "--host", "127.0.0.1:18081"]
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            TemporalContextCliVerb::parse("query").expect("verb"),
            TemporalContextCliVerb::Query
        );
        assert_eq!(TemporalContextCliVerb::Query.as_str(), "query");
        assert_eq!(
            TemporalContextCliVerb::parse("QUERY"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            TemporalContextCliVerb::parse("cancel"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            TemporalContextCliVerb::parse("authorize"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_host_credentials_and_metrics() {
        assert_eq!(
            TemporalContextCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCliInvocation::from_args(["query"], query_body()).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCliInvocation::from_args(
                ["query", "--host", "8.8.8.8:80"],
                query_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            TemporalContextCliInvocation::from_args(
                [
                    "query",
                    "--host",
                    "127.0.0.1:18081",
                    "--authorization",
                    "secret"
                ],
                query_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            TemporalContextCliInvocation::from_args(query_args(), r#"{"rmse":1.0}"#).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            TemporalContextCliInvocation::from_args(query_args(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn compose_posts_temporal_context_without_credentials_or_metrics() {
        let invocation =
            TemporalContextCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let http = compose_temporal_context_cli_http(&invocation).expect("http");
        assert!(http.starts_with("POST /v1/temporal-context HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(!http.contains("idempotency-key"));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("/analysis-runs"));
        assert!(!http.contains("/v1/exports"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!http.contains("rmse"));
    }

    #[test]
    fn dispatch_returns_association_not_causal_without_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let invocation =
            TemporalContextCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let got = dispatch_temporal_context_cli(&mut service, &invocation).expect("dispatch");
        assert_eq!(got.status_code, 200);
        let stdout = render_temporal_context_cli_stdout(&invocation, &got).expect("stdout");
        assert!(stdout.contains(TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY));
        assert!(stdout.contains("candidate_not_causal") || stdout.contains("timeline_events"));
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("causal_score"));
    }

    #[test]
    fn render_refuses_metrics_schema_and_empty_bodies() {
        let invocation =
            TemporalContextCliInvocation::from_args(query_args(), query_body()).expect("inv");
        assert_eq!(
            render_temporal_context_cli_stdout(
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
        assert_eq!(
            render_temporal_context_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"claim_boundary":"association_not_causal","rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_temporal_context_cli_stdout(
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
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation =
            TemporalContextCliInvocation::from_args(query_args(), query_body()).expect("inv");
        invocation.host = addr.to_string();
        let response = execute_temporal_context_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_temporal_context_cli(&invocation).unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let parsed =
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}").expect("parse");
        assert_eq!(parsed.status_code, 200);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let empty = read_temporal_context_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped =
            read_temporal_context_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("piped");
        assert_eq!(piped, "{}");
    }
}
