//! Realistic contracts: mentions are not instances; promotion is explicit.

use event_core::{
    EventConfidence, EventError, EventEvidenceLayer, EventInstance, EventMention, EventRegistry,
    EventRoleKind, MentionEvidenceClocks, MentionReviewStatus, refuse_mention_as_instance,
    refuse_span_mention_as_instance,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

const DOCUMENT_TEXT: &str = "contract award announced on 2026-03-01 merger filed";

fn documentary_record() -> DocumentRecord {
    let artifact = SourceArtifact::from_bytes(DOCUMENT_TEXT.as_bytes()).expect("artifact");
    DocumentRecord::from_text(artifact.id(), DOCUMENT_TEXT).expect("document")
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

fn eligible_clocks() -> MentionEvidenceClocks {
    MentionEvidenceClocks::new(
        EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("event"),
        AssertionTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("assertion"),
        DocumentTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("document"),
        SystemTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("system"),
        AvailableTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("available"),
        KnowledgeCutoff::parse_rfc3339("2026-03-31T00:00:00Z").expect("cutoff"),
    )
    .expect("eligible clocks")
}

fn grounded_mention(surface: &str, confidence: EventConfidence) -> EventMention {
    let document = documentary_record();
    EventMention::new(
        &document,
        span_for(&document, surface),
        confidence,
        eligible_clocks(),
        "ace-extent-extractor/1",
        MentionReviewStatus::Proposed,
    )
    .expect("grounded mention")
}

#[test]
fn mention_cannot_be_cast_to_instance_without_promotion() {
    let mention = grounded_mention(
        "contract award announced",
        EventConfidence::new(0.72).expect("confidence"),
    );

    assert_eq!(
        refuse_mention_as_instance(mention.mention_id()),
        Err(EventError::MentionIsNotEventInstance)
    );
    assert_eq!(
        refuse_span_mention_as_instance(&mention),
        Err(EventError::SpanMentionIsNotEventInstance)
    );
}

#[test]
fn registry_requires_supporting_mentions_before_instance_insert() {
    let mention = grounded_mention(
        "contract award announced",
        EventConfidence::new(0.8).expect("confidence"),
    );
    let start = EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("start");
    let end = EventTime::parse_rfc3339("2026-03-01T23:59:59Z").expect("end");

    let mut registry = EventRegistry::new();
    let orphan = EventInstance::promote_from_mentions(
        vec![mention.mention_id()],
        start,
        end,
        EventConfidence::certain().expect("certain"),
        EventEvidenceLayer::PromotedTransition,
    )
    .expect("instance");
    assert_eq!(
        registry.insert_instance(orphan),
        Err(EventError::UnknownEventInstance)
    );

    registry
        .insert_mention(mention.clone())
        .expect("insert mention");
    let mut instance = EventInstance::promote_from_mentions(
        vec![mention.mention_id()],
        start,
        end,
        EventConfidence::certain().expect("certain"),
        EventEvidenceLayer::PromotedTransition,
    )
    .expect("instance");
    instance.assign_role(EventRoleKind::Product, "contract award");
    instance.assign_role(EventRoleKind::Agent, "procurement office");
    registry
        .insert_instance(instance.clone())
        .expect("insert instance");

    assert_eq!(registry.mention_count(), 1);
    assert_eq!(registry.instance_count(), 1);
    let stored_mention = registry
        .mention(mention.mention_id())
        .expect("stored mention");
    assert_eq!(stored_mention.surface_form(), "contract award announced");
    assert_eq!(
        stored_mention.source_span().byte_end() - stored_mention.source_span().byte_start(),
        "contract award announced".len()
    );
    let stored = registry.instance(instance.instance_id()).expect("stored");
    assert_eq!(stored.supporting_mentions(), &[mention.mention_id()]);
    assert_eq!(
        stored.evidence_layer(),
        EventEvidenceLayer::PromotedTransition
    );
    assert_eq!(stored.roles().len(), 2);
    assert!(stored.is_active_at(start));
}

#[test]
fn empty_extractor_and_empty_mention_sets_fail_closed() {
    let document = documentary_record();
    let span = span_for(&document, "award");
    assert_eq!(
        EventMention::new(
            &document,
            span,
            EventConfidence::new(0.5).expect("c"),
            eligible_clocks(),
            "   ",
            MentionReviewStatus::Proposed,
        )
        .map(|_| ()),
        Err(EventError::EmptyExtractorVersion)
    );
    let start = EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("start");
    let end = EventTime::parse_rfc3339("2026-03-02T00:00:00Z").expect("end");
    assert_eq!(
        EventInstance::promote_from_mentions(
            Vec::new(),
            start,
            end,
            EventConfidence::certain().expect("c"),
            EventEvidenceLayer::PromotedTransition,
        )
        .map(|_| ()),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn accessors_and_duplicate_identity_paths_are_covered() {
    let mention = grounded_mention("merger filed", EventConfidence::certain().expect("certain"));
    assert_eq!(mention.surface_form(), "merger filed");
    assert_eq!(mention.document_id(), mention.evidence_id());
    assert_eq!(mention.extractor_version(), "ace-extent-extractor/1");
    assert_eq!(mention.review_status(), MentionReviewStatus::Proposed);
    assert!((mention.confidence().value() - 1.0).abs() < f64::EPSILON);
    assert_eq!(event_core::EVENT_INSTANCE_WIRE_SCHEMA_VERSION, 1);

    let start = EventTime::parse_rfc3339("2026-04-01T00:00:00Z").expect("start");
    let end = EventTime::parse_rfc3339("2026-04-02T00:00:00Z").expect("end");
    let instance = EventInstance::promote_from_mentions(
        vec![mention.mention_id()],
        start,
        end,
        EventConfidence::new(0.9).expect("c"),
        EventEvidenceLayer::PromotedTransition,
    )
    .expect("instance");
    assert!((instance.confidence().value() - 0.9).abs() < f64::EPSILON);
    assert_eq!(
        instance.event_time().certainty(),
        temporal_core::TemporalCertainty::Bounded
    );

    let mut registry = EventRegistry::new();
    registry.insert_mention(mention.clone()).expect("insert");
    assert_eq!(
        registry.insert_mention(mention.clone()),
        Err(EventError::DuplicateEventIdentity)
    );
    registry
        .insert_instance(instance.clone())
        .expect("instance");
    assert_eq!(
        registry.insert_instance(instance),
        Err(EventError::DuplicateEventIdentity)
    );
}

#[test]
fn promotion_rejects_every_non_promoted_evidence_layer() {
    let mention = grounded_mention(
        "contract award announced",
        EventConfidence::certain().expect("confidence"),
    );
    let start = EventTime::parse_rfc3339("2026-03-01T00:00:00Z").expect("start");
    let end = EventTime::parse_rfc3339("2026-03-01T23:59:59Z").expect("end");

    for (layer, expected) in [
        (
            EventEvidenceLayer::ObservedMention,
            EventError::DetectionIsNotTransition,
        ),
        (
            EventEvidenceLayer::TdtDetection,
            EventError::DetectionIsNotTransition,
        ),
        (
            EventEvidenceLayer::ChronosPrediction,
            EventError::PredictionIsNotFact,
        ),
        (
            EventEvidenceLayer::TemporalConsistency,
            EventError::DetectionIsNotTransition,
        ),
    ] {
        assert_eq!(
            EventInstance::promote_from_mentions(
                vec![mention.mention_id()],
                start,
                end,
                EventConfidence::certain().expect("confidence"),
                layer,
            )
            .map(|_| ()),
            Err(expected)
        );
    }
}
