//! Versioned terminal analysis-run result contracts.
//!
//! Submission acceptance and scientific completion are separate facts. An
//! [`AnalysisRunAccepted`] value proves only that TEPP accepted a durable run.
//! This module publishes a distinct terminal contract that binds any result
//! artifact back to the immutable request, snapshot, cutoff, model contract,
//! output profile, and accepted remote run identity.

use crate::wire::{
    from_json, require_byte_limit, require_contract_version, require_nonempty, to_json,
};
use crate::{AnalysisRunAccepted, AnalysisRunRequest, ApiError};
use serde::{Deserialize, Serialize};
use temporal_core::{KnowledgeCutoff, SystemTime};

/// Supported terminal analysis-result contract version.
pub const ANALYSIS_RESULT_CONTRACT_VERSION: u16 = 1;

/// Default maximum terminal analysis-result JSON payload size in bytes.
pub const DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT: usize = 64 * 1024;

const MAXIMUM_SUMMARY_COUNT: u64 = 1_000_000_000;
const MAXIMUM_FAILURE_CODE_BYTES: usize = 64;

/// Canonical terminal lifecycle state for an analysis run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisRunTerminalState {
    /// Computation completed and a digest-bound result artifact is available.
    Succeeded,
    /// Computation ended without a result artifact.
    Failed,
}

/// Bounded, identity-free summary of a completed measurement artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResultSummary {
    /// Versioned analysis family, such as `temporal_topic_measurement`.
    pub analysis_family: String,
    /// Number of evidence units represented by the result.
    pub evidence_count: u64,
    /// Number of reported statistics or parameters.
    pub statistic_count: u64,
    /// Provider-authored validation state, such as `validated`.
    pub validation_status: String,
}

impl AnalysisResultSummary {
    /// Construct and validate an identity-free result summary.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for empty labels or unbounded
    /// counts.
    pub fn new(
        analysis_family: impl Into<String>,
        evidence_count: u64,
        statistic_count: u64,
        validation_status: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let summary = Self {
            analysis_family: analysis_family.into(),
            evidence_count,
            statistic_count,
            validation_status: validation_status.into(),
        };
        summary.validate();
        Ok(summary)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_nonempty(&self.analysis_family)?;
        require_nonempty(&self.validation_status)?;
        if self.evidence_count > MAXIMUM_SUMMARY_COUNT
            || self.statistic_count > MAXIMUM_SUMMARY_COUNT
        {
            return Err(ApiError::LimitExceeded);
      }
        Ok(())
    }
}

/// A request-bound terminal analysis outcome.
///
/// A succeeded value carries only artifact identity, canonical digest, schema,
/// and a bounded summary. It deliberately excludes source text, credentials,
/// direct identity, respondent records, item records, and unrestricted model
/// output. A failed value carries only a stable redacted failure code.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunTerminalResult {
    /// Semantic contract version for this payload family.
    pub contract_version: u16,
    /// Server-assigned opaque run identity from [`AnalysisRunAccepted`].
    pub run_id: String,
    /// Canonical terminal lifecycle state.
    pub run_state: AnalysisRunTerminalState,
    /// Echo of the validated request idempotency key.
    pub idempotency_key: String,
    /// Authorized tenant or workspace opaque identity.
    pub tenant_workspace_id: String,
    /// Immutable corpus/evidence snapshot identity.
    pub snapshot_id: String,
    /// Exact request knowledge cutoff.
    pub knowledge_cutoff: String,
    /// Versioned model/backend contract identity.
    pub model_contract_version: String,
    /// Exact requested output profile.
    pub output_profile: String,
    /// Opaque immutable result artifact identity for a succeeded run.
    pub result_artifact_id: Option<String>,
    /// Canonical lowercase SHA-256 digest for a succeded result artifact.
    pub result_sha256: Option<String>,
    /// Versioned result schema identity for a succeeded run.
    pub result_schema_version: Option<String>,
    /// Strict RFC 3339 system time at which the run became terminal.
    pub completed_at: String,
    /// Bounded identity-free summary for a succeeded run.
    pub summary: Option<AnalysisResultSummary>,
    /// Stable snake-case failure code for a failed run.
    pub failure_code: Option<String>,
}

