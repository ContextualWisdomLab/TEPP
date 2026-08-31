//! End-to-end contract for the completed TDT/CHRONOS workflow artifact.

use analysis_engine::{
    AnalysisEngineError, EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION,
    EVENT_INTELLIGENCE_MODEL_CONTRACT_VERSION, EVENT_INTELLIGENCE_OUTPUT_PROFILE,
    EventIntelligenceRunInput, execute_event_intelligence_run,
};
use event_core::{
    ChronosOccurrenceForecast, ChronosPredictionId, EVENT_INTELLIGENCE_WORKFLOW_VERSION,
    EventConfidence, EventError, EventEvidenceLayer, EventIntelligenceWorkflowConfig,
    EventLinkPair, EventMention, EventRoleKind, EventTrackAssignment, EventTrackId,
    FirstStoryLabel, MentionEvidenceClocks, MentionReviewStatus, SchemaSlotAssignment,
    StorySegmentation,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

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
            award_original,
            protest_original,
            award_noisy,
            award_revised,
        }
    }

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

    fn input(&self) -> EventIntelligenceRunInput {
        EventIntelligenceRunInput::new(
            workflow_config(),
            StorySegmentation::new(3, vec![false, true]).expect("segmentation"),
            self.mentions(),
            self.links(),
            Self::first_story_labels(),
            self.track_assignments(),
            Self::schema_slots(),
            Self::forecasts(),
        )
    }
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "event-intelligence-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-event-intelligence".into(),
        knowledge_cutoff: "2026-03-31T00:00:00Z".into(),
        model_contract_version: EVENT_INTELLIGENCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: EVENT_INTELLIGENCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-event-intelligence",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff")
}

#[test]
fn composed_workflow_emits_digest_bound_cutoff_safe_counts() {
    let fixture = KnownTruthFixture::build();
    let request = request();
    let accepted = accepted(&request);
    let input = fixture.input();
    assert_eq!(
        input.config().version(),
        EVENT_INTELLIGENCE_WORKFLOW_VERSION
    );
    assert_eq!(input.mentions().len(), 3);
    assert_eq!(input.links().len(), 2);
    assert_eq!(input.first_story_labels().len(), 3);
    assert_eq!(input.track_assignments().len(), 3);
    assert_eq!(input.schema_slot_assignments().len(), 2);
    assert_eq!(input.occurrence_forecasts().len(), 1);
    assert_eq!(input.segmentation().unit_count(), 3);

    let execution = execute_event_intelligence_run(
        &request,
        &accepted,
        "snapshot-event-intelligence",
        cutoff(),
        input,
        "2026-04-01T00:00:00Z",
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.mention_count, 3);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 0);
    assert_eq!(execution.artifact.link_count, 2);
    assert_eq!(execution.artifact.first_story_count, 3);
    assert_eq!(execution.artifact.track_count, 3);
    assert_eq!(execution.artifact.schema_slot_count, 2);
    assert_eq!(execution.artifact.forecast_count, 1);
    assert_eq!(
        execution.artifact.envelope_layer,
        EventEvidenceLayer::TdtDetection.wire_name()
    );
    assert_eq!(
        execution.artifact.hypothesis_layer,
        EventEvidenceLayer::ChronosPrediction.wire_name()
    );
    assert_ne!(
        execution.artifact.envelope_layer,
        EventEvidenceLayer::PromotedTransition.wire_name()
    );
    assert_eq!(
        execution.artifact.inference_status,
        "composed_workflow_not_instance_or_transition"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(EVENT_INTELLIGENCE_ARTIFACT_SCHEMA_VERSION)
    );
    assert!(execution.artifact.to_json().is_ok());
}

