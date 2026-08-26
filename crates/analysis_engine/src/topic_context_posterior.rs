//! Posterior TRSL-TM producer contract for downstream context influence.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use uuid::Uuid;

use crate::{AnalysisEngineError, format_digest, valid_identifier};

/// Exact posterior artifact schema.
pub const TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION: &str = "tepp.topic_context_posterior.v1";
/// Maximum canonical JSON size.
pub const TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT: usize = 16 * 1024 * 1024;
const ENTRY_LIMIT: usize = 1_000_000;
const DIMENSIONS: [&str; 4] = ["business_unit", "process_unit", "team", "person"];
type PosteriorDraws = BTreeMap<Uuid, BTreeSet<u64>>;
type DocumentEventTimes = BTreeMap<Uuid, KnowledgeCutoff>;

/// One explicit active, dormant, or reactivated interval for a global topic.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicActivityInterval {
    /// Opaque stable topic identity.
    pub topic_id: String,
    /// Activity state: `active`, `dormant`, or `reactivated`.
    pub state_code: String,
    /// Inclusive event-time start.
    pub valid_from: String,
    /// Inclusive event-time end.
    pub valid_to: String,
}

/// One explicit topic birth/split/merge/retirement event, when fitted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicLineageEvent {
    /// Event kind supplied by TEPP, never inferred by a consumer.
    pub event_code: String,
    /// Source stable topic identity.
    pub source_topic_id: String,
    /// Optional target stable topic identity.
    pub target_topic_id: Option<String>,
    /// Event time.
    pub event_time: String,
    /// Digest of event evidence.
    pub evidence_sha256: String,
    /// Opaque evidence resource identity.
    pub evidence_resource_id: String,
    /// Opaque provenance assertion identity.
    pub provenance_assertion_id: String,
}

/// One admitted Event Lineage or document-relation record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicDocumentRelation {
    /// Opaque source document identity.
    pub source_document_id: String,
    /// Opaque target document identity.
    pub target_document_id: String,
    /// Producer-owned closed relation kind.
    pub relation_kind_code: String,
    /// Event time at which the relation is admitted.
    pub event_time: String,
    /// Digest of relation evidence.
    pub evidence_sha256: String,
    /// Opaque evidence resource identity.
    pub evidence_resource_id: String,
    /// Opaque provenance assertion identity.
    pub provenance_assertion_id: String,
}

/// One document posterior plausible value for one draw.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicPostPlausibleValue {
    /// Opaque document identity.
    pub document_id: String,
    /// Posterior draw index.
    pub draw_index: u64,
    /// Event time used by the model.
    pub event_time: String,
    /// Full-rank logistic-normal coordinates of length `topic_count - 1`.
    pub logistic_normal_coordinates: Vec<f64>,
}

/// One time-valid provenance-bound organizational membership.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicContextMembership {
    /// Opaque document identity.
    pub document_id: String,
    /// `business_unit`, `process_unit`, `team`, or `person`.
    pub dimension_code: String,
    /// Opaque context identity within the dimension.
    pub context_id: String,
    /// Source-derived multiple-membership weight.
    pub weight: f64,
    /// Inclusive event-time validity start.
    pub valid_from: String,
    /// Inclusive event-time validity end.
    pub valid_to: String,
    /// Digest of membership source evidence.
    pub evidence_sha256: String,
    /// Opaque evidence resource identity.
    pub evidence_resource_id: String,
    /// Opaque provenance assertion identity.
    pub provenance_assertion_id: String,
}

/// Digest-bound posterior artifact consumed by fast-mlsirm.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicContextPosteriorArtifact {
    /// Exact schema identity.
    pub schema_version: String,
    /// Opaque model-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Canonical source snapshot SHA-256.
    pub source_snapshot_sha256: String,
    /// Historical knowledge cutoff.
    pub knowledge_cutoff: String,
    /// Exact event-time clock represented by all temporal fields.
    pub event_clock_code: String,
    /// Exact model contract version.
    pub model_contract_version: String,
    /// Opaque posterior draw-set identity.
    pub posterior_draw_set_id: String,
    /// Number of draws present for every document.
    pub posterior_draw_count: u64,
    /// Number of global topics.
    pub topic_count: u64,
    /// Stable topic identities in logistic-normal coordinate order.
    pub topic_ids: Vec<String>,
    /// Explicit topic-state intervals.
    pub activity_intervals: Vec<TopicActivityInterval>,
    /// Explicit topic lineage events, when present.
    pub lineage_events: Vec<TopicLineageEvent>,
    /// Complete admitted Event Lineage/document relations.
    pub document_relations: Vec<TopicDocumentRelation>,
    /// Complete document-by-draw posterior coordinates.
    pub plausible_values: Vec<TopicPostPlausibleValue>,
    /// Time-valid BU/PU/team/person memberships.
    pub memberships: Vec<TopicContextMembership>,
    /// Fixed interpretation boundary.
    pub inference_status: String,
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn provenance_binding(fields: &[&[u8]]) -> String {
    let mut digest = Sha256::new();
    for field in fields {
        digest.update(
            u64::try_from(field.len())
                .expect("bounded artifact field length fits u64")
                .to_le_bytes(),
        );
        digest.update(field);
    }
    format_digest(digest.finalize())
}

