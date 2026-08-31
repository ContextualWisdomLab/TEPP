//! Digest-bound fitted candidate-`K` selection composed with topic lineage.

use model_selection::{FittedCandidateKConfig, select_fitted_candidate_model};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_measurement::ReferenceTopicInput;

use crate::topic_lineage_artifact::{
    TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TOPIC_LINEAGE_OUTPUT_PROFILE,
    topic_lineage_execution_from_model,
};
use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed composed fitted-lineage artifact.
pub const COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.composed_fitted_lineage.v1";
/// Model contract required by the composed fitted-lineage execution path.
pub const COMPOSED_FITTED_LINEAGE_MODEL_CONTRACT_VERSION: &str = "composed_fitted_lineage_v1";
/// Analysis-run output profile required for a composed fitted-lineage artifact.
pub const COMPOSED_FITTED_LINEAGE_OUTPUT_PROFILE: &str = "composed_fitted_lineage_v1";
/// Maximum canonical artifact JSON size.
pub const COMPOSED_FITTED_LINEAGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS: &str =
    "fitted_k_composed_lineage_not_bayesian_sampler";

/// Cutoff-safe composition payload: fitted selection plus production lineage.
#[derive(Clone, Debug)]
pub struct ComposedFittedLineageInput<'a> {
    input: &'a ReferenceTopicInput,
    selection: &'a FittedCandidateKConfig,
    method_name: &'a str,
    llm_votes: &'a [u32],
}

impl<'a> ComposedFittedLineageInput<'a> {
    /// Construct a composition payload from existing scientific-crate values.
    #[must_use]
    pub const fn new(
        input: &'a ReferenceTopicInput,
        selection: &'a FittedCandidateKConfig,
        method_name: &'a str,
        llm_votes: &'a [u32],
    ) -> Self {
        Self {
            input,
            selection,
            method_name,
            llm_votes,
        }
    }

    /// Borrow the cutoff-safe reference-topic input.
    #[must_use]
    pub const fn input(&self) -> &'a ReferenceTopicInput {
        self.input
    }

    /// Borrow the fitted candidate-`K` configuration.
    #[must_use]
    pub const fn selection(&self) -> &'a FittedCandidateKConfig {
        self.selection
    }

    /// Return the declared statistical method identity.
    #[must_use]
    pub const fn method_name(&self) -> &'a str {
        self.method_name
    }

    /// Borrow optional LLM votes. They cannot define the numerical optimum.
    #[must_use]
    pub const fn llm_votes(&self) -> &'a [u32] {
        self.llm_votes
    }
}

