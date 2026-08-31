//! Operator loopback CLI for contextual-orchestrator interpretation-run POST.
//!
//! Operators run `tepp-interpretation-runs create` to mint a typed
//! `contextual_orchestrator_interpretation_run_exchange` onto spawned
//! `tepp-orchestrator-loopback` TCP. Stdout is a metric-free `202 Accepted`
//! body with `claim_status` `hypothetical` and `scientific_authority` false.
//! `tepp.scientific_acceptance.v1` never appears. The CLI does not call a
//! model provider, infer causality, or promote scientific authority.
//! Naruon and `LineageWeave` are refused on this orchestrator-owned adapter.
//! `NaruonLiveService` stays POST-only for analysis-run and export.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::http::{header_is_credential, map_io_error};
use crate::request::{host_implies_table_access, require_nonempty};
use crate::{
    HYPOTHETICAL_CLAIM_STATUS, INTERPRETATION_RUN_PATH, InterpretationRunAccepted,
    InterpretationRunRequest, OrchestratorLiveError, OrchestratorLiveResponse,
    OrchestratorLiveService,
};

/// Published modular consumer for the interpretation-run adapter.
pub const CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE: &str = "contextual-orchestrator";

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const CLI_IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Supported operator verbs for the loopback interpretation-run CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterpretationRunCliVerb {
    /// `POST /v1/interpretation-runs`.
    Create,
}

impl InterpretationRunCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, OrchestratorLiveError> {
        match token {
            "create" => Ok(Self::Create),
            _ => Err(OrchestratorLiveError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
        }
    }
}

/// Typed HTTPS interpretation-run exchange before loopback Host rewrite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunHttpExchange {
    /// HTTP method, always `POST` for a valid exchange.
    pub method: &'static str,
    /// Absolute HTTPS target ending in [`INTERPRETATION_RUN_PATH`].
    pub target_url: String,
    /// Credential-free consumer, version, content, and idempotency headers.
    pub headers: Vec<(String, String)>,
    /// Validated JSON request body.
    pub body: String,
}

/// One operator CLI invocation against a loopback interpretation-run listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpretationRunCliInvocation {
    /// CLI verb to execute.
    pub verb: InterpretationRunCliVerb,
    /// Loopback `host:port` of `tepp-orchestrator-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed exchange.
    pub origin: String,
    /// Published modular consumer. Interpretation-run admits
    /// `contextual-orchestrator` only.
    pub consumer: String,
    /// Validated hypothetical interpretation-run request.
    pub request: InterpretationRunRequest,
}

