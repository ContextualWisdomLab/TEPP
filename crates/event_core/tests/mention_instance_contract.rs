//! Realistic contracts: mentions are not instances; promotion is explicit.

use event_core::{
    EventConfidence, EventError, EventEvidenceLayer, EventInstance, EventMention, EventRegistry,
    EventRoleKind, refuse_mention_as_instance,
};
use evidence_core::{DocumentRecord, EvidenceId, SourceArtifact};
use temporal_core::EventTime;

fn document_evidence() -> EvidenceId {
    let artifact =
        SourceArtifact::from_bytes(b"contract award announced on 2026-03-01").expect("artifact");
    let document =
        DocumentRecord::from_text(artifact.id(), "contract award announced on 2026-03-01")
            .expect("document");
    document.id()
}

#[test]
fn mention_cannot_be_cast_to_instance_without_promotion() {
    let evidence_id = document_evidence();
    let mention = EventMention::new(
        evidence_id,
        "contract award announced",
        EventConfidence::new(0.72).expect("confidence"),
    )
    .expect("mention");

    assert_eq!(
        refuse_mention_as_instance(mention.mention_id()),
        Err(EventError::MentionIsNotEventInstance)
    );
}

#[test]
fn registry_requires_supporting_mentions_before_instance_insert() {
    let evidence_id = document_evidence();
    let mention = EventMention::new(
        evidence_id,
        "contract award announced",
        EventConfidence::new(0.8).expect("confidence"),
    )
    .expect("mention");
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
    assert!(registry.mention(mention.mention_id()).is_some());
    let stored = registry.instance(instance.instance_id()).expect("stored");
    assert_eq!(stored.supporting_mentions(), &[mention.mention_id()]);
    assert_eq!(stored.roles().len(), 2);
    assert!(stored.is_active_at(start));
}

#[test]
fn empty_surface_and_empty_mention_sets_fail_closed() {
    let evidence_id = document_evidence();
    assert_eq!(
        EventMention::new(evidence_id, "   ", EventConfidence::new(0.5).expect("c")).map(|_| ()),
        Err(EventError::InvalidWirePayload)
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
    let evidence_id = document_evidence();
    let mention = EventMention::new(
        evidence_id,
        "merger filed",
        EventConfidence::certain().expect("certain"),
    )
    .expect("mention");
    assert_eq!(mention.evidence_id(), evidence_id);
    assert_eq!(mention.surface_form(), "merger filed");
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
    let evidence_id = document_evidence();
    let mention = EventMention::new(
        evidence_id,
        "contract award announced",
        EventConfidence::certain().expect("confidence"),
    )
    .expect("mention");
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
