//! Versioned TDT/CHRONOS composition recovers known-truth metrics and refuses promotion.
//!
//! Fixture mirrors Allan (2002) noisy duplicate stories plus a delayed revised
//! document, Li et al. (2021) schema/forecast hypotheses, and Anagnostopoulos,
//! Batsakis, and Petrakis (2013) separation of prediction from observed fact.

use event_core::{
    ChronosOccurrenceForecast, ChronosPredictionId, EVENT_INTELLIGENCE_WORKFLOW_VERSION,
    EventConfidence, EventError, EventEvidenceLayer, EventIntelligenceComposition,
    EventIntelligenceWorkflowConfig, EventLinkPair, EventMention, EventRoleKind,
    EventTrackAssignment, EventTrackId, FirstStoryLabel, MentionEvidenceClocks,
    MentionReviewStatus, OccurrenceTruth, SchemaSlotAssignment, StorySegmentation,
    admit_state_transition, chronos_prediction_brier_score, compose_event_intelligence,
    event_link_precision, event_link_recall, first_story_false_alarm_rate, first_story_miss_rate,
    mention_span_precision, mention_span_recall, refuse_composition_as_instance,
    refuse_composition_as_transition, schema_slot_precision, schema_slot_recall, story_pk,
    story_window_diff, tracking_pair_precision, tracking_pair_recall,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

const STORY_A: &str = "The procurement office awarded the river-crossing contract on 1 March 2026 after the earlier protest was withdrawn.";
const STORY_A_NOISY: &str =
    "Procurement office awarded river-crossing contract 1 March 2026; earlier protest withdrawn.";
const STORY_A_REVISED: &str = "Revised notice: the procurement office awarded the river-crossing contract on 1 March 2026 after the earlier protest was withdrawn.";

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

fn clocks_at(available: &str) -> MentionEvidenceClocks {
    MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T12:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339(available).expect("assertion"),
        DocumentTime::parse_rfc3339(available).expect("document"),
        SystemTime::parse_rfc3339(available).expect("system"),
        AvailableTime::parse_rfc3339(available).expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    )
    .expect("clocks")
}