impl InterpretationRunCliInvocation {
    /// Parse argv plus stdin JSON into a validated loopback create invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing flags, a
    /// non-loopback host, a non-`https` origin, an unpublished consumer,
    /// credential-shaped flags, metric keys, or an invalid body.
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
        let verb = InterpretationRunCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        let body = body.into();
        refuse_scientific_acceptance(&body)?;
        refuse_metrics_on_interpretation_run_cli_payload(&body)?;
        let request = InterpretationRunRequest::from_json(&body)?;
        let invocation = Self {
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
            request,
        };
        invocation.validate()?;
        Ok(invocation)
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile origin.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
    /// host and [`OrchestratorLiveError::InvalidWirePayload`] when the origin is
    /// not `https` or the consumer is not `contextual-orchestrator`.
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
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, OrchestratorLiveError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
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

fn compose_https_target(origin: &str, path: &str) -> Result<String, OrchestratorLiveError> {
    require_nonempty(origin)?;
    if !origin.starts_with("https://") || origin.ends_with('/') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let rest = origin
        .strip_prefix("https://")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if rest.contains('@') || rest.contains('?') || rest.contains('#') || rest.contains('\\') {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if host_implies_table_access(rest) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(format!("{origin}{path}"))
}

/// Build a credential-free contextual-orchestrator interpretation-run exchange.
///
/// # Errors
///
/// Returns a fail-closed origin, request, or scientific-authority error.
pub fn contextual_orchestrator_interpretation_run_exchange(
    origin: &str,
    request: &InterpretationRunRequest,
) -> Result<InterpretationRunHttpExchange, OrchestratorLiveError> {
    let target_url = compose_https_target(origin, INTERPRETATION_RUN_PATH)?;
    let body = request.to_json()?;
    Ok(InterpretationRunHttpExchange {
        method: "POST",
        target_url,
        headers: vec![
            ("content-type".into(), "application/json".into()),
            (
                "tepp-consumer".into(),
                CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE.into(),
            ),
            ("tepp-contract-version".into(), "1".into()),
            (
                "idempotency-key".into(),
                request.idempotency_key().to_owned(),
            ),
        ],
        body,
    })
}

/// Render a typed interpretation-run exchange as HTTP/1.1 for a loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::AuthorizationDenied`] for a non-loopback
/// host or a credential-bearing header, and
/// [`OrchestratorLiveError::InvalidWirePayload`] when the exchange is not a
/// POST `/v1/interpretation-runs`.
pub fn loopback_http1_from_interpretation_run_exchange(
    exchange: &InterpretationRunHttpExchange,
    loopback_host: &str,
) -> Result<String, OrchestratorLiveError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "POST" {
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
    write!(
        request,
        "content-length: {}\r\n\r\n{}",
        exchange.body.len(),
        exchange.body
    )
    .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 interpretation-run POST from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`InterpretationRunCliInvocation::validate`].
pub fn compose_interpretation_run_cli_http(
    invocation: &InterpretationRunCliInvocation,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    let exchange = contextual_orchestrator_interpretation_run_exchange(
        &invocation.origin,
        &invocation.request,
    )?;
    loopback_http1_from_interpretation_run_exchange(&exchange, &invocation.host)
}

/// Dispatch one interpretation-run CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_interpretation_run_cli(
    service: &mut OrchestratorLiveService,
    invocation: &InterpretationRunCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let request = compose_interpretation_run_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one interpretation-run CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_interpretation_run_cli(
    invocation: &InterpretationRunCliInvocation,
) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_interpretation_run_cli_http(invocation)?;
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
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|error| map_io_error(&error))?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so interpretation-run never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when the body is empty,
/// carries metric keys, or a success body is not a hypothetical accepted run
/// for the requested idempotency key.
pub fn render_interpretation_run_cli_stdout(
    invocation: &InterpretationRunCliInvocation,
    response: &OrchestratorLiveResponse,
) -> Result<String, OrchestratorLiveError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics_on_interpretation_run_cli_payload(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    if response.status_code != 202 {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    let accepted = InterpretationRunAccepted::from_json(&response.body)?;
    if accepted.idempotency_key() != invocation.request.idempotency_key()
        || accepted.claim_status() != HYPOTHETICAL_CLAIM_STATUS
        || accepted.scientific_authority()
    {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    accepted.to_json()
}

fn refuse_scientific_acceptance(body: &str) -> Result<(), OrchestratorLiveError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(OrchestratorLiveError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

/// Refuse interpretation-run JSON that already carries scientific-metric keys.
///
/// Empty payloads are admitted so missing stdin can fail later as invalid
/// wire. Non-object JSON fails closed.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when a forbidden
/// metric or causal key is present or the payload is a non-empty non-object.
pub fn refuse_metrics_on_interpretation_run_cli_payload(
    payload: &str,
) -> Result<(), OrchestratorLiveError> {
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
        serde_json::from_str(payload).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    if !value.is_object() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if contains_forbidden(&value, &FORBIDDEN) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
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

fn parse_http_response(bytes: &[u8]) -> Result<OrchestratorLiveResponse, OrchestratorLiveError> {
    let text = std::str::from_utf8(bytes).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    let (header_block, body) = text
        .split_once("\r\n\r\n")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
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
    for line in lines {
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

/// Read stdin leftover bytes on a non-terminal; create requires JSON.
///
/// # Errors
///
/// Returns [`OrchestratorLiveError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_interpretation_run_cli_stdin(
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, OrchestratorLiveError> {
    if stdin_is_terminal {
        Ok(String::new())
    } else {
        let mut body = String::new();
        stdin
            .read_to_string(&mut body)
            .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
        Ok(body)
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, InterpretationRunCliInvocation,
        InterpretationRunCliVerb, InterpretationRunHttpExchange, SCIENTIFIC_ACCEPTANCE_SCHEMA,
        compose_interpretation_run_cli_http, contextual_orchestrator_interpretation_run_exchange,
        dispatch_interpretation_run_cli, execute_interpretation_run_cli,
        loopback_http1_from_interpretation_run_exchange, parse_http_response,
        read_interpretation_run_cli_stdin, render_interpretation_run_cli_stdout, static_reason,
    };
    use crate::{
        HYPOTHETICAL_CLAIM_STATUS, INTERPRETATION_RUN_CONTRACT_VERSION, InterpretationRunAccepted,
        InterpretationRunRequest, OrchestrationMode, OrchestratorLiveError,
        OrchestratorLiveResponse, OrchestratorLiveService,
    };

    const ORIGIN: &str = "https://tepp.example.test";

    fn query_body() -> String {
        InterpretationRunRequest::new(
            INTERPRETATION_RUN_CONTRACT_VERSION,
            "orch-cli-idem-1",
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

    fn query_args() -> [&'static str; 7] {
        [
            "create",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
        ]
    }

    #[test]
    fn verbs_and_from_args_fail_closed() {
        assert_eq!(
            InterpretationRunCliVerb::parse("create").expect("verb"),
            InterpretationRunCliVerb::Create
        );
        assert_eq!(InterpretationRunCliVerb::Create.as_str(), "create");
        assert_eq!(
            InterpretationRunCliVerb::parse("CREATE"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            InterpretationRunCliVerb::parse("query"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "8.8.8.8:80",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    "http://tepp.example.test",
                    "--consumer",
                    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "localhost:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--authorization",
                    "secret"
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_naruon_metrics_and_empty_body() {
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon"
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(
                [
                    "create",
                    "--host",
                    "127.0.0.1:18082",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "lineageweave"
                ],
                query_body()
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(query_args(), r#"{"rmse":1.0}"#).unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            InterpretationRunCliInvocation::from_args(query_args(), "").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
    }

    #[test]
    fn compose_is_typed_https_post_without_credentials() {
        let invocation =
            InterpretationRunCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let http = compose_interpretation_run_cli_http(&invocation).expect("http");
        assert!(http.starts_with("POST /v1/interpretation-runs HTTP/1.1"));
        assert!(http.contains("tepp-consumer: contextual-orchestrator"));
        assert!(http.contains("idempotency-key: orch-cli-idem-1"));
        assert!(!http.to_ascii_lowercase().contains("authorization"));
        assert!(!http.contains("/analysis-runs"));
        assert!(!http.contains("/v1/exports"));
        assert!(!http.contains("/v1/project-histories"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!http.contains("rmse"));
    }

    #[test]
    fn dispatch_returns_hypothetical_accepted_run() {
        let mut service = OrchestratorLiveService::new();
        let invocation =
            InterpretationRunCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let got = dispatch_interpretation_run_cli(&mut service, &invocation).expect("dispatch");
        assert_eq!(got.status_code, 202, "{}", got.body);
        let stdout = render_interpretation_run_cli_stdout(&invocation, &got).expect("stdout");
        let accepted = InterpretationRunAccepted::from_json(&stdout).expect("accepted");
        assert_eq!(accepted.idempotency_key(), "orch-cli-idem-1");
        assert_eq!(accepted.claim_status(), HYPOTHETICAL_CLAIM_STATUS);
        assert!(!accepted.scientific_authority());
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("causal_score"));
    }

    #[test]
    fn loopback_http1_refuses_non_post_and_wrong_path() {
        let invocation =
            InterpretationRunCliInvocation::from_args(query_args(), query_body()).expect("inv");
        let mut exchange = contextual_orchestrator_interpretation_run_exchange(
            &invocation.origin,
            &invocation.request,
        )
        .expect("exchange");
        exchange.method = "GET";
        assert_eq!(
            loopback_http1_from_interpretation_run_exchange(&exchange, &invocation.host)
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let collection = InterpretationRunHttpExchange {
            method: "POST",
            target_url: "https://tepp.example.test/v1/analysis-runs".into(),
            headers: exchange.headers.clone(),
            body: exchange.body.clone(),
        };
        assert_eq!(
            loopback_http1_from_interpretation_run_exchange(&collection, &invocation.host)
                .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
    }

    #[test]
    fn render_refuses_metrics_schema_and_identity_mismatch() {
        let invocation =
            InterpretationRunCliInvocation::from_args(query_args(), query_body()).expect("inv");
        assert_eq!(
            render_interpretation_run_cli_stdout(
                &invocation,
                &OrchestratorLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            render_interpretation_run_cli_stdout(
                &invocation,
                &OrchestratorLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: r#"{"claim_status":"hypothetical","rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            render_interpretation_run_cli_stdout(
                &invocation,
                &OrchestratorLiveResponse {
                    status_code: 202,
                    reason_phrase: "Accepted",
                    body: format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#),
                }
            )
            .unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
    }

    #[test]
    fn execute_over_tcp_and_stdin_reader() {
        let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation =
            InterpretationRunCliInvocation::from_args(query_args(), query_body()).expect("inv");
        invocation.host = addr.to_string();
        let response = execute_interpretation_run_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 202, "{}", response.body);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_interpretation_run_cli(&invocation).unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );

        let parsed = parse_http_response(b"HTTP/1.1 202 Accepted\r\ncontent-length: 2\r\n\r\n{}")
            .expect("parse");
        assert_eq!(parsed.status_code, 202);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(static_reason(202).expect("202"), "Accepted");
        assert_eq!(
            static_reason(500).unwrap_err(),
            OrchestratorLiveError::InvalidWirePayload
        );
        let empty = read_interpretation_run_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped =
            read_interpretation_run_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("piped");
        assert_eq!(piped, "{}");
    }
}
