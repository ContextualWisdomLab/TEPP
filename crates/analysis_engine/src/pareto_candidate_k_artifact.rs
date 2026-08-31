//! Digest-bound Pareto candidate-`K` selection as an analysis-run profile.

use model_selection::{ModelCandidate, select_candidate_k, selected_k_root_mean_square_error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed Pareto candidate-`K` artifact.
pub const PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION: &str = "tepp.pareto_candidate_k.v1";
/// Model contract required by the Pareto candidate-`K` execution path.
pub const PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION: &str = "pareto_candidate_k_v1";
/// Analysis-run output profile required for a Pareto candidate-`K` artifact.
pub const PARETO_CANDIDATE_K_OUTPUT_PROFILE: &str = "pareto_candidate_k_v1";
/// Maximum canonical artifact JSON size.
pub const PARETO_CANDIDATE_K_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const PARETO_CANDIDATE_K_INFERENCE_STATUS: &str =
    "pareto_statistical_front_not_fitted_schwarz_sampler";

/// Cutoff-safe Pareto-front input bound to offered candidates and known truth.
#[derive(Clone, Debug, PartialEq)]
pub struct ParetoCandidateKInput {
    candidates: Vec<ModelCandidate>,
    selected_replications: Vec<u32>,
    truth_k: u32,
}

impl ParetoCandidateKInput {
    /// Construct a Pareto-front selection payload.
    #[must_use]
    pub fn new(
        candidates: Vec<ModelCandidate>,
        selected_replications: Vec<u32>,
        truth_k: u32,
    ) -> Self {
        Self {
            candidates,
            selected_replications,
            truth_k,
        }
    }

    /// Borrow the offered candidates.
    #[must_use]
    pub fn candidates(&self) -> &[ModelCandidate] {
        &self.candidates
    }

    /// Borrow selected-`K` replications used for RMSE.
    #[must_use]
    pub fn selected_replications(&self) -> &[u32] {
        &self.selected_replications
    }

    /// Return the known-truth topic count.
    #[must_use]
    pub const fn truth_k(&self) -> u32 {
        self.truth_k
    }
}

/// Completed, bounded Pareto candidate-`K` selection for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParetoCandidateKArtifact {
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
    /// Number of candidates offered to the Pareto gate.
    pub candidate_count: u64,
    /// Number of statistically supported candidates.
    pub statistical_count: u64,
    /// Known-truth topic count used for RMSE.
    pub truth_k: u64,
    /// RMSE of selected-`K` replications against known truth.
    pub selected_k_rmse: f64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl ParetoCandidateKArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidParetoCandidateKArtifact`] when the
    /// schema, identifiers, counts, RMSE, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > PARETO_CANDIDATE_K_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidParetoCandidateKArtifact)?;
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
        if payload.len() > PARETO_CANDIDATE_K_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.selected_k < 2
            || self.candidate_count == 0
            || self.statistical_count == 0
            || self.statistical_count > self.candidate_count
            || self.truth_k < 2
            || !self.selected_k_rmse.is_finite()
            || self.selected_k_rmse < 0.0
            || self.inference_status != PARETO_CANDIDATE_K_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidParetoCandidateKArtifact);
        }
        Ok(())
    }
}

/// One completed Pareto candidate-`K` artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct ParetoCandidateKExecution {
    /// Digest-bound completed selection artifact.
    pub artifact: ParetoCandidateKArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe Pareto candidate-`K` selection as one analysis-run profile.
///
/// The executor invokes [`select_candidate_k`] and
/// [`selected_k_root_mean_square_error`] and does not reimplement Pareto
/// dominance or RMSE. LLM votes cannot define the numerical optimum. This is
/// not Schwarz fitted selection, not a Bayesian sampler, and not GPU execution.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, model-selection
/// failure, or invalid artifact error.
pub fn execute_pareto_candidate_k_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ParetoCandidateKInput,
    completed_at: impl Into<String>,
) -> Result<ParetoCandidateKExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION
        || request.output_profile != PARETO_CANDIDATE_K_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let selected_k = u64::from(select_candidate_k(input.candidates())?);
    let selected_k_rmse =
        selected_k_root_mean_square_error(input.selected_replications(), input.truth_k())?;
    let candidate_count = u64::try_from(input.candidates().len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let statistical_count = u64::try_from(
        input
            .candidates()
            .iter()
            .filter(|candidate| candidate.is_statistically_supported())
            .count(),
    )
    .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let artifact = ParetoCandidateKArtifact {
        schema_version: PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        selected_k,
        candidate_count,
        statistical_count,
        truth_k: u64::from(input.truth_k()),
        selected_k_rmse,
        inference_status: PARETO_CANDIDATE_K_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "pareto_candidate_k",
        candidate_count,
        2,
        PARETO_CANDIDATE_K_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("pareto_candidate_k_artifact_{}", &digest[..16]),
        digest,
        PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(ParetoCandidateKExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        PARETO_CANDIDATE_K_ARTIFACT_BYTE_LIMIT, PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
        PARETO_CANDIDATE_K_INFERENCE_STATUS, ParetoCandidateKArtifact, ParetoCandidateKInput,
    };
    use crate::AnalysisEngineError;
    use model_selection::ModelCandidate;

    fn artifact() -> ParetoCandidateKArtifact {
        ParetoCandidateKArtifact {
            schema_version: PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            selected_k: 2,
            candidate_count: 2,
            statistical_count: 2,
            truth_k: 2,
            selected_k_rmse: 0.0,
            inference_status: PARETO_CANDIDATE_K_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &ParetoCandidateKArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidParetoCandidateKArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            ParetoCandidateKArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            ParetoCandidateKArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidParetoCandidateKArtifact)
        );
        assert_eq!(
            ParetoCandidateKArtifact::from_json(
                &"x".repeat(PARETO_CANDIDATE_K_ARTIFACT_BYTE_LIMIT + 1)
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
                value.statistical_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.statistical_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.truth_k = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.selected_k_rmse = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.selected_k_rmse = -0.1;
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

    #[test]
    fn input_accessors_expose_candidates_and_truth() {
        let a = ModelCandidate::statistical(2, -30.0, 8.0).expect("a");
        let input = ParetoCandidateKInput::new(vec![a], vec![2], 2);
        assert_eq!(input.candidates(), &[a]);
        assert_eq!(input.selected_replications(), &[2]);
        assert_eq!(input.truth_k(), 2);
    }
}
