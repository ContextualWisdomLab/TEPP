//! Versioned TDT/CHRONOS workflow composition over admitted artifacts.
//!
//! Allan (2002) defines Topic Detection and Tracking as linked detection
//! tasks—segmentation, link detection, first-story detection, and tracking—
//! rather than a single opaque model. Li et al. (2021) treat schema and next-
//! event forecasts as graph hypotheses. Anagnostopoulos, Batsakis, and
//! Petrakis (2013) keep CHRONOS-style reasoning distinct from observed fact.
//! This module admits already-extracted artifacts into one versioned workflow
//! and never invents a new extractor or a silent promotion path.

use crate::{
    ChronosOccurrenceForecast, EventConfidence, EventError, EventEvidenceLayer, EventInstanceId,
    EventLinkPair, EventMention, EventTrackAssignment, FirstStoryLabel, SchemaSlotAssignment,
    StorySegmentation,
};

/// Wire schema version for the unified event-intelligence workflow.
pub const EVENT_INTELLIGENCE_WORKFLOW_VERSION: u16 = 1;

/// Named thresholds and version for one reproducible TDT/CHRONOS run.
///
/// Callers pass these thresholds into the existing `decide_*` helpers for
/// link, first-story, track, schema-slot, boundary, and occurrence forecasts.
/// An empty (`0`) or unsupported version fails closed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventIntelligenceWorkflowConfig {
    version: u16,
    link_threshold: EventConfidence,
    first_story_threshold: EventConfidence,
    track_threshold: EventConfidence,
    schema_threshold: EventConfidence,
    boundary_threshold: EventConfidence,
    forecast_threshold: EventConfidence,
}

impl EventIntelligenceWorkflowConfig {
    /// Validate a workflow version and named decision thresholds.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] when `version` is `0`
    /// (empty). Returns [`EventError::UnsupportedWireVersion`] when `version`
    /// is not [`EVENT_INTELLIGENCE_WORKFLOW_VERSION`].
    pub fn new(
        version: u16,
        link_threshold: EventConfidence,
        first_story_threshold: EventConfidence,
        track_threshold: EventConfidence,
        schema_threshold: EventConfidence,
        boundary_threshold: EventConfidence,
        forecast_threshold: EventConfidence,
    ) -> Result<Self, EventError> {
        if version == 0 {
            return Err(EventError::InvalidWirePayload);
        }
        if version != EVENT_INTELLIGENCE_WORKFLOW_VERSION {
            return Err(EventError::UnsupportedWireVersion);
        }
        Ok(Self {
            version,
            link_threshold,
            first_story_threshold,
            track_threshold,
            schema_threshold,
            boundary_threshold,
            forecast_threshold,
        })
    }

    /// Return the validated workflow version.
    #[must_use]
    pub const fn version(self) -> u16 {
        self.version
    }

    /// Return the link-decision threshold for [`crate::decide_event_link`].
    #[must_use]
    pub const fn link_threshold(self) -> EventConfidence {
        self.link_threshold
    }

    /// Return the first-story threshold for [`crate::decide_first_story`].
    #[must_use]
    pub const fn first_story_threshold(self) -> EventConfidence {
        self.first_story_threshold
    }

    /// Return the track-continue threshold for [`crate::decide_track_continue`].
    #[must_use]
    pub const fn track_threshold(self) -> EventConfidence {
        self.track_threshold
    }

    /// Return the schema-slot threshold for [`crate::decide_schema_slot`].
    #[must_use]
    pub const fn schema_threshold(self) -> EventConfidence {
        self.schema_threshold
    }

    /// Return the story-boundary threshold for [`crate::decide_story_boundary`].
    #[must_use]
    pub const fn boundary_threshold(self) -> EventConfidence {
        self.boundary_threshold
    }

    /// Return the occurrence-forecast threshold used with forecast probabilities.
    #[must_use]
    pub const fn forecast_threshold(self) -> EventConfidence {
        self.forecast_threshold
    }
}

