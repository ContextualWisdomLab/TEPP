//! Operator loopback CLI that waits for analysis-run terminal status.
//!
//! GAP-003A operator-visible client of `GET /v1/analysis-runs/{run_id}` (ADR
//! 0027 / live #359, status CLI #392). Operators run `tepp-analysis-runs wait`
//! to poll until succeeded or failed without writing a poll loop.
//! Accepted/running/failed stdout stays metric-free.
//! `tepp.scientific_acceptance.v1` appears only on a succeeded GET whose
//! request profile is `scientific_acceptance_v1`. This module does not
//! duplicate status CLI, GET-by-id HTTP, lifecycle POST, cancel/create/retry
//! CLIs, or lookup CLI. Persistence remains GAP-003B.

use std::thread;
use std::time::{Duration, Instant};

use crate::analysis_run_status_cli::{
    AnalysisRunStatusCliInvocation, dispatch_analysis_run_status_cli,
    execute_analysis_run_status_cli, render_analysis_run_status_cli_stdout,
};
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunLiveService, AnalysisRunStatus, AnalysisRunStatusState, ApiError, NaruonLiveResponse,
};

/// Default wait budget in milliseconds.
pub const ANALYSIS_RUN_WAIT_DEFAULT_TIMEOUT_MS: u64 = 1_000;
/// Maximum wait budget in milliseconds.
pub const ANALYSIS_RUN_WAIT_MAX_TIMEOUT_MS: u64 = 60_000;
/// Default poll interval in milliseconds.
pub const ANALYSIS_RUN_WAIT_DEFAULT_INTERVAL_MS: u64 = 10;
/// Maximum poll interval in milliseconds.
pub const ANALYSIS_RUN_WAIT_MAX_INTERVAL_MS: u64 = 1_000;

/// Supported operator verbs for the loopback wait CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunWaitCliVerb {
    /// Poll `GET /v1/analysis-runs/{run_id}` until terminal or timeout.
    Wait,
}

impl AnalysisRunWaitCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "wait" => Ok(Self::Wait),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wait => "wait",
        }
    }
}

/// One operator CLI invocation that polls loopback status until terminal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunWaitCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunWaitCliVerb,
    /// Underlying status GET invocation.
    pub status: AnalysisRunStatusCliInvocation,
    /// Inclusive wait budget.
    pub timeout: Duration,
    /// Sleep between non-terminal polls.
    pub interval: Duration,
}

impl AnalysisRunWaitCliInvocation {
    /// Parse argv plus stdin body into a validated loopback wait invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, unpublished consumers, credential-shaped flags,
    /// hostile identities, a nonempty body, or an oversized wait budget.
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
        let verb = AnalysisRunWaitCliVerb::parse(verb_token)?;
        let (status_args, timeout_ms, interval_ms) = split_wait_flags(rest)?;
        let mut status_tokens = vec!["status".to_owned()];
        status_tokens.extend(status_args);
        let status = AnalysisRunStatusCliInvocation::from_args(status_tokens, body)?;
        let invocation = Self {
            verb,
            status,
            timeout: Duration::from_millis(timeout_ms),
            interval: Duration::from_millis(interval_ms),
        };
        invocation.validate()?;
        Ok(invocation)
    }

    /// Reject an interval longer than the wait budget.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] when the interval exceeds the
    /// timeout.
    pub fn validate(&self) -> Result<(), ApiError> {
        self.status.validate()?;
        if self.interval > self.timeout && self.timeout != Duration::ZERO {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }
}

fn split_wait_flags(rest: &[String]) -> Result<(Vec<String>, u64, u64), ApiError> {
    let mut timeout_ms = ANALYSIS_RUN_WAIT_DEFAULT_TIMEOUT_MS;
    let mut interval_ms = ANALYSIS_RUN_WAIT_DEFAULT_INTERVAL_MS;
    let mut seen_timeout = false;
    let mut seen_interval = false;
    let mut status_args = Vec::new();
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
        if index + 1 >= rest.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        let value = rest[index + 1].as_str();
        require_nonempty(value)?;
        match name {
            "timeout-ms" => {
                if seen_timeout {
                    return Err(ApiError::InvalidWirePayload);
                }
                timeout_ms = parse_bounded_ms(value, ANALYSIS_RUN_WAIT_MAX_TIMEOUT_MS)?;
                seen_timeout = true;
            }
            "poll-interval-ms" => {
                if seen_interval {
                    return Err(ApiError::InvalidWirePayload);
                }
                interval_ms = parse_bounded_ms(value, ANALYSIS_RUN_WAIT_MAX_INTERVAL_MS)?;
                seen_interval = true;
            }
            _ => {
                status_args.push(flag.to_owned());
                status_args.push(value.to_owned());
            }
        }
        index += 2;
    }
    Ok((status_args, timeout_ms, interval_ms))
}

