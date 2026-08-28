//! Digest-bound completed artifacts from the ADR-0012 topic estimator.

use std::collections::BTreeSet;

use model_selection::{FittedCandidateKConfig, select_fitted_candidate_model};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_measurement::{
    ReferenceTopicInput, ReferenceTopicModel, ReferenceTopicModelConfig, fit_reference_topic_model,
};
use uuid::Uuid;

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed TRSL topic-lineage artifact.
pub const TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.trsl_topic_lineage.v1";
/// Model contract required by the CPU `f64` reference execution path.
pub const TOPIC_LINEAGE_MODEL_CONTRACT_VERSION: &str = "trsl_tm_cpu_f64_v1";
/// Analysis-run output profile required for a topic-lineage artifact.
pub const TOPIC_LINEAGE_OUTPUT_PROFILE: &str = "trsl_topic_lineage_v1";
/// Maximum canonical artifact JSON size.
pub const TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const TOPIC_LINEAGE_EDGE_LIMIT: usize = 100_000;

/// One fitted same-topic predecessor/successor association.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageArtifactEdge {
    /// Opaque predecessor document identity.
    pub predecessor_document_id: String,
    /// Opaque successor document identity.
    pub successor_document_id: String,
    /// Artifact-local global topic index.
    pub topic_index: u64,
    /// Minimum dominant-topic posterior mean across the linked documents.
    pub association_strength: f64,
}