/// Ordered TDT/CHRONOS artifacts admitted under one workflow version.
///
/// TDT segmentation, links, first-story labels, and tracks remain
/// [`EventEvidenceLayer::TdtDetection`]. Schema-slot assignments and
/// occurrence forecasts remain [`EventEvidenceLayer::ChronosPrediction`].
/// The composition itself is never [`EventEvidenceLayer::PromotedTransition`].
#[derive(Clone, Debug, PartialEq)]
pub struct EventIntelligenceComposition {
    config: EventIntelligenceWorkflowConfig,
    envelope_layer: EventEvidenceLayer,
    hypothesis_layer: EventEvidenceLayer,
    mentions: Vec<EventMention>,
    segmentation: StorySegmentation,
    links: Vec<EventLinkPair>,
    first_story_labels: Vec<FirstStoryLabel>,
    track_assignments: Vec<EventTrackAssignment>,
    schema_slot_assignments: Vec<SchemaSlotAssignment>,
    occurrence_forecasts: Vec<ChronosOccurrenceForecast>,
}

impl EventIntelligenceComposition {
    /// Return the workflow configuration version stored with this composition.
    #[must_use]
    pub const fn config_version(&self) -> u16 {
        self.config.version()
    }

    /// Return the validated workflow configuration.
    #[must_use]
    pub const fn config(&self) -> EventIntelligenceWorkflowConfig {
        self.config
    }

    /// Return the ordered span-grounded mentions.
    #[must_use]
    pub fn mentions(&self) -> &[EventMention] {
        &self.mentions
    }

    /// Return the admitted story/event segmentation.
    #[must_use]
    pub const fn segmentation(&self) -> &StorySegmentation {
        &self.segmentation
    }

    /// Return the admitted TDT link pairs.
    #[must_use]
    pub fn links(&self) -> &[EventLinkPair] {
        &self.links
    }

    /// Return the admitted first-story labels aligned to mentions.
    #[must_use]
    pub fn first_story_labels(&self) -> &[FirstStoryLabel] {
        &self.first_story_labels
    }

    /// Return the admitted track assignments aligned to mentions.
    #[must_use]
    pub fn track_assignments(&self) -> &[EventTrackAssignment] {
        &self.track_assignments
    }

    /// Return the admitted CHRONOS schema-slot fills.
    #[must_use]
    pub fn schema_slot_assignments(&self) -> &[SchemaSlotAssignment] {
        &self.schema_slot_assignments
    }

    /// Return the admitted CHRONOS occurrence forecasts.
    #[must_use]
    pub fn occurrence_forecasts(&self) -> &[ChronosOccurrenceForecast] {
        &self.occurrence_forecasts
    }

    /// Epistemic layer of the composed workflow envelope.
    ///
    /// TDT artifacts remain [`EventEvidenceLayer::TdtDetection`]. Schema slots
    /// and occurrence forecasts remain [`EventEvidenceLayer::ChronosPrediction`]
    /// hypotheses. The composition itself is never a promoted transition.
    #[must_use]
    pub const fn evidence_layer(&self) -> EventEvidenceLayer {
        self.envelope_layer
    }

    /// Epistemic layer retained by composed CHRONOS schema/forecast artifacts.
    #[must_use]
    pub const fn chronos_evidence_layer(&self) -> EventEvidenceLayer {
        self.hypothesis_layer
    }

    /// Append a later-arriving revised-document mention without rewriting earlier
    /// mention identities, spans, or track assignments.
    ///
    /// # Errors
    ///
    /// Returns [`EventError::InvalidWirePayload`] when the track assignment does
    /// not cite the appended mention identity.
    pub fn append_revised_mention(
        &mut self,
        mention: EventMention,
        first_story_label: FirstStoryLabel,
        track_assignment: EventTrackAssignment,
    ) -> Result<(), EventError> {
        if track_assignment.mention_id() != mention.mention_id() {
            return Err(EventError::InvalidWirePayload);
        }
        self.mentions.push(mention);
        self.first_story_labels.push(first_story_label);
        self.track_assignments.push(track_assignment);
        Ok(())
    }
}