fn parse_bounded_ms(value: &str, maximum: u64) -> Result<u64, ApiError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| ApiError::InvalidWirePayload)?;
    if parsed > maximum {
        Err(ApiError::LimitExceeded)
    } else {
        Ok(parsed)
    }
}

fn is_terminal(state: AnalysisRunStatusState) -> bool {
    matches!(
        state,
        AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
    )
}

/// Dispatch wait against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors, [`ApiError::LimitExceeded`] when the
/// run stays accepted/running past the budget, or status-path errors.
pub fn dispatch_analysis_run_wait_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunWaitCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    invocation.validate()?;
    let started = Instant::now();
    loop {
        let response = dispatch_analysis_run_status_cli(service, &invocation.status)?;
        if is_wait_complete(&response)? {
            return Ok(response);
        }
        if started.elapsed() >= invocation.timeout {
            return Err(ApiError::LimitExceeded);
        }
        if !invocation.interval.is_zero() {
            thread::sleep(invocation.interval);
        }
    }
}

/// Execute wait over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, timeout, or response errors.
pub fn execute_analysis_run_wait_cli(
    invocation: &AnalysisRunWaitCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    invocation.validate()?;
    let started = Instant::now();
    loop {
        let response = execute_analysis_run_status_cli(&invocation.status)?;
        if is_wait_complete(&response)? {
            return Ok(response);
        }
        if started.elapsed() >= invocation.timeout {
            return Err(ApiError::LimitExceeded);
        }
        if !invocation.interval.is_zero() {
            thread::sleep(invocation.interval);
        }
    }
}

/// Render wait stdout through the status CLI metric-free gates.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`render_analysis_run_status_cli_stdout`].
pub fn render_analysis_run_wait_cli_stdout(
    invocation: &AnalysisRunWaitCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    render_analysis_run_status_cli_stdout(&invocation.status, response)
}

