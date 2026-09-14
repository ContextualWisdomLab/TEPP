//! Digest-bound completed artifacts from the ADR-0012 topic estimator.

use std::collections::BTreeSet;

use model_selection::{
    FittedCandidateKConfig, FittedCandidateOutcome, FittedCandidateSelection, ModelCandidate,
    select_candidate_k, select_fitted_candidate_model, statistical_candidate_from_fit,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use topic_measurement::{
    MAX_REFERENCE_FIT_BUDGET, MAX_REFERENCE_WORKING_CELLS, ReferenceTopicInput,
    ReferenceTopicModel, ReferenceTopicModelConfig, fit_reference_topic_model,
};
use uuid::Uuid;

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed TRSL topic-lineage artifact.
pub const TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.trsl_topic_lineage.v2";
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

/// One compact, reason-bearing candidate fit result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageCandidateOutcome {
    /// Requested topic count.
    pub candidate_k: u32,
    /// Finite Schwarz score when fitting succeeded.
    pub statistical_score: Option<f64>,
    /// Finite free-parameter count when fitting succeeded.
    pub complexity: Option<f64>,
    /// Stable estimator failure code when fitting failed.
    pub failure_code: Option<String>,
    /// Converged seed when fitting succeeded.
    pub seed: Option<u64>,
    /// Converged iteration count when fitting succeeded.
    pub iterations: Option<u64>,
    /// Final penalized objective when fitting succeeded.
    pub objective: Option<f64>,
}

/// Canonical configuration and evidence for the fitted model decision.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageFitManifest {
    /// Versioned fixed-K or candidate-selection method.
    pub method_code: String,
    /// Requested candidates and their actual outcomes.
    pub candidate_outcomes: Vec<TopicLineageCandidateOutcome>,
    /// Separated non-authoritative LLM recommendations.
    pub llm_recommendations: Vec<u32>,
    /// Deterministic initialization seeds.
    pub seeds: Vec<u64>,
    /// Per-seed iteration budget.
    pub maximum_iterations: u64,
    /// Relative-objective convergence tolerance.
    pub tolerance: f64,
    /// Gaussian prior variance.
    pub prior_variance: f64,
    /// Relation penalty strength.
    pub relation_strength: f64,
    /// Coefficient ridge penalty.
    pub ridge: f64,
    /// Multinomial smoothing floor.
    pub topic_smoothing: f64,
    /// Bounded GEM step size.
    pub step_size: f64,
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
    /// Canonical digest of ordered snapshot identities and availability times.
    pub source_snapshot_sha256: String,
    /// Canonical digest of the complete validated numerical input.
    pub model_input_sha256: String,
    /// Exact numerical model contract.
    pub model_contract_version: String,
    /// Digest-bound fitted configuration and candidate evidence.
    pub fit_manifest: TopicLineageFitManifest,
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

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_fit_manifest(
    manifest: &TopicLineageFitManifest,
    selected_k: u64,
    selected_seed: u64,
    iterations: u64,
    objective: f64,
) -> bool {
    let methods_ok = match manifest.method_code.as_str() {
        "fixed_k_reference_v1" => {
            manifest.candidate_outcomes.len() == 1 && manifest.llm_recommendations.is_empty()
        }
        "trsl_tm_reference_bic_v1" => true,
        _ => false,
    };
    let controls = [
        manifest.tolerance,
        manifest.prior_variance,
        manifest.topic_smoothing,
        manifest.step_size,
    ];
    let mut candidate_counts = BTreeSet::new();
    let mut statistical_candidates = Vec::new();
    let mut successful_receipts = Vec::new();
    methods_ok
        && !manifest.candidate_outcomes.is_empty()
        && !manifest.seeds.is_empty()
        && manifest.candidate_outcomes.len() <= MAX_REFERENCE_FIT_BUDGET
        && manifest.seeds.len() <= MAX_REFERENCE_FIT_BUDGET
        && manifest.llm_recommendations.len() <= MAX_REFERENCE_FIT_BUDGET
        && manifest.maximum_iterations >= 2
        && u128::from(manifest.maximum_iterations) <= MAX_REFERENCE_FIT_BUDGET as u128
        && manifest.seeds.contains(&selected_seed)
        && iterations > 0
        && iterations <= manifest.maximum_iterations
        && (manifest.candidate_outcomes.len() as u128)
            * (manifest.seeds.len() as u128)
            * u128::from(manifest.maximum_iterations)
            <= MAX_REFERENCE_WORKING_CELLS as u128
        && controls
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
        && manifest.relation_strength.is_finite()
        && manifest.relation_strength >= 0.0
        && manifest.ridge.is_finite()
        && manifest.ridge >= 0.0
        && manifest
            .llm_recommendations
            .iter()
            .all(|candidate| *candidate >= 2)
        && manifest.candidate_outcomes.iter().all(|outcome| {
            let candidate = outcome.statistical_score.zip(outcome.complexity).and_then(
                |(score, complexity)| {
                    ModelCandidate::statistical(outcome.candidate_k, score, complexity).ok()
                },
            );
            let receipt = outcome.seed.zip(outcome.iterations).zip(outcome.objective);
            let success = candidate.is_some()
                && outcome.failure_code.is_none()
                && receipt.is_some_and(|((seed, iterations), objective)| {
                    manifest.seeds.contains(&seed)
                        && iterations > 0
                        && iterations <= manifest.maximum_iterations
                        && objective.is_finite()
                });
            if let Some(candidate) = candidate {
                statistical_candidates.push(candidate);
            }
            if let (true, Some(seed), Some(iterations), Some(objective)) =
                (success, outcome.seed, outcome.iterations, outcome.objective)
            {
                successful_receipts.push((outcome.candidate_k, seed, iterations, objective));
            }
            let failure = outcome.statistical_score.is_none()
                && outcome.complexity.is_none()
                && outcome.seed.is_none()
                && outcome.iterations.is_none()
                && outcome.objective.is_none()
                && outcome.failure_code.as_deref().is_some_and(|code| {
                    matches!(
                        code,
                        "invalid_model_input"
                            | "non_finite_estimate"
                            | "did_not_converge"
                            | "unsupported_estimator_failure"
                    )
                });
            outcome.candidate_k >= 2
                && candidate_counts.insert(outcome.candidate_k)
                && (success || failure)
        })
        && select_candidate_k(&statistical_candidates)
            .is_ok_and(|winner| u64::from(winner) == selected_k)
        && successful_receipts.iter().any(
            |&(candidate_k, seed, candidate_iterations, candidate_objective)| {
                u64::from(candidate_k) == selected_k
                    && seed == selected_seed
                    && candidate_iterations == iterations
                    && candidate_objective.to_bits() == objective.to_bits()
            },
        )
}