/// Admit already-extracted TDT/CHRONOS artifacts into one versioned workflow.
///
/// Sequence retained for audit: segmentation → span-grounded mentions → links →
/// first-story → tracks → schema slots → forecasts. This function does not
/// invent a new extractor; callers supply validated artifacts.
///
/// # Errors
///
/// Returns [`EventError::InvalidWirePayload`] when mentions are empty, when
/// first-story or track streams are not length-aligned to mentions, or when a
/// track assignment is not index-aligned to the matching mention identity.
/// Propagates config version errors from
/// [`EventIntelligenceWorkflowConfig::new`] when the supplied config is reused
/// only after validation (callers must construct config first).
#[allow(clippy::too_many_arguments, reason = "audited TDT/CHRONOS sequence")]
pub fn compose_event_intelligence(
    config: EventIntelligenceWorkflowConfig,
    segmentation: StorySegmentation,
    mentions: Vec<EventMention>,
    links: Vec<EventLinkPair>,
    first_story_labels: Vec<FirstStoryLabel>,
    track_assignments: Vec<EventTrackAssignment>,
    schema_slot_assignments: Vec<SchemaSlotAssignment>,
    occurrence_forecasts: Vec<ChronosOccurrenceForecast>,
) -> Result<EventIntelligenceComposition, EventError> {
    if mentions.is_empty() {
        return Err(EventError::InvalidWirePayload);
    }
    if first_story_labels.len() != mentions.len() {
        return Err(EventError::InvalidWirePayload);
    }
    if track_assignments.len() != mentions.len() {
        return Err(EventError::InvalidWirePayload);
    }
    let mention_ids: Vec<_> = mentions.iter().map(EventMention::mention_id).collect();
    for (mention, assignment) in mentions.iter().zip(&track_assignments) {
        if mention.mention_id() != assignment.mention_id() {
            return Err(EventError::InvalidWirePayload);
        }
    }
    for link in &links {
        let left_known = mention_ids.iter().any(|id| *id == link.left());
        let right_known = mention_ids.iter().any(|id| *id == link.right());
        if !left_known {
            return Err(EventError::InvalidWirePayload);
        }
        if !right_known {
            return Err(EventError::InvalidWirePayload);
        }
    }
    Ok(EventIntelligenceComposition {
        config,
        envelope_layer: EventEvidenceLayer::TdtDetection,
        hypothesis_layer: EventEvidenceLayer::ChronosPrediction,
        mentions,
        segmentation,
        links,
        first_story_labels,
        track_assignments,
        schema_slot_assignments,
        occurrence_forecasts,
    })
}

/// Explicit refusal to treat a composed workflow as an event instance.
///
/// # Errors
///
/// Always returns [`EventError::IntelligenceWorkflowIsNotEventInstance`].
pub fn refuse_composition_as_instance(
    _composition: &EventIntelligenceComposition,
) -> Result<EventInstanceId, EventError> {
    Err(EventError::IntelligenceWorkflowIsNotEventInstance)
}

/// Explicit refusal to treat a composed workflow as a state transition.
///
/// # Errors
///
/// Always returns [`EventError::IntelligenceWorkflowIsNotStateTransition`].
pub fn refuse_composition_as_transition(
    _composition: &EventIntelligenceComposition,
) -> Result<(), EventError> {
    Err(EventError::IntelligenceWorkflowIsNotStateTransition)
}

#[cfg(test)]
mod tests {
    use super::{
        EVENT_INTELLIGENCE_WORKFLOW_VERSION, EventIntelligenceWorkflowConfig,
        compose_event_intelligence, refuse_composition_as_instance,
        refuse_composition_as_transition,
    };
    use crate::{
        EventConfidence, EventError, EventEvidenceLayer, EventLinkPair, EventMention,
        EventTrackAssignment, EventTrackId, FirstStoryLabel, MentionEvidenceClocks,
        MentionReviewStatus, StorySegmentation,
    };
    use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
    use temporal_core::{
        AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
    };

