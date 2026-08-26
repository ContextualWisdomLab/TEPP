//! TDT tracks are not instances; pair P/R and switch rate come from truth.

use event_core::{
    EventConfidence, EventError, EventMention, EventMentionId, EventTrackAssignment, EventTrackId,
    EventTrackLabel, MentionEvidenceClocks, MentionReviewStatus, decide_track_continue,
    refuse_track_as_instance, refuse_track_as_transition, tracking_identity_switch_rate,
    tracking_pair_precision, tracking_pair_recall,
};
use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};
use temporal_core::{
    AssertionTime, AvailableTime, DocumentTime, EventTime, KnowledgeCutoff, SystemTime,
};

const DOCUMENT_TEXT: &str = "alpha bravo charlie delta echo foxtrot golf hotel";

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

fn mention() -> EventMentionId {
    grounded_mention("alpha").mention_id()
}

fn assignment(mention_id: EventMentionId, track: u32) -> EventTrackAssignment {
    EventTrackAssignment::new(mention_id, EventTrackId::from_raw(track))
}

#[test]
fn event_track_cannot_be_cast_to_an_instance_or_transition() {
    let track = EventTrackId::from_raw(1);
    assert_eq!(
        refuse_track_as_instance(track),
        Err(EventError::EventTrackIsNotEventInstance)
    );
    assert_eq!(
        refuse_track_as_transition(track),
        Err(EventError::EventTrackIsNotStateTransition)
    );
}

#[test]
fn pair_precision_and_recall_are_computed_from_known_truth_assignments() {
    let a = mention();
    let b = mention();
    let c = mention();
    let d = mention();
    let truth = [
        assignment(a, 1),
        assignment(b, 1),
        assignment(c, 2),
        assignment(d, 2),
    ];
    let calibrated = [
        assignment(a, 1),
        assignment(b, 1),
        assignment(c, 2),
        assignment(d, 3),
    ];
    let always_one_track = [
        assignment(a, 1),
        assignment(b, 1),
        assignment(c, 1),
        assignment(d, 1),
    ];

    let calibrated_precision = tracking_pair_precision(&truth, &calibrated).expect("precision");
    let naive_precision = tracking_pair_precision(&truth, &always_one_track).expect("naive p");
    let calibrated_recall = tracking_pair_recall(&truth, &calibrated).expect("recall");
    let naive_recall = tracking_pair_recall(&truth, &always_one_track).expect("naive r");

    assert!(
        calibrated_precision > naive_precision,
        "computed precision {calibrated_precision} must exceed always-one-track precision {naive_precision}"
    );
    assert!(calibrated_recall <= naive_recall);
}

#[test]
fn identity_switch_rate_is_lower_for_stable_tracks_than_always_switch() {
    let a = mention();
    let b = mention();
    let c = mention();
    let d = mention();
    let truth = [
        assignment(a, 1),
        assignment(b, 1),
        assignment(c, 2),
        assignment(d, 2),
    ];
    let stable = truth;
    let always_switch = [
        assignment(a, 1),
        assignment(b, 2),
        assignment(c, 3),
        assignment(d, 4),
    ];

    let stable_rate = tracking_identity_switch_rate(&truth, &stable).expect("stable");
    let switch_rate = tracking_identity_switch_rate(&truth, &always_switch).expect("switch");
    assert!(
        stable_rate < switch_rate,
        "computed switch rate {stable_rate} must be below always-switch rate {switch_rate}"
    );
}

#[test]
fn calibrated_same_track_scores_have_lower_rmse_than_always_one_track() {
    let truth = [1.0_f64, 1.0, 0.0, 0.0, 0.0, 1.0];
    let calibrated = [0.90_f64, 0.85, 0.15, 0.10, 0.20, 0.88];
    let always_one = [1.0_f64, 1.0, 1.0, 1.0, 1.0, 1.0];
    let calibrated_rmse = computed_rmse(&truth, &calibrated);
    let naive_rmse = computed_rmse(&truth, &always_one);
    assert!(
        calibrated_rmse < naive_rmse,
        "computed calibrated RMSE {calibrated_rmse} must be below always-one-track RMSE {naive_rmse}"
    );
}

#[test]
fn assignment_helpers_fail_closed_on_empty_mismatch_duplicate_and_missing_pairs() {
    let a = mention();
    let b = mention();
    let one = [assignment(a, 1)];
    let two = [assignment(a, 1), assignment(b, 1)];
    assert_eq!(
        tracking_pair_precision(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_pair_recall(&one, &two),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_pair_precision(&one, &one),
        Err(EventError::InvalidWirePayload)
    );
    let duplicate = [assignment(a, 1), assignment(a, 2)];
    assert_eq!(
        tracking_pair_recall(&duplicate, &duplicate),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_identity_switch_rate(&one, &one),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_identity_switch_rate(&[], &[]),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_identity_switch_rate(&two, &one),
        Err(EventError::InvalidWirePayload)
    );
    let a2 = mention();
    let b2 = mention();
    let c2 = mention();
    let d2 = mention();
    let three = [assignment(a2, 1), assignment(b2, 1), assignment(c2, 2)];
    let four = [
        assignment(a2, 1),
        assignment(b2, 1),
        assignment(c2, 2),
        assignment(d2, 2),
    ];
    assert_eq!(
        tracking_pair_precision(&three, &four),
        Err(EventError::InvalidWirePayload)
    );
    assert_eq!(
        tracking_pair_recall(&three, &four),
        Err(EventError::InvalidWirePayload)
    );
}

#[test]
fn labels_round_trip_and_threshold_is_inclusive() {
    assert_eq!(EventTrackLabel::Continue.wire_name(), "continue");
    assert_eq!(EventTrackLabel::Switch.wire_name(), "switch");
    assert_eq!(
        EventTrackLabel::from_wire_name("continue").expect("parse"),
        EventTrackLabel::Continue
    );
    assert_eq!(
        EventTrackLabel::from_wire_name("switch").expect("parse"),
        EventTrackLabel::Switch
    );
    assert_eq!(
        EventTrackLabel::from_wire_name("same_track"),
        Err(EventError::UnknownEventTrackLabel)
    );
    assert!(EventTrackLabel::Continue.is_continue());
    assert!(!EventTrackLabel::Switch.is_continue());
    assert!((EventTrackLabel::Continue.as_probability_target() - 1.0).abs() < f64::EPSILON);
    assert!((EventTrackLabel::Switch.as_probability_target() - 0.0).abs() < f64::EPSILON);

    let half = EventConfidence::new(0.5).expect("half");
    assert_eq!(decide_track_continue(half, half), EventTrackLabel::Continue);
    assert_eq!(
        decide_track_continue(EventConfidence::new(0.49).expect("below"), half),
        EventTrackLabel::Switch
    );

    let mention_id = mention();
    let assigned = assignment(mention_id, 7);
    assert_eq!(assigned.mention_id(), mention_id);
    assert_eq!(assigned.track_id(), EventTrackId::from_raw(7));
    assert_eq!(EventTrackId::from_raw(7).raw(), 7);
}