fn valid_runtime_dimensions(topic_count: u64, evidence_count: u64) -> bool {
    let topics = u128::from(topic_count);
    let documents = u128::from(evidence_count);
    topics >= 2
        && documents <= MAX_REFERENCE_FIT_BUDGET as u128
        && topics * topics <= MAX_REFERENCE_WORKING_CELLS as u128
        && documents * topics <= MAX_REFERENCE_WORKING_CELLS as u128
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
            || !is_sha256(&self.source_snapshot_sha256)
            || !is_sha256(&self.model_input_sha256)
            || self.model_contract_version != TOPIC_LINEAGE_MODEL_CONTRACT_VERSION
            || !self.objective.is_finite()
            || !valid_fit_manifest(
                &self.fit_manifest,
                self.topic_count,
                self.selected_seed,
                self.iterations,
                self.objective,
            )
            || !valid_runtime_dimensions(self.topic_count, self.evidence_count)
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
    validate_topic_lineage_request(request, accepted, snapshot_id, knowledge_cutoff, input)?;
    let model = fit_reference_topic_model(input, config)?;
    let candidate_k = u32::try_from(model.topic_term_probabilities.len())
        .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let candidate = statistical_candidate_from_fit(input, candidate_k, &model)
        .map_err(AnalysisEngineError::ModelSelection)?;
    let manifest = fit_manifest(
        "fixed_k_reference_v1",
        vec![TopicLineageCandidateOutcome {
            candidate_k,
            statistical_score: candidate.held_out_log_likelihood(),
            complexity: candidate.complexity(),
            failure_code: None,
            seed: Some(model.seed),
            iterations: Some(
                u64::try_from(model.iterations)
                    .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
            ),
            objective: Some(model.objective),
        }],
        Vec::new(),
        config.seeds(),
        config.maximum_iterations(),
        config.tolerance(),
        config.hyperparameters(),
    )?;
    complete_topic_lineage_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        input,
        &model,
        manifest,
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
    validate_topic_lineage_request(request, accepted, snapshot_id, knowledge_cutoff, input)?;
    let selection = select_fitted_candidate_model(input, selection_config, method_name, llm_votes)
        .map_err(AnalysisEngineError::FittedModelSelection)?;
    let manifest = selection_manifest(&selection, selection_config)?;
    let model = selection.into_model();
    complete_topic_lineage_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        input,
        &model,
        manifest,
        completed_at,
    )
}