    fn record(text: &str) -> DocumentRecord {
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        DocumentRecord::from_text(artifact.id(), text).expect("document")
    }

    fn span_for(document: &DocumentRecord, surface: &str) -> SourceSpan {
        let byte_start = document.text().find(surface).expect("surface present");
        let byte_end = byte_start + surface.len();
        let scalar_start = document.text()[..byte_start].chars().count();
        let scalar_end = scalar_start + surface.chars().count();
        SourceSpan::new(
            document,
            byte_start,
            byte_end,
            scalar_start,
            scalar_end,
            None,
        )
        .expect("span")
    }

    fn clocks() -> MentionEvidenceClocks {
        MentionEvidenceClocks::new(
            EventTime::parse_rfc3339("2026-03-01T12:00:00Z").expect("event"),
            AssertionTime::parse_rfc3339("2026-03-02T09:00:00Z").expect("assertion"),
            DocumentTime::parse_rfc3339("2026-03-02T09:00:00Z").expect("document"),
            SystemTime::parse_rfc3339("2026-03-02T09:00:00Z").expect("system"),
            AvailableTime::parse_rfc3339("2026-03-02T09:00:00Z").expect("available"),
            KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
        )
        .expect("clocks")
    }

    fn grounded(document: &DocumentRecord, surface: &str) -> EventMention {
        EventMention::new(
            document,
            span_for(document, surface),
            EventConfidence::new(0.9).expect("confidence"),
            clocks(),
            "ace-extent-extractor/1",
            MentionReviewStatus::Proposed,
        )
        .expect("grounded mention")
    }

    fn half() -> EventConfidence {
        EventConfidence::new(0.5).expect("half")
    }

    fn workflow_config() -> EventIntelligenceWorkflowConfig {
        EventIntelligenceWorkflowConfig::new(
            EVENT_INTELLIGENCE_WORKFLOW_VERSION,
            half(),
            half(),
            half(),
            half(),
            half(),
            half(),
        )
        .expect("workflow config")
    }

    #[test]
    fn workflow_config_rejects_empty_and_unsupported_versions() {
        assert_eq!(
            EventIntelligenceWorkflowConfig::new(0, half(), half(), half(), half(), half(), half()),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            EventIntelligenceWorkflowConfig::new(
                99,
                half(),
                half(),
                half(),
                half(),
                half(),
                half()
            ),
            Err(EventError::UnsupportedWireVersion)
        );
        let config = workflow_config();
        assert_eq!(config.version(), EVENT_INTELLIGENCE_WORKFLOW_VERSION);
        assert_eq!(config.link_threshold(), half());
        assert_eq!(config.first_story_threshold(), half());
        assert_eq!(config.track_threshold(), half());
        assert_eq!(config.schema_threshold(), half());
        assert_eq!(config.boundary_threshold(), half());
        assert_eq!(config.forecast_threshold(), half());
    }

    #[test]
    fn compose_refuses_empty_mentions_and_stream_misalignment() {
        let original = record("award protest later");
        let award = grounded(&original, "award");
        let protest = grounded(&original, "protest");
        let segmentation = StorySegmentation::new(3, vec![false, true]).expect("seg");
        let mentions = vec![award.clone(), protest.clone()];
        let labels = vec![FirstStoryLabel::FirstStory, FirstStoryLabel::FollowUp];
        let tracks = vec![
            EventTrackAssignment::new(award.mention_id(), EventTrackId::from_raw(1)),
            EventTrackAssignment::new(protest.mention_id(), EventTrackId::from_raw(1)),
        ];
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation.clone(),
                mentions.clone(),
                Vec::new(),
                vec![FirstStoryLabel::FirstStory],
                tracks.clone(),
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation,
                mentions,
                Vec::new(),
                labels,
                vec![tracks[0]],
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
    }

