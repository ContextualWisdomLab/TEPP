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

fn valid_activity_interval(interval: &TopicActivityInterval, topic_count: u64) -> bool {
    interval.topic_index < topic_count
        && ["active", "dormant", "reactivated"].contains(&interval.state_code.as_str())
        && time(&interval.valid_from)
            .zip(time(&interval.valid_to))
            .is_some_and(|(valid_from, valid_to)| valid_from <= valid_to)
}

fn within_entry_limits(lengths: [usize; 4]) -> bool {
    lengths.into_iter().all(|length| length <= ENTRY_LIMIT)
}

impl TopicContextPosteriorArtifact {
    fn has_valid_header(&self) -> bool {
        self.schema_version == TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION
            && valid_identifier(&self.run_id)
            && valid_identifier(&self.snapshot_id)
            && digest(&self.source_snapshot_sha256)
            && time(&self.knowledge_cutoff).is_some()
            && valid_identifier(&self.model_contract_version)
            && valid_identifier(&self.posterior_draw_set_id)
            && self.posterior_draw_count > 0
            && self.topic_count >= 2
            && within_entry_limits([
                self.activity_intervals.len(),
                self.lineage_events.len(),
                self.plausible_values.len(),
                self.memberships.len(),
            ])
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
        if !self.has_valid_header() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        for interval in &self.activity_intervals {
            if !valid_activity_interval(interval, self.topic_count) {
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
            // `event_times` is keyed by exactly the plausible-value
            // documents, so the membership document is guaranteed to be a
            // known draw document here; no redundant contains-key probe.
            if !DIMENSIONS.contains(&membership.dimension_code.as_str())
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
    use super::AnalysisEngineError;
    use super::{
        ENTRY_LIMIT, TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT, TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION,
        TopicActivityInterval, TopicContextMembership, TopicContextPosteriorArtifact,
        TopicLineageEvent, TopicPostPlausibleValue, within_entry_limits,
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
            source_snapshot_sha256: "a0".repeat(32),
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

    #[test]
    fn rejects_every_foreign_header_and_bound() {
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.schema_version.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.run_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.snapshot_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .source_snapshot_sha256
            .replace_range(..1, "A"));
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.knowledge_cutoff.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.model_contract_version.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.posterior_draw_set_id.clear());
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.posterior_draw_count = 0);
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.topic_count = 1);
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.inference_status.clear());
        assert!(within_entry_limits([ENTRY_LIMIT; 4]));
        assert!(!within_entry_limits([ENTRY_LIMIT + 1, 0, 0, 0]));
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
            |value: &mut TopicContextPosteriorArtifact| value.activity_intervals[0].topic_index = 2
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