fn validate_topic_lineage_request(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
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
    let binding = input
        .source_binding()
        .ok_or(AnalysisEngineError::InvalidEvidence)?;
    if binding.snapshot_id() != snapshot_id
        || binding.knowledge_cutoff() != knowledge_cutoff.to_rfc3339()
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(())
}

fn selection_manifest(
    selection: &FittedCandidateSelection,
    config: &FittedCandidateKConfig,
) -> Result<TopicLineageFitManifest, AnalysisEngineError> {
    let outcomes = selection
        .candidate_outcomes()
        .iter()
        .copied()
        .map(candidate_outcome)
        .collect();
    fit_manifest(
        selection.method_code(),
        outcomes,
        selection.llm_recommendations().to_vec(),
        config.seeds(),
        config.maximum_iterations(),
        config.tolerance(),
        config.hyperparameters(),
    )
}

fn candidate_outcome(outcome: FittedCandidateOutcome) -> TopicLineageCandidateOutcome {
    let fit_receipt = outcome.fit_receipt();
    TopicLineageCandidateOutcome {
        candidate_k: outcome.candidate_k(),
        statistical_score: outcome.statistical_score(),
        complexity: outcome.complexity(),
        failure_code: outcome.failure_code().map(str::to_owned),
        seed: fit_receipt.map(|receipt| receipt.0),
        iterations: fit_receipt.and_then(|receipt| u64::try_from(receipt.1).ok()),
        objective: fit_receipt.map(|receipt| receipt.2),
    }
}

#[allow(clippy::too_many_arguments)]
fn fit_manifest(
    method_code: &str,
    candidate_outcomes: Vec<TopicLineageCandidateOutcome>,
    llm_recommendations: Vec<u32>,
    seeds: &[u64],
    maximum_iterations: usize,
    tolerance: f64,
    hyperparameters: [f64; 5],
) -> Result<TopicLineageFitManifest, AnalysisEngineError> {
    Ok(TopicLineageFitManifest {
        method_code: method_code.to_owned(),
        candidate_outcomes,
        llm_recommendations,
        seeds: seeds.to_vec(),
        maximum_iterations: u64::try_from(maximum_iterations)
            .map_err(|_| AnalysisEngineError::ArithmeticOverflow)?,
        tolerance,
        prior_variance: hyperparameters[0],
        relation_strength: hyperparameters[1],
        ridge: hyperparameters[2],
        topic_smoothing: hyperparameters[3],
        step_size: hyperparameters[4],
    })
}

