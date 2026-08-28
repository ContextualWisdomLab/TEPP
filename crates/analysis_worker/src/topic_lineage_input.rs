//! Manifest-bound input adapter for the existing CPU topic-lineage estimator.

use crate::{AnalysisWorkerError, MAX_WORKER_INPUT_BYTES};
use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix};
use uuid::Uuid;

/// Version of the topic-lineage scientific input contract.
pub const TOPIC_LINEAGE_WORKER_INPUT_VERSION: u16 = 1;
const MAX_DOCUMENTS: usize = 10_000;
const MAX_VOCABULARY: usize = 100_000;
const MAX_SEEDS: usize = 32;
const MAX_FIT_CELLS: usize = 1_000_000_000;

/// One document row in the bounded scientific input.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicDocumentInput {
    /// Opaque analytical document identity.
    pub document_id: Uuid,
    /// Event-valid instant.
    pub event_time: String,
    /// First availability instant.
    pub available_time: String,
}

/// Canonical compressed sparse row matrix.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicSparseInput {
    /// Matrix column count.
    pub columns: usize,
    /// CSR row offsets.
    pub offsets: Vec<usize>,
    /// CSR column indices.
    pub indices: Vec<usize>,
    /// Finite cell values.
    pub values: Vec<f64>,
}

/// One provenance-bound multiple-membership assignment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicMembershipInput {
    /// Modeled document identity.
    pub document_id: Uuid,
    /// Opaque contextual group identity.
    pub group_id: Uuid,
    /// Closed membership-role wire name.
    pub role: String,
    /// Finite non-negative membership weight.
    pub weight: f64,
    /// Inclusive event-valid start.
    pub valid_from: String,
    /// Inclusive event-valid end.
    pub valid_to: String,
    /// First availability of the supporting assertion.
    pub available_time: String,
    /// Opaque supporting assertion identity.
    pub evidence_id: Uuid,
    /// SHA-256 of the supporting assertion.
    pub evidence_sha256: String,
}

/// One provenance-bound relation between modeled documents.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicRelationInput {
    /// Source document identity.
    pub source_document_id: Uuid,
    /// Target document identity.
    pub target_document_id: Uuid,
    /// Closed relation-kind wire name.
    pub kind: String,
    /// Observed or inferred evidence status.
    pub evidence_status: RelationEvidenceStatus,
    /// Inclusive source interval end; its start is the source document event time.
    pub source_event_end: String,
    /// Inclusive target interval end; its start is the target document event time.
    pub target_event_end: String,
    /// First availability of the supporting assertion.
    pub available_time: String,
    /// Opaque supporting assertion identity.
    pub evidence_id: Uuid,
    /// SHA-256 of the supporting assertion.
    pub evidence_sha256: String,
}

/// Explicit deterministic reference-estimator configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicModelInput {
    /// Requested number of topics.
    pub topic_count: usize,
    /// Ordered deterministic restart seeds.
    pub seeds: Vec<u64>,
    /// Maximum coordinate iterations.
    pub maximum_iterations: usize,
    /// Convergence tolerance.
    pub tolerance: f64,
    /// Gaussian prior variance.
    pub prior_variance: f64,
    /// Relational likelihood strength.
    pub relation_strength: f64,
    /// Numerical ridge penalty.
    pub ridge: f64,
    /// Topic smoothing constant.
    pub topic_smoothing: f64,
    /// Coordinate update step size.
    pub step_size: f64,
}

/// Complete manifest-bound topic-lineage estimator input.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageWorkerInput {
    /// Semantic contract version.
    pub contract_version: u16,
    /// Reproducibility manifest authorizing these exact scientific bytes.
    pub reproducibility_manifest_id: Uuid,
    /// Immutable snapshot identity.
    pub snapshot_id: String,
    /// SHA-256 of this payload with this field omitted.
    pub scientific_input_sha256: String,
    /// Documents in estimator row order.
    pub documents: Vec<TopicDocumentInput>,
    /// Sparse document-term counts.
    pub document_term: TopicSparseInput,
    /// Optional sparse document covariates.
    pub covariates: Option<TopicSparseInput>,
    /// Cross-classified and multiple-membership assignments.
    pub memberships: Vec<TopicMembershipInput>,
    /// Typed document relations.
    pub relations: Vec<TopicRelationInput>,
    /// Explicit reference-estimator configuration.
    pub model: TopicModelInput,
}

