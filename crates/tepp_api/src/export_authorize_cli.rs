//! Operator loopback CLI for purpose-bound export authorization POST.
//!
//! Operator-visible client of `POST /v1/exports` on `NaruonLiveService`
//! (ADR 0009 / ADR 0011). Operators run `tepp-exports authorize` against
//! `tepp-naruon-live` without writing raw HTTP. `tepp-loopback` is
//! `AnalysisRunLiveService` and does not serve `/v1/exports`.
//! `tepp.scientific_acceptance.v1` never appears. This module does not
//! duplicate analysis-run CLIs, GET-by-id, Leiden, or GAP-010 Figma/export.
//! Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_EXPORT_PATH, header_is_credential};
use crate::wire::{from_json, require_nonempty, to_json};
use crate::{
    AnalyticalPurpose, ApiError, ExportAuthorizationRequest, NARUON_CONSUMER_CODE,
    NARUON_LIVE_IO_TIMEOUT, NaruonLiveResponse, NaruonLiveService,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";
const FORBIDDEN_EXPORT_KEYS: [&str; 11] = [
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
    "terminal_result",
];

/// Supported operator verbs for the loopback export CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportAuthorizeCliVerb {
    /// `POST /v1/exports`.
    Authorize,
}

impl ExportAuthorizeCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "authorize" => Ok(Self::Authorize),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authorize => "authorize",
        }
    }
}

/// One operator CLI invocation against a loopback export-authorize listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportAuthorizeCliInvocation {
    /// CLI verb to execute.
    pub verb: ExportAuthorizeCliVerb,
    /// Loopback `host:port` of `tepp-naruon-live`.
    pub host: String,
    /// Per-export operation key; never equal to `principal_id`.
    pub idempotency_key: String,
    /// Validated purpose-bound export request.
    pub request: ExportAuthorizationRequest,
}