#[allow(clippy::too_many_arguments)]
fn complete_topic_lineage_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &ReferenceTopicInput,
    model: &ReferenceTopicModel,
    fit_manifest: TopicLineageFitManifest,
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
        source_snapshot_sha256: input
            .source_binding()
            .ok_or(AnalysisEngineError::InvalidEvidence)?
            .source_snapshot_sha256()
            .to_owned(),
        model_input_sha256: input
            .source_binding()
            .ok_or(AnalysisEngineError::InvalidEvidence)?
            .model_input_sha256()
            .to_owned(),
        model_contract_version: TOPIC_LINEAGE_MODEL_CONTRACT_VERSION.into(),
        fit_manifest,
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
        MAX_REFERENCE_FIT_BUDGET, TOPIC_LINEAGE_ARTIFACT_BYTE_LIMIT,
        TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION, TOPIC_LINEAGE_EDGE_LIMIT,
        TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TopicLineageArtifact, TopicLineageArtifactEdge,
        TopicLineageCandidateOutcome, TopicLineageFitManifest, valid_fit_manifest,
        valid_runtime_dimensions,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> TopicLineageArtifact {
        TopicLineageArtifact {
            schema_version: TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            source_snapshot_sha256: "0".repeat(64),
            model_input_sha256: "1".repeat(64),
            model_contract_version: TOPIC_LINEAGE_MODEL_CONTRACT_VERSION.into(),
            fit_manifest: TopicLineageFitManifest {
                method_code: "fixed_k_reference_v1".into(),
                candidate_outcomes: vec![TopicLineageCandidateOutcome {
                    candidate_k: 2,
                    statistical_score: Some(-5.0),
                    complexity: Some(4.0),
                    failure_code: None,
                    seed: Some(7),
                    iterations: Some(4),
                    objective: Some(-1.0),
                }],
                llm_recommendations: Vec::new(),
                seeds: vec![7],
                maximum_iterations: 20,
                tolerance: 1e-5,
                prior_variance: 1.0,
                relation_strength: 0.25,
                ridge: 0.01,
                topic_smoothing: 0.05,
                step_size: 0.2,
            },
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
        assert!(valid_runtime_dimensions(2, 2));
        assert!(!valid_runtime_dimensions(1, 2));
        assert!(!valid_runtime_dimensions(4_096, 2));
        assert!(!valid_runtime_dimensions(2_048, 4_096));
        assert!(!valid_runtime_dimensions(
            2,
            u64::try_from(MAX_REFERENCE_FIT_BUDGET + 1).expect("bounded limit"),
        ));
        let mut oversized_k = artifact.clone();
        oversized_k.topic_count = 2_049;
        oversized_k.fit_manifest.candidate_outcomes[0].candidate_k = 2_049;
        assert_invalid(&oversized_k);
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            TopicLineageArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        let mut selected_with_failure = artifact.clone();
        selected_with_failure.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
        selected_with_failure
            .fit_manifest
            .candidate_outcomes
            .push(TopicLineageCandidateOutcome {
                candidate_k: 3,
                statistical_score: None,
                complexity: None,
                failure_code: Some("did_not_converge".into()),
                seed: None,
                iterations: None,
                objective: None,
            });
        assert!(selected_with_failure.to_json().is_ok());
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
                value.source_snapshot_sha256.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.source_snapshot_sha256 = "G".repeat(64);
                value
            },
            {
                let mut value = artifact.clone();
                value.model_input_sha256.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.model_contract_version.clear();
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
    #[allow(clippy::too_many_lines)]
    fn fit_manifest_tampering_fails_closed() {
        let artifact = artifact();
        let mut oversized_candidates = artifact.fit_manifest.clone();
        oversized_candidates.method_code = "trsl_tm_reference_bic_v1".into();
        oversized_candidates.candidate_outcomes.extend(
            (3..=u32::try_from(MAX_REFERENCE_FIT_BUDGET + 2).expect("bounded limit")).map(
                |candidate_k| TopicLineageCandidateOutcome {
                    candidate_k,
                    statistical_score: None,
                    complexity: None,
                    failure_code: Some("did_not_converge".into()),
                    seed: None,
                    iterations: None,
                    objective: None,
                },
            ),
        );
        assert!(!valid_fit_manifest(&oversized_candidates, 2, 7, 4, -1.0,));

        let mut losing_first = artifact.fit_manifest.clone();
        losing_first.method_code = "trsl_tm_reference_bic_v1".into();
        losing_first.candidate_outcomes.insert(
            0,
            TopicLineageCandidateOutcome {
                candidate_k: 3,
                statistical_score: Some(-10.0),
                complexity: Some(10.0),
                failure_code: None,
                seed: Some(7),
                iterations: Some(4),
                objective: Some(-2.0),
            },
        );
        assert!(valid_fit_manifest(&losing_first, 2, 7, 4, -1.0));
        let invalid_artifacts = vec![
            {
                let mut value = artifact.clone();
                value.fit_manifest.method_code.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes.clear();
                value.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.llm_recommendations.push(3);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.seeds.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.maximum_iterations = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.maximum_iterations =
                    u64::try_from(MAX_REFERENCE_FIT_BUDGET + 1).expect("bounded limit");
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.seeds = vec![7; MAX_REFERENCE_FIT_BUDGET + 1];
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.llm_recommendations = vec![2; MAX_REFERENCE_FIT_BUDGET + 1];
                value.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.seeds = vec![7; 2_049];
                value.fit_manifest.maximum_iterations = 2_048;
                value
            },
            {
                let mut value = artifact.clone();
                value.selected_seed = 8;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.seeds.push(8);
                value.selected_seed = 8;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].seed = Some(8);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].iterations = Some(0);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].iterations = Some(21);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].objective = Some(f64::NAN);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = None;
                value.fit_manifest.candidate_outcomes[0].complexity = None;
                value.fit_manifest.candidate_outcomes[0].failure_code =
                    Some("did_not_converge".into());
                value.fit_manifest.candidate_outcomes[0].iterations = None;
                value.fit_manifest.candidate_outcomes[0].objective = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = None;
                value.fit_manifest.candidate_outcomes[0].complexity = None;
                value.fit_manifest.candidate_outcomes[0].failure_code =
                    Some("did_not_converge".into());
                value.fit_manifest.candidate_outcomes[0].seed = None;
                value.fit_manifest.candidate_outcomes[0].objective = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = None;
                value.fit_manifest.candidate_outcomes[0].complexity = None;
                value.fit_manifest.candidate_outcomes[0].failure_code =
                    Some("did_not_converge".into());
                value.fit_manifest.candidate_outcomes[0].seed = None;
                value.fit_manifest.candidate_outcomes[0].iterations = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.iterations = 5;
                value
            },
            {
                let mut value = artifact.clone();
                value.objective = -2.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.iterations = value.fit_manifest.maximum_iterations + 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.tolerance = 0.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.prior_variance = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.relation_strength = -1.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.relation_strength = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.ridge = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.ridge = -1.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.topic_smoothing = 0.0;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.step_size = f64::INFINITY;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
                value.fit_manifest.llm_recommendations.push(1);
                value
            },
            {
                let mut value = artifact.clone();
                value
                    .fit_manifest
                    .candidate_outcomes
                    .push(TopicLineageCandidateOutcome {
                        candidate_k: 3,
                        statistical_score: Some(-10.0),
                        complexity: Some(10.0),
                        failure_code: None,
                        seed: Some(7),
                        iterations: Some(4),
                        objective: Some(-2.0),
                    });
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].candidate_k = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = Some(f64::NAN);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].complexity = Some(f64::INFINITY);
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].complexity = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = None;
                value.fit_manifest.candidate_outcomes[0].failure_code =
                    Some("did_not_converge".into());
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].statistical_score = None;
                value.fit_manifest.candidate_outcomes[0].complexity = None;
                value.fit_manifest.candidate_outcomes[0].failure_code =
                    Some("unsupported_estimator_failure".into());
                value.fit_manifest.candidate_outcomes[0].seed = None;
                value.fit_manifest.candidate_outcomes[0].iterations = None;
                value.fit_manifest.candidate_outcomes[0].objective = None;
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].failure_code = Some("unknown".into());
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
                value
                    .fit_manifest
                    .candidate_outcomes
                    .push(value.fit_manifest.candidate_outcomes[0].clone());
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.method_code = "trsl_tm_reference_bic_v1".into();
                value
                    .fit_manifest
                    .candidate_outcomes
                    .push(TopicLineageCandidateOutcome {
                        candidate_k: 3,
                        statistical_score: Some(-1.0),
                        complexity: Some(3.0),
                        failure_code: None,
                        seed: Some(7),
                        iterations: Some(4),
                        objective: Some(-2.0),
                    });
                value
            },
            {
                let mut value = artifact.clone();
                value.fit_manifest.candidate_outcomes[0].candidate_k = 3;
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
