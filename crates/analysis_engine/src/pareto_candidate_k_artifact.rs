//! Digest-bound Pareto candidate-`K` selection as an analysis-run profile.

use model_selection::{ModelCandidate, select_candidate_k, selected_k_root_mean_square_error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

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
// `select_candidate_k` performs an O(n^2) dominance scan. This application-path
// ceiling bounds one request to at most 65,536 ordered candidate comparisons;
// it is an operational resource contract, not a scientific claim about valid K.
const MAX_PARETO_CANDIDATES: usize = 256;

/// Provenance-bound Pareto-front input over one historical evidence universe.
///
/// `source_evidence_available_times` is the complete availability-time vector
/// for the evidence universe used to construct both the candidate diagnostics
/// and the selected-`K` replications. Construction fails closed when any source
/// evidence was unavailable at the bound knowledge cutoff. The engine therefore
/// never attempts to subtract future evidence from already-aggregated model
/// diagnostics.
#[derive(Clone, Debug, PartialEq)]
pub struct ParetoCandidateKInput {
    snapshot_id: String,
    knowledge_cutoff: KnowledgeCutoff,
    source_evidence_available_times: Vec<AvailableTime>,
    candidates: Vec<ModelCandidate>,
    selected_replications: Vec<u32>,
    truth_k: u32,
}

impl ParetoCandidateKInput {
    /// Construct a Pareto-front selection payload with exact historical
    /// provenance for the evidence used to create its diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] for invalid snapshot
    /// identity, empty evidence provenance, or post-cutoff source evidence.
    /// Returns [`AnalysisEngineError::LimitExceeded`] before selection when the
    /// evidence, replication, or O(n²) candidate population exceeds its
    /// application-path bound.
    pub fn new(
        snapshot_id: impl Into<String>,
        knowledge_cutoff: KnowledgeCutoff,
        source_evidence_available_times: Vec<AvailableTime>,
        candidates: Vec<ModelCandidate>,
        selected_replications: Vec<u32>,
        truth_k: u32,
    ) -> Result<Self, AnalysisEngineError> {
        let value = Self {
            snapshot_id: snapshot_id.into(),
            knowledge_cutoff,
            source_evidence_available_times,
            candidates,
            selected_replications,
            truth_k,
        };
        value.validate_provenance()?;
        Ok(value)
    }

    /// Return the immutable snapshot identity of the diagnostic evidence.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the knowledge cutoff used when constructing the diagnostics.
    #[must_use]
    pub const fn knowledge_cutoff(&self) -> KnowledgeCutoff {
        self.knowledge_cutoff
    }

    /// Return the number of source evidence units represented by the
    /// candidate diagnostics and replication RMSE.
    #[must_use]
    pub fn evidence_count(&self) -> usize {
        self.source_evidence_available_times.len()
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

    fn validate_provenance(&self) -> Result<(), AnalysisEngineError> {
        if !valid_identifier(&self.snapshot_id) || self.source_evidence_available_times.is_empty() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if self.source_evidence_available_times.len() > MAX_EVIDENCE_UNITS
            || self.selected_replications.len() > MAX_EVIDENCE_UNITS
            || self.candidates.len() > MAX_PARETO_CANDIDATES
        {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        if self
            .source_evidence_available_times
            .iter()
            .any(|available_time| available_time.instant() > self.knowledge_cutoff.instant())
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(())
    }

    fn validate_against(
        &self,
        snapshot_id: &str,
        knowledge_cutoff: KnowledgeCutoff,
    ) -> Result<(), AnalysisEngineError> {
        self.validate_provenance()?;
        if self.snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::SnapshotMismatch);
        }
        if self.knowledge_cutoff.instant() != knowledge_cutoff.instant() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(())
    }
}

/// Completed, bounded Pareto candidate-`K` selection for analysis-run clients.
///
/// Fields are private so a validated completed artifact cannot be mutated into
/// an unchecked in-memory state after execution. Consumers read through the
/// accessors and obtain untrusted artifacts through [`Self::from_json`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParetoCandidateKArtifact {
    schema_version: String,
    run_id: String,
    snapshot_id: String,
    knowledge_cutoff: String,
    selected_k: u64,
    candidate_count: u64,
    statistical_count: u64,
    truth_k: u64,
    selected_k_rmse: f64,
    inference_status: String,
}