#[test]
fn execution_excludes_mentions_unavailable_at_the_request_cutoff() {
    let fixture = KnownTruthFixture::build();
    let mut request = request();
    request.knowledge_cutoff = "2026-03-05T00:00:00Z".into();
    let accepted = accepted(&request);
    let early_cutoff = KnowledgeCutoff::parse_rfc3339("2026-03-05T00:00:00Z").expect("cutoff");
    let input = EventIntelligenceRunInput::new(
        workflow_config(),
        StorySegmentation::new(3, vec![false, true]).expect("segmentation"),
        vec![
            fixture.award_original.clone(),
            fixture.protest_original.clone(),
            fixture.award_noisy.clone(),
            fixture.award_revised.clone(),
        ],
        {
            let mut links = fixture.links();
            links.push(
                EventLinkPair::new(
                    fixture.award_original.mention_id(),
                    fixture.award_revised.mention_id(),
                )
                .expect("revised link"),
            );
            links
        },
        vec![
            FirstStoryLabel::FirstStory,
            FirstStoryLabel::FollowUp,
            FirstStoryLabel::FollowUp,
            FirstStoryLabel::FollowUp,
        ],
        vec![
            EventTrackAssignment::new(
                fixture.award_original.mention_id(),
                EventTrackId::from_raw(1),
            ),
            EventTrackAssignment::new(
                fixture.protest_original.mention_id(),
                EventTrackId::from_raw(1),
            ),
            EventTrackAssignment::new(fixture.award_noisy.mention_id(), EventTrackId::from_raw(1)),
            EventTrackAssignment::new(
                fixture.award_revised.mention_id(),
                EventTrackId::from_raw(1),
            ),
        ],
        KnownTruthFixture::schema_slots(),
        KnownTruthFixture::forecasts(),
    );
    let execution = execute_event_intelligence_run(
        &request,
        &accepted,
        "snapshot-event-intelligence",
        early_cutoff,
        input,
        "2026-04-01T00:00:00Z",
    )
    .expect("execution");
    assert_eq!(execution.artifact.mention_count, 3);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 1);
    assert_eq!(execution.artifact.link_count, 2);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let fixture = KnownTruthFixture::build();
    let request = request();
    let accepted = accepted(&request);

    assert_eq!(
        execute_event_intelligence_run(
            &request,
            &accepted,
            "other-snapshot",
            cutoff(),
            fixture.input(),
            "2026-04-01T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    for invalid_request in [
        {
            let mut value = request.clone();
            value.knowledge_cutoff = "2026-03-05T00:00:00Z".into();
            value
        },
        {
            let mut value = request.clone();
            value.model_contract_version = "other-model".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "other-profile".into();
            value
        },
    ] {
        assert_eq!(
            execute_event_intelligence_run(
                &invalid_request,
                &accepted,
                "snapshot-event-intelligence",
                cutoff(),
                fixture.input(),
                "2026-04-01T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn execution_refuses_stream_misalignment() {
    let fixture = KnownTruthFixture::build();
    let request = request();
    let accepted = accepted(&request);

    let mismatched_labels = EventIntelligenceRunInput::new(
        workflow_config(),
        StorySegmentation::new(3, vec![false, true]).expect("segmentation"),
        fixture.mentions(),
        fixture.links(),
        vec![FirstStoryLabel::FirstStory],
        fixture.track_assignments(),
        KnownTruthFixture::schema_slots(),
        KnownTruthFixture::forecasts(),
    );
    assert_eq!(
        execute_event_intelligence_run(
            &request,
            &accepted,
            "snapshot-event-intelligence",
            cutoff(),
            mismatched_labels,
            "2026-04-01T00:00:00Z",
        ),
        Err(AnalysisEngineError::Event(EventError::InvalidWirePayload))
    );

    let mismatched_tracks = EventIntelligenceRunInput::new(
        workflow_config(),
        StorySegmentation::new(3, vec![false, true]).expect("segmentation"),
        fixture.mentions(),
        fixture.links(),
        KnownTruthFixture::first_story_labels(),
        vec![EventTrackAssignment::new(
            fixture.award_original.mention_id(),
            EventTrackId::from_raw(1),
        )],
        KnownTruthFixture::schema_slots(),
        KnownTruthFixture::forecasts(),
    );
    assert_eq!(
        execute_event_intelligence_run(
            &request,
            &accepted,
            "snapshot-event-intelligence",
            cutoff(),
            mismatched_tracks,
            "2026-04-01T00:00:00Z",
        ),
        Err(AnalysisEngineError::Event(EventError::InvalidWirePayload))
    );
}

#[test]
fn execution_refuses_empty_cutoff_receipt_mismatch_and_compose_failure() {
    let fixture = KnownTruthFixture::build();
    let request = request();
    let accepted = accepted(&request);
    let input = fixture.input();
    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-03-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-03-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute_event_intelligence_run(
            &early_request,
            &accepted,
            "snapshot-event-intelligence",
            too_early,
            input,
            "2026-04-01T00:00:00Z",
        ),
        Err(AnalysisEngineError::Event(
            EventError::MentionIneligibleAtCutoff
        ))
    );

    let wrong_receipt = AnalysisRunAccepted::new("run-event-intelligence", "accepted", "other-key")
        .expect("accepted");
    assert_eq!(
        execute_event_intelligence_run(
            &request,
            &wrong_receipt,
            "snapshot-event-intelligence",
            cutoff(),
            fixture.input(),
            "2026-04-01T00:00:00Z",
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let mut tracks = fixture.track_assignments();
    tracks[0] =
        EventTrackAssignment::new(fixture.award_noisy.mention_id(), EventTrackId::from_raw(1));
    let misaligned_track = EventIntelligenceRunInput::new(
        workflow_config(),
        StorySegmentation::new(3, vec![false, true]).expect("segmentation"),
        fixture.mentions(),
        fixture.links(),
        KnownTruthFixture::first_story_labels(),
        tracks,
        KnownTruthFixture::schema_slots(),
        KnownTruthFixture::forecasts(),
    );
    assert_eq!(
        execute_event_intelligence_run(
            &request,
            &accepted,
            "snapshot-event-intelligence",
            cutoff(),
            misaligned_track,
            "2026-04-01T00:00:00Z",
        ),
        Err(AnalysisEngineError::Event(EventError::InvalidWirePayload))
    );
}
