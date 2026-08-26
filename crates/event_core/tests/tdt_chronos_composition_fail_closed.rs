//! Fail-closed TDT/CHRONOS composition paths stay refuse-first.

use event_core::{
    EVENT_INTELLIGENCE_WORKFLOW_VERSION, EventConfidence, EventError,
    EventIntelligenceWorkflowConfig, EventLinkPair, EventMention, EventTrackAssignment,
    EventTrackId, FirstStoryLabel, MentionEvidenceClocks, MentionReviewStatus, StorySegmentation,
    compose_event_intelligence, decide_event_link, decide_first_story, decide_schema_slot,
    decide_story_boundary, decide_track_continue,
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
fn compose_refuses_unknown_track_mentions_and_foreign_links() {
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
    let link = EventLinkPair::new(award.mention_id(), protest.mention_id()).expect("link");
    let stranger = EventTrackAssignment::new(later.mention_id(), EventTrackId::from_raw(9));
    assert_eq!(
        compose_event_intelligence(
            workflow_config(),
            segmentation.clone(),
            mentions.clone(),
            vec![link],
            labels.clone(),
            vec![tracks[0], stranger],
            Vec::new(),
            Vec::new(),
        )
        .map(|_| ()),
        Err(EventError::InvalidWirePayload)
    );
    let foreign = EventLinkPair::new(award.mention_id(), later.mention_id()).expect("foreign");
    assert_eq!(
        compose_event_intelligence(
            workflow_config(),
            segmentation,
            mentions,
            vec![foreign],
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
fn workflow_config_thresholds_drive_existing_decide_helpers() {
    let config = workflow_config();
    assert_eq!(config.version(), EVENT_INTELLIGENCE_WORKFLOW_VERSION);
    let cut = half();
    let _ = decide_event_link(cut, config.link_threshold());
    let _ = decide_first_story(cut, config.first_story_threshold());
    let _ = decide_track_continue(cut, config.track_threshold());
    let _ = decide_schema_slot(cut, config.schema_threshold());
    let _ = decide_story_boundary(cut, config.boundary_threshold());
    assert!(cut.value() >= config.forecast_threshold().value());
    assert!((config.link_threshold().value() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn empty_mentions_and_bad_versions_fail_closed_before_composition() {
    assert_eq!(
        EventIntelligenceWorkflowConfig::new(0, half(), half(), half(), half(), half(), half()),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        EventIntelligenceWorkflowConfig::new(99, half(), half(), half(), half(), half(), half()),
        Err(EventError::UnsupportedWireVersion)
    );
    let segmentation = StorySegmentation::new(2, vec![true]).expect("seg");
    assert_eq!(
        compose_event_intelligence(
            workflow_config(),
            segmentation,
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
}
