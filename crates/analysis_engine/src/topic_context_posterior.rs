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

/// One explicit active, dormant, or reactivated interval for a global topic.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicActivityInterval {
    /// Artifact-local global topic index.
    pub topic_index: u64,
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
    /// Source global topic index.
    pub source_topic_index: u64,
    /// Optional target global topic index.
    pub target_topic_index: Option<u64>,
    /// Event time.
    pub event_time: String,
    /// Digest of event evidence.
    pub evidence_sha256: String,
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
    /// Exact model contract version.
    pub model_contract_version: String,
    /// Opaque posterior draw-set identity.
    pub posterior_draw_set_id: String,
    /// Number of draws present for every document.
    pub posterior_draw_count: u64,
    /// Number of global topics.
    pub topic_count: u64,
    /// Explicit topic-state intervals.
    pub activity_intervals: Vec<TopicActivityInterval>,
    /// Explicit topic lineage events, when present.
    pub lineage_events: Vec<TopicLineageEvent>,
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

fn time(value: &str) -> Option<KnowledgeCutoff> {
    KnowledgeCutoff::parse_rfc3339(value).ok()
}

impl TopicContextPosteriorArtifact {
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
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
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
        if self.schema_version != TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || !digest(&self.source_snapshot_sha256)
            || time(&self.knowledge_cutoff).is_none()
            || !valid_identifier(&self.model_contract_version)
            || !valid_identifier(&self.posterior_draw_set_id)
            || self.posterior_draw_count == 0
            || self.topic_count < 2
            || self.activity_intervals.len() > ENTRY_LIMIT
            || self.lineage_events.len() > ENTRY_LIMIT
            || self.plausible_values.len() > ENTRY_LIMIT
            || self.memberships.len() > ENTRY_LIMIT
            || self.inference_status != "posterior_topic_coordinates_not_importance"
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        for interval in &self.activity_intervals {
            if interval.topic_index >= self.topic_count
                || !["active", "dormant", "reactivated"].contains(&interval.state_code.as_str())
                || time(&interval.valid_from)
                    .zip(time(&interval.valid_to))
                    .is_none_or(|(valid_from, valid_to)| valid_from > valid_to)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
        for event in &self.lineage_events {
            if !["birth", "split", "merge", "retirement"].contains(&event.event_code.as_str())
                || event.source_topic_index >= self.topic_count
                || event
                    .target_topic_index
                    .is_some_and(|target| target >= self.topic_count)
                || time(&event.event_time).is_none()
                || !digest(&event.evidence_sha256)
            {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
        let mut draws: BTreeMap<Uuid, BTreeSet<u64>> = BTreeMap::new();
        let mut event_times = BTreeMap::new();
        for value in &self.plausible_values {
            let document = Uuid::parse_str(&value.document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let event_time = time(&value.event_time).ok_or(AnalysisEngineError::InvalidEvidence)?;
            if value.draw_index >= self.posterior_draw_count
                || value.logistic_normal_coordinates.len() != (self.topic_count - 1) as usize
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
                .any(|indices| indices.len() != self.posterior_draw_count as usize)
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        let mut dimensions: BTreeMap<Uuid, BTreeSet<&str>> = BTreeMap::new();
        let mut membership_keys = BTreeSet::new();
        for membership in &self.memberships {
            let document = Uuid::parse_str(&membership.document_id)
                .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
            let valid_from =
                time(&membership.valid_from).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let valid_to =
                time(&membership.valid_to).ok_or(AnalysisEngineError::InvalidEvidence)?;
            let document_event_time = event_times
                .get(&document)
                .ok_or(AnalysisEngineError::InvalidEvidence)?;
            if !draws.contains_key(&document)
                || !DIMENSIONS.contains(&membership.dimension_code.as_str())
                || !valid_identifier(&membership.context_id)
                || !membership.weight.is_finite()
                || membership.weight <= 0.0
                || valid_from > valid_to
                || *document_event_time < valid_from
                || *document_event_time > valid_to
                || !digest(&membership.evidence_sha256)
                || !membership_keys.insert((
                    document,
                    membership.dimension_code.as_str(),
                    membership.context_id.as_str(),
                    membership.valid_from.as_str(),
                    membership.valid_to.as_str(),
                ))
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
    use super::{
        TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION, TopicActivityInterval, TopicContextMembership,
        TopicContextPosteriorArtifact, TopicPostPlausibleValue,
    };

    fn artifact() -> TopicContextPosteriorArtifact {
        let documents = [
            "018f3f7a-7b7c-7d00-8000-000000000001",
            "018f3f7a-7b7c-7d00-8000-000000000002",
        ];
        TopicContextPosteriorArtifact {
            schema_version: TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            source_snapshot_sha256: "a".repeat(64),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "trsl-tm-v1".into(),
            posterior_draw_set_id: "draw-set-1".into(),
            posterior_draw_count: 2,
            topic_count: 2,
            activity_intervals: vec![TopicActivityInterval {
                topic_index: 0,
                state_code: "active".into(),
                valid_from: "2026-07-01T00:00:00Z".into(),
                valid_to: "2026-08-01T00:00:00Z".into(),
            }],
            lineage_events: vec![],
            plausible_values: documents
                .iter()
                .flat_map(|document| {
                    (0..2).map(|draw| TopicPostPlausibleValue {
                        document_id: (*document).into(),
                        draw_index: draw,
                        event_time: "2026-07-15T00:00:00Z".into(),
                        logistic_normal_coordinates: vec![draw as f64 / 10.0],
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
        assert_eq!(
            TopicContextPosteriorArtifact::from_json(&json).expect("parse"),
            artifact
        );
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
    }

    #[test]
    fn compares_absolute_instants_and_requires_membership_coverage() {
        let mut equivalent_offsets = artifact();
        equivalent_offsets.activity_intervals[0].valid_from = "2026-07-01T09:00:00+09:00".into();
        equivalent_offsets.activity_intervals[0].valid_to = "2026-07-01T00:00:00Z".into();
        assert!(equivalent_offsets.to_json().is_ok());

        let mut reversed = artifact();
        reversed.activity_intervals[0].valid_from = "2026-07-02T00:00:00Z".into();
        reversed.activity_intervals[0].valid_to = "2026-07-01T23:00:00Z".into();
        assert!(reversed.to_json().is_err());

        let mut uncovered = artifact();
        uncovered.memberships[0].valid_to = "2026-07-14T23:59:59Z".into();
        assert!(uncovered.to_json().is_err());
    }
}