/// Domain-validated estimator arguments ready for `execute_topic_lineage_run`.
pub struct ValidatedTopicLineageInput {
    /// Cutoff-safe, relational topic input.
    pub input: ReferenceTopicInput,
    /// Validated deterministic estimator configuration.
    pub config: ReferenceTopicModelConfig,
}

impl TopicLineageWorkerInput {
    /// Parse and size-bound untrusted JSON.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] for oversized or malformed input.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisWorkerError> {
        if payload.len() > MAX_WORKER_INPUT_BYTES {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        serde_json::from_str(payload).map_err(|_| AnalysisWorkerError::InvalidInput)
    }

    /// Compute the digest over every scientific field except the digest itself.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] if canonical serialization fails.
    pub fn scientific_digest(&self) -> Result<String, AnalysisWorkerError> {
        let mut value =
            serde_json::to_value(self).map_err(|_| AnalysisWorkerError::InvalidInput)?;
        value
            .as_object_mut()
            .ok_or(AnalysisWorkerError::InvalidInput)?
            .remove("scientific_input_sha256");
        let bytes = serde_json::to_vec(&value).map_err(|_| AnalysisWorkerError::InvalidInput)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    /// Validate cutoff/provenance/domain contracts and build existing estimator types.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisWorkerError::InvalidInput`] when any trust-boundary or
    /// estimator invariant fails.
    #[allow(clippy::too_many_lines)]
    pub fn validate(
        &self,
        cutoff: KnowledgeCutoff,
    ) -> Result<ValidatedTopicLineageInput, AnalysisWorkerError> {
        if self.contract_version != TOPIC_LINEAGE_WORKER_INPUT_VERSION
            || self.scientific_input_sha256 != self.scientific_digest()?
            || self.documents.len() < 2
            || self.documents.len() > MAX_DOCUMENTS
            || self.document_term.columns > MAX_VOCABULARY
            || self.model.topic_count < 2
            || self.model.topic_count > self.document_term.columns
            || self.model.seeds.is_empty()
            || self.model.seeds.len() > MAX_SEEDS
            || self.model.seeds.iter().collect::<BTreeSet<_>>().len() != self.model.seeds.len()
            || self
                .documents
                .len()
                .checked_mul(self.document_term.columns)
                .and_then(|cells| cells.checked_mul(self.model.topic_count))
                .and_then(|cells| cells.checked_mul(self.model.seeds.len()))
                .is_none_or(|cells| cells > MAX_FIT_CELLS)
        {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        let mut snapshot = CorpusSnapshot::new();
        let mut document_ids = Vec::with_capacity(self.documents.len());
        let mut event_times = Vec::with_capacity(self.documents.len());
        let mut document_set = BTreeSet::new();
        for document in &self.documents {
            let event = EventTime::parse_rfc3339(&document.event_time)
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
            let available = AvailableTime::parse_rfc3339(&document.available_time)
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
            if !document_set.insert(document.document_id) {
                return Err(AnalysisWorkerError::InvalidInput);
            }
            snapshot
                .insert_if_eligible(
                    CorpusDocument::new(document.document_id, available),
                    &cutoff,
                )
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
            document_ids.push(document.document_id);
            event_times.push(event);
        }
        let terms = sparse(&self.document_term, self.documents.len())?;
        let covariates = self
            .covariates
            .as_ref()
            .map(|matrix| sparse(matrix, self.documents.len()))
            .transpose()?;
        let mut memberships = MembershipNetwork::new();
        for assignment in &self.memberships {
            require_provenance(
                &assignment.available_time,
                &assignment.evidence_sha256,
                cutoff,
            )?;
            if !document_set.contains(&assignment.document_id) {
                return Err(AnalysisWorkerError::InvalidInput);
            }
            memberships
                .insert(
                    MembershipAssignment::new(
                        MemberId::from_uuid(assignment.document_id),
                        GroupId::from_uuid(assignment.group_id),
                        MembershipRole::from_wire_name(&assignment.role)
                            .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                        MembershipWeight::new(assignment.weight)
                            .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                        EventTime::parse_rfc3339(&assignment.valid_from)
                            .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                        EventTime::parse_rfc3339(&assignment.valid_to)
                            .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                    )
                    .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                )
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
        }
        for (&id, &event) in document_ids.iter().zip(&event_times) {
            if memberships
                .active_memberships_for(MemberId::from_uuid(id), event)
                .iter()
                .all(|item| item.weight().value() <= 0.0)
            {
                return Err(AnalysisWorkerError::InvalidInput);
            }
        }
        let mut relations = RelationGraph::new();
        let mut transition_count = 0_usize;
        let mut relation_keys = BTreeSet::new();
        for relation in &self.relations {
            require_provenance(&relation.available_time, &relation.evidence_sha256, cutoff)?;
            let source_index = document_ids
                .iter()
                .position(|id| *id == relation.source_document_id)
                .ok_or(AnalysisWorkerError::InvalidInput)?;
            let target_index = document_ids
                .iter()
                .position(|id| *id == relation.target_document_id)
                .ok_or(AnalysisWorkerError::InvalidInput)?;
            let kind = RelationKind::from_wire_name(&relation.kind)
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
            if !relation_keys.insert((
                relation.source_document_id,
                relation.target_document_id,
                kind,
                relation.evidence_status,
            )) {
                return Err(AnalysisWorkerError::InvalidInput);
            }
            transition_count += usize::from(kind.is_transition_edge());
            relations
                .insert(
                    RelationEdge::new(
                        kind,
                        RelationEndpointId::from_uuid(relation.source_document_id),
                        RelationEndpointId::from_uuid(relation.target_document_id),
                        relation.evidence_status,
                        TemporalInterval::bounded(
                            TemporalBoundary::Included(event_times[source_index]),
                            TemporalBoundary::Included(
                                EventTime::parse_rfc3339(&relation.source_event_end)
                                    .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                            ),
                            TemporalPrecision::Second,
                        )
                        .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                        TemporalInterval::bounded(
                            TemporalBoundary::Included(event_times[target_index]),
                            TemporalBoundary::Included(
                                EventTime::parse_rfc3339(&relation.target_event_end)
                                    .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                            ),
                            TemporalPrecision::Second,
                        )
                        .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                    )
                    .map_err(|_| AnalysisWorkerError::InvalidInput)?,
                )
                .map_err(|_| AnalysisWorkerError::InvalidInput)?;
        }
        if transition_count == 0 {
            return Err(AnalysisWorkerError::InvalidInput);
        }
        let input = ReferenceTopicInput::new(
            &snapshot,
            document_ids,
            &terms,
            &event_times,
            covariates.as_ref(),
            &memberships,
            &relations,
        )
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
        let config = ReferenceTopicModelConfig::new(
            self.model.topic_count,
            self.model.seeds.clone(),
            self.model.maximum_iterations,
            self.model.tolerance,
        )
        .map_err(|_| AnalysisWorkerError::InvalidInput)?
        .with_hyperparameters(
            self.model.prior_variance,
            self.model.relation_strength,
            self.model.ridge,
            self.model.topic_smoothing,
            self.model.step_size,
        )
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
        Ok(ValidatedTopicLineageInput { input, config })
    }
}

fn sparse(input: &TopicSparseInput, rows: usize) -> Result<SparseMatrix, AnalysisWorkerError> {
    SparseMatrix::from_csr(
        rows,
        input.columns,
        input.offsets.clone(),
        input.indices.clone(),
        input.values.clone(),
    )
    .map_err(|_| AnalysisWorkerError::InvalidInput)
}

fn require_provenance(
    available_time: &str,
    digest: &str,
    cutoff: KnowledgeCutoff,
) -> Result<(), AnalysisWorkerError> {
    let available = AvailableTime::parse_rfc3339(available_time)
        .map_err(|_| AnalysisWorkerError::InvalidInput)?;
    if available.instant() > cutoff.instant()
        || digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AnalysisWorkerError::InvalidInput);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        TOPIC_LINEAGE_WORKER_INPUT_VERSION, TopicDocumentInput, TopicLineageWorkerInput,
        TopicMembershipInput, TopicModelInput, TopicRelationInput, TopicSparseInput,
    };
    use crate::AnalysisWorkerError;
    use relation_graph::RelationEvidenceStatus;
    use temporal_core::KnowledgeCutoff;
    use uuid::Uuid;

