//! Digest-bound TDT/CHRONOS workflow artifacts from ADR 0016 composition.

use std::collections::BTreeSet;

use event_core::{
    ChronosOccurrenceForecast, EVENT_INTELLIGENCE_WORKFLOW_VERSION, EventError, EventEvidenceLayer,
    EventIntelligenceWorkflowConfig, EventLinkPair, EventMention, EventTrackAssignment,
    FirstStoryLabel, SchemaSlotAssignment, StorySegmentation, compose_event_intelligence,
    refuse_composition_as_instance, refuse_composition_as_transition,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed TDT/CHRONOS workflow artifact.
pub const EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.tdt_chronos_workflow.v1";
/// Model contract required by the versioned event-intelligence workflow.
pub const EVENT_INTELLIGENCE_MODEL_CONTRACT_VERSION: &str = "event_intelligence_workflow_v1";
/// Analysis-run output profile required for a TDT/CHRONOS workflow artifact.
pub const EVENT_INTELLIGENCE_OUTPUT_PROFILE: &str = "tdt_chronos_workflow_v1";
/// Maximum canonical artifact JSON size.
pub const EVENT_INTELLIGENCE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const EVENT_INTELLIGENCE_INFERENCE_STATUS: &str = "composed_workflow_not_instance_or_transition";

/// Already-extracted TDT/CHRONOS artifacts admitted to one analysis run.
///
/// This input is not an extractor. Callers supply validated mentions, links,
/// first-story labels, tracks, schema slots, and forecasts.
#[derive(Clone, Debug, PartialEq)]
pub struct EventIntelligenceRunInput {
    config: EventIntelligenceWorkflowConfig,
    segmentation: StorySegmentation,
    mentions: Vec<EventMention>,
    links: Vec<EventLinkPair>,
    first_story_labels: Vec<FirstStoryLabel>,
    track_assignments: Vec<EventTrackAssignment>,
    schema_slot_assignments: Vec<SchemaSlotAssignment>,
    occurrence_forecasts: Vec<ChronosOccurrenceForecast>,
}

impl EventIntelligenceRunInput {
    /// Bundle already-extracted TDT/CHRONOS artifacts for one analysis run.
    #[must_use]
    #[allow(clippy::too_many_arguments, reason = "audited TDT/CHRONOS sequence")]
    pub fn new(
        config: EventIntelligenceWorkflowConfig,
        segmentation: StorySegmentation,
        mentions: Vec<EventMention>,
        links: Vec<EventLinkPair>,
        first_story_labels: Vec<FirstStoryLabel>,
        track_assignments: Vec<EventTrackAssignment>,
        schema_slot_assignments: Vec<SchemaSlotAssignment>,
        occurrence_forecasts: Vec<ChronosOccurrenceForecast>,
    ) -> Self {
        Self {
            config,
            segmentation,
            mentions,
            links,
            first_story_labels,
            track_assignments,
            schema_slot_assignments,
            occurrence_forecasts,
        }
    }

    /// Return the workflow configuration.
    #[must_use]
    pub const fn config(&self) -> EventIntelligenceWorkflowConfig {
        self.config
    }

    /// Return the admitted story segmentation.
    #[must_use]
    pub const fn segmentation(&self) -> &StorySegmentation {
        &self.segmentation
    }

    /// Return the offered mentions before cutoff filtering.
    #[must_use]
    pub fn mentions(&self) -> &[EventMention] {
        &self.mentions
    }

    /// Return the offered TDT links before cutoff filtering.
    #[must_use]
    pub fn links(&self) -> &[EventLinkPair] {
        &self.links
    }

    /// Return the offered first-story labels before cutoff filtering.
    #[must_use]
    pub fn first_story_labels(&self) -> &[FirstStoryLabel] {
        &self.first_story_labels
    }

    /// Return the offered track assignments before cutoff filtering.
    #[must_use]
    pub fn track_assignments(&self) -> &[EventTrackAssignment] {
        &self.track_assignments
    }

    /// Return the offered CHRONOS schema-slot assignments.
    #[must_use]
    pub fn schema_slot_assignments(&self) -> &[SchemaSlotAssignment] {
        &self.schema_slot_assignments
    }

    /// Return the offered CHRONOS occurrence forecasts.
    #[must_use]
    pub fn occurrence_forecasts(&self) -> &[ChronosOccurrenceForecast] {
        &self.occurrence_forecasts
    }
}

/// Completed, bounded TDT/CHRONOS workflow result consumed by analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventIntelligenceArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the analysis run.
    pub knowledge_cutoff: String,
    /// Versioned TDT/CHRONOS workflow identity.
    pub workflow_version: u16,
    /// Mentions admitted at the request cutoff.
    pub mention_count: u64,
    /// Mentions excluded because availability was after the request cutoff.
    pub excluded_after_cutoff_count: u64,
    /// TDT links whose both mentions remained eligible.
    pub link_count: u64,
    /// First-story labels aligned to admitted mentions.
    pub first_story_count: u64,
    /// Track assignments aligned to admitted mentions.
    pub track_count: u64,
    /// CHRONOS schema-slot assignments admitted with the workflow.
    pub schema_slot_count: u64,
    /// CHRONOS occurrence forecasts admitted with the workflow.
    pub forecast_count: u64,
    /// Epistemic layer of the composed TDT envelope.
    pub envelope_layer: String,
    /// Epistemic layer retained by composed CHRONOS hypotheses.
    pub hypothesis_layer: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl EventIntelligenceArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEventIntelligenceArtifact`] when the
    /// schema, identifiers, counts, layers, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > EVENT_INTELLIGENCE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidEventIntelligenceArtifact)?;
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
        if self.schema_version != EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.workflow_version != EVENT_INTELLIGENCE_WORKFLOW_VERSION
            || self.mention_count == 0
            || self.first_story_count != self.mention_count
            || self.track_count != self.mention_count
            || self.envelope_layer != EventEvidenceLayer::TdtDetection.wire_name()
            || self.hypothesis_layer != EventEvidenceLayer::ChronosPrediction.wire_name()
            || self.inference_status != EVENT_INTELLIGENCE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidEventIntelligenceArtifact);
        }
        Ok(())
    }
}

/// One completed event-intelligence artifact and its request-bound terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct EventIntelligenceExecution {
    /// Digest-bound completed workflow artifact.
    pub artifact: EventIntelligenceArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe TDT/CHRONOS composition as one analysis-run profile.
///
/// The caller supplies already-extracted artifacts. This executor does not
/// invent a new extractor, persist the composition, or promote it to an event
/// instance or state transition. Mentions whose availability is later than the
/// request cutoff are excluded before composition.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, composition
/// failure, or invalid/oversized artifact error.
pub fn execute_event_intelligence_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: EventIntelligenceRunInput,
    completed_at: impl Into<String>,
) -> Result<EventIntelligenceExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    require_event_intelligence_binding(request, snapshot_id, knowledge_cutoff)?;
    let admitted = admit_mentions_at_cutoff(input, knowledge_cutoff)?;
    let composition = compose_event_intelligence(
        admitted.config,
        admitted.segmentation,
        admitted.mentions,
        admitted.links,
        admitted.first_story_labels,
        admitted.track_assignments,
        admitted.schema_slot_assignments,
        admitted.occurrence_forecasts,
    )?;
    let _ = refuse_composition_as_instance(&composition);
    let _ = refuse_composition_as_transition(&composition);
    let artifact = EventIntelligenceArtifact::from_composition(
        accepted,
        snapshot_id,
        knowledge_cutoff,
        admitted.excluded_after_cutoff_count,
        &composition,
    )?;
    let digest = artifact.sha256()?;
    let statistic_count = artifact
        .mention_count
        .checked_add(artifact.link_count)
        .and_then(|value| value.checked_add(artifact.schema_slot_count))
        .and_then(|value| value.checked_add(artifact.forecast_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    let summary = AnalysisResultSummary::new(
        "tdt_chronos_workflow",
        artifact.mention_count,
        statistic_count,
        EVENT_INTELLIGENCE_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("event_intelligence_artifact_{}", &digest[..16]),
        digest,
        EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(EventIntelligenceExecution {
        artifact,
        terminal_result,
    })
}

fn require_event_intelligence_binding(
    request: &AnalysisRunRequest,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<(), AnalysisEngineError> {
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != EVENT_INTELLIGENCE_MODEL_CONTRACT_VERSION
        || request.output_profile != EVENT_INTELLIGENCE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(())
}

struct AdmittedEventIntelligence {
    config: EventIntelligenceWorkflowConfig,
    segmentation: StorySegmentation,
    mentions: Vec<EventMention>,
    links: Vec<EventLinkPair>,
    first_story_labels: Vec<FirstStoryLabel>,
    track_assignments: Vec<EventTrackAssignment>,
    schema_slot_assignments: Vec<SchemaSlotAssignment>,
    occurrence_forecasts: Vec<ChronosOccurrenceForecast>,
    excluded_after_cutoff_count: u64,
}

fn admit_mentions_at_cutoff(
    input: EventIntelligenceRunInput,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<AdmittedEventIntelligence, AnalysisEngineError> {
    if input.first_story_labels.len() != input.mentions.len()
        || input.track_assignments.len() != input.mentions.len()
    {
        return Err(AnalysisEngineError::Event(EventError::InvalidWirePayload));
    }
    let mut mentions = Vec::new();
    let mut first_story_labels = Vec::new();
    let mut track_assignments = Vec::new();
    let mut excluded_after_cutoff_count = 0_u64;
    for ((mention, first_story), track) in input
        .mentions
        .into_iter()
        .zip(input.first_story_labels)
        .zip(input.track_assignments)
    {
        if mention.clocks().available_time().instant() <= knowledge_cutoff.instant() {
            mentions.push(mention);
            first_story_labels.push(first_story);
            track_assignments.push(track);
        } else {
            excluded_after_cutoff_count += 1;
        }
    }
    if mentions.is_empty() {
        return Err(AnalysisEngineError::Event(
            EventError::MentionIneligibleAtCutoff,
        ));
    }
    let eligible_ids: BTreeSet<_> = mentions.iter().map(EventMention::mention_id).collect();
    let links: Vec<_> = input
        .links
        .into_iter()
        .filter(|link| eligible_ids.contains(&link.left()) && eligible_ids.contains(&link.right()))
        .collect();
    Ok(AdmittedEventIntelligence {
        config: input.config,
        segmentation: input.segmentation,
        mentions,
        links,
        first_story_labels,
        track_assignments,
        schema_slot_assignments: input.schema_slot_assignments,
        occurrence_forecasts: input.occurrence_forecasts,
        excluded_after_cutoff_count,
    })
}

fn count_or_overflow(len: usize) -> Result<u64, AnalysisEngineError> {
    u64::try_from(len).map_err(|_| AnalysisEngineError::ArithmeticOverflow)
}

impl EventIntelligenceArtifact {
    fn from_composition(
        accepted: &AnalysisRunAccepted,
        snapshot_id: &str,
        knowledge_cutoff: KnowledgeCutoff,
        excluded_after_cutoff_count: u64,
        composition: &event_core::EventIntelligenceComposition,
    ) -> Result<Self, AnalysisEngineError> {
        let artifact = Self {
            schema_version: EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: accepted.run_id.clone(),
            snapshot_id: snapshot_id.to_owned(),
            knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
            workflow_version: composition.config_version(),
            mention_count: count_or_overflow(composition.mentions().len())?,
            excluded_after_cutoff_count,
            link_count: count_or_overflow(composition.links().len())?,
            first_story_count: count_or_overflow(composition.first_story_labels().len())?,
            track_count: count_or_overflow(composition.track_assignments().len())?,
            schema_slot_count: count_or_overflow(composition.schema_slot_assignments().len())?,
            forecast_count: count_or_overflow(composition.occurrence_forecasts().len())?,
            envelope_layer: composition.evidence_layer().wire_name().into(),
            hypothesis_layer: composition.chronos_evidence_layer().wire_name().into(),
            inference_status: EVENT_INTELLIGENCE_INFERENCE_STATUS.into(),
        };
        artifact.validate()?;
        Ok(artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EVENT_INTELLIGENCE_ARTIFACT_BYTE_LIMIT, EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION,
        EVENT_INTELLIGENCE_INFERENCE_STATUS, EventIntelligenceArtifact,
    };
    use crate::AnalysisEngineError;
    use event_core::EventEvidenceLayer;

    fn artifact() -> EventIntelligenceArtifact {
        EventIntelligenceArtifact {
            schema_version: EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-03-31T00:00:00Z".into(),
            workflow_version: 1,
            mention_count: 3,
            excluded_after_cutoff_count: 0,
            link_count: 2,
            first_story_count: 3,
            track_count: 3,
            schema_slot_count: 2,
            forecast_count: 1,
            envelope_layer: EventEvidenceLayer::TdtDetection.wire_name().into(),
            hypothesis_layer: EventEvidenceLayer::ChronosPrediction.wire_name().into(),
            inference_status: EVENT_INTELLIGENCE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &EventIntelligenceArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidEventIntelligenceArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            EventIntelligenceArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            EventIntelligenceArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidEventIntelligenceArtifact)
        );
        assert_eq!(
            EventIntelligenceArtifact::from_json(
                &"x".repeat(EVENT_INTELLIGENCE_ARTIFACT_BYTE_LIMIT + 1)
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
                value.workflow_version = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.mention_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.first_story_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.track_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.envelope_layer = EventEvidenceLayer::PromotedTransition.wire_name().into();
                value
            },
            {
                let mut value = artifact.clone();
                value.hypothesis_layer = EventEvidenceLayer::PromotedTransition.wire_name().into();
                value
            },
            {
                let mut value = artifact.clone();
                value.envelope_layer.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.hypothesis_layer.clear();
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
