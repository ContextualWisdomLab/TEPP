//! Production loopback lifecycle POST contracts for analysis-run status.
//!
//! GAP-003A fourth slice: `POST /v1/analysis-runs/{run_id}/running` and
//! `POST /v1/analysis-runs/{run_id}/terminal` are the operator-visible
//! status-update path. Accepted and running bodies stay metric-free. Only a
//! succeeded terminal whose request profile is `scientific_acceptance_v1` may
//! attach canonical `tepp.scientific_acceptance.v1` bytes. This module does
//! not copy the terminal-result DTO, does not persist, and does not execute
//! psychometric estimation.

use crate::analysis_run_status_http::{ANALYSIS_RUN_ID_MAX_LEN, encode_path_segment};
use crate::naruon_http::{NaruonHttpExchange, compose_https_target, standard_headers};
use crate::scientific_acceptance_http::refuse_metrics_on_receipt;
use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{
    ANALYSIS_RUN_STATUS_PATH, AnalysisRunStatusState, AnalysisRunTerminalResult,
    AnalysisRunTerminalState, ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT,
};
use serde::{Deserialize, Serialize};

/// Supported lifecycle-transition contract version.
pub const ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION: u16 = 1;

/// Production HTTP transition that records running or terminal status.
///
/// `scientific_acceptance_json` stores canonical artifact bytes as a JSON
/// string so `result_sha256` hashes those exact bytes. The field is absent on
/// running and failed transitions and on succeeded profiles that do not bind
/// scientific acceptance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunLifecycleTransition {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Requested lifecycle state. `accepted` is refused.
    pub run_state: AnalysisRunStatusState,
    /// Exact request idempotency key.
    pub idempotency_key: String,
    /// Request-bound terminal result, required for terminal states.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_result: Option<AnalysisRunTerminalResult>,
    /// Canonical scientific-acceptance JSON bytes, when authorized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scientific_acceptance_json: Option<String>,
}

impl AnalysisRunLifecycleTransition {
    /// Construct a metric-free running transition.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for empty identities.
    pub fn running(
        run_id: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let transition = Self {
            contract_version: ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state: AnalysisRunStatusState::Running,
            idempotency_key: idempotency_key.into(),
            terminal_result: None,
            scientific_acceptance_json: None,
        };
        transition.validate()?;
        Ok(transition)
    }

    /// Construct a terminal transition bound to a request-bound result.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error when the result, identities, or optional
    /// canonical artifact bytes are invalid.
    pub fn terminal(
        run_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        terminal_result: AnalysisRunTerminalResult,
        scientific_acceptance_json: Option<String>,
    ) -> Result<Self, ApiError> {
        let run_state = match terminal_result.run_state {
            AnalysisRunTerminalState::Succeeded => AnalysisRunStatusState::Succeeded,
            AnalysisRunTerminalState::Failed => AnalysisRunStatusState::Failed,
        };
        let transition = Self {
            contract_version: ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION,
            run_id: run_id.into(),
            run_state,
            idempotency_key: idempotency_key.into(),
            terminal_result: Some(terminal_result),
            scientific_acceptance_json,
        };
        transition.validate()?;
        Ok(transition)
    }

    /// Parse and validate a lifecycle transition with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
    }

    /// Parse and validate a lifecycle transition with a caller-supplied limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, metric-key, or field-validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        refuse_metrics_on_receipt(payload)?;
        let transition: Self = from_json(payload)?;
        transition.validate()?;
        Ok(transition)
    }

    /// Serialize this transition after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        let payload = to_json(self)?;
        require_byte_limit(&payload, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)?;
        Ok(payload)
    }

    pub(crate) fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(
            self.contract_version,
            ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION,
        )?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        match self.run_state {
            AnalysisRunStatusState::Accepted => Err(ApiError::InvalidWirePayload),
            AnalysisRunStatusState::Running => {
                if self.terminal_result.is_some() || self.scientific_acceptance_json.is_some() {
                    return Err(ApiError::InvalidWirePayload);
                }
                Ok(())
            }
            AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed => {
                let result = self
                    .terminal_result
                    .as_ref()
                    .ok_or(ApiError::InvalidWirePayload)?;
                result.validate()?;
                let expected = match result.run_state {
                    AnalysisRunTerminalState::Succeeded => AnalysisRunStatusState::Succeeded,
                    AnalysisRunTerminalState::Failed => AnalysisRunStatusState::Failed,
                };
                if expected != self.run_state
                    || result.run_id != self.run_id
                    || result.idempotency_key != self.idempotency_key
                {
                    return Err(ApiError::InvalidWirePayload);
                }
                if self.run_state == AnalysisRunStatusState::Failed
                    && self.scientific_acceptance_json.is_some()
                {
                    return Err(ApiError::InvalidWirePayload);
                }
                if let Some(artifact) = self.scientific_acceptance_json.as_deref() {
                    require_nonempty(artifact)?;
                }
                Ok(())
            }
        }
    }
}