fn time(value: &str) -> Option<KnowledgeCutoff> {
    KnowledgeCutoff::parse_rfc3339(value).ok()
}

fn canonical_time(value: &str) -> Option<KnowledgeCutoff> {
    time(value).filter(|instant| instant.to_rfc3339() == value)
}

fn valid_activity_interval(interval: &TopicActivityInterval, topic_ids: &BTreeSet<&str>) -> bool {
    topic_ids.contains(interval.topic_id.as_str())
        && ["active", "dormant", "reactivated"].contains(&interval.state_code.as_str())
        && canonical_time(&interval.valid_from)
            .zip(canonical_time(&interval.valid_to))
            .is_some_and(|(valid_from, valid_to)| valid_from <= valid_to)
}

fn within_entry_limits(lengths: [usize; 5], entry_limit: usize) -> bool {
    lengths.into_iter().all(|length| length <= entry_limit)
}

impl TopicContextPosteriorArtifact {
    fn has_valid_header(&self) -> bool {
        self.has_valid_header_with_entry_limit(ENTRY_LIMIT)
    }

    fn has_valid_header_with_entry_limit(&self, entry_limit: usize) -> bool {
        self.schema_version == TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION
            && valid_identifier(&self.run_id)
            && valid_identifier(&self.snapshot_id)
            && digest(&self.source_snapshot_sha256)
            && canonical_time(&self.knowledge_cutoff).is_some()
            && self.event_clock_code == "event_time_rfc3339"
            && valid_identifier(&self.model_contract_version)
            && valid_identifier(&self.posterior_draw_set_id)
            && self.posterior_draw_count > 0
            && self.topic_count >= 2
            && usize::try_from(self.topic_count) == Ok(self.topic_ids.len())
            && within_entry_limits(
                [
                    self.activity_intervals.len(),
                    self.lineage_events.len(),
                    self.document_relations.len(),
                    self.plausible_values.len(),
                    self.memberships.len(),
                ],
                entry_limit,
            )
            && self.inference_status == "posterior_topic_coordinates_not_importance"
    }

