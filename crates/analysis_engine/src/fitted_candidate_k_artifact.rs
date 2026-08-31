//! Digest-bound fitted candidate-`K` selection as an analysis-run profile.

use model_selection::{FittedCandidateKConfig, select_fitted_candidate_k};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_measurement::ReferenceTopicInput;

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed fitted candidate-`K` artifact.
pub const FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION: &str = "tepp.fitted_candidate_k.v1";
/// Model contract required by the fitted candidate-`K` execution path.
pub const FITTED_CANDIDATE_K_MODEL_CONTRACT_VERSION: &str = "fitted_candidate_k_v1";
/// Analysis-run output profile required for a fitted candidate-`K` artifact.
pub const FITTED_CANDIDATE_K_OUTPUT_PROFILE: &str = "fitted_candidate_k_v1";
/// Maximum canonical artifact JSON size.
pub const FITTED_CANDIDATE_K_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const FITTED_CANDIDATE_K_INFERENCE_STATUS: &str = "fitted_schwarz_candidate_k_not_bayesian_sampler";

/// Completed, bounded fitted candidate-`K` selection for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FittedCandidateKArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the selection.
    pub knowledge_cutoff: String,
    /// Statistically selected topic count `K`.
    pub selected_k: u64,
    /// Number of candidate topic counts offered to the selector.
    pub candidate_count: u64,
    /// Number of modeled evidence documents.
    pub evidence_count: u64,
    /// Declared statistical method identity (not an LLM label).
    pub method_name: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl FittedCandidateKArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidFittedCandidateKArtifact`] when the
    /// schema, identifiers, counts, method, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > FITTED_CANDIDATE_K_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidFittedCandidateKArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, serialization, or size failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > FITTED_CANDIDATE_K_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        if self.schema_version != FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.selected_k < 2
            || self.candidate_count == 0
            || self.evidence_count < 2
            || !valid_identifier(&self.method_name)
            || self.inference_status != FITTED_CANDIDATE_K_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidFittedCandidateKArtifact);
        }
        Ok(())
    }
}

/// One completed fitted candidate-`K` artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedCandidateKExecution {
    /// Digest-bound completed selection artifact.
    pub artifact: FittedCandidateKArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe fitted candidate-`K` selection as one analysis-run profile.
///
/// The executor invokes [`select_fitted_candidate_k`] and does not reimplement
/// Schwarz scoring, Pareto admission, or the CPU `f64` reference fit. LLM votes
/// cannot define the numerical optimum. This is not a Bayesian sampler, not GPU
/// execution, and not topic birth/split/merge.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, model-selection
/// failure, or invalid artifact error.
#[allow(
    clippy::too_many_arguments,
    reason = "audited cutoff, method, vote, and selection-config gates"
)]
pub fn execute_fitted_candidate_k_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    config: &FittedCandidateKConfig,
    method_name: &str,
    llm_votes: &[u32],
    completed_at: impl Into<String>,
) -> Result<FittedCandidateKExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != FITTED_CANDIDATE_K_MODEL_CONTRACT_VERSION
        || request.output_profile != FITTED_CANDIDATE_K_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if !valid_identifier(method_name) {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let selected_k = u64::from(select_fitted_candidate_k(
        input,
        config,
        method_name,
        llm_votes,
    )?);
    let candidate_count = u64::try_from(config.candidate_topic_counts().len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let evidence_count = u64::try_from(input.document_count())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = FittedCandidateKArtifact {
        schema_version: FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        selected_k,
        candidate_count,
        evidence_count,
        method_name: method_name.to_owned(),
        inference_status: FITTED_CANDIDATE_K_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "fitted_candidate_k",
        evidence_count,
        2,
        FITTED_CANDIDATE_K_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("fitted_candidate_k_artifact_{}", &digest[..16]),
        digest,
        FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(FittedCandidateKExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        FITTED_CANDIDATE_K_ARTIFACT_BYTE_LIMIT, FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
        FITTED_CANDIDATE_K_INFERENCE_STATUS, FittedCandidateKArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> FittedCandidateKArtifact {
        FittedCandidateKArtifact {
            schema_version: FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            selected_k: 2,
            candidate_count: 2,
            evidence_count: 6,
            method_name: "trsl_tm_reference".into(),
            inference_status: FITTED_CANDIDATE_K_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &FittedCandidateKArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidFittedCandidateKArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            FittedCandidateKArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            FittedCandidateKArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidFittedCandidateKArtifact)
        );
        assert_eq!(
            FittedCandidateKArtifact::from_json(
                &"x".repeat(FITTED_CANDIDATE_K_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn artifact_metadata_tampering_fails_closed() {
        let artifact = artifact();
        let invalid_artifacts = [
            {
                let mut value = artifact.clone();
                value.schema_version.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.run_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.snapshot_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.knowledge_cutoff = "invalid".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.selected_k = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.candidate_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.evidence_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.method_name.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.inference_status.clear();
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }
}