/// Build a provider-owned `POST` running-status exchange.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-running transition or a
/// non-`https` origin, and [`ApiError::LimitExceeded`] when the run identity
/// exceeds [`ANALYSIS_RUN_ID_MAX_LEN`].
pub fn naruon_analysis_run_running_exchange(
    origin: &str,
    transition: &AnalysisRunLifecycleTransition,
) -> Result<NaruonHttpExchange, ApiError> {
    if transition.run_state != AnalysisRunStatusState::Running {
        return Err(ApiError::InvalidWirePayload);
    }
    build_lifecycle_exchange(origin, transition, "running")
}

/// Build a provider-owned `POST` terminal-status exchange.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for a non-terminal transition or a
/// non-`https` origin, and [`ApiError::LimitExceeded`] when the run identity
/// exceeds [`ANALYSIS_RUN_ID_MAX_LEN`].
pub fn naruon_analysis_run_terminal_exchange(
    origin: &str,
    transition: &AnalysisRunLifecycleTransition,
) -> Result<NaruonHttpExchange, ApiError> {
    if !matches!(
        transition.run_state,
        AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed
    ) {
        return Err(ApiError::InvalidWirePayload);
    }
    build_lifecycle_exchange(origin, transition, "terminal")
}

fn build_lifecycle_exchange(
    origin: &str,
    transition: &AnalysisRunLifecycleTransition,
    suffix: &str,
) -> Result<NaruonHttpExchange, ApiError> {
    transition.validate()?;
    if transition.run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
        return Err(ApiError::LimitExceeded);
    }
    let encoded_run_id = encode_path_segment(&transition.run_id);
    let target_path = format!("{ANALYSIS_RUN_STATUS_PATH}/{encoded_run_id}/{suffix}");
    let target_url = compose_https_target(origin, &target_path)?;
    Ok(NaruonHttpExchange {
        method: "POST",
        target_url,
        headers: standard_headers(&transition.idempotency_key),
        body: transition.to_json()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION, AnalysisRunLifecycleTransition,
        naruon_analysis_run_running_exchange, naruon_analysis_run_terminal_exchange,
    };
    use crate::analysis_run_status_http::ANALYSIS_RUN_ID_MAX_LEN;
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted,
        AnalysisRunRequest, AnalysisRunStatusState, AnalysisRunTerminalResult, ApiError,
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
        SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    };

    fn request(profile: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "lifecycle-idem-1".into(),
            tenant_workspace_id: "lifecycle-tenant-1".into(),
            snapshot_id: "lifecycle-snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "validation_cpu_f64_v1".into(),
            output_profile: profile.into(),
        }
    }

    fn accepted() -> AnalysisRunAccepted {
        AnalysisRunAccepted::new("tepp-run-1", "accepted", "lifecycle-idem-1").expect("accepted")
    }

    fn succeeded_result() -> AnalysisRunTerminalResult {
        let request = request(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE);
        let accepted = accepted();
        AnalysisRunTerminalResult::succeeded(
            &request,
            &accepted,
            "artifact-lifecycle-1",
            "ab".repeat(32),
            SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
            "2026-08-02T03:04:05Z",
            AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated")
                .expect("summary"),
        )
        .expect("succeeded")
    }

    fn failed_result() -> AnalysisRunTerminalResult {
        AnalysisRunTerminalResult::failed(
            &request("calibrated_event_measurement"),
            &accepted(),
            "2026-08-02T03:04:05Z",
            "estimation_failed",
        )
        .expect("failed")
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn running_and_terminal_transitions_round_trip_and_refuse_hostile_shapes() {
        let running = AnalysisRunLifecycleTransition::running("tepp-run-1", "lifecycle-idem-1")
            .expect("running");
        let running_json = running.to_json().expect("running json");
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json(&running_json).expect("decode"),
            running
        );
        assert!(!running_json.contains("rmse"));
        assert!(!running_json.contains("scientific_acceptance_json"));

        let artifact = r#"{"schema_version":"tepp.scientific_acceptance.v1"}"#;
        let terminal = AnalysisRunLifecycleTransition::terminal(
            "tepp-run-1",
            "lifecycle-idem-1",
            succeeded_result(),
            Some(artifact.into()),
        )
        .expect("terminal");
        let terminal_json = terminal.to_json().expect("terminal json");
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json(&terminal_json).expect("decode terminal"),
            terminal
        );
        assert!(terminal_json.contains("scientific_acceptance_json"));

        let failed = AnalysisRunLifecycleTransition::terminal(
            "tepp-run-1",
            "lifecycle-idem-1",
            failed_result(),
            None,
        )
        .expect("failed");
        assert_eq!(failed.run_state, AnalysisRunStatusState::Failed);

        assert_eq!(
            AnalysisRunLifecycleTransition::running("", "lifecycle-idem-1"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::running("tepp-run-1", ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::terminal(
                "tepp-run-1",
                "lifecycle-idem-1",
                failed_result(),
                Some(artifact.into()),
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::terminal(
                "tepp-run-other",
                "lifecycle-idem-1",
                succeeded_result(),
                None,
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json(&running_json.replacen(
                '{',
                r#"{"rmse":0.1,"#,
                1
            )),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json_with_limit(&running_json, 8),
            Err(ApiError::LimitExceeded)
        );
        assert!(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT >= running_json.len());

        let mut accepted_state = running.clone();
        accepted_state.run_state = AnalysisRunStatusState::Accepted;
        assert_eq!(accepted_state.to_json(), Err(ApiError::InvalidWirePayload));

        let mut running_with_result = running.clone();
        running_with_result.terminal_result = Some(failed_result());
        assert_eq!(
            running_with_result.to_json(),
            Err(ApiError::InvalidWirePayload)
        );

        let mut running_with_artifact = running.clone();
        running_with_artifact.scientific_acceptance_json = Some(artifact.into());
        assert_eq!(
            running_with_artifact.to_json(),
            Err(ApiError::InvalidWirePayload)
        );

        let mut missing_result = terminal.clone();
        missing_result.terminal_result = None;
        assert_eq!(missing_result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut mismatched_state = terminal.clone();
        mismatched_state.run_state = AnalysisRunStatusState::Failed;
        assert_eq!(
            mismatched_state.to_json(),
            Err(ApiError::InvalidWirePayload)
        );

        let mut mismatched_idem = terminal.clone();
        mismatched_idem.idempotency_key = "other-key".into();
        assert_eq!(mismatched_idem.to_json(), Err(ApiError::InvalidWirePayload));

        let mut empty_artifact = terminal.clone();
        empty_artifact.scientific_acceptance_json = Some(String::new());
        assert_eq!(empty_artifact.to_json(), Err(ApiError::InvalidWirePayload));

        let extra = running_json.replacen('{', r#"{"extra":true,"#, 1);
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json(&extra),
            Err(ApiError::InvalidWirePayload)
        );
        let wrong_version = running_json.replace(
            &format!("\"contract_version\":{ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION}"),
            "\"contract_version\":9",
        );
        assert_eq!(
            AnalysisRunLifecycleTransition::from_json(&wrong_version),
            Err(ApiError::UnsupportedContractVersion)
        );
    }

    #[test]
    fn running_and_terminal_exchanges_encode_paths_and_refuse_hostile_origins() {
        let running = AnalysisRunLifecycleTransition::running("tepp-run-1", "lifecycle-idem-1")
            .expect("running");
        let running_exchange =
            naruon_analysis_run_running_exchange("https://tepp.example.com", &running)
                .expect("running exchange");
        assert_eq!(running_exchange.method, "POST");
        assert_eq!(
            running_exchange.target_url,
            "https://tepp.example.com/v1/analysis-runs/tepp-run-1/running"
        );
        assert!(!running_exchange.body.contains("rmse"));

        let unsafe_id =
            AnalysisRunLifecycleTransition::running("run/../../etc", "lifecycle-idem-1")
                .expect("unsafe");
        let encoded = naruon_analysis_run_running_exchange("https://tepp.example.com", &unsafe_id)
            .expect("encoded");
        assert!(encoded.target_url.contains("run%2F..%2F..%2Fetc/running"));

        let terminal = AnalysisRunLifecycleTransition::terminal(
            "tepp-run-1",
            "lifecycle-idem-1",
            failed_result(),
            None,
        )
        .expect("terminal");
        let terminal_exchange =
            naruon_analysis_run_terminal_exchange("https://tepp.example.com", &terminal)
                .expect("terminal exchange");
        assert!(
            terminal_exchange
                .target_url
                .ends_with("/v1/analysis-runs/tepp-run-1/terminal")
        );

        assert_eq!(
            naruon_analysis_run_running_exchange("http://tepp.example.com", &running)
                .expect_err("http"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            naruon_analysis_run_running_exchange("https://tepp.example.com", &terminal)
                .expect_err("terminal on running"),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            naruon_analysis_run_terminal_exchange("https://tepp.example.com", &running)
                .expect_err("running on terminal"),
            ApiError::InvalidWirePayload
        );

        let oversized_id = "a".repeat(ANALYSIS_RUN_ID_MAX_LEN + 1);
        let mut oversized = running.clone();
        oversized.run_id = oversized_id;
        // bypass validate by calling builder after mutating
        oversized.contract_version = ANALYSIS_RUN_LIFECYCLE_CONTRACT_VERSION;
        assert_eq!(
            naruon_analysis_run_running_exchange("https://tepp.example.com", &oversized)
                .expect_err("limit"),
            ApiError::LimitExceeded
        );
    }
}