/// Completed, bounded fitted-`K` plus topic-lineage composition.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComposedFittedLineageArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by selection and the lineage fit.
    pub knowledge_cutoff: String,
    /// Statistically selected topic count `K`.
    pub selected_k: u64,
    /// Number of candidate topic counts offered to fitted selection.
    pub candidate_count: u64,
    /// Number of modeled evidence documents.
    pub evidence_count: u64,
    /// Topic count of the production lineage fit at selected `K`.
    pub lineage_topic_count: u64,
    /// Number of fitted same-topic sequence edges.
    pub lineage_edge_count: u64,
    /// Documents incident to at least one fitted sequence edge.
    pub connected_post_count: u64,
    /// SHA-256 digest of the inner topic-lineage artifact.
    pub lineage_artifact_sha256: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl ComposedFittedLineageArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidComposedFittedLineageArtifact`]
    /// when the schema, identifiers, counts, digest, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > COMPOSED_FITTED_LINEAGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidComposedFittedLineageArtifact)?;
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
        if self.schema_version != COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.selected_k < 2
            || self.candidate_count == 0
            || self.evidence_count < 2
            || self.lineage_topic_count != self.selected_k
            || self.connected_post_count > self.evidence_count
            || self.lineage_edge_count > self.evidence_count.saturating_mul(self.evidence_count - 1)
            || self.lineage_artifact_sha256.len() != 64
            || !self
                .lineage_artifact_sha256
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            || self.inference_status != COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidComposedFittedLineageArtifact);
        }
        Ok(())
    }
}

/// One completed composed fitted-lineage artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct ComposedFittedLineageExecution {
    /// Digest-bound composed selection-plus-lineage artifact.
    pub artifact: ComposedFittedLineageArtifact,
    /// Terminal result carrying the composed artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute fitted candidate-`K` selection then the CPU `f64` topic-lineage fit.
///
/// The executor invokes [`select_fitted_candidate_model`] and reuses its exact
/// winning fit to build topic lineage; it does not reimplement Schwarz scoring or
/// lineage edges. LLM votes cannot define the numerical optimum. This is not
/// a standalone fitted-`K` profile, not a Pareto-front profile, and not a
/// Bayesian sampler.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, model-selection
/// failure, estimator failure, or invalid artifact error.
pub fn execute_composed_fitted_lineage_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    composition: &ComposedFittedLineageInput<'_>,
    completed_at: impl Into<String>,
) -> Result<ComposedFittedLineageExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != COMPOSED_FITTED_LINEAGE_MODEL_CONTRACT_VERSION
        || request.output_profile != COMPOSED_FITTED_LINEAGE_OUTPUT_PROFILE
        || !valid_identifier(composition.method_name())
        || !composition.input().is_eligible_at(&knowledge_cutoff)
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let (selected_k, selected_model) = select_fitted_candidate_model(
        composition.input(),
        composition.selection(),
        composition.method_name(),
        composition.llm_votes(),
    )?;
    let mut lineage_request = request.clone();
    lineage_request.model_contract_version = TOPIC_LINEAGE_MODEL_CONTRACT_VERSION.into();
    lineage_request.output_profile = TOPIC_LINEAGE_OUTPUT_PROFILE.into();
    let completed_at = completed_at.into();
    #[rustfmt::skip]
    let lineage = topic_lineage_execution_from_model(&lineage_request, accepted, snapshot_id, knowledge_cutoff, composition.input(), &selected_model, completed_at.clone())?;
    let connected_post_count = lineage.artifact.connected_post_count;
    let artifact = ComposedFittedLineageArtifact {
        schema_version: COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        selected_k: u64::from(selected_k),
        candidate_count: u64::try_from(composition.selection().candidate_topic_counts().len())
            .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
        evidence_count: lineage.artifact.evidence_count,
        lineage_topic_count: lineage.artifact.topic_count,
        lineage_edge_count: u64::try_from(lineage.artifact.sequence_edges.len())
            .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
        connected_post_count,
        lineage_artifact_sha256: lineage.artifact.sha256()?,
        inference_status: COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    #[rustfmt::skip]
    let summary = AnalysisResultSummary::new("composed_fitted_lineage", artifact.evidence_count, 4, COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS)?;
    #[rustfmt::skip]
    let terminal_result = AnalysisRunTerminalResult::succeeded(request, accepted, format!("composed_fitted_lineage_artifact_{}", &digest[..16]), digest, COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION, completed_at, summary)?;
    Ok(ComposedFittedLineageExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        COMPOSED_FITTED_LINEAGE_ARTIFACT_BYTE_LIMIT,
        COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION, COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS,
        ComposedFittedLineageArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> ComposedFittedLineageArtifact {
        ComposedFittedLineageArtifact {
            schema_version: COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            selected_k: 2,
            candidate_count: 2,
            evidence_count: 4,
            lineage_topic_count: 2,
            lineage_edge_count: 2,
            connected_post_count: 4,
            lineage_artifact_sha256: "ab".repeat(32),
            inference_status: COMPOSED_FITTED_LINEAGE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &ComposedFittedLineageArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidComposedFittedLineageArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            ComposedFittedLineageArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            ComposedFittedLineageArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidComposedFittedLineageArtifact)
        );
        assert_eq!(
            ComposedFittedLineageArtifact::from_json(
                &"x".repeat(COMPOSED_FITTED_LINEAGE_ARTIFACT_BYTE_LIMIT + 1)
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
                value.lineage_topic_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.connected_post_count = 5;
                value
            },
            {
                let mut value = artifact.clone();
                value.lineage_edge_count = 13;
                value
            },
            {
                let mut value = artifact.clone();
                value.lineage_artifact_sha256.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.lineage_artifact_sha256 = "GG".repeat(32);
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
