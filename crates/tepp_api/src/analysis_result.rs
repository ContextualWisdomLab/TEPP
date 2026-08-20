//! Versioned terminal analysis-run result contracts.
//!
//! Submission acceptance and scientific completion are separate facts.
//! [`AnalysisRunAccepted`] is only a durable receipt. This module defines a
//! distinct, request-bound terminal result with a digest-bound artifact or a
//! redacted failure code.

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
    /// Computation completed with a digest-bound result artifact.
    Succeeded,
    /// Computation ended without a result artifact.
    Failed,
}

/// Bounded, identity-free summary of one completed measurement artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResultSummary {
    /// Versioned analysis family.
    pub analysis_family: String,
    /// Number of evidence units represented by the result.
    pub evidence_count: u64,
    /// Number of reported statistics or parameters.
    pub statistic_count: u64,
    /// Provider-authored validation status.
    pub validation_status: String,
}

impl AnalysisResultSummary {
    /// Construct and validate a bounded, identity-free summary.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed contract error for empty labels or unbounded
    /// counts.
    pub fn new(
        analysis_family: impl Into<String>,
        evidence_count: u64,
        statistic_count: u64,
        validation_status: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let value = Self {
            analysis_family: analysis_family.into(),
            evidence_count,
            statistic_count,
            validation_status: validation_status.into(),
        };
        value.validate()?;
        Ok(value)
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

/// Request-bound terminal outcome for one accepted analysis run.
///
/// The succeeded shape excludes source text, credentials, direct identity,
/// respondent/item records, and unrestricted model output. The failed shape
/// contains no measurement artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRunTerminalResult {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Opaque remote run identity from [`AnalysisRunAccepted`].
    pub run_id: String,
    /// Terminal lifecycle state.
    pub run_state: AnalysisRunTerminalState,
    /// Exact request idempotency key.
    pub idempotency_key: String,
    /// Authorized tenant/workspace opaque identity.
    pub tenant_workspace_id: String,
    /// Immutable corpus/evidence snapshot identity.
    pub snapshot_id: String,
    /// Exact request knowledge cutoff.
    pub knowledge_cutoff: String,
    /// Exact model/backend contract identity.
    pub model_contract_version: String,
    /// Exact requested output profile.
    pub output_profile: String,
    /// Opaque result artifact identity for a succeeded run.
    pub result_artifact_id: Option<String>,
    /// Canonical lowercase SHA-256 result digest.
    pub result_sha256: Option<String>,
    /// Versioned result-schema identity.
    pub result_schema_version: Option<String>,
    /// Strict RFC 3339 system time at terminal completion.
    pub completed_at: String,
    /// Bounded summary for a succeeded run.
    pub summary: Option<AnalysisResultSummary>,
    /// Stable snake-case code for a failed run.
    pub failure_code: Option<String>,
}

impl AnalysisRunTerminalResult {
    /// Construct a succeeded terminal result bound to request and receipt.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for invalid shape, digest, time, summary, or
    /// request/receipt binding.
    pub fn succeeded(
        request: &AnalysisRunRequest,
        accepted: &AnalysisRunAccepted,
        result_artifact_id: impl Into<String>,
        result_sha256: impl Into<String>,
        result_schema_version: impl Into<String>,
        completed_at: impl Into<String>,
        summary: AnalysisResultSummary,
    ) -> Result<Self, ApiError> {
        let value = Self {
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
        value.validate()?;
        require_terminal_binding(request, accepted, &value)?;
        Ok(value)
    }

    /// Construct a failed terminal result bound to request and receipt.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for invalid time, failure code, or
    /// request/receipt binding.
    pub fn failed(
        request: &AnalysisRunRequest,
        accepted: &AnalysisRunAccepted,
        completed_at: impl Into<String>,
        failure_code: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let value = Self {
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
        value.validate()?;
        require_terminal_binding(request, accepted, &value)?;
        Ok(value)
    }

    /// Parse and validate a terminal result with the default byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, time, digest, shape, or field errors.
    pub fn from_json(payload: &str) -> Result<Self, ApiError> {
        Self::from_json_with_limit(payload, DEFAULT_ANALYSIS_RESULT_BYTE_LIMIT)
    }

    /// Parse and validate a terminal result with a caller-supplied byte limit.
    ///
    /// # Errors
    ///
    /// Returns wire, version, limit, time, digest, shape, or field errors.
    pub fn from_json_with_limit(payload: &str, maximum_bytes: usize) -> Result<Self, ApiError> {
        require_byte_limit(payload, maximum_bytes)?;
        let value: Self = from_json(payload)?;
        value.validate()?;
        Ok(value)
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

    pub(crate) fn validate(&self) -> Result<(), ApiError> {
        require_contract_version(self.contract_version, ANALYSIS_RESULT_CONTRACT_VERSION)?;
        for value in [
            &self.run_id,
            &self.idempotency_key,
            &self.tenant_workspace_id,
            &self.snapshot_id,
            &self.knowledge_cutoff,
            &self.model_contract_version,
            &self.output_profile,
            &self.completed_at,
        ] {
            require_nonempty(value)?;
        }
        KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        SystemTime::parse_rfc3339(&self.completed_at).map_err(|_| ApiError::InvalidWirePayload)?;

        match self.run_state {
            AnalysisRunTerminalState::Succeeded => self.validate_succeeded(),
            AnalysisRunTerminalState::Failed => self.validate_failed(),
        }
    }

    fn validate_succeeded(&self) -> Result<(), ApiError> {
        let artifact_id = self
            .result_artifact_id
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let digest = self
            .result_sha256
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let schema = self
            .result_schema_version
            .as_deref()
            .ok_or(ApiError::InvalidWirePayload)?;
        let summary = self.summary.as_ref().ok_or(ApiError::InvalidWirePayload)?;
        require_nonempty(artifact_id)?;
        require_nonempty(schema)?;
        require_canonical_sha256(digest)?;
        summary.validate()?;
        if self.failure_code.is_some() {
            return Err(ApiError::InvalidWirePayload);
        }
        Ok(())
    }

    fn validate_failed(&self) -> Result<(), ApiError> {
        if self.result_artifact_id.is_some()
            || self.result_sha256.is_some()
            || self.result_schema_version.is_some()
            || self.summary.is_some()
        {
            return Err(ApiError::InvalidWirePayload);
        }
        require_failure_code(
            self.failure_code
                .as_deref()
                .ok_or(ApiError::InvalidWirePayload)?,
        )
    }
}

/// Return whether a terminal result exactly binds to its submitted request.
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

/// Return whether a terminal result exactly binds to an accepted receipt.
#[must_use]
pub fn terminal_result_matches_accepted(
    accepted: &AnalysisRunAccepted,
    result: &AnalysisRunTerminalResult,
) -> bool {
    result.run_id == accepted.run_id && result.idempotency_key == accepted.idempotency_key
}

/// Require exact request and accepted-receipt binding.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when either binding differs.
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
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}

fn require_failure_code(value: &str) -> Result<(), ApiError> {
    let bytes = value.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= MAXIMUM_FAILURE_CODE_BYTES
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_');
    if valid {
        Ok(())
    } else {
        Err(ApiError::InvalidWirePayload)
    }
}