    #[test]
    fn compose_refuses_unknown_tracks_and_foreign_links() {
        let original = record("award protest later");
        let revised = record("revised award later");
        let award = grounded(&original, "award");
        let protest = grounded(&original, "protest");
        let later = grounded(&revised, "award");
        let segmentation = StorySegmentation::new(3, vec![false, true]).expect("seg");
        let mentions = vec![award.clone(), protest.clone()];
        let labels = vec![FirstStoryLabel::FirstStory, FirstStoryLabel::FollowUp];
        let tracks = vec![
            EventTrackAssignment::new(award.mention_id(), EventTrackId::from_raw(1)),
            EventTrackAssignment::new(protest.mention_id(), EventTrackId::from_raw(1)),
        ];
        let stranger = EventTrackAssignment::new(later.mention_id(), EventTrackId::from_raw(9));
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation.clone(),
                mentions.clone(),
                Vec::new(),
                labels.clone(),
                vec![tracks[0], stranger],
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
        let foreign_right =
            EventLinkPair::new(award.mention_id(), later.mention_id()).expect("foreign right");
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation.clone(),
                mentions.clone(),
                vec![foreign_right],
                labels.clone(),
                tracks.clone(),
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
        let foreign_left =
            EventLinkPair::new(later.mention_id(), award.mention_id()).expect("foreign left");
        assert_eq!(
            compose_event_intelligence(
                workflow_config(),
                segmentation,
                mentions,
                vec![foreign_left],
                labels,
                tracks,
                Vec::new(),
                Vec::new(),
            )
            .map(|_| ()),
            Err(EventError::InvalidWirePayload)
        );
    }

    #[test]
    fn compose_exposes_layers_and_refuses_mismatched_append() {
        let original = record("award protest later");
        let revised = record("revised award later");
        let award = grounded(&original, "award");
        let protest = grounded(&original, "protest");
        let later = grounded(&revised, "award");
        let segmentation = StorySegmentation::new(3, vec![false, true]).expect("seg");
        let mentions = vec![award.clone(), protest.clone()];
        let labels = vec![FirstStoryLabel::FirstStory, FirstStoryLabel::FollowUp];
        let tracks = vec![
            EventTrackAssignment::new(award.mention_id(), EventTrackId::from_raw(1)),
            EventTrackAssignment::new(protest.mention_id(), EventTrackId::from_raw(1)),
        ];
        let mut composition = compose_event_intelligence(
            workflow_config(),
            segmentation,
            mentions,
            Vec::new(),
            labels,
            tracks,
            Vec::new(),
            Vec::new(),
        )
        .expect("compose");
        assert_eq!(
            composition.config_version(),
            EVENT_INTELLIGENCE_WORKFLOW_VERSION
        );
        assert_eq!(composition.config(), workflow_config());
        assert_eq!(
            composition.evidence_layer(),
            EventEvidenceLayer::TdtDetection
        );
        assert_eq!(
            composition.chronos_evidence_layer(),
            EventEvidenceLayer::ChronosPrediction
        );
        assert_eq!(composition.mentions().len(), 2);
        assert!(composition.links().is_empty());
        assert_eq!(composition.first_story_labels().len(), 2);
        assert_eq!(composition.track_assignments().len(), 2);
        assert!(composition.schema_slot_assignments().is_empty());
        assert!(composition.occurrence_forecasts().is_empty());
        let mismatched = EventTrackAssignment::new(award.mention_id(), EventTrackId::from_raw(1));
        assert_eq!(
            composition.append_revised_mention(
                later.clone(),
                FirstStoryLabel::FollowUp,
                mismatched
            ),
            Err(EventError::InvalidWirePayload)
        );
        composition
            .append_revised_mention(
                later.clone(),
                FirstStoryLabel::FollowUp,
                EventTrackAssignment::new(later.mention_id(), EventTrackId::from_raw(1)),
            )
            .expect("matching append");
        assert_eq!(
            refuse_composition_as_instance(&composition),
            Err(EventError::IntelligenceWorkflowIsNotEventInstance)
        );
        assert_eq!(
            refuse_composition_as_transition(&composition),
            Err(EventError::IntelligenceWorkflowIsNotStateTransition)
        );
    }
}