fn grounded(
    document: &DocumentRecord,
    surface: &str,
    available: &str,
    confidence: f64,
) -> EventMention {
    EventMention::new(
        document,
        span_for(document, surface),
        EventConfidence::new(confidence).expect("confidence"),
        clocks_at(available),
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

struct KnownTruthFixture {
    original: DocumentRecord,
    noisy: DocumentRecord,
    revised: DocumentRecord,
    award_original: EventMention,
    protest_original: EventMention,
    award_noisy: EventMention,
    award_revised: EventMention,
}

impl KnownTruthFixture {
    fn build() -> Self {
        let original = record(STORY_A);
        let noisy = record(STORY_A_NOISY);
        let revised = record(STORY_A_REVISED);
        let award_original = grounded(
            &original,
            "awarded the river-crossing contract",
            "2026-03-02T09:00:00Z",
            0.91,
        );
        let protest_original = grounded(&original, "protest", "2026-03-02T09:00:00Z", 0.88);
        let award_noisy = grounded(
            &noisy,
            "awarded river-crossing contract",
            "2026-03-02T12:00:00Z",
            0.80,
        );
        let award_revised = grounded(
            &revised,
            "awarded the river-crossing contract",
            "2026-03-10T08:00:00Z",
            0.93,
        );
        Self {
            original,
            noisy,
            revised,
            award_original,
            protest_original,
            award_noisy,
            award_revised,
        }
    }
}

impl KnownTruthFixture {
    fn mentions(&self) -> Vec<EventMention> {
        vec![
            self.award_original.clone(),
            self.protest_original.clone(),
            self.award_noisy.clone(),
        ]
    }

    fn links(&self) -> Vec<EventLinkPair> {
        vec![
            EventLinkPair::new(
                self.award_original.mention_id(),
                self.award_noisy.mention_id(),
            )
            .expect("duplicate link"),
            EventLinkPair::new(
                self.award_original.mention_id(),
                self.protest_original.mention_id(),
            )
            .expect("same-document link"),
        ]
    }

    fn first_story_labels() -> Vec<FirstStoryLabel> {
        vec![
            FirstStoryLabel::FirstStory,
            FirstStoryLabel::FollowUp,
            FirstStoryLabel::FollowUp,
        ]
    }

    fn track_assignments(&self) -> Vec<EventTrackAssignment> {
        vec![
            EventTrackAssignment::new(self.award_original.mention_id(), EventTrackId::from_raw(1)),
            EventTrackAssignment::new(
                self.protest_original.mention_id(),
                EventTrackId::from_raw(1),
            ),
            EventTrackAssignment::new(self.award_noisy.mention_id(), EventTrackId::from_raw(1)),
        ]
    }

    fn schema_slots() -> Vec<SchemaSlotAssignment> {
        vec![
            SchemaSlotAssignment::new(EventRoleKind::Agent, "procurement office").expect("agent"),
            SchemaSlotAssignment::new(EventRoleKind::Product, "river-crossing contract")
                .expect("product"),
        ]
    }

    fn forecasts() -> Vec<ChronosOccurrenceForecast> {
        vec![ChronosOccurrenceForecast::new(
            ChronosPredictionId::from_raw(1),
            EventConfidence::new(0.75).expect("forecast"),
        )]
    }

    fn compose(&self) -> EventIntelligenceComposition {
        compose_event_intelligence(
            workflow_config(),
            StorySegmentation::new(3, vec![false, true]).expect("recovered segmentation"),
            self.mentions(),
            self.links(),
            Self::first_story_labels(),
            self.track_assignments(),
            Self::schema_slots(),
            Self::forecasts(),
        )
        .expect("compose workflow")
    }
}

#[test]
fn composition_recovers_known_truth_span_and_segmentation_metrics() {
    let fixture = KnownTruthFixture::build();
    let truth_spans = [
        span_for(&fixture.original, "awarded the river-crossing contract"),
        span_for(&fixture.original, "protest"),
        span_for(&fixture.noisy, "awarded river-crossing contract"),
    ];
    let recovered_spans = [
        fixture.award_original.source_span(),
        fixture.protest_original.source_span(),
        fixture.award_noisy.source_span(),
    ];
    let mention_precision =
        mention_span_precision(&truth_spans, &recovered_spans).expect("mention p");
    let mention_recall = mention_span_recall(&truth_spans, &recovered_spans).expect("mention r");
    assert!((mention_precision - 1.0).abs() < f64::EPSILON);
    assert!((mention_recall - 1.0).abs() < f64::EPSILON);
    let truth_segmentation =
        StorySegmentation::new(3, vec![false, true]).expect("truth segmentation");
    let composition = fixture.compose();
    let window_diff =
        story_window_diff(&truth_segmentation, composition.segmentation(), 1).expect("wd");
    let pk = story_pk(&truth_segmentation, composition.segmentation(), 1).expect("pk");
    assert!(window_diff.abs() < f64::EPSILON);
    assert!(pk.abs() < f64::EPSILON);
}

#[test]
fn composition_recovers_link_track_first_story_schema_and_brier() {
    let fixture = KnownTruthFixture::build();
    let links = fixture.links();
    let first_story_labels = KnownTruthFixture::first_story_labels();
    let track_assignments = fixture.track_assignments();
    let schema_slots = KnownTruthFixture::schema_slots();
    let composition = fixture.compose();
    let link_precision = event_link_precision(&links, composition.links()).expect("link p");
    let link_recall = event_link_recall(&links, composition.links()).expect("link r");
    assert!((link_precision - 1.0).abs() < f64::EPSILON);
    assert!((link_recall - 1.0).abs() < f64::EPSILON);
    let track_precision =
        tracking_pair_precision(&track_assignments, composition.track_assignments())
            .expect("track p");
    let track_recall =
        tracking_pair_recall(&track_assignments, composition.track_assignments()).expect("track r");
    assert!((track_precision - 1.0).abs() < f64::EPSILON);
    assert!((track_recall - 1.0).abs() < f64::EPSILON);
    let miss =
        first_story_miss_rate(&first_story_labels, composition.first_story_labels()).expect("miss");
    let far = first_story_false_alarm_rate(&first_story_labels, composition.first_story_labels())
        .expect("far");
    assert!(miss.abs() < f64::EPSILON);
    assert!(far.abs() < f64::EPSILON);
    let slot_precision =
        schema_slot_precision(&schema_slots, composition.schema_slot_assignments())
            .expect("slot p");
    let slot_recall =
        schema_slot_recall(&schema_slots, composition.schema_slot_assignments()).expect("slot r");
    assert!((slot_precision - 1.0).abs() < f64::EPSILON);
    assert!((slot_recall - 1.0).abs() < f64::EPSILON);
    let outcomes = [OccurrenceTruth::Occurred];
    let brier = chronos_prediction_brier_score(composition.occurrence_forecasts(), &outcomes)
        .expect("brier");
    let expected_brier = (0.75_f64 - 1.0).powi(2);
    assert!((brier - expected_brier).abs() < 1e-15);
}

#[test]
fn composition_refuses_promotion_and_preserves_earlier_mention_identity() {
    let fixture = KnownTruthFixture::build();
    let mut composition = fixture.compose();
    assert_eq!(
        refuse_composition_as_instance(&composition),
        Err(EventError::IntelligenceWorkflowIsNotEventInstance)
    );
    assert_eq!(
        refuse_composition_as_transition(&composition),
        Err(EventError::IntelligenceWorkflowIsNotStateTransition)
    );
    assert_ne!(
        composition.evidence_layer(),
        EventEvidenceLayer::PromotedTransition
    );
    assert_eq!(
        admit_state_transition(composition.evidence_layer()),
        Err(EventError::DetectionIsNotTransition)
    );
    assert_eq!(
        composition.chronos_evidence_layer(),
        EventEvidenceLayer::ChronosPrediction
    );
    assert_eq!(
        admit_state_transition(composition.chronos_evidence_layer()),
        Err(EventError::PredictionIsNotFact)
    );
    let earlier_mention_id = composition.mentions()[0].mention_id();
    let earlier_span = composition.mentions()[0].source_span();
    let earlier_track = composition.track_assignments()[0].track_id();
    let earlier_surface = composition.mentions()[0].surface_form().to_string();
    composition
        .append_revised_mention(
            fixture.award_revised.clone(),
            FirstStoryLabel::FollowUp,
            EventTrackAssignment::new(
                fixture.award_revised.mention_id(),
                EventTrackId::from_raw(1),
            ),
        )
        .expect("append revised document mention");
    assert_eq!(composition.mentions()[0].mention_id(), earlier_mention_id);
    assert_eq!(composition.mentions()[0].source_span(), earlier_span);
    assert_eq!(composition.mentions()[0].surface_form(), earlier_surface);
    assert_eq!(composition.track_assignments()[0].track_id(), earlier_track);
    assert_eq!(
        composition.mentions().last().map(EventMention::mention_id),
        Some(fixture.award_revised.mention_id())
    );
    assert_eq!(composition.mentions().len(), 4);
    assert_eq!(composition.first_story_labels().len(), 4);
    assert_eq!(composition.track_assignments().len(), 4);
    let _ = (&fixture.noisy, &fixture.revised);
}