        let event = TopicLineageEvent {
            event_code: "birth".into(),
            source_topic_index: 0,
            target_topic_index: Some(1),
            event_time: "2026-07-15T00:00:00Z".into(),
            evidence_sha256: "c".repeat(64),
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
            value.lineage_events[0].source_topic_index = 2;
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].target_topic_index = Some(2);
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event.clone());
            value.lineage_events[0].event_time.clear();
        });
        invalid!(|value: &mut TopicContextPosteriorArtifact| {
            value.lineage_events.push(event);
            value.lineage_events[0].evidence_sha256.clear();
        });
    }

    #[test]
    fn rejects_invalid_posterior_and_membership_records() {
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
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.plausible_values.truncate(2));

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
                "2026-08-02T00:00:00Z".into()
        );
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0]
                .evidence_sha256
                .clear()
        );
        invalid!(|value: &mut TopicContextPosteriorArtifact| value
            .memberships
            .push(value.memberships[0].clone()));
        invalid!(|value: &mut TopicContextPosteriorArtifact| value.memberships.remove(0));
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].document_id =
                "018f3f7a-7b7c-7d00-8000-000000000003".into()
        );
        // Document event time strictly after the membership window start:
        // the membership would claim context before the document existed.
        invalid!(
            |value: &mut TopicContextPosteriorArtifact| value.memberships[0].valid_from =
                "2026-07-15T00:00:01Z".into()
        );
        // A draw document with no membership rows at all breaks the
        // document-coverage reconciliation even though every surviving row
        // is individually valid.
        let mut stripped = artifact();
        let orphaned_document = stripped.memberships[0].document_id.clone();
        stripped
            .memberships
            .retain(|membership| membership.document_id != orphaned_document);
        assert!(stripped.to_json().is_err());
    }

    #[test]
    fn rejects_header_when_any_entry_collection_exceeds_the_cap() {
        // The header's entry-cap conjunct must fail closed before per-entry
        // validation when a collection alone grows past ENTRY_LIMIT.
        let mut oversized = artifact();
        let template = oversized.activity_intervals[0].clone();
        let target = ENTRY_LIMIT + 1;
        oversized.activity_intervals.reserve(target);
        while oversized.activity_intervals.len() <= target {
            oversized.activity_intervals.push(template.clone());
        }
        assert!(oversized.to_json().is_err());
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
            let document = ::uuid::Uuid::from_u128(0x8000_0000_0000_0000_u128).to_string();
            for draw in 0..one.posterior_draw_count {
                one.plausible_values.push(TopicPostPlausibleValue {
                    document_id: document.clone(),
                    draw_index: draw,
                    event_time: "2026-07-15T00:00:00Z".into(),
                    logistic_normal_coordinates: vec![0.0],
                });
            }
            for dimension in ["business_unit", "process_unit", "team", "person"] {
                one.memberships.push(TopicContextMembership {
                    document_id: document.clone(),
                    dimension_code: dimension.into(),
                    context_id: "context-probe".into(),
                    weight: 1.0,
                    valid_from: "2026-07-01T00:00:00Z".into(),
                    valid_to: "2026-08-01T00:00:00Z".into(),
                    evidence_sha256: "b".repeat(64),
                });
            }
            serde_json::to_string(&one).expect("probe").len()
                - serde_json::to_string(&probe).expect("base").len()
        };
        assert!(per_document_bytes > 0);
        let extra_documents = TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT / per_document_bytes + 2;
        let projected_entries = extra_documents
            * usize::try_from(probe.posterior_draw_count + 4)
                .expect("posterior draw count fits usize");
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
                0x8000_0000_0000_0000_u128 + u128::try_from(index).expect("index fits u128"),
            )
            .to_string();
            for draw in 0..oversized.posterior_draw_count {
                oversized.plausible_values.push(TopicPostPlausibleValue {
                    document_id: document.clone(),
                    draw_index: draw,
                    event_time: "2026-07-15T00:00:00Z".into(),
                    logistic_normal_coordinates: vec![0.0],
                });
            }
            for dimension in ["business_unit", "process_unit", "team", "person"] {
                oversized.memberships.push(TopicContextMembership {
                    document_id: document.clone(),
                    dimension_code: dimension.into(),
                    context_id: format!("context-{index}"),
                    weight: 1.0,
                    valid_from: "2026-07-01T00:00:00Z".into(),
                    valid_to: "2026-08-01T00:00:00Z".into(),
                    evidence_sha256: "b".repeat(64),
                });
            }
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
                0x8000_0000_0000_0000_u128 + u128::try_from(top_up).expect("fits"),
            )
            .to_string();
            for draw in 0..oversized.posterior_draw_count {
                oversized.plausible_values.push(TopicPostPlausibleValue {
                    document_id: document.clone(),
                    draw_index: draw,
                    event_time: "2026-07-15T00:00:00Z".into(),
                    logistic_normal_coordinates: vec![0.0],
                });
            }
            for dimension in ["business_unit", "process_unit", "team", "person"] {
                oversized.memberships.push(TopicContextMembership {
                    document_id: document.clone(),
                    dimension_code: dimension.into(),
                    context_id: format!("context-topup-{top_up}"),
                    weight: 1.0,
                    valid_from: "2026-07-01T00:00:00Z".into(),
                    valid_to: "2026-08-01T00:00:00Z".into(),
                    evidence_sha256: "b".repeat(64),
                });
            }
            top_up += 1;
        }
        assert_eq!(oversized.to_json(), Err(AnalysisEngineError::LimitExceeded));
    }
}