    /// Parse and validate one bounded posterior artifact.
    ///
    /// # Errors
    ///
    /// Returns a size or evidence error for any foreign, incomplete,
    /// non-finite, temporally invalid, or mixed-identity payload.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact =
            serde_json::from_str(payload).map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        Self::validate(&artifact)?;
        Ok(artifact)
    }

    /// Serialize canonical validated JSON.
    ///
    /// # Errors
    ///
    /// Returns a validation, serialization, or size error.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let mut canonical = self.clone();
        canonical.activity_intervals.sort_by(|a, b| {
            (&a.topic_id, &a.valid_from, &a.valid_to, &a.state_code).cmp(&(
                &b.topic_id,
                &b.valid_from,
                &b.valid_to,
                &b.state_code,
            ))
        });
        canonical.lineage_events.sort_by(|a, b| {
            (
                &a.event_code,
                &a.source_topic_id,
                &a.target_topic_id,
                &a.event_time,
                &a.evidence_resource_id,
                &a.provenance_assertion_id,
            )
                .cmp(&(
                    &b.event_code,
                    &b.source_topic_id,
                    &b.target_topic_id,
                    &b.event_time,
                    &b.evidence_resource_id,
                    &b.provenance_assertion_id,
                ))
        });
        canonical.document_relations.sort_by(|a, b| {
            (
                &a.source_document_id,
                &a.target_document_id,
                &a.relation_kind_code,
                &a.event_time,
                &a.evidence_resource_id,
                &a.provenance_assertion_id,
            )
                .cmp(&(
                    &b.source_document_id,
                    &b.target_document_id,
                    &b.relation_kind_code,
                    &b.event_time,
                    &b.evidence_resource_id,
                    &b.provenance_assertion_id,
                ))
        });
        canonical
            .plausible_values
            .sort_by(|a, b| (&a.document_id, a.draw_index).cmp(&(&b.document_id, b.draw_index)));
        canonical.memberships.sort_by(|a, b| {
            (
                &a.document_id,
                &a.dimension_code,
                &a.context_id,
                &a.valid_from,
                &a.valid_to,
            )
                .cmp(&(
                    &b.document_id,
                    &b.dimension_code,
                    &b.context_id,
                    &b.valid_from,
                    &b.valid_to,
                ))
        });
        let payload = serde_json::to_string(&canonical)
            .map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(payload)
    }

    /// Return the canonical artifact SHA-256.
    ///
    /// # Errors
    ///
    /// Returns a validation or serialization error.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        if !self.has_valid_header() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let cutoff =
            canonical_time(&self.knowledge_cutoff).ok_or(AnalysisEngineError::InvalidEvidence)?;
        let topic_ids: BTreeSet<&str> = self.topic_ids.iter().map(String::as_str).collect();
        if topic_ids.len() != self.topic_ids.len()
            || self
                .topic_ids
                .iter()
                .any(|topic_id| Uuid::parse_str(topic_id).is_err())
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        self.validate_topic_records(cutoff, &topic_ids)?;
        let (draws, event_times) = self.validate_plausible_values(cutoff)?;
        self.validate_document_relations(cutoff, &draws, &event_times)?;
        self.validate_memberships(&draws, &event_times)?;
        self.validate_provenance_bindings()
    }

    fn validate_provenance_bindings(&self) -> Result<(), AnalysisEngineError> {
        let mut bindings = BTreeMap::new();
        let mut bind = |id: &str, value: String| {
            if bindings
                .insert(id.to_owned(), value.clone())
                .is_some_and(|existing| existing != value)
            {
                Err(AnalysisEngineError::InvalidEvidence)
            } else {
                Ok(())
            }
        };
        for event in &self.lineage_events {
            let fields: [&[u8]; 8] = [
                b"topic",
                event.event_code.as_bytes(),
                event.source_topic_id.as_bytes(),
                event.target_topic_id.as_deref().unwrap_or("").as_bytes(),
                event.event_time.as_bytes(),
                event.evidence_resource_id.as_bytes(),
                event.evidence_sha256.as_bytes(),
                self.source_snapshot_sha256.as_bytes(),
            ];
            bind(&event.provenance_assertion_id, provenance_binding(&fields))?;
        }
        for relation in &self.document_relations {
            let fields: [&[u8]; 8] = [
                b"document",
                relation.relation_kind_code.as_bytes(),
                relation.source_document_id.as_bytes(),
                relation.target_document_id.as_bytes(),
                relation.event_time.as_bytes(),
                relation.evidence_resource_id.as_bytes(),
                relation.evidence_sha256.as_bytes(),
                self.source_snapshot_sha256.as_bytes(),
            ];
            bind(
                &relation.provenance_assertion_id,
                provenance_binding(&fields),
            )?;
        }
        for membership in &self.memberships {
            let weight = membership.weight.to_bits().to_le_bytes();
            let fields: [&[u8]; 10] = [
                b"membership",
                membership.dimension_code.as_bytes(),
                membership.document_id.as_bytes(),
                membership.context_id.as_bytes(),
                &weight,
                membership.valid_from.as_bytes(),
                membership.valid_to.as_bytes(),
                membership.evidence_resource_id.as_bytes(),
                membership.evidence_sha256.as_bytes(),
                self.source_snapshot_sha256.as_bytes(),
            ];
            bind(
                &membership.provenance_assertion_id,
                provenance_binding(&fields),
            )?;
        }
        Ok(())
    }

    fn validate_topic_records(
        &self,
        cutoff: KnowledgeCutoff,
        topic_ids: &BTreeSet<&str>,
    ) -> Result<(), AnalysisEngineError> {
        let mut activity_by_topic: BTreeMap<&str, Vec<(KnowledgeCutoff, KnowledgeCutoff, &str)>> =
            BTreeMap::new();
        let mut seen_activity = BTreeSet::new();
        for interval in &self.activity_intervals {
            if !valid_activity_interval(interval, topic_ids) {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            let valid_from =
                canonical_time(&interval.valid_from).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let valid_to =
                canonical_time(&interval.valid_to).ok_or(AnalysisEngineError::InvalidEvidence)?;
            if valid_to > cutoff {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            let key = (
                interval.topic_id.as_str(),
                valid_from,
                valid_to,
                interval.state_code.as_str(),
            );
            if !seen_activity.insert(key) {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            activity_by_topic
                .entry(&interval.topic_id)
                .or_default()
                .push((valid_from, valid_to, interval.state_code.as_str()));
        }
        if activity_by_topic.len() != topic_ids.len() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        for intervals in activity_by_topic.values_mut() {
            intervals.sort();
            if intervals.first().map(|interval| interval.2) != Some("active") {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            for pair in intervals.windows(2) {
                let valid_transition = matches!(
                    (pair[0].2, pair[1].2),
                    ("active" | "reactivated", "dormant") | ("dormant", "reactivated")
                );
                if pair[1].0 <= pair[0].1 || !valid_transition {
                    return Err(AnalysisEngineError::InvalidEvidence);
                }
            }
        }
        let mut seen_lineage = BTreeSet::new();
        for event in &self.lineage_events {
            let event_time =
                canonical_time(&event.event_time).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let key = (
                event.event_code.as_str(),
                event.source_topic_id.as_str(),
                event.target_topic_id.as_deref(),
                event_time,
                event.evidence_resource_id.as_str(),
                event.provenance_assertion_id.as_str(),
            );
            if !["birth", "split", "merge", "retirement"].contains(&event.event_code.as_str())
                || !topic_ids.contains(event.source_topic_id.as_str())
                || event
                    .target_topic_id
                    .as_deref()
                    .is_some_and(|target| !topic_ids.contains(target))
                || match event.event_code.as_str() {
                    "birth" | "retirement" => event.target_topic_id.is_some(),
                    // Only "split"/"merge" remain; their target must be None
                    // or self-referencing.
                    _ => event
                        .target_topic_id
                        .as_deref()
                        .is_none_or(|target| target == event.source_topic_id),
                }
                || event_time > cutoff
                || !digest(&event.evidence_sha256)
                || !valid_identifier(&event.evidence_resource_id)
                || !valid_identifier(&event.provenance_assertion_id)
                || !seen_lineage.insert(key)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
        Ok(())
    }

    fn validate_plausible_values(
        &self,
        cutoff: KnowledgeCutoff,
    ) -> Result<(PosteriorDraws, DocumentEventTimes), AnalysisEngineError> {
        let mut draws: PosteriorDraws = BTreeMap::new();
        let mut event_times = BTreeMap::new();
        for value in &self.plausible_values {
            let document = Uuid::parse_str(&value.document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let event_time =
                canonical_time(&value.event_time).ok_or(AnalysisEngineError::InvalidEvidence)?;
            if value.draw_index >= self.posterior_draw_count
                || event_time > cutoff
                || value.logistic_normal_coordinates.len()
                    != usize::try_from(self.topic_count - 1)
                        .map_err(|_| AnalysisEngineError::InvalidEvidence)?
                || value
                    .logistic_normal_coordinates
                    .iter()
                    .any(|coordinate| !coordinate.is_finite())
                || !draws.entry(document).or_default().insert(value.draw_index)
                || event_times
                    .insert(document, event_time)
                    .is_some_and(|previous| previous != event_time)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
        if draws.len() < 2
            || draws
                .values()
                .any(|indices| usize::try_from(self.posterior_draw_count) != Ok(indices.len()))
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok((draws, event_times))
    }

    fn validate_document_relations(
        &self,
        cutoff: KnowledgeCutoff,
        draws: &PosteriorDraws,
        event_times: &DocumentEventTimes,
    ) -> Result<(), AnalysisEngineError> {
        let mut seen_relations = BTreeSet::new();
        for relation in &self.document_relations {
            let source = Uuid::parse_str(&relation.source_document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let target = Uuid::parse_str(&relation.target_document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let event_time =
                canonical_time(&relation.event_time).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let key = (
                source,
                target,
                relation.relation_kind_code.as_str(),
                event_time,
                relation.evidence_resource_id.as_str(),
                relation.provenance_assertion_id.as_str(),
            );
            if source == target
                || !draws.contains_key(&source)
                || !draws.contains_key(&target)
                || relation.relation_kind_code != "event_lineage_precedes"
                || event_times.get(&source) > event_times.get(&target)
                || event_time > cutoff
                || !digest(&relation.evidence_sha256)
                || !valid_identifier(&relation.evidence_resource_id)
                || !valid_identifier(&relation.provenance_assertion_id)
                || !seen_relations.insert(key)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
        Ok(())
    }

    fn validate_memberships(
        &self,
        draws: &PosteriorDraws,
        event_times: &DocumentEventTimes,
    ) -> Result<(), AnalysisEngineError> {
        let mut dimensions: BTreeMap<Uuid, BTreeSet<&str>> = BTreeMap::new();
        let mut seen_memberships = BTreeSet::new();
        for membership in &self.memberships {
            let document = Uuid::parse_str(&membership.document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let valid_from = canonical_time(&membership.valid_from)
                .ok_or(AnalysisEngineError::InvalidEvidence)?;
            let valid_to =
                canonical_time(&membership.valid_to).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let document_event_time = event_times
                .get(&document)
                .ok_or(AnalysisEngineError::InvalidEvidence)?;
            let dimension_ordinal = DIMENSIONS
                .iter()
                .position(|dimension| *dimension == membership.dimension_code)
                .ok_or(AnalysisEngineError::InvalidEvidence)?;
            let key = (
                document,
                dimension_ordinal,
                membership.context_id.as_str(),
                valid_from,
                valid_to,
            );
            if !valid_identifier(&membership.context_id)
                || !membership.weight.is_finite()
                || membership.weight <= 0.0
                || valid_from > valid_to
                || *document_event_time < valid_from
                || *document_event_time > valid_to
                || !digest(&membership.evidence_sha256)
                || !valid_identifier(&membership.evidence_resource_id)
                || !valid_identifier(&membership.provenance_assertion_id)
                || !seen_memberships.insert(key)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            dimensions
                .entry(document)
                .or_default()
                .insert(membership.dimension_code.as_str());
        }
        if dimensions.len() != draws.len()
            || dimensions.values().any(|present| {
                DIMENSIONS
                    .iter()
                    .any(|required| !present.contains(required))
            })
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisEngineError;
    use super::{
        ENTRY_LIMIT, TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT, TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION,
        TopicActivityInterval, TopicContextMembership, TopicContextPosteriorArtifact,
        TopicDocumentRelation, TopicLineageEvent, TopicPostPlausibleValue, within_entry_limits,
    };

    macro_rules! invalid {
        ($change:expr) => {{
            let mut candidate = artifact();
            $change(&mut candidate);
            assert!(candidate.to_json().is_err());
        }};
    }

    fn artifact() -> TopicContextPosteriorArtifact {
        let documents = [
            "018f3f7a-7b7c-7d00-8000-000000000001",
            "018f3f7a-7b7c-7d00-8000-000000000002",
        ];
        TopicContextPosteriorArtifact {
            schema_version: TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            source_snapshot_sha256: "0".repeat(64),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            event_clock_code: "event_time_rfc3339".into(),
            model_contract_version: "trsl-tm-v1".into(),
            posterior_draw_set_id: "draw-set-1".into(),
            posterior_draw_count: 2,
            topic_count: 2,
            topic_ids: vec![
                "018f3f7a-7b7c-7d00-8000-000000000101".into(),
                "018f3f7a-7b7c-7d00-8000-000000000102".into(),
            ],
            activity_intervals: [
                "018f3f7a-7b7c-7d00-8000-000000000101",
                "018f3f7a-7b7c-7d00-8000-000000000102",
            ]
            .map(|topic_id| TopicActivityInterval {
                topic_id: topic_id.into(),
                state_code: "active".into(),
                valid_from: "2026-07-01T00:00:00Z".into(),
                valid_to: "2026-07-15T00:00:00Z".into(),
            })
            .into(),
            lineage_events: vec![],
            document_relations: vec![TopicDocumentRelation {
                source_document_id: documents[0].into(),
                target_document_id: documents[1].into(),
                relation_kind_code: "event_lineage_precedes".into(),
                event_time: "2026-07-15T00:00:00Z".into(),
                evidence_sha256: "c".repeat(64),
                evidence_resource_id: "evidence-relation-1".into(),
                provenance_assertion_id: "provenance-relation-1".into(),
            }],
            plausible_values: documents
                .iter()
                .flat_map(|document| {
                    (0..2).map(|draw| TopicPostPlausibleValue {
                        document_id: (*document).into(),
                        draw_index: draw,
                        event_time: "2026-07-15T00:00:00Z".into(),
                        logistic_normal_coordinates: vec![if draw == 0 { 0.0 } else { 0.1 }],
                    })
                })
                .collect(),
            memberships: documents
                .iter()
                .flat_map(|document| {
                    ["business_unit", "process_unit", "team", "person"].map(|dimension| {
                        TopicContextMembership {
                            document_id: (*document).into(),
                            dimension_code: dimension.into(),
                            context_id: format!("{dimension}-{document}"),
                            weight: 1.0,
                            valid_from: "2026-07-01T00:00:00Z".into(),
                            valid_to: "2026-08-01T00:00:00Z".into(),
                            evidence_sha256: "b".repeat(64),
                            evidence_resource_id: format!("evidence-{dimension}-{document}"),
                            provenance_assertion_id: format!("provenance-{dimension}-{document}"),
                        }
                    })
                })
                .collect(),
            inference_status: "posterior_topic_coordinates_not_importance".into(),
        }
    }

    #[test]
    fn round_trip_preserves_plausible_values() {
        let artifact = artifact();
        let json = artifact.to_json().expect("json");
        let parsed = TopicContextPosteriorArtifact::from_json(&json).expect("parse");
        assert_eq!(parsed.to_json().expect("canonical"), json);
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
    }

    #[test]
    fn rejects_missing_draw_instead_of_collapsing_uncertainty() {
        let mut incomplete = artifact();
        incomplete.plausible_values.pop();
        assert!(incomplete.to_json().is_err());

        let mut duplicate = artifact();
        duplicate
            .plausible_values
            .push(duplicate.plausible_values[0].clone());
        assert!(duplicate.to_json().is_err());

        let mut reordered = artifact();
        reordered.plausible_values.swap(0, 1);
        assert_eq!(reordered.to_json(), artifact().to_json());
        assert_eq!(reordered.sha256(), artifact().sha256());

        let mut distinct_evidence = artifact();
        let mut relation = distinct_evidence.document_relations[0].clone();
        relation.evidence_resource_id = "evidence-relation-0".into();
        distinct_evidence.document_relations.push(relation);
        let lineage = TopicLineageEvent {
            event_code: "split".into(),
            source_topic_id: distinct_evidence.topic_ids[0].clone(),
            target_topic_id: Some(distinct_evidence.topic_ids[1].clone()),
            event_time: "2026-07-15T00:00:00Z".into(),
            evidence_sha256: "d".repeat(64),
            evidence_resource_id: "evidence-lineage-1".into(),
            provenance_assertion_id: "provenance-lineage-canonical".into(),
        };
        distinct_evidence.lineage_events.extend([
            lineage.clone(),
            TopicLineageEvent {
                evidence_resource_id: "evidence-lineage-0".into(),
                ..lineage
            },
        ]);
        let canonical_json = distinct_evidence.to_json();
        distinct_evidence.document_relations.swap(0, 1);
        distinct_evidence.lineage_events.swap(0, 1);
        assert_eq!(distinct_evidence.to_json(), canonical_json);
    }

    #[test]
    fn compares_absolute_instants_and_requires_membership_coverage() {
        let mut equivalent_offsets = artifact();
        equivalent_offsets.activity_intervals[0].valid_from = "2026-07-01T09:00:00+09:00".into();
        equivalent_offsets.activity_intervals[0].valid_to = "2026-07-01T00:00:00Z".into();
        assert!(equivalent_offsets.to_json().is_err());

        let mut reversed = artifact();
        reversed.activity_intervals[0].valid_from = "2026-07-02T00:00:00Z".into();
        reversed.activity_intervals[0].valid_to = "2026-07-01T23:00:00Z".into();
        assert!(reversed.to_json().is_err());

        let mut uncovered = artifact();
        uncovered.memberships[0].valid_to = "2026-07-14T23:59:59Z".into();
        assert!(uncovered.to_json().is_err());
    }

    #[test]
    fn rejects_every_foreign_header_and_bound() {
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.schema_version.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.run_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.snapshot_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .source_snapshot_sha256
            .replace_range(..1, "A"));
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.knowledge_cutoff.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.event_clock_code.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.model_contract_version.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.posterior_draw_set_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.posterior_draw_count = 0);
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.topic_count = 1);
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.topic_ids.pop());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.topic_ids[0].clear());
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.topic_ids[1] =
                value.topic_ids[0].clone()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.inference_status.clear());
        assert!(within_entry_limits([ENTRY_LIMIT; 5], ENTRY_LIMIT));
        assert!(!within_entry_limits(
            [ENTRY_LIMIT + 1, 0, 0, 0, 0],
            ENTRY_LIMIT
        ));
        assert!(!artifact().has_valid_header_with_entry_limit(0));
        assert!(TopicContextPosteriorArtifact::from_json("{").is_err());
        assert!(
            TopicContextPosteriorArtifact::from_json(
                &"x".repeat(TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT + 1)
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_activity_and_lineage_records() {
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0].topic_id =
                "018f3f7a-7b7c-7d00-8000-000000000999".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0]
                .state_code
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0]
                .valid_from
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0].valid_to =
                "2026-08-01T00:00:01Z".into()
        );

        let event = TopicLineageEvent {
            event_code: "birth".into(),
            source_topic_id: "018f3f7a-7b7c-7d00-8000-000000000101".into(),
            target_topic_id: None,
            event_time: "2026-07-15T00:00:00Z".into(),
            evidence_sha256: "c".repeat(64),
            evidence_resource_id: "evidence-lineage-1".into(),
            provenance_assertion_id: "provenance-lineage-1".into(),
        };
        let mut valid = artifact();
        valid.lineage_events.push(event.clone());
        assert!(valid.to_json().is_ok());
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].event_code.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].source_topic_id.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].target_topic_id = Some("missing-topic".into());
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].event_time.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].evidence_sha256.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].evidence_resource_id.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].provenance_assertion_id.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].event_time = "2026-08-01T00:00:01Z".into();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events.push(event.clone());
        });

        let mut overlap = artifact();
        let mut non_overlapping = artifact();
        let topic_id = non_overlapping.activity_intervals[0].topic_id.clone();
        non_overlapping.activity_intervals.extend([
            TopicActivityInterval {
                topic_id: topic_id.clone(),
                state_code: "dormant".into(),
                valid_from: "2026-07-15T00:00:01Z".into(),
                valid_to: "2026-07-20T00:00:00Z".into(),
            },
            TopicActivityInterval {
                topic_id,
                state_code: "reactivated".into(),
                valid_from: "2026-07-20T00:00:01Z".into(),
                valid_to: "2026-08-01T00:00:00Z".into(),
            },
        ]);
        assert!(non_overlapping.to_json().is_ok());
        let canonical_json = non_overlapping.to_json();
        non_overlapping.activity_intervals.swap(0, 1);
        assert_eq!(non_overlapping.to_json(), canonical_json);
        overlap.activity_intervals.push(TopicActivityInterval {
            topic_id: overlap.activity_intervals[0].topic_id.clone(),
            state_code: "dormant".into(),
            valid_from: "2026-07-15T00:00:00Z".into(),
            valid_to: "2026-07-15T00:00:00Z".into(),
        });
        assert!(overlap.to_json().is_err());
    }

    #[test]
    fn rejects_incomplete_topic_state_and_lineage_shapes() {
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .activity_intervals
            .push(value.activity_intervals[0].clone()));
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0].state_code =
                "reactivated".into()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.activity_intervals.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.activity_intervals.push(TopicActivityInterval {
                topic_id: value.activity_intervals[0].topic_id.clone(),
                state_code: "active".into(),
                valid_from: "2026-07-15T00:00:01Z".into(),
                valid_to: "2026-07-16T00:00:00Z".into(),
            });
        });
        let source_topic_id = artifact().topic_ids[0].clone();
        for (event_code, target_topic_id) in
            [("split", None), ("merge", Some(source_topic_id.clone()))]
        {
            invalid!(|value: &mut TopicContextPosteriorArtifact| {
                value.lineage_events.push(TopicLineageEvent {
                    event_code: event_code.into(),
                    source_topic_id: source_topic_id.clone(),
                    target_topic_id: target_topic_id.clone(),
                    event_time: "2026-07-15T00:00:00Z".into(),
                    evidence_sha256: "c".repeat(64),
                    evidence_resource_id: "evidence-lineage-shape".into(),
                    provenance_assertion_id: "provenance-lineage-shape".into(),
                });
            });
        }
    }

    #[test]
    fn rejects_invalid_posterior_records() {
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0]
                .document_id
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0]
                .event_time
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0].draw_index = 2
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0]
                .logistic_normal_coordinates
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0]
                .logistic_normal_coordinates[0] =
                f64::NAN
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[1].event_time =
                "2026-07-16T00:00:00Z".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.plausible_values[0].event_time =
                "2026-08-01T00:00:01Z".into()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.plausible_values.truncate(2));
    }

    #[test]
    fn rejects_invalid_membership_and_relation_records() {
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].document_id.clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].dimension_code.clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].context_id.clear()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.memberships[0].weight = 0.0);
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].weight = f64::NAN
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].valid_from.clear()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.memberships[0].valid_to.clear());
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].valid_from =
                "2026-07-16T00:00:00Z".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].valid_from =
                "2026-08-02T00:00:00Z".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0]
                .evidence_sha256
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0]
                .evidence_resource_id
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0]
                .provenance_assertion_id
                .clear()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .memberships
            .push(value.memberships[0].clone()));
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.memberships.remove(0));
        let mut reordered = artifact();
        reordered.memberships.swap(0, 1);
        assert_eq!(reordered.to_json(), artifact().to_json());
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].document_id =
                "018f3f7a-7b7c-7d00-8000-000000000003".into()
        );

        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .target_document_id =
                value.document_relations[0].source_document_id.clone()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .source_document_id =
                "018f3f7a-7b7c-7d00-8000-000000000003".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .target_document_id =
                "018f3f7a-7b7c-7d00-8000-000000000003".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .relation_kind_code
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0].event_time =
                "2026-08-01T00:00:01Z".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .evidence_sha256
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .evidence_resource_id
                .clear()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .provenance_assertion_id
                .clear()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .document_relations
            .push(value.document_relations[0].clone()));
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.memberships.truncate(4));
    }

    #[test]
    fn rejects_ambiguous_relation_semantics_and_provenance() {
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.document_relations[0]
                .relation_kind_code =
                "associated_with".into()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            for plausible_value in value
                .plausible_values
                .iter_mut()
                .filter(|item| item.document_id.ends_with("0001"))
            {
                plausible_value.event_time = "2026-07-16T00:00:00Z".into();
            }
        });
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0]
                .provenance_assertion_id =
                value.document_relations[0].provenance_assertion_id.clone()
        );

        let mut relation_time_reuse = artifact();
        let mut relation = relation_time_reuse.document_relations[0].clone();
        relation.event_time = "2026-07-14T00:00:00Z".into();
        relation_time_reuse.document_relations.push(relation);
        assert!(relation_time_reuse.to_json().is_err());

        let mut lineage_time_reuse = artifact();
        let lineage = TopicLineageEvent {
            event_code: "birth".into(),
            source_topic_id: lineage_time_reuse.topic_ids[0].clone(),
            target_topic_id: None,
            event_time: "2026-07-15T00:00:00Z".into(),
            evidence_sha256: "c".repeat(64),
            evidence_resource_id: "evidence-lineage-time".into(),
            provenance_assertion_id: "provenance-lineage-time".into(),
        };
        lineage_time_reuse.lineage_events.push(lineage.clone());
        lineage_time_reuse.lineage_events.push(TopicLineageEvent {
            event_time: "2026-07-14T00:00:00Z".into(),
            ..lineage
        });
        assert!(lineage_time_reuse.to_json().is_err());

        let mut membership_window_reuse = artifact();
        let mut membership = membership_window_reuse.memberships[0].clone();
        membership.valid_from = "2026-06-30T00:00:00Z".into();
        membership_window_reuse.memberships.push(membership);
        assert!(membership_window_reuse.to_json().is_err());
    }

    fn append_synthetic_document(
        artifact: &mut TopicContextPosteriorArtifact,
        document: &str,
        context_id: &str,
    ) {
        for draw in 0..artifact.posterior_draw_count {
            artifact.plausible_values.push(TopicPostPlausibleValue {
                document_id: document.to_string(),
                draw_index: draw,
                event_time: "2026-07-15T00:00:00Z".into(),
                logistic_normal_coordinates: vec![0.0],
            });
        }
        for dimension in ["business_unit", "process_unit", "team", "person"] {
            artifact.memberships.push(TopicContextMembership {
                document_id: document.to_string(),
                dimension_code: dimension.into(),
                context_id: context_id.to_string(),
                weight: 1.0,
                valid_from: "2026-07-01T00:00:00Z".into(),
                valid_to: "2026-08-01T00:00:00Z".into(),
                evidence_sha256: "b".repeat(64),
                evidence_resource_id: format!("evidence-{dimension}-{document}"),
                provenance_assertion_id: format!("provenance-{dimension}-{document}"),
            });
        }
    }

    #[test]
    fn to_json_refuses_payloads_over_the_canonical_byte_limit() {
        // Grow the artifact with fully valid synthetic documents — each with
        // the full draw set and all four membership dimensions — until the
        // canonical serialization crosses 16 MiB, so the size branch (and
        // only the size branch) refuses with LimitExceeded. The count is
        // derived from one measured per-document delta, so the loop runs
        // once instead of re-serializing the whole payload per step.
        let probe = artifact();
        let per_document_bytes = {
            let mut one = probe.clone();
            let document =
                ::uuid::Uuid::from_u128(0x8000_0000_0000_0000_0000_0000_0000_0000_u128).to_string();
            append_synthetic_document(&mut one, &document, "context-probe");
            serde_json::to_string(&one).expect("probe").len()
                - serde_json::to_string(&probe).expect("base").len()
        };
        assert!(per_document_bytes > 0);
        let extra_documents = TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT / per_document_bytes + 2;
        let entries_per_document =
            usize::try_from(probe.posterior_draw_count + 4).expect("bounded draw count fits usize");
        let projected_entries = extra_documents * entries_per_document;
        assert!(
            projected_entries <= ENTRY_LIMIT,
            "derived documents must stay inside the entry cap"
        );
        let mut oversized = probe;
        // Bulk-fill up to the estimate, then top up one document at a time
        // (near the limit, only a handful of iterations remain) so rounding
        // differences between the probe and the real entries cannot leave the
        // payload under the threshold.
        for index in 0..extra_documents {
            let document = ::uuid::Uuid::from_u128(
                0x8000_0000_0000_0000_0000_0000_0000_0000_u128
                    + u128::try_from(index).expect("index fits u128"),
            )
            .to_string();
            append_synthetic_document(&mut oversized, &document, &format!("context-{index}"));
        }
        let mut top_up = extra_documents;
        while serde_json::to_string(&oversized)
            .expect("canonical serialization")
            .len()
            <= TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT
        {
            assert!(
                top_up < extra_documents + 1_000,
                "byte limit not crossed within the top-up budget"
            );
            let document = ::uuid::Uuid::from_u128(
                0x8000_0000_0000_0000_0000_0000_0000_0000_u128
                    + u128::try_from(top_up).expect("fits"),
            )
            .to_string();
            append_synthetic_document(
                &mut oversized,
                &document,
                &format!("context-topup-{top_up}"),
            );
            top_up += 1;
        }
        assert_eq!(oversized.to_json(), Err(AnalysisEngineError::LimitExceeded));
    }

    #[test]
    fn canonicalises_out_of_order_lineage_and_relation_sorts() {
        let mut value = artifact();
        let topic_a = value.topic_ids[0].clone();
        let topic_b = value.topic_ids[1].clone();

        // birth/retirement require no target; split/merge may self-reference.
        let event_later = TopicLineageEvent {
            event_code: "retirement".into(),
            source_topic_id: topic_b.clone(),
            target_topic_id: None,
            event_time: "2026-07-20T00:00:00Z".into(),
            evidence_sha256: "b".repeat(64),
            evidence_resource_id: "evidence-lineage-later".into(),
            provenance_assertion_id: "provenance-lineage-later".into(),
        };
        let event_earlier = TopicLineageEvent {
            event_code: "birth".into(),
            source_topic_id: topic_a.clone(),
            target_topic_id: None,
            event_time: "2026-07-10T00:00:00Z".into(),
            evidence_sha256: "a".repeat(64),
            evidence_resource_id: "evidence-lineage-earlier".into(),
            provenance_assertion_id: "provenance-lineage-earlier".into(),
        };

        let relation_later = TopicDocumentRelation {
            source_document_id: "018f3f7a-7b7c-7d00-8000-000000000002".into(),
            target_document_id: "018f3f7a-7b7c-7d00-8000-000000000001".into(),
            relation_kind_code: "event_lineage_precedes".into(),
            event_time: "2026-07-20T00:00:00Z".into(),
            evidence_sha256: "d".repeat(64),
            evidence_resource_id: "evidence-relation-later".into(),
            provenance_assertion_id: "provenance-relation-later".into(),
        };
        let relation_earlier = TopicDocumentRelation {
            source_document_id: "018f3f7a-7b7c-7d00-8000-000000000001".into(),
            target_document_id: "018f3f7a-7b7c-7d00-8000-000000000002".into(),
            relation_kind_code: "event_lineage_precedes".into(),
            event_time: "2026-07-10T00:00:00Z".into(),
            evidence_sha256: "c".repeat(64),
            evidence_resource_id: "evidence-relation-earlier".into(),
            provenance_assertion_id: "provenance-relation-earlier".into(),
        };

        value.lineage_events = vec![event_later, event_earlier];
        value.document_relations = vec![relation_later, relation_earlier];
        let json = value.to_json().expect("canonical serialisation");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        let events = parsed["lineage_events"].as_array().expect("events array");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event_code"], "birth");
        assert_eq!(events[1]["event_code"], "retirement");

        let relations = parsed["document_relations"].as_array().expect("relations");
        assert_eq!(relations.len(), 2);
        assert_eq!(
            relations[0]["source_document_id"],
            "018f3f7a-7b7c-7d00-8000-000000000001"
        );
        assert_eq!(
            relations[1]["source_document_id"],
            "018f3f7a-7b7c-7d00-8000-000000000002"
        );

        let round_tripped = TopicContextPosteriorArtifact::from_json(&json).expect("round-trip");
        assert_eq!(
            round_tripped.lineage_events[0].event_code, "birth",
            "canonical lineage order survives round-trip"
        );
        assert_eq!(
            round_tripped.document_relations[0].source_document_id,
            "018f3f7a-7b7c-7d00-8000-000000000001",
            "canonical relation order survives round-trip"
        );
    }
}