    fn fixture() -> TopicLineageWorkerInput {
        let documents = (1_u128..=4)
            .map(|value| TopicDocumentInput {
                document_id: Uuid::from_u128(value),
                event_time: format!("2026-07-{value:02}T00:00:00Z"),
                available_time: "2026-07-01T00:00:00Z".into(),
            })
            .collect::<Vec<_>>();
        let memberships = documents
            .iter()
            .map(|document| TopicMembershipInput {
                document_id: document.document_id,
                group_id: Uuid::from_u128(100),
                role: "project".into(),
                weight: 1.0,
                valid_from: "2026-07-01T00:00:00Z".into(),
                valid_to: "2026-07-09T00:00:00Z".into(),
                available_time: "2026-07-01T00:00:00Z".into(),
                evidence_id: Uuid::from_u128(1_000 + document.document_id.as_u128()),
                evidence_sha256: "ab".repeat(32),
            })
            .collect();
        let relations = (1_u128..=3)
            .map(|value| TopicRelationInput {
                source_document_id: Uuid::from_u128(value),
                target_document_id: Uuid::from_u128(value + 1),
                kind: "transitions_to".into(),
                evidence_status: RelationEvidenceStatus::Observed,
                source_event_end: format!("2026-07-{value:02}T12:00:00Z"),
                target_event_end: format!("2026-07-{:02}T12:00:00Z", value + 1),
                available_time: "2026-07-05T00:00:00Z".into(),
                evidence_id: Uuid::from_u128(2_000 + value),
                evidence_sha256: "cd".repeat(32),
            })
            .collect();
        let mut input = TopicLineageWorkerInput {
            contract_version: TOPIC_LINEAGE_WORKER_INPUT_VERSION,
            reproducibility_manifest_id: Uuid::nil(),
            snapshot_id: "snapshot-topic-lineage".into(),
            scientific_input_sha256: String::new(),
            documents,
            document_term: TopicSparseInput {
                columns: 4,
                offsets: vec![0, 2, 4, 6, 8],
                indices: vec![0, 1, 0, 1, 2, 3, 2, 3],
                values: vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
            },
            covariates: None,
            memberships,
            relations,
            model: TopicModelInput {
                topic_count: 2,
                seeds: vec![7, 11],
                maximum_iterations: 2_000,
                tolerance: 1e-5,
                prior_variance: 1.0,
                relation_strength: 0.5,
                ridge: 0.01,
                topic_smoothing: 0.05,
                step_size: 0.2,
            },
        };
        input.scientific_input_sha256 = input.scientific_digest().expect("digest");
        input
    }

    #[test]
    fn builds_existing_reference_input_and_rejects_future_provenance() {
        let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
        let input = fixture();
        assert_eq!(
            input.scientific_input_sha256,
            input.scientific_digest().expect("digest")
        );
        let validated = input.validate(cutoff).expect("validated");
        assert_eq!(validated.input.document_count(), 4);
        assert_eq!(validated.input.vocabulary_size(), 4);

        let mut future = fixture();
        future.memberships[0].available_time = "2026-08-02T00:00:00Z".into();
        future.scientific_input_sha256 = future.scientific_digest().expect("digest");
        assert!(matches!(
            future.validate(cutoff),
            Err(AnalysisWorkerError::InvalidInput)
        ));
    }
}