impl ParetoCandidateKArtifact {
    /// Return the exact versioned schema identity.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Return the opaque accepted-run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Return the immutable source snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return the canonical historical evidence cutoff.
    #[must_use]
    pub fn knowledge_cutoff(&self) -> &str {
        &self.knowledge_cutoff
    }

    /// Return the statistically selected topic count `K`.
    #[must_use]
    pub const fn selected_k(&self) -> u64 {
        self.selected_k
    }

    /// Return the number of candidates offered to the Pareto gate.
    #[must_use]
    pub const fn candidate_count(&self) -> u64 {
        self.candidate_count
    }

    /// Return the number of statistically supported candidates.
    #[must_use]
    pub const fn statistical_count(&self) -> u64 {
        self.statistical_count
    }

    /// Return the known-truth topic count used for RMSE.
    #[must_use]
    pub const fn truth_k(&self) -> u64 {
        self.truth_k
    }

    /// Return the RMSE of selected-`K` replications against known truth.
    #[must_use]
    pub const fn selected_k_rmse(&self) -> f64 {
        self.selected_k_rmse
    }

    /// Return the fixed scientific claim boundary.
    #[must_use]
    pub fn inference_status(&self) -> &str {
        &self.inference_status
    }

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
            || self.candidate_count > MAX_PARETO_CANDIDATES as u64
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
/// dominance or RMSE. Candidate diagnostics and RMSE replications are admitted
/// only when their construction provenance is bound to this snapshot and cutoff.
/// LLM votes cannot define the numerical optimum. This is not Schwarz fitted
/// selection, not a Bayesian sampler, and not GPU execution.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, provenance or
/// resource-bound failure, model-selection failure, or invalid artifact error.
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
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION
        || request.output_profile != PARETO_CANDIDATE_K_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    input.validate_against(snapshot_id, knowledge_cutoff)?;

    let candidate_count = u64::try_from(input.candidates().len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let evidence_count = u64::try_from(input.evidence_count())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let selected_k = u64::from(select_candidate_k(input.candidates())?);
    let selected_k_rmse =
        selected_k_root_mean_square_error(input.selected_replications(), input.truth_k())?;
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
    let summary =
        AnalysisResultSummary::new("pareto_candidate_k", evidence_count, 2, "validated")?;
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
    use temporal_core::{AvailableTime, KnowledgeCutoff};

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

    fn cutoff() -> KnowledgeCutoff {
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
    }

    fn available() -> AvailableTime {
        AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available")
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
        assert_eq!(artifact.schema_version(), PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION);
        assert_eq!(artifact.run_id(), "run-1");
        assert_eq!(artifact.snapshot_id(), "snapshot-1");
        assert_eq!(artifact.knowledge_cutoff(), "2026-08-01T00:00:00Z");
        assert_eq!(artifact.selected_k(), 2);
        assert_eq!(artifact.candidate_count(), 2);
        assert_eq!(artifact.statistical_count(), 2);
        assert_eq!(artifact.truth_k(), 2);
        assert_eq!(artifact.selected_k_rmse(), 0.0);
        assert_eq!(artifact.inference_status(), PARETO_CANDIDATE_K_INFERENCE_STATUS);
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
                value.candidate_count = 257;
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
    fn input_accessors_expose_provenance_candidates_and_truth() {
        let a = ModelCandidate::statistical(2, -30.0, 8.0).expect("a");
        let input = ParetoCandidateKInput::new(
            "snapshot-1",
            cutoff(),
            vec![available()],
            vec![a],
            vec![2],
            2,
        )
        .expect("input");
        assert_eq!(input.snapshot_id(), "snapshot-1");
        assert_eq!(input.knowledge_cutoff(), cutoff());
        assert_eq!(input.evidence_count(), 1);
        assert_eq!(input.candidates(), &[a]);
        assert_eq!(input.selected_replications(), &[2]);
        assert_eq!(input.truth_k(), 2);
    }
}