/// Completed, bounded topic-lineage result consumed by product-history clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the estimator.
    pub knowledge_cutoff: String,
    /// Selected deterministic initialization seed.
    pub selected_seed: u64,
    /// Iterations used by the selected converged fit.
    pub iterations: u64,
    /// Final finite penalized objective.
    pub objective: f64,
    /// Number of global topics in the fitted model.
    pub topic_count: u64,
    /// Number of modeled evidence documents.
    pub evidence_count: u64,
    /// Documents incident to at least one fitted same-topic sequence edge.
    pub connected_post_count: u64,
    /// Topics represented by at least one fitted sequence edge.
    pub lineage_count: u64,
    /// Fitted edges restricted to explicit forward predecessor/successor input.
    pub sequence_edges: Vec<TopicLineageArtifactEdge>,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl TopicLineageArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidTopicLineageArtifact`] when the
    /// schema, dimensions, identifiers, counts, edges, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidTopicLineageArtifact)?;
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
        if payload.len() > TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.iterations == 0
            || !self.objective.is_finite()
            || self.topic_count < 2
            || self.evidence_count < 2
            || self.connected_post_count > self.evidence_count
            || self.lineage_count > self.topic_count
            || self.sequence_edges.len() > TOPIC_LINEAGE_EDGE_LIMIT
            || self.inference_status != "fitted_topic_association_not_causation"
        {
            return Err(AnalysisEngineError::InvalidTopicLineageArtifact);
        }
        let mut pairs = BTreeSet::new();
        let mut connected = BTreeSet::new();
        let mut lineages = BTreeSet::new();
        for edge in &self.sequence_edges {
            let predecessor = Uuid::parse_str(&edge.predecessor_document_id)
                .map_err(|_| AnalysisEngineError::InvalidTopicLineageArtifact)?;
            let successor = Uuid::parse_str(&edge.successor_document_id)
                .map_err(|_| AnalysisEngineError::InvalidTopicLineageArtifact)?;
            if predecessor == successor
                || edge.topic_index >= self.topic_count
                || !edge.association_strength.is_finite()
                || edge.association_strength <= 0.0
                || edge.association_strength > 1.0
                || !pairs.insert((predecessor, successor))
            {
                return Err(AnalysisEngineError::InvalidTopicLineageArtifact);
            }
            connected.insert(predecessor);
            connected.insert(successor);
            lineages.insert(edge.topic_index);
        }
        if self.connected_post_count != connected.len() as u64
            || self.lineage_count != lineages.len() as u64
        {
            return Err(AnalysisEngineError::InvalidTopicLineageArtifact);
        }
        Ok(())
    }
}

/// One completed topic-lineage artifact and its request-bound terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct TopicLineageExecution {
    /// Digest-bound completed model artifact.
    pub artifact: TopicLineageArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute the validated ADR-0012 CPU `f64` reference estimator.
///
/// The caller supplies the exact snapshot identity and cutoff used to construct
/// `input`; both must exactly match the already-validated analysis request.
/// This executor preserves the estimator result and does not select `K`, infer
/// causal edges, or emit a partial artifact.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, estimator failure,
/// arithmetic error, or invalid/oversized artifact error.
pub fn execute_topic_lineage_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    config: &ReferenceTopicModelConfig,
    completed_at: impl Into<String>,
) -> Result<TopicLineageExecution, AnalysisEngineError> {
    validate_topic_lineage_request(request, accepted, snapshot_id, knowledge_cutoff)?;
    let model = fit_reference_topic_model(input, config)?;
    complete_topic_lineage_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        input,
        &model,
        completed_at,
    )
}

/// Select `K` from fitted diagnostics and complete one topic-lineage run.
///
/// The winning converged CPU `f64` fit is reused directly. LLM votes may be
/// recorded as recommenders but cannot create or replace a statistical fit.
///
/// # Errors
///
/// Returns request-binding, model-selection, estimator, arithmetic, or artifact
/// validation failures without emitting a completed result.
#[allow(clippy::too_many_arguments)]
pub fn execute_selected_topic_lineage_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    selection_config: &FittedCandidateKConfig,
    method_name: &str,
    llm_votes: &[u32],
    completed_at: impl Into<String>,
) -> Result<TopicLineageExecution, AnalysisEngineError> {
    validate_topic_lineage_request(request, accepted, snapshot_id, knowledge_cutoff)?;
    let model = select_fitted_candidate_model(input, selection_config, method_name, llm_votes)
        .map_err(AnalysisEngineError::ModelSelection)?;
    complete_topic_lineage_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        input,
        &model,
        completed_at,
    )
}

fn validate_topic_lineage_request(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<(), AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != TOPIC_LINEAGE_MODEL_CONTRACT_VERSION
        || request.output_profile != TOPIC_LINEAGE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(())
}

fn complete_topic_lineage_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    model: &ReferenceTopicModel,
    completed_at: impl Into<String>,
) -> Result<TopicLineageExecution, AnalysisEngineError> {
    let topic_count = u64::try_from(model.topic_term_probabilities.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let evidence_count = u64::try_from(input.document_count())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let connected_post_count = u64::try_from(model.connected_post_count)
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let lineage_count =
        u64::try_from(model.lineage_count).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let sequence_edges: Vec<_> = model
        .sequence_edges
        .iter()
        .map(|edge| {
            Ok(TopicLineageArtifactEdge {
                predecessor_document_id: edge.predecessor_document_id.to_string(),
                successor_document_id: edge.successor_document_id.to_string(),
                topic_index: u64::try_from(edge.topic_index)
                    .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
                association_strength: edge.association_strength,
            })
        })
        .collect::<Result<_, AnalysisEngineError>>()?;
    let artifact = TopicLineageArtifact {
        schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        selected_seed: model.seed,
        iterations: u64::try_from(model.iterations)
            .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
        objective: model.objective,
        topic_count,
        evidence_count,
        connected_post_count,
        lineage_count,
        sequence_edges,
        inference_status: "fitted_topic_association_not_causation".into(),
    };
    let digest = artifact.sha256()?;
    let statistic_count = u64::try_from(artifact.sequence_edges.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?
        .checked_add(2)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let summary = AnalysisResultSummary::new(
        "trsl_topic_lineage",
        evidence_count,
        statistic_count,
        "reference_estimator_converged",
    );
    let summary = summary?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("topic_lineage_artifact_{}", &digest[..16]),
        digest,
        TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    );
    let terminal_result = terminal_result?;
    Ok(TopicLineageExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT, TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION,
        TOPIC_LINEAGE_EDGE_LIMIT, TopicLineageArtifact, TopicLineageArtifactEdge,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> TopicLineageArtifact {
        TopicLineageArtifact {
            schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            selected_seed: 7,
            iterations: 4,
            objective: -1.0,
            topic_count: 2,
            evidence_count: 2,
            connected_post_count: 2,
            lineage_count: 1,
            sequence_edges: vec![TopicLineageArtifactEdge {
                predecessor_document_id: "00000000-0000-0000-0000-000000000001".into(),
                successor_document_id: "00000000-0000-0000-0000-000000000002".into(),
                topic_index: 0,
                association_strength: 0.8,
            }],
            inference_status: "fitted_topic_association_not_causation".into(),
        }
    }

    fn assert_invalid(artifact: &TopicLineageArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidTopicLineageArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            TopicLineageArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            TopicLineageArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidTopicLineageArtifact)
        );
        assert_eq!(
            TopicLineageArtifact::from_json(&"x".repeat(TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT + 1)),
            Err(AnalysisEngineError::LimitExceeded)
        );

        let mut oversized = artifact;
        oversized.sequence_edges = (1_u128..=2_000)
            .map(|index| TopicLineageArtifactEdge {
                predecessor_document_id: uuid::Uuid::from_u128(index).to_string(),
                successor_document_id: uuid::Uuid::from_u128(index + 1).to_string(),
                topic_index: 0,
                association_strength: 0.8,
            })
            .collect();
        oversized.evidence_count = 2_001;
        oversized.connected_post_count = 2_001;
        assert_eq!(oversized.to_json(), Err(AnalysisEngineError::LimitExceeded));
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
                value.iterations = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.objective = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.topic_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.evidence_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.connected_post_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.lineage_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges =
                    vec![value.sequence_edges[0].clone(); TOPIC_LINEAGE_EDGE_LIMIT + 1];
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
    fn artifact_edge_tampering_fails_closed() {
        let artifact = artifact();
        let invalid_artifacts = [
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].successor_document_id =
                    value.sequence_edges[0].predecessor_document_id.clone();
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].topic_index = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].association_strength = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].association_strength = 0.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].association_strength = 1.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges.push(value.sequence_edges[0].clone());
                value
            },
            {
                let mut value = artifact.clone();
                value.connected_post_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.lineage_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].predecessor_document_id = "invalid".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.sequence_edges[0].successor_document_id = "invalid".into();
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }
}
