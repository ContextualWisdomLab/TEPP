//! Operator loopback CLI for scientific-acceptance execute.
//!
//! GAP-003A execute-CLI slice: operators run `tepp-execute execute` to POST
//! `/v1/analysis-runs/{run_id}/execute` from the typed naruon/`LineageWeave`
//! execute exchange onto spawned `tepp-loopback` TCP. This module does not
//! duplicate the TCP renderer (#382), execute builders (#381), published
//! binary (#375), engine-execute (#370), lifecycle CLI (#362), create CLI
//! (#385), cancel consumer-parity (#373), cancel CLI (#378), stored-request
//! GET (#377), stored-request consumer-parity (#387), retry-children (#379),
//! idempotency (#380), retry-parent (#384), collection GET (#368), GET
//! (#359), lifecycle POST (#360), cancel HTTP (#361), collection CLI (#371),
//! retry (#369), engine-library (#356), DTO (#358), persistence (#287),
//! Leiden (#351), Driver p.16, CWC/Rubin/ESEM/OLS, GAP-010, or GAP-003C.
//! Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use tepp_api::{
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NaruonLiveResponse,
    SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
};

use crate::{
    ScientificAcceptanceExecuteRequest, ScientificAcceptanceLoopbackService,
    lineageweave_analysis_run_execute_exchange, loopback_http1_from_execute_exchange,
    naruon_analysis_run_execute_exchange,
};

/// Bound for one CLI TCP round-trip against spawned `tepp-loopback`.
const EXECUTE_CLI_IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Supported operator verbs for the loopback execute CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScientificAcceptanceExecuteCliVerb {
    /// `POST /v1/analysis-runs/{run_id}/execute`.
    Execute,
}

impl ScientificAcceptanceExecuteCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "execute" => Ok(Self::Execute),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Execute => "execute",
        }
    }
}

/// One operator CLI invocation against spawned `tepp-loopback` TCP.
#[derive(Clone, Debug, PartialEq)]
pub struct ScientificAcceptanceExecuteCliInvocation {
    /// CLI verb to execute.
    pub verb: ScientificAcceptanceExecuteCliVerb,
    /// Loopback `host:port` printed by `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin kept on the typed execute exchange.
    pub origin: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Typed metric-free execute body.
    pub execute: ScientificAcceptanceExecuteRequest,
}