impl ExportAuthorizeCliInvocation {
    /// Parse argv plus stdin JSON into a validated loopback export invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing flags, a
    /// non-loopback host, credential-shaped flags, a nonempty-incompatible
    /// purpose, metric keys, or an idempotency key equal to `principal_id`.
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
        let verb = ExportAuthorizeCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        let body = body.into();
        refuse_scientific_acceptance(&body)?;
        refuse_metrics(&body)?;
        let request: ExportAuthorizationRequest = from_json(&body)?;
        let invocation = Self {
            verb,
            host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
            idempotency_key: flags.idempotency_key.ok_or(ApiError::InvalidWirePayload)?,
            request,
        };
        invocation.validate()?;
        Ok(invocation)
    }

    /// Reject a non-loopback host or a purpose the live listener will not serve.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
    /// non-modular purpose, and [`ApiError::InvalidWirePayload`] when the
    /// idempotency key equals `principal_id`.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.request.tenant_workspace_id)?;
        require_nonempty(&self.request.principal_id)?;
        require_nonempty(&self.request.artifact_id)?;
        if self.idempotency_key == self.request.principal_id {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.request.purpose != AnalyticalPurpose::ModularServiceConsumer {
            return Err(ApiError::AuthorizationDenied);
        }
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
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

fn require_loopback_host(host: &str) -> Result<SocketAddr, ApiError> {
    let addr: SocketAddr = host.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

/// Compose one HTTP/1.1 export-authorize POST for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ExportAuthorizeCliInvocation::validate`].
pub fn compose_export_authorize_cli_http(
    invocation: &ExportAuthorizeCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let body = to_json(&invocation.request)?;
    refuse_scientific_acceptance(&body)?;
    refuse_metrics(&body)?;
    Ok(format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        invocation.host,
        invocation.idempotency_key,
        body.len()
    ))
}

/// Dispatch one export CLI invocation against an in-process naruon live service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_export_authorize_cli(
    service: &mut NaruonLiveService,
    invocation: &ExportAuthorizeCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_export_authorize_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one export CLI invocation over loopback TCP against `tepp-naruon-live`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_export_authorize_cli(
    invocation: &ExportAuthorizeCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_export_authorize_cli_http(invocation)?;
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

/// Filter CLI stdout so export never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when the body is empty or carries
/// metric keys or the scientific-acceptance schema.
pub fn render_export_authorize_cli_stdout(
    invocation: &ExportAuthorizeCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance(&response.body)?;
    refuse_metrics(&response.body)?;
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
    if FORBIDDEN_EXPORT_KEYS
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

/// Read stdin leftover bytes on a non-terminal; export authorize requires JSON.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_export_authorize_cli_stdin(
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
        ExportAuthorizeCliInvocation, ExportAuthorizeCliVerb, SCIENTIFIC_ACCEPTANCE_SCHEMA,
        compose_export_authorize_cli_http, dispatch_export_authorize_cli,
        execute_export_authorize_cli, parse_http_response, read_export_authorize_cli_stdin,
        render_export_authorize_cli_stdout, static_reason,
    };
    use crate::{
        AnalyticalPurpose, ApiError, ExportAuthorizationRequest, NaruonLiveResponse,
        NaruonLiveService,
    };

    fn allowed_body() -> String {
        serde_json::to_string(&ExportAuthorizationRequest {
            tenant_workspace_id: "tenant-a".into(),
            principal_id: "naruon-service".into(),
            purpose: AnalyticalPurpose::ModularServiceConsumer,
            artifact_id: "artifact-a".into(),
            includes_source_text: false,
        })
        .expect("json")
    }

    fn authorize_args() -> [&'static str; 5] {
        [
            "authorize",
            "--host",
            "127.0.0.1:18082",
            "--idempotency-key",
            "export-idem-1",
        ]
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            ExportAuthorizeCliVerb::parse("authorize").expect("verb"),
            ExportAuthorizeCliVerb::Authorize
        );
        assert_eq!(ExportAuthorizeCliVerb::Authorize.as_str(), "authorize");
        assert_eq!(
            ExportAuthorizeCliVerb::parse("AUTHORIZE"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportAuthorizeCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ExportAuthorizeCliVerb::parse("wait"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_host_credentials_purpose_and_metrics() {
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(["authorize"], allowed_body()).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(
                [
                    "authorize",
                    "--host",
                    "8.8.8.8:80",
                    "--idempotency-key",
                    "export-idem-1"
                ],
                allowed_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(
                [
                    "authorize",
                    "--host",
                    "127.0.0.1:18082",
                    "--idempotency-key",
                    "export-idem-1",
                    "--authorization",
                    "secret"
                ],
                allowed_body()
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        let monitoring = serde_json::to_string(&ExportAuthorizationRequest {
            tenant_workspace_id: "tenant-a".into(),
            principal_id: "naruon-service".into(),
            purpose: AnalyticalPurpose::OperationalMonitoring,
            artifact_id: "artifact-a".into(),
            includes_source_text: false,
        })
        .expect("json");
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(authorize_args(), monitoring).unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(authorize_args(), r#"{"rmse":1.0}"#)
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(
                [
                    "authorize",
                    "--host",
                    "127.0.0.1:18082",
                    "--idempotency-key",
                    "naruon-service"
                ],
                allowed_body()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ExportAuthorizeCliInvocation::from_args(authorize_args(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn compose_posts_exports_without_credentials_or_metrics() {
        let invocation =
            ExportAuthorizeCliInvocation::from_args(authorize_args(), allowed_body()).expect("inv");
        let http = compose_export_authorize_cli_http(&invocation).expect("http");
        assert!(http.starts_with("POST /v1/exports HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(http.contains("idempotency-key: export-idem-1"));
        assert!(!http.contains("authorization"));
        assert!(!http.contains("/analysis-runs"));
        assert!(!http.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!http.contains("rmse"));
    }

    #[test]
    fn dispatch_allows_modular_export_without_scientific_acceptance() {
        let mut service = NaruonLiveService::new();
        let invocation =
            ExportAuthorizeCliInvocation::from_args(authorize_args(), allowed_body()).expect("inv");
        let got = dispatch_export_authorize_cli(&mut service, &invocation).expect("dispatch");
        assert_eq!(got.status_code, 200);
        let stdout = render_export_authorize_cli_stdout(&invocation, &got).expect("stdout");
        assert!(stdout.contains("purpose_bound_export_allowed"));
        assert!(stdout.contains("\"allowed\":true"));
        assert!(!stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!stdout.contains("rmse"));
        assert!(!stdout.contains("source_text"));
    }

    #[test]
    fn render_refuses_metrics_schema_and_empty_bodies() {
        let invocation =
            ExportAuthorizeCliInvocation::from_args(authorize_args(), allowed_body()).expect("inv");
        assert_eq!(
            render_export_authorize_cli_stdout(
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
            render_export_authorize_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"allowed":true,"rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_export_authorize_cli_stdout(
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
        let mut service = NaruonLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation =
            ExportAuthorizeCliInvocation::from_args(authorize_args(), allowed_body()).expect("inv");
        invocation.host = addr.to_string();
        let response = execute_export_authorize_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_export_authorize_cli(&invocation).unwrap_err(),
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
        assert_eq!(static_reason(403).expect("403"), "Forbidden");
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let empty = read_export_authorize_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped =
            read_export_authorize_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("piped");
        assert_eq!(piped, "{}");
    }
}
