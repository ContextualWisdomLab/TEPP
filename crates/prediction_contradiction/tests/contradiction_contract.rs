//! Predicted intervals stay hypothetical unless later-available evidence overlaps.

use prediction_contradiction::{
    PredictionContradictionError, contradiction_agreement_rate, intervals_contradict,
    refuse_promotion,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};

fn event_at(second: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-01T00:00:{second:02}Z")).expect("event time")
}

fn closed_event_interval(start: u8, end: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_at(start)),
        TemporalBoundary::Included(event_at(end)),
        TemporalPrecision::Second,
    )
    .expect("closed proper interval")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available time")
}

fn cutoff(stamp: &str) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(stamp).expect("knowledge cutoff")
}

fn eligible_clocks() -> (AvailableTime, KnowledgeCutoff) {
    (
        available("2026-01-02T00:00:00Z"),
        cutoff("2026-01-03T00:00:00Z"),
    )
}

#[test]
fn before_and_after_cannot_become_observed_fact() {
    let predicted = closed_event_interval(0, 10);
    let later_observed = closed_event_interval(20, 30);
    let earlier_observed = closed_event_interval(40, 50);
    let predicted_later = closed_event_interval(0, 10);
    let (observed_available, knowledge_cutoff) = eligible_clocks();

    assert!(intervals_contradict(&predicted, &later_observed).expect("before"));
    assert_eq!(
        refuse_promotion(
            &predicted,
            &later_observed,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );

    assert!(intervals_contradict(&earlier_observed, &predicted_later).expect("after"));
    assert_eq!(
        refuse_promotion(
            &earlier_observed,
            &predicted_later,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
}

#[test]
fn meeting_intervals_are_adjacent_not_allen_contradiction() {
    let predicted = closed_event_interval(0, 10);
    let meeting = closed_event_interval(10, 20);
    let met_by = closed_event_interval(10, 20);
    let earlier = closed_event_interval(0, 10);
    let (observed_available, knowledge_cutoff) = eligible_clocks();

    assert!(!intervals_contradict(&predicted, &meeting).expect("meets"));
    assert_eq!(
        refuse_promotion(&predicted, &meeting, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
    assert!(!intervals_contradict(&met_by, &earlier).expect("met_by"));
    assert_eq!(
        refuse_promotion(&met_by, &earlier, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
}

#[test]
fn overlapping_observation_may_support_promotion() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    let (observed_available, knowledge_cutoff) = eligible_clocks();
    assert!(!intervals_contradict(&predicted, &overlapping).expect("overlaps"));
    refuse_promotion(
        &predicted,
        &overlapping,
        observed_available,
        knowledge_cutoff,
    )
    .expect("overlapping support");
}

#[test]
fn evidence_available_after_cutoff_is_ineligible() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    assert_eq!(
        refuse_promotion(
            &predicted,
            &overlapping,
            available("2026-01-04T00:00:00Z"),
            cutoff("2026-01-03T00:00:00Z"),
        ),
        Err(PredictionContradictionError::EvidenceAfterCutoff)
    );
}

#[test]
fn half_open_intervals_are_not_allen_inputs() {
    let predicted = TemporalInterval::bounded(
        TemporalBoundary::Included(event_at(0)),
        TemporalBoundary::Excluded(event_at(10)),
        TemporalPrecision::Second,
    )
    .expect("half-open interval is representable");
    let observed = closed_event_interval(20, 30);
    assert_eq!(
        intervals_contradict(&predicted, &observed),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
}

#[test]
fn agreement_rate_matches_known_allen_labels_not_promote_all() {
    let pairs = [
        (closed_event_interval(0, 10), closed_event_interval(20, 30)),
        (closed_event_interval(0, 10), closed_event_interval(5, 15)),
        (closed_event_interval(0, 10), closed_event_interval(10, 20)),
        (closed_event_interval(40, 50), closed_event_interval(0, 10)),
    ];
    let truth = [true, false, false, true];
    let decided = [
        intervals_contradict(&pairs[0].0, &pairs[0].1).expect("before"),
        intervals_contradict(&pairs[1].0, &pairs[1].1).expect("overlaps"),
        intervals_contradict(&pairs[2].0, &pairs[2].1).expect("meets"),
        intervals_contradict(&pairs[3].0, &pairs[3].1).expect("after"),
    ];
    let collapsed = [false, false, false, false];
    let agreed = contradiction_agreement_rate(&truth, &decided).expect("agreement");
    let collapsed_rate = contradiction_agreement_rate(&truth, &collapsed).expect("collapsed");
    assert!((agreed - 1.0).abs() < f64::EPSILON);
    assert!(agreed > collapsed_rate);
}

#[test]
fn empty_or_mismatched_agreement_slices_fail_closed() {
    assert_eq!(
        contradiction_agreement_rate(&[], &[]),
        Err(PredictionContradictionError::AgreementSliceMismatch)
    );
    assert_eq!(
        contradiction_agreement_rate(&[true], &[]),
        Err(PredictionContradictionError::AgreementSliceMismatch)
    );
    assert_eq!(
        contradiction_agreement_rate(&[true, false], &[true]),
        Err(PredictionContradictionError::AgreementSliceMismatch)
    );
}