impl ScientificAcceptanceExecuteCliInvocation {
    /// Parse argv plus stdin body into a validated loopback execute invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, `localhost`, an unpublished consumer, credential-shaped
    /// flags, empty stdin, LLM recovery, metric keys, or a body that disagrees
    /// with optional `--run-id` / `--idempotency-key`.
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
        let verb = ScientificAcceptanceExecuteCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        let body = body.into();
        assemble_invocation(verb, flags, &body)
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile execute body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] for empty or unpublished fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        require_nonempty(&self.consumer)?;
        if self.consumer != NARUON_CONSUMER_CODE && self.consumer != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        self.execute.to_json()?;
        self.typed_execute_exchange()?;
        Ok(())
    }

    fn typed_execute_exchange(&self) -> Result<tepp_api::NaruonHttpExchange, ApiError> {
        if self.consumer == LINEAGEWEAVE_CONSUMER_CODE {
            lineageweave_analysis_run_execute_exchange(&self.origin, &self.execute)
        } else if self.consumer == NARUON_CONSUMER_CODE {
            naruon_analysis_run_execute_exchange(&self.origin, &self.execute)
        } else {
            Err(ApiError::InvalidWirePayload)
        }
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    idempotency_key: Option<String>,
    run_id: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        idempotency_key: None,
        run_id: None,
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        if !flag.starts_with("--") {
            return Err(ApiError::InvalidWirePayload);
        }
        let name = &flag[2..];
        if flag_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        let slot = match name {
            "host" => &mut flags.host,
            "origin" => &mut flags.origin,
            "consumer" => &mut flags.consumer,
            "idempotency-key" => &mut flags.idempotency_key,
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
    verb: ScientificAcceptanceExecuteCliVerb,
    flags: ParsedFlags,
    body: &str,
) -> Result<ScientificAcceptanceExecuteCliInvocation, ApiError> {
    if body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    let execute = ScientificAcceptanceExecuteRequest::from_json(body)?;
    if let Some(run_id) = flags.run_id.as_deref()
        && run_id != execute.run_id
    {
        return Err(ApiError::InvalidWirePayload);
    }
    if let Some(idempotency_key) = flags.idempotency_key.as_deref()
        && idempotency_key != execute.idempotency_key
    {
        return Err(ApiError::InvalidWirePayload);
    }
    let invocation = ScientificAcceptanceExecuteCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
        execute,
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

fn require_nonempty(value: &str) -> Result<(), ApiError> {
    if value.is_empty() {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn flag_is_credential(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("authorization") || lower.contains("token") || lower.contains("copilot")
}

/// Compose HTTP/1.1 from the typed naruon/`LineageWeave` execute exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as the typed execute builders and
/// [`loopback_http1_from_execute_exchange`].
pub fn compose_scientific_acceptance_execute_cli_http(
    invocation: &ScientificAcceptanceExecuteCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    loopback_http1_from_execute_exchange(&invocation.typed_execute_exchange()?, &invocation.host)
}

/// Dispatch one execute CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_scientific_acceptance_execute_cli(
    service: &mut ScientificAcceptanceLoopbackService,
    invocation: &ScientificAcceptanceExecuteCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_scientific_acceptance_execute_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_scientific_acceptance_execute_cli(
    invocation: &ScientificAcceptanceExecuteCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_scientific_acceptance_execute_cli_http(invocation)?;
    let mut stream = TcpStream::connect(addr).map_err(|_| ApiError::InvalidWirePayload)?;
    stream
        .set_read_timeout(Some(EXECUTE_CLI_IO_TIMEOUT))
        .map_err(|_| ApiError::InvalidWirePayload)?;
    stream
        .set_write_timeout(Some(EXECUTE_CLI_IO_TIMEOUT))
        .map_err(|_| ApiError::InvalidWirePayload)?;
    stream
        .write_all(request.as_bytes())
        .map_err(|_| ApiError::InvalidWirePayload)?;
    stream.flush().map_err(|_| ApiError::InvalidWirePayload)?;
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|_| ApiError::InvalidWirePayload)?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so a non-success execute never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a success body omits the
/// engine-produced schema or a non-success body still carries it.
pub fn render_scientific_acceptance_execute_cli_stdout(
    invocation: &ScientificAcceptanceExecuteCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    if (200..300).contains(&response.status_code) {
        if !response.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA) {
            return Err(ApiError::InvalidWirePayload);
        }
    } else if response.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(response.body.clone())
}

/// Read stdin leftover bytes on a non-terminal; empty execute POST is refused.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_scientific_acceptance_execute_cli_stdin(
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

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        ScientificAcceptanceExecuteCliInvocation, ScientificAcceptanceExecuteCliVerb,
        compose_scientific_acceptance_execute_cli_http, flag_is_credential,
        read_scientific_acceptance_execute_cli_stdin,
        render_scientific_acceptance_execute_cli_stdout,
    };
    use crate::{
        ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
        VALIDATION_CPU_F64_MODEL,
    };
    use tepp_api::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
        NARUON_CONSUMER_CODE, NaruonLiveResponse, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
        SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA, naruon_analysis_run_exchange,
    };

    use super::dispatch_scientific_acceptance_execute_cli;
    use crate::{ScientificAcceptanceLoopbackService, loopback_http1_from_naruon_exchange};

    const HTTPS_ORIGIN: &str = "https://tepp.example.com";

    fn execute_json(run_id: &str, idempotency_key: &str) -> String {
        serde_json::json!({
            "contract_version": ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
            "run_id": run_id,
            "idempotency_key": idempotency_key,
            "seed": 42,
            "se_gate_k": 3.0,
            "completed_at": "2026-08-31T13:00:00Z",
            "study_label": "loopback-cli-recovery",
            "authored_by_llm": false,
            "corpus": {
                "snapshot_id": "snapshot-execute-cli",
                "evidence_units": [
                    {
                        "evidence_id": "evidence-1",
                        "event_time": "2026-07-01T00:00:00Z",
                        "available_time": "2026-07-10T00:00:00Z",
                        "membership_count": 1
                    },
                    {
                        "evidence_id": "evidence-2",
                        "event_time": "2026-07-01T00:00:00Z",
                        "available_time": "2026-07-20T00:00:00Z",
                        "membership_count": 1
                    },
                    {
                        "evidence_id": "future",
                        "event_time": "2026-07-01T00:00:00Z",
                        "available_time": "2026-08-02T00:00:00Z",
                        "membership_count": 1
                    }
                ]
            },
            "truth": [0.70, 0.55, 0.40, -0.20, 0.85],
            "recovered": [0.70, 0.55, 0.40, -0.20, 0.85],
            "interval_lower": [0.50, 0.35, 0.20, -0.40, 0.65],
            "interval_upper": [0.90, 0.75, 0.60, 0.00, 1.00],
            "truth_times": [1.0, 2.0, 3.0, 4.0, 5.0],
            "recovered_times": [1.1, 1.9, 3.2, 3.8, 5.1]
        })
        .to_string()
    }

    fn invocation(
        host: &str,
        consumer: &str,
        run_id: &str,
        idempotency_key: &str,
    ) -> Result<ScientificAcceptanceExecuteCliInvocation, ApiError> {
        ScientificAcceptanceExecuteCliInvocation::from_args(
            [
                "execute",
                "--host",
                host,
                "--origin",
                HTTPS_ORIGIN,
                "--consumer",
                consumer,
                "--run-id",
                run_id,
                "--idempotency-key",
                idempotency_key,
            ],
            execute_json(run_id, idempotency_key),
        )
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            ScientificAcceptanceExecuteCliVerb::parse("execute").expect("execute"),
            ScientificAcceptanceExecuteCliVerb::Execute
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliVerb::Execute.as_str(),
            "execute"
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliVerb::parse("EXECUTE"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliVerb::parse("status"),
            Err(ApiError::InvalidWirePayload)
        );
        assert!(flag_is_credential("authorization"));
        assert!(flag_is_credential("admin-token"));
        assert!(flag_is_credential("copilot"));
        assert!(!flag_is_credential("host"));
    }

    #[test]
    fn from_args_refuses_empty_public_bind_localhost_and_credentials() {
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(Vec::<String>::new(), "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            invocation("8.8.8.8:80", NARUON_CONSUMER_CODE, "tepp-run-1", "idem-1").unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            invocation(
                "localhost:18081",
                NARUON_CONSUMER_CODE,
                "tepp-run-1",
                "idem-1"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    HTTPS_ORIGIN,
                    "--authorization",
                    "Bearer secret"
                ],
                execute_json("tepp-run-1", "idem-1")
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    HTTPS_ORIGIN
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let mut llm =
            serde_json::from_str::<serde_json::Value>(&execute_json("tepp-run-1", "idem-1"))
                .expect("json");
        llm["authored_by_llm"] = serde_json::json!(true);
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    HTTPS_ORIGIN
                ],
                llm.to_string()
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    HTTPS_ORIGIN
                ],
                r#"{"rmse":1.0}"#
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    "http://tepp.example.com"
                ],
                execute_json("tepp-run-1", "idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ScientificAcceptanceExecuteCliInvocation::from_args(
                [
                    "execute",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    HTTPS_ORIGIN,
                    "--run-id",
                    "other-run"
                ],
                execute_json("tepp-run-1", "idem-1")
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            read_scientific_acceptance_execute_cli_stdin(true, std::io::empty()).expect("terminal"),
            ""
        );
    }

    #[test]
    fn compose_uses_typed_execute_exchange_without_credentials() {
        let naruon = invocation(
            "127.0.0.1:18081",
            NARUON_CONSUMER_CODE,
            "tepp-run-1",
            "idem-naruon-execute-cli",
        )
        .expect("naruon");
        let http = compose_scientific_acceptance_execute_cli_http(&naruon).expect("http");
        assert!(http.starts_with("POST /v1/analysis-runs/tepp-run-1/execute HTTP/1.1"));
        assert!(http.contains("Host: 127.0.0.1:18081"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(!http.to_ascii_lowercase().contains("authorization"));
        assert!(!http.contains("scientific_acceptance_json"));

        let lineage = invocation(
            "127.0.0.1:18081",
            LINEAGEWEAVE_CONSUMER_CODE,
            "tepp-run-1",
            "idem-lineage-execute-cli",
        )
        .expect("lineage");
        let lineage_http =
            compose_scientific_acceptance_execute_cli_http(&lineage).expect("lineage http");
        assert!(lineage_http.contains("tepp-consumer: lineageweave"));
        assert!(!lineage_http.contains("tepp-consumer: naruon"));
    }

    #[test]
    fn in_process_cli_execute_returns_scientific_acceptance() {
        let mut service = ScientificAcceptanceLoopbackService::new();
        let request = AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "idem-naruon-execute-cli".into(),
            tenant_workspace_id: "tenant-workspace-execute-cli".into(),
            snapshot_id: "snapshot-execute-cli".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: VALIDATION_CPU_F64_MODEL.into(),
            output_profile: SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into(),
        };
        let create = naruon_analysis_run_exchange(HTTPS_ORIGIN, &request).expect("create");
        let create_http =
            loopback_http1_from_naruon_exchange(&create, "127.0.0.1:18081").expect("create http");
        let accepted = service.handle_http_request(&create_http);
        assert_eq!(accepted.status_code, 202);
        let run_id = serde_json::from_str::<serde_json::Value>(&accepted.body)
            .expect("accepted json")["run_id"]
            .as_str()
            .expect("run_id")
            .to_owned();
        let invocation = invocation(
            "127.0.0.1:18081",
            NARUON_CONSUMER_CODE,
            &run_id,
            "idem-naruon-execute-cli",
        )
        .expect("invocation");
        let response = dispatch_scientific_acceptance_execute_cli(&mut service, &invocation)
            .expect("dispatch");
        assert_eq!(response.status_code, 200);
        let stdout = render_scientific_acceptance_execute_cli_stdout(&invocation, &response)
            .expect("stdout");
        assert!(stdout.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
        assert!(stdout.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
        assert_eq!(
            render_scientific_acceptance_execute_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: format!("{{\"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}\":true}}"),
                }
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
