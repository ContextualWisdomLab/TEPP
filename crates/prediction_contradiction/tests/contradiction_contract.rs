//! Predicted intervals stay hypothetical unless later-available evidence overlaps.

use prediction_contradiction::{
    PredictionContradictionError, PromotionSupport, classify_promotion_support,
    contradiction_agreement_rate, intervals_contradict, refuse_contradiction_or_adjacency,
    refuse_promotion, require_observed_coverage,
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
        refuse_contradiction_or_adjacency(
            &predicted,
            &later_observed,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
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
        refuse_contradiction_or_adjacency(
            &earlier_observed,
            &predicted_later,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
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
        refuse_contradiction_or_adjacency(
            &predicted,
            &meeting,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
    assert_eq!(
        refuse_promotion(&predicted, &meeting, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
    assert!(!intervals_contradict(&met_by, &earlier).expect("met_by"));
    assert_eq!(
        refuse_contradiction_or_adjacency(&met_by, &earlier, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
    assert_eq!(
        refuse_promotion(&met_by, &earlier, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
    );
}

#[test]
fn overlapping_observation_is_not_contradiction_and_is_not_coverage() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    let (observed_available, knowledge_cutoff) = eligible_clocks();
    assert!(!intervals_contradict(&predicted, &overlapping).expect("overlaps"));
    refuse_contradiction_or_adjacency(
        &predicted,
        &overlapping,
        observed_available,
        knowledge_cutoff,
    )
    .expect("overlap is not Allen contradiction");
    assert_eq!(
        classify_promotion_support(&predicted, &overlapping).expect("overlaps"),
        PromotionSupport::PartialOverlap
    );
    assert_eq!(
        require_observed_coverage(
            &predicted,
            &overlapping,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionNotCoveredByObservation)
    );
}

#[test]
fn refuse_promotion_refuses_unmatched_predicted_mass() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    let contained = closed_event_interval(2, 8);
    let (observed_available, knowledge_cutoff) = eligible_clocks();
    assert_eq!(
        refuse_promotion(
            &predicted,
            &overlapping,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionNotCoveredByObservation)
    );
    assert_eq!(
        refuse_promotion(
            &predicted,
            &contained,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionNotCoveredByObservation)
    );
}

#[test]
fn evidence_available_after_cutoff_is_ineligible() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    assert_eq!(
        refuse_contradiction_or_adjacency(
            &predicted,
            &overlapping,
            available("2026-01-04T00:00:00Z"),
            cutoff("2026-01-03T00:00:00Z"),
        ),
        Err(PredictionContradictionError::EvidenceAfterCutoff)
    );
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

#[test]
fn availability_equal_to_cutoff_remains_eligible() {
    let predicted = closed_event_interval(0, 10);
    let overlapping = closed_event_interval(5, 15);
    let covering = closed_event_interval(0, 10);
    let observed_available = available("2026-01-03T00:00:00Z");
    let knowledge_cutoff = cutoff("2026-01-03T00:00:00Z");
    refuse_contradiction_or_adjacency(
        &predicted,
        &overlapping,
        observed_available,
        knowledge_cutoff,
    )
    .expect("available == cutoff is eligible for the contradiction filter");
    require_observed_coverage(&predicted, &covering, observed_available, knowledge_cutoff)
        .expect("available == cutoff is eligible for coverage");
    refuse_promotion(&predicted, &covering, observed_available, knowledge_cutoff)
        .expect("available == cutoff is eligible for promotion");
}

#[test]
fn after_and_overlapped_by_use_the_same_promotion_rules() {
    let predicted = closed_event_interval(20, 30);
    let earlier_observed = closed_event_interval(0, 10);
    let overlapped_by = closed_event_interval(5, 15);
    let later_predicted = closed_event_interval(10, 20);
    let (observed_available, knowledge_cutoff) = eligible_clocks();

    assert!(intervals_contradict(&predicted, &earlier_observed).expect("after"));
    assert_eq!(
        refuse_contradiction_or_adjacency(
            &predicted,
            &earlier_observed,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
    assert_eq!(
        refuse_promotion(
            &predicted,
            &earlier_observed,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
    assert_eq!(
        classify_promotion_support(&later_predicted, &overlapped_by).expect("overlapped_by"),
        PromotionSupport::PartialOverlap
    );
    refuse_contradiction_or_adjacency(
        &later_predicted,
        &overlapped_by,
        observed_available,
        knowledge_cutoff,
    )
    .expect("overlapped_by is not Allen contradiction");
    assert_eq!(
        refuse_promotion(
            &later_predicted,
            &overlapped_by,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionNotCoveredByObservation)
    );
    assert_eq!(
        require_observed_coverage(
            &later_predicted,
            &overlapped_by,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::PredictionNotCoveredByObservation)
    );
}

#[test]
fn half_open_observed_interval_is_not_an_allen_input() {
    let predicted = closed_event_interval(0, 10);
    let observed = TemporalInterval::bounded(
        TemporalBoundary::Included(event_at(5)),
        TemporalBoundary::Excluded(event_at(15)),
        TemporalPrecision::Second,
    )
    .expect("half-open observed interval is representable");
    let (observed_available, knowledge_cutoff) = eligible_clocks();
    assert_eq!(
        intervals_contradict(&predicted, &observed),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        refuse_contradiction_or_adjacency(
            &predicted,
            &observed,
            observed_available,
            knowledge_cutoff
        ),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        refuse_promotion(&predicted, &observed, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        classify_promotion_support(&predicted, &observed),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        require_observed_coverage(&predicted, &observed, observed_available, knowledge_cutoff),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
}

#[test]
fn known_allen_pairs_recover_coverage_and_contradiction_labels() {
    let cases = [
        (
            closed_event_interval(0, 10),
            closed_event_interval(20, 30),
            PromotionSupport::ContradictoryDisjoint,
        ),
        (
            closed_event_interval(20, 30),
            closed_event_interval(0, 10),
            PromotionSupport::ContradictoryDisjoint,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(10, 20),
            PromotionSupport::AdjacentWithoutOverlap,
        ),
        (
            closed_event_interval(10, 20),
            closed_event_interval(0, 10),
            PromotionSupport::AdjacentWithoutOverlap,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(5, 15),
            PromotionSupport::PartialOverlap,
        ),
        (
            closed_event_interval(10, 20),
            closed_event_interval(5, 15),
            PromotionSupport::PartialOverlap,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(0, 8),
            PromotionSupport::PartialOverlap,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(2, 8),
            PromotionSupport::PartialOverlap,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(2, 10),
            PromotionSupport::PartialOverlap,
        ),
        (
            closed_event_interval(0, 8),
            closed_event_interval(0, 20),
            PromotionSupport::ObservedCoversPrediction,
        ),
        (
            closed_event_interval(2, 8),
            closed_event_interval(0, 20),
            PromotionSupport::ObservedCoversPrediction,
        ),
        (
            closed_event_interval(2, 10),
            closed_event_interval(0, 10),
            PromotionSupport::ObservedCoversPrediction,
        ),
        (
            closed_event_interval(0, 10),
            closed_event_interval(0, 10),
            PromotionSupport::ObservedCoversPrediction,
        ),
    ];
    let (observed_available, knowledge_cutoff) = eligible_clocks();
    let mut truth = Vec::new();
    let mut decided = Vec::new();
    for (predicted, observed, expected) in cases {
        let support = classify_promotion_support(&predicted, &observed).expect("label");
        assert_eq!(support, expected);
        truth.push(expected == PromotionSupport::ObservedCoversPrediction);
        decided.push(
            require_observed_coverage(&predicted, &observed, observed_available, knowledge_cutoff)
                .is_ok(),
        );
    }
    let agreed = contradiction_agreement_rate(&truth, &decided).expect("coverage agreement");
    assert!((agreed - 1.0).abs() < f64::EPSILON);
}