impl AnalysisRunTerminalResult {
    /// Construct a validated succeeded result bound to request and acceptance.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed contract error when request binding, acceptance
    /// binding, timestamp, digest, or summary validation fails.
    pub fn succeeded(
        request: &AnalysisRunRequest,
        accepted: &AnalysisRunAccepted,
        result_artifact_id: impl Into<String>,
        result_sha256: impl Into<String>,
        result_schema_version: impl Into<String>,
        completed_at: impl Into<String>,
        summary: AnalysisResultSummary,
    ) -> Result<Self, ApiError> {
        let result = Self {
            contract_version: ANALYSIS_RESULT_CONTRACT_VERSION,
            run_id: accepted.run_id.clone(),
            run_state: AnalysisRunTerminalState::Succeeded,
            idempotency_key: request.idempotency_key.clone(),
            tenant_workspace_id: request.tenant_workspace_id.clone(),
            snapshot_id: request.snapshot_id.clone(),
            knowledge_cutoff: request.knowledge_cutoff.clone(),
            model_contract_version: request.model_contract_version.clone(),
            output_profile: request.output_profile.clone(),
            result_artifact_id: Some(result_artifact_id.into()),
            result_sha256: Some(result_sha256.into()),
            result_schema_version: Some(result_schema_version.into()),
            completed_at: completed_at.into(),
            summary: Some(summary),
            failure_code: None,
        };
        result.validate()?;
        require_terminal_binding(request, accepted, &result)?;
        Ok(result)
    }

    /// Construct a validated terminal failure bound to request and acceptance.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed contract error when request binding, acceptance
    /// binding, timestamp, or failure-code validation fails.
    pub fn failed(
        request: &AnalysisRunRequest,
        accepted: &AnalysisRunAccepted,
        completed_at: impl Into<String>,
        failure_code: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let result = Self {
            contract_version: ANALYSIS_RESULT_CONTRACT_VERSION,
            run_id: accepted.run_id.clone(),
            run_state: AnalysisRunTerminalState::Failed,
            idempotency_key: request.idempotency_key.clone(),
            tenant_workspace_id: request.tenant_workspace_id.clone(),
            snapshot_id: request.snapshot_id.clone(),
            knowledge_cutoff: request.knowledge_cutoff.clone(),
            model_contract_version: request.model_contract_version.clone(),
            output_profile: request.output_profile.clone(),
            result_artifact_id: None,
            result_sha256: None,
            result_schema_version: None,
            completed_at: completed_at.into(),
            summary: None,
            failure_code: Some(failure_code.into()),
        };
        result.validate()?;
        require_terminal_binding(request, accepted, &result)?;
        Ok(result)
    }

    /// Parse and validate a terminal result with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, timestamp, digest, state-shape, or field
    /// validation errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT)
    }

    /// Parse and validate a terminal result with a caller-supplied byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, timestamp, digest, state-shape, or field
    /// validation errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let result: Self = from_json(payload)?;
        result.validate()?;
        Ok(result)
    }

    /// Serialize this terminal result after complete validation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization errors.
    pub fn to_json(&self) -> Result<String, ApiError> {
        self.validate()?;
        to_json(self)
    }

    fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, ANALYSIS_RESULT_CONTRACT_VERSION)?;
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        require_nonempty(&self.tenant_workspace_id)?;
        require_nonempty(&self.snapshot_id)?;
        require_nonempty(&self.knowledge_cutoff)?;
        require_nonempty(&self.model_contract_version)?;
        require_nonempty(&self.output_profile)?;
        require_nonempty(&self.completed_at)?;
        KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        SystemTime::parse_rfc3339(&self.completed_at).map_err(|_| ApiError::InvalidWirePayload)?;

        match self.run_state {
            AnalysisRunTerminalState::Succeeded => self.validate_succeeded_shape(),
            AnalysisRunTerminalState::Failed => self.validate_failed_shape(),
        }
    }

    fn validate_suceeded_shape(&self) -> Result<(), ApiError> {
        let artifact_id = self
            .result_artifact_id
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let digest = self
            .result_sha256
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let schema_version = self
            .result_schema_version
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let summary = self
            .summary
            .as_ref()
            .ok_or(ApiError::InvalidWirePayload)?;
        require_nonempty(artifact_id)?;
        require_nonempty(schema_version)?;
        require_canonical_sha256(digest)?;
        summary.validate()?;
        if self.failure_code.is_some() {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }

    fn validate_failed_shape(&self) -> Result<(), ApiError> {
        if self.result_artifact_id.is_some()
            || self.result_sha256.is_some()
            || self.result_schema_version.is_some()
            || self.summary.is_some()
        {
            return Err(ApiError::InvalidWirePayload);
        }
        let failure_code = self
            .failure_code
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        require_failure_code(failure_code)
    }
}