fn is_wait_complete(response: &NaruonLiveResponse) -> Result<bool, ApiError> {
    if !(200..300).contains(&response.status_code) {
        return Ok(true);
    }
    let status = AnalysisRunStatus::from_json(&response.body)?;
    Ok(is_terminal(status.run_state))
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        ANALYSIS_RUN_WAIT_DEFAULT_INTERVAL_MS, ANALYSIS_RUN_WAIT_DEFAULT_TIMEOUT_MS,
        ANALYSIS_RUN_WAIT_MAX_INTERVAL_MS, ANALYSIS_RUN_WAIT_MAX_TIMEOUT_MS,
        AnalysisRunWaitCliInvocation, AnalysisRunWaitCliVerb, dispatch_analysis_run_wait_cli,
        execute_analysis_run_wait_cli, render_analysis_run_wait_cli_stdout,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLiveService,
        AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, AnalysisRunTerminalResult,
        ApiError, NARUON_CONSUMER_CODE,
    };
    use std::time::Duration;

    fn request(idempotency_key: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "cli-wait-tenant".into(),
            snapshot_id: "cli-wait-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn create_http(run: &AnalysisRunRequest, host: &str) -> String {
        let body = run.to_json().expect("json");
        format!(
            "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    }

    fn wait_args(run_id: &str, key: &str, extra: &[&str]) -> Vec<String> {
        let mut args = vec![
            "wait".into(),
            "--host".into(),
            "127.0.0.1:18081".into(),
            "--run-id".into(),
            run_id.into(),
            "--idempotency-key".into(),
            key.into(),
        ];
        args.extend(extra.iter().map(|value| (*value).to_owned()));
        args
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunWaitCliVerb::parse("wait").expect("verb"),
            AnalysisRunWaitCliVerb::Wait
        );
        assert_eq!(AnalysisRunWaitCliVerb::Wait.as_str(), "wait");
        assert_eq!(
            AnalysisRunWaitCliVerb::parse("WAIT"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunWaitCliVerb::parse("status"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunWaitCliVerb::parse("lookup"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_host_credentials_and_oversized_budgets() {
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                [
                    "wait",
                    "--host",
                    "8.8.8.8:80",
                    "--run-id",
                    "tepp-run-1",
                    "--idempotency-key",
                    "idem-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                wait_args("tepp-run-1", "idem-1", &["--authorization", "secret"]),
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                wait_args("tepp-run-1", "idem-1", &["--timeout-ms", "not-a-number"]),
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                wait_args(
                    "tepp-run-1",
                    "idem-1",
                    &[
                        "--timeout-ms",
                        &(ANALYSIS_RUN_WAIT_MAX_TIMEOUT_MS + 1).to_string()
                    ]
                ),
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                wait_args(
                    "tepp-run-1",
                    "idem-1",
                    &[
                        "--poll-interval-ms",
                        &(ANALYSIS_RUN_WAIT_MAX_INTERVAL_MS + 1).to_string()
                    ]
                ),
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(
                wait_args(
                    "tepp-run-1",
                    "idem-1",
                    &["--timeout-ms", "10", "--poll-interval-ms", "20"]
                ),
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunWaitCliInvocation::from_args(wait_args("tepp-run-1", "idem-1", &[]), "{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let defaults =
            AnalysisRunWaitCliInvocation::from_args(wait_args("tepp-run-1", "idem-1", &[]), "")
                .expect("defaults");
        assert_eq!(defaults.verb, AnalysisRunWaitCliVerb::Wait);
        assert_eq!(
            defaults.timeout,
            Duration::from_millis(ANALYSIS_RUN_WAIT_DEFAULT_TIMEOUT_MS)
        );
        assert_eq!(
            defaults.interval,
            Duration::from_millis(ANALYSIS_RUN_WAIT_DEFAULT_INTERVAL_MS)
        );
    }

    #[test]
    fn wait_times_out_on_accepted_and_returns_failed_terminal() {
        let mut service = AnalysisRunLiveService::new();
        let run = request("cli-wait-idem-1");
        let created = service.handle_http_request(&create_http(&run, "127.0.0.1:18081"));
        assert_eq!(created.status_code, 202);
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let timeout = AnalysisRunWaitCliInvocation::from_args(
            wait_args(
                &accepted.run_id,
                "cli-wait-idem-1",
                &["--timeout-ms", "0", "--poll-interval-ms", "0"],
            ),
            "",
        )
        .expect("timeout inv");
        assert_eq!(
            dispatch_analysis_run_wait_cli(&mut service, &timeout).unwrap_err(),
            ApiError::LimitExceeded
        );

        let failed = AnalysisRunTerminalResult::failed(
            &run,
            &accepted,
            "2026-08-02T03:04:05Z",
            "non_convergence",
        )
        .expect("failed");
        let terminal = AnalysisRunStatus::terminal(&run, &accepted, failed).expect("terminal");
        service
            .record_loopback_status(&accepted.run_id, terminal, None)
            .expect("recorded");
        let wait = AnalysisRunWaitCliInvocation::from_args(
            wait_args(
                &accepted.run_id,
                "cli-wait-idem-1",
                &["--timeout-ms", "1000", "--poll-interval-ms", "0"],
            ),
            "",
        )
        .expect("wait");
        let got = dispatch_analysis_run_wait_cli(&mut service, &wait).expect("terminal wait");
        assert_eq!(got.status_code, 200);
        let stdout = render_analysis_run_wait_cli_stdout(&wait, &got).expect("stdout");
        assert!(stdout.contains("\"failed\""));
        assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
        assert!(!stdout.contains("rmse"));
        let status = AnalysisRunStatus::from_json(&stdout).expect("status");
        assert_eq!(status.run_state, AnalysisRunStatusState::Failed);
        assert_eq!(status.run_id, accepted.run_id);
    }

    #[test]
    fn execute_times_out_over_tcp_on_accepted() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let run = request("cli-wait-tcp");
        let created = service.handle_http_request(&create_http(&run, &addr.to_string()));
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = AnalysisRunWaitCliInvocation::from_args(
            wait_args(
                &accepted.run_id,
                "cli-wait-tcp",
                &["--timeout-ms", "0", "--poll-interval-ms", "0"],
            ),
            "",
        )
        .expect("inv");
        invocation.status.host = addr.to_string();
        assert_eq!(
            execute_analysis_run_wait_cli(&invocation).unwrap_err(),
            ApiError::LimitExceeded
        );
        handle.join().expect("join");
    }
}
