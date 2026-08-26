//! TDT link detections are not instances; precision/recall come from truth.

use event_core::{
    EventConfidence, EventError, EventLinkLabel, EventLinkPair, EventMention, EventMentionId,
    MentionEvidenceClocks, MentionReviewStatus, decide_event_link, event_link_precision,
    event_link_recall, refuse_event_link_as_instance, refuse_event_link_as_transition,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

const DOCUMENT_TEXT: &str = "alpha bravo charlie delta echo foxtrot";

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

fn grounded_mention(surface: &str) -> EventMention {
    let document = documentary_record();
    EventMention::new(
        &document,
        span_for(&document, surface),
        EventConfidence::new(0.8).expect("confidence"),
        eligible_clocks(),
        "ace-extent-extractor/1",
        MentionReviewStatus::Proposed,
    )
    .expect("grounded mention")
}

fn mention_id(surface: &str) -> EventMentionId {
    grounded_mention(surface).mention_id()
}

fn computed_rmse(truth: &[f64], recovered: &[f64]) -> f64 {
    assert_eq!(truth.len(), recovered.len());
    let n = f64::from(u32::try_from(truth.len()).expect("tiny fixture"));
    let sse: f64 = truth
        .iter()
        .zip(recovered)
        .map(|(truth_value, recovered_value)| {
            let residual = truth_value - recovered_value;
            residual * residual
        })
        .sum();
    (sse / n).sqrt()
}

fn pair(left: EventMentionId, right: EventMentionId) -> EventLinkPair {
    EventLinkPair::new(left, right).expect("distinct mentions")
}

#[test]
fn event_link_detection_cannot_be_cast_to_an_instance_or_transition() {
    let left = mention_id("alpha");
    let right = mention_id("bravo");
    let link = pair(left, right);
    assert_eq!(
        refuse_event_link_as_instance(link),
        Err(EventError::EventLinkIsNotEventInstance)
    );
    assert_eq!(
        refuse_event_link_as_transition(link),
        Err(EventError::EventLinkIsNotStateTransition)
    );
}

#[test]
fn precision_and_recall_are_computed_from_known_truth_pairs() {
    let a = mention_id("alpha");
    let b = mention_id("bravo");
    let c = mention_id("charlie");
    let d = mention_id("delta");
    let truth = [pair(a, b), pair(b, c)];
    let calibrated = [pair(a, b)];
    let always_link = [pair(a, b), pair(b, c), pair(c, d), pair(a, d)];

    let calibrated_precision = event_link_precision(&truth, &calibrated).expect("precision");
    let naive_precision = event_link_precision(&truth, &always_link).expect("naive precision");
    let calibrated_recall = event_link_recall(&truth, &calibrated).expect("recall");
    let naive_recall = event_link_recall(&truth, &always_link).expect("naive recall");

    assert!(
        calibrated_precision > naive_precision,
        "computed precision {calibrated_precision} must exceed always-link precision {naive_precision}"
    );
    assert!(
        calibrated_recall < naive_recall,
        "computed recall {calibrated_recall} must stay below the always-link recall {naive_recall}"
    );
}

#[test]
fn calibrated_link_scores_have_lower_rmse_than_always_link() {
    let truth = [1.0_f64, 0.0, 0.0, 1.0, 0.0, 0.0];
    let calibrated = [0.90_f64, 0.10, 0.15, 0.85, 0.20, 0.05];
    let always_link = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_link);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-link RMSE {naive_rmse}"
    );
}

#[test]
fn pair_helpers_fail_closed_on_self_links_empty_and_missing_sets() {
    let mention = mention_id("echo");
    assert_eq!(
        EventLinkPair::new(mention, mention),
        Err(EventError::InvalidWirePayload)
    );
    let a = mention_id("alpha");
    let b = mention_id("bravo");
    let truth = [pair(a, b)];
    assert_eq!(
        event_link_precision(&truth, &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        event_link_recall(&[], &truth),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(EventLinkLabel::Linked.wire_name(), "linked");
    assert_eq!(EventLinkLabel::Unlinked.wire_name(), "unlinked");
    assert_eq!(
        EventLinkLabel::from_wire_name("linked").expect("parse"),
        EventLinkLabel::Linked
    );
    assert_eq!(
        EventLinkLabel::from_wire_name("unlinked").expect("parse"),
        EventLinkLabel::Unlinked
    );
    assert_eq!(
        EventLinkLabel::from_wire_name("same_event"),
        Err(EventError::UnknownEventLinkLabel)
    );
    assert!(EventLinkLabel::Linked.is_linked());
    assert!(!EventLinkLabel::Unlinked.is_linked());
    assert!((EventLinkLabel::Linked.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!((EventLinkLabel::Unlinked.as_probability_target() - 0.0).abs() < f64::EPSILON);

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(decide_event_link(half, half), EventLinkLabel::Linked);
    assert_eq!(
        decide_event_link(EventConfidence::new(0.49).expect("below"), half),
        EventLinkLabel::Unlinked
    );

    let left = mention_id("echo");
    let right = mention_id("foxtrot");
    assert_eq!(pair(left, right), pair(right, left));
    assert_ne!(pair(left, right).left(), pair(left, right).right());
}