/// Return whether a terminal outcome exactly binds to its submitted request.
#[must_use]
pub fn terminal_result_matches_request(
    request: &AnalysisRunRequest,
    result: &AnalysisRunTerminalResult,
) -> bool {
    result.idempotency_key == request.idempotency_key
        && result.tenant_workspace_id == request.tenant_workspace_id
        && result.snapshot_id == request.snapshot_id
        && result.knowledge_cutoff == request.knowledge_cutoff
        && result.model_contract_version == request.model_contract_version
        && result.output_profile == request.output_profile
}

/// Return whether a terminal outcome exactly binds to an accepted receipt.
#[must_use]
pub fn terminal_result_matches_accepted(
    accepted: &AnalysisRunAccepted,
    result: &AnalysisRunTerminalResult,
) -> bool {
    result.run_id == accepted.run_id && result.idempotency_key == accepted.idempotency_key
}

/// Require exact request and accepted-receipt binding for a terminal outcome.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] if either binding differs.
pub fn require_terminal_binding(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    result: &AnalysisRunTerminalResult,
) -> Result<(), ApiError> {
    if terminal_result_matches_request(request, result)
        && terminal_result_matches_accepted(accepted, result)
    {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn require_canonical_sha256(value: &str) -> Result<(), ApiError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn require_failure_code(value: &str) -> Result<(), ApiError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > MAXIMUM_FAILURE_CODE_BYTES
        || !bytes[0].is_ascii_lowercase()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ANALYSIS_RESULT_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunTerminalResult,
        AnalysisRunTerminalState, DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT, require_terminal_binding,
        terminal_result_matches_accepted, terminal_result_matches_request,
    };
    use crate::{
        ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunRequest, ApiError,
    };

    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn sample_request() -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: "idem-1".into(),
            tenant_workspace_id: "tenant-ws-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "temporal-model-v1".into(),
            output_profile: "validation-report".into(),
        }
    }

    fn sample_accepted() -> AnalysisRunAccepted {
        AnalysisRunAccepted::new("run-1", "accepted", "idem-1").expect("accepted")
    }

    fn sample_summary() -> AnalysisResultSummary {
        AnalysisResultSummary::new("temporal_topic_measurement", 120, 42, "validated")
            .expect("summary")
    }

    fn sample_succeeded() -> AnalysisRunTerminalResult {
        AnalysisRunTerminalResult::succeeded(
            &sample_request(),
            &sample_accepted(),
            "artifact-1",
            DIGEST,
            "tepp-result-v1",
            "2026-08-02T03:04:05Z",
            sample_summary(),
        )
        .expect("succeeded")
    }

    #[test]
    fn succeeded_result_round_trips_and_binds_request_and_receipt() {
        let request = sample_request();
        let accepted = sample_accepted();
        let result = AnalysisRunTerminalResult::succeeded(
            &request,
            &accepted,
            "artifact-1",
            DIGEST,
            "tepp-result-v1",
            "2026-08-02T03:04:05+00:00",
            sample_summary(),
        )
        .expect("succeed");
        assert_eq!(result.run_state, AnalysisRunTerminalState::Succeeded);
        assert!(terminal_result_matches_request(&request, &result));
        assert!(terminal_result_matches_accepted(&accepted, &result));
        assert_eq!(require_terminal_binding(&request, &accepted, &result), Ok(()));
        let json = result.to_json().expect("json");
        assert_eq!(
            AnalysisRunTerminalResult::from_json(&ajson).expect("decoded"),
            result
        );
        assert!(DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT >= json.len());
    }

    #[test]
    fn failed_result_round_trips_without_measurement_artifact() {
        let request = sample_request();
        let accepted = sample_accepted();
        let result = AnalysisRunTerminalResult::failed(
            &request,
            &accepted,
            "2026-08-02T03:04:05Z",
            "estimation_failed",
        )
        .expect("failed");
        assert_eq!(result.run_state, AnalysisRunTerminalState::Failed);
        assert_eq!(result.result_artifact_id, None);
        assert_eq!(result.summary, None);
        let json = result.to_json().expect("json");
        assert_eq!(
            AnalysisRunTerminalResult::from_json(&ajson).expect("decoded"),
            result
        );
    }

    #[test]
    fn accepted_receipt_and_extended_or_oversized_payloads_fail_closed() {
        let accepted_json = sample_accepted().to_json().expect("accepted json");
        assert_eq!(
            AnalysisRunTerminalResult::from_json(&accepted_json),
            Err(ApiError::InvalidWirePayload)
        );

        let mut value: serde_json::Value =
            serde_json::from_str(&sample_succeeded().to_json().expect( "json" )).expect("value");
        value["extra"] = serde_json::json!(true);
        assert_eq!(
            AnalysisRunTerminalResult::from_json(&value.to_string()),
            Err(ApiError::InvalidWirePayload)
        );

        let json = sample_succeeded().to_json().expect("json");
        assert_eq!(
            AnalysisRunTerminalResult::from_json_with_limit(&json, 8),
            Err(ApiError::LimitExceeded)
        );
    }

    #[test]
    fn version_required_fields_and_timestamps_fail_closed() {
        let mut result = sample_succeeded();
        result.contract_version = ANALYSIS_RESULT_CONTRACT_VERSION + 1;
        assert_eq!(result.to_json(), Err(ApiError::UnsupportedContractVersion));

        for clear in 0..8 {
            let mut invalid = sample_succeeded();
            match clear {
                0 => invalid.run_id.clear(),
                1 => invalid.idempotency_key.clear(),
                2 => invalid.tenant_workspace_id.clear(),
                3 => invalid.snapshot_id.clear(),
                4 => invalid.knowledge_cutoff.clear(),
                5 => invalid.model_contract_version.clear(),
                6 => invalid.output_profile.clear(),
                7 => invalid.completed_at.clear(),
                _ => unreachable!(),
            }
            assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));
        }

        let mut invalid_cutoff = sample_succeeded();
        invalid_cutoff.knowledge_cutoff = "yesterday".into();
        assert_eq!(invalid_cutoff.to_json(), Err(ApiError::InvalidWirePayload));

        let mut invalid_completion = sample_succeeded();
        invalid_completion.completed_at = "2026-99-99T25:00:00Z".into();
        assert_eq!(invalid_completion.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn succeeded_shape_requires_complete_digest_bound_result_and_no_failure() {
        let mut result = sample_succeeded();
        result.result_artifact_id = None;
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut result = sample_succeeded();
        result.result_artifact_id = Some(String::new());
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut result = sample_succeeded();
        result.result_sha256 = None;
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        for digest in [
            "abcd".to_string(),
            DIGEST.to_uppercase(),
            format!("{DIGEST}0"),
        ] {
            let mut result = sample_succeeded();
            result.result_sha256 = Some(digest);
            assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));
        }

        let mut result = sample_succeeded();
        result.result_schema_version = None;
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut result = sample_succeeded();
        result.result_schema_version = Some(String::new());
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut result = sample_succeeded();
        result.summary = None;
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));

        let mut result = sample_succeeded();
        result.failure_code = Some("unexpected_failure".into();
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn summary_is_bounded_and_nonempty() {
        assert_eq!(
            AnalysisResultSummary::new("", 0, 0, "validated"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisResultSummary::new("family", 0, 0, ""),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisResultSummary::new("family", 1_000_000_001, 0, "validated"),
            Err(ApiError::LimitExceeded)
        );
        assert_eq!(
            AnalysisResultSummary::new("family", 0, 1_000_000_001, "validated"),
            Err(ApiError::LimitExceeded)
        );

        let mut result = sample_succeeded();
        result.summary = Some(AnalysisResultSummary {
            analysis_family: String::new(),
            evidence_count: 0,
            statistic_count: 0,
            validation_status: "validated".into(),
        });
        assert_eq!(result.to_json(), Err(ApiError::InvalidWirePayload));
    }

    #[test]
    fn failed_shape_refuses_result_fields_and_invalid_failure_codes() {
        let request = sample_request();
        let accepted = sample_accepted();
        let base = AnalysisRunTerminalResult::failed(
            &request,
            &accepted,
            "2026-08-02T03:04:05Z",
            "provider_timeout",
        )
        .expect("failed");

        let mut with_artifact = base.clone();
        with_artifact.result_artifact_id = Some("artifact".into());
        assert_eq!(with_artifact.to_json(), Err(ApiError::InvalidWirePayload));

        let mut with_digest = base.clone();
        with_digest.result_sha256 = Some(DIGEST.into();
        assert_eq!(with_digest.to_json(), Err(ApiError::InvalidWirePayload));

        let mut with_schema = base.clone();
        with_schema.result_schema_version = Some("schema".into();
        assert_eq!(with_schema.to_json(), Err(ApiError:InvalidWirePayload));

        let mut with_summary = base.clone();
        with_summary.summary = Some(sample_summary());
        assert_eq!(with_summary.to_json(), Err(ApiError::InvalidWirePayload));

        for failure_code in [
            None,
            Some(String::new()),
            Some("UPPER_CASE".into()),
            Some("_leading".into(),
            Some("contains-hyphen".into()),
            Some("x".repeat(65)),
        ] {
            let mut invalid = base.clone();
            invalid.failure_code = failure_code;
            assert_eq!(invalid.to_json(), Err(ApiError:InvalidWirePayload));
        }
    }

    #[test]
    fn request_and_acceptance_mismatches_are_rejected() {
        let request = sample_request();
        let accepted = sample_accepted();
        let result = sample_succeeded();

        let mut mismatched_request = request.clone();
        mismatched_request.snapshot_id = "other-snapshot".into();
        assert!(!terminal_result_matches_request(&mismatched_request, &result));
        assert_eq!(
            require_terminal_binding(&mismatched_request, &accepted, &result),
            Err(ApiError:InvalidWirePayload)
        );

        let mismatched_accepted =
            AnalysisRunAccepted::new("other-run", "accepted", "idem-1").expect("accepted");
        assert!(!terminal_result_matches_accepted(&mismatched_accepted, &result));
        assert_eq!(
            require_terminal_binding(&request, &mismatched_accepted, &result),
            Err(ApiError::InvalidWirePayload)
        );

        let mismatched_idempotency =
            AnalysisRunAccepted::new(brun-1", "accepted", "other-idem").expect("accepted");
        assert!(!terminal_result_matches_accepted(&mismatched_idempotency, &result));
        assert_eq!(
            AnalysisRunTerminalResult::succeeded(
                &request,
                &mismatched_idempotency,
                "artifact-1",
                DIGEST,
                "tepp-result-v1",
                "2026-08-02T03:04:05Z",
                sample_summary(),
            ),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunTerminalResult::failed(
                &request,
                &mismatched_idempotency,
                "2026-08-02T03:04:05Z",
                "provider_timeout",
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }
}
