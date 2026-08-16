//! Predicted-versus-observed promotion using `temporal_core` Allen classification.

use crate::PredictionContradictionError;
use temporal_core::{
    AllenRelation, AvailableTime, EventTime, KnowledgeCutoff, TemporalError, TemporalInterval,
    classify_interval_relation,
};

fn map_temporal(error: TemporalError) -> PredictionContradictionError {
    let _ = error;
    PredictionContradictionError::InvalidIntervalPayload
}

/// How later-observed evidence relates to a predicted event-time interval.
///
/// `Ok(())` from [`refuse_promotion`] or [`require_observed_coverage`] means
/// every predicted instant has observed support. `Ok(())` from
/// [`refuse_contradiction_or_adjacency`] only means the pair is not an Allen
/// contradiction or adjacency refusal. Only
/// [`PromotionSupport::ObservedCoversPrediction`] authorizes promotion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionSupport {
    /// Observed interval covers every instant of the predicted interval.
    ObservedCoversPrediction,
    /// Interiors overlap, but some predicted mass has no observed support.
    PartialOverlap,
    /// Intervals share an endpoint and have no interior overlap.
    AdjacentWithoutOverlap,
    /// Intervals are strictly disjoint with a gap.
    ContradictoryDisjoint,
}

/// Classify predicted-versus-observed support without applying cutoff policy.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::InvalidIntervalPayload`] when either
/// interval is not a closed proper Allen input.
pub fn classify_promotion_support(
    predicted: &TemporalInterval<EventTime>,
    observed: &TemporalInterval<EventTime>,
) -> Result<PromotionSupport, PredictionContradictionError> {
    match classify_interval_relation(predicted, observed).map_err(map_temporal)? {
        AllenRelation::Before | AllenRelation::After => Ok(PromotionSupport::ContradictoryDisjoint),
        AllenRelation::Meets | AllenRelation::MetBy => Ok(PromotionSupport::AdjacentWithoutOverlap),
        AllenRelation::Overlaps
        | AllenRelation::OverlappedBy
        | AllenRelation::Contains
        | AllenRelation::StartedBy
        | AllenRelation::FinishedBy => Ok(PromotionSupport::PartialOverlap),
        AllenRelation::Starts
        | AllenRelation::During
        | AllenRelation::Finishes
        | AllenRelation::Equals => Ok(PromotionSupport::ObservedCoversPrediction),
    }
}

/// Return whether two closed proper intervals are Allen `before` or `after`.
///
/// Adjacent `meets` / `met_by` pairs are not contradictions. They share an
/// endpoint and remain consistent under Allen (1983); they still lack interior
/// overlap and therefore cannot support promotion.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::InvalidIntervalPayload`] when either
/// interval is not a closed proper Allen input.
pub fn intervals_contradict(
    predicted: &TemporalInterval<EventTime>,
    observed: &TemporalInterval<EventTime>,
) -> Result<bool, PredictionContradictionError> {
    match classify_interval_relation(predicted, observed).map_err(map_temporal)? {
        AllenRelation::Before | AllenRelation::After => Ok(true),
        AllenRelation::Meets
        | AllenRelation::MetBy
        | AllenRelation::Overlaps
        | AllenRelation::OverlappedBy
        | AllenRelation::Starts
        | AllenRelation::StartedBy
        | AllenRelation::During
        | AllenRelation::Contains
        | AllenRelation::Finishes
        | AllenRelation::FinishedBy
        | AllenRelation::Equals => Ok(false),
    }
}

/// Refuse only Allen contradiction or adjacency; this is not promotion authority.
///
/// Success means the pair is not Allen `before` / `after` and is not merely
/// adjacent. Partial overlap still leaves unmatched predicted mass. Call
/// [`refuse_promotion`] or [`require_observed_coverage`] before recording a
/// forecast as observed fact.
///
/// This function classifies intervals with
/// [`temporal_core::classify_interval_relation`]. It does not run the
/// path-consistency reasoner.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::EvidenceAfterCutoff`] when
/// `observed_available` is later than `cutoff`. Returns
/// [`PredictionContradictionError::PredictionContradictsObservation`] for
/// Allen `before` / `after`. Returns
/// [`PredictionContradictionError::PredictionLacksOverlappingSupport`] for
/// `meets` / `met_by`. Returns
/// [`PredictionContradictionError::InvalidIntervalPayload`] when either
/// interval is not a closed proper Allen input.
pub fn refuse_contradiction_or_adjacency(
    predicted: &TemporalInterval<EventTime>,
    observed: &TemporalInterval<EventTime>,
    observed_available: AvailableTime,
    cutoff: KnowledgeCutoff,
) -> Result<(), PredictionContradictionError> {
    if observed_available.instant() > cutoff.instant() {
        return Err(PredictionContradictionError::EvidenceAfterCutoff);
    }
    match classify_interval_relation(predicted, observed).map_err(map_temporal)? {
        AllenRelation::Before | AllenRelation::After => {
            Err(PredictionContradictionError::PredictionContradictsObservation)
        }
        AllenRelation::Meets | AllenRelation::MetBy => {
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        }
        AllenRelation::Overlaps
        | AllenRelation::OverlappedBy
        | AllenRelation::Starts
        | AllenRelation::StartedBy
        | AllenRelation::During
        | AllenRelation::Contains
        | AllenRelation::Finishes
        | AllenRelation::FinishedBy
        | AllenRelation::Equals => Ok(()),
    }
}

/// Refuse promotion unless later-observed evidence covers the prediction.
///
/// This is the promotion-authority entry point. It is identical to
/// [`require_observed_coverage`]: unmatched predicted mass stays hypothetical.
///
/// # Errors
///
/// Returns the same errors as [`require_observed_coverage`].
pub fn refuse_promotion(
    predicted: &TemporalInterval<EventTime>,
    observed: &TemporalInterval<EventTime>,
    observed_available: AvailableTime,
    cutoff: KnowledgeCutoff,
) -> Result<(), PredictionContradictionError> {
    require_observed_coverage(predicted, observed, observed_available, cutoff)
}

/// Refuse promotion unless later-observed evidence covers the prediction.
///
/// Coverage requires Allen `during`, `starts`, `finishes`, or `equals`.
/// Partial overlap leaves unmatched predicted mass and stays hypothetical.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::EvidenceAfterCutoff`] when
/// `observed_available` is later than `cutoff`. Returns
/// [`PredictionContradictionError::PredictionContradictsObservation`] for
/// Allen `before` / `after`. Returns
/// [`PredictionContradictionError::PredictionLacksOverlappingSupport`] for
/// `meets` / `met_by`. Returns
/// [`PredictionContradictionError::PredictionNotCoveredByObservation`] for
/// partial overlap. Returns
/// [`PredictionContradictionError::InvalidIntervalPayload`] when either
/// interval is not a closed proper Allen input.
pub fn require_observed_coverage(
    predicted: &TemporalInterval<EventTime>,
    observed: &TemporalInterval<EventTime>,
    observed_available: AvailableTime,
    cutoff: KnowledgeCutoff,
) -> Result<(), PredictionContradictionError> {
    if observed_available.instant() > cutoff.instant() {
        return Err(PredictionContradictionError::EvidenceAfterCutoff);
    }
    match classify_promotion_support(predicted, observed)? {
        PromotionSupport::ObservedCoversPrediction => Ok(()),
        PromotionSupport::PartialOverlap => {
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        }
        PromotionSupport::AdjacentWithoutOverlap => {
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        }
        PromotionSupport::ContradictoryDisjoint => {
            Err(PredictionContradictionError::PredictionContradictsObservation)
        }
    }
}

/// Fraction of contradiction flags that match independently supplied labels.
///
/// This is a label-agreement helper for the promotion gate. It is not RMSE,
/// bias, or interval-coverage recovery against a generative truth process.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::AgreementSliceMismatch`] when
/// either slice is empty or the lengths differ.
pub fn contradiction_agreement_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, PredictionContradictionError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PredictionContradictionError::AgreementSliceMismatch);
    }
    let mut matches = 0_u32;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let rate = f64::from(matches) / truth.len() as f64;
    Ok(rate)
}

#[cfg(test)]
mod tests {
    use super::{
        PromotionSupport, classify_promotion_support, contradiction_agreement_rate,
        intervals_contradict, refuse_contradiction_or_adjacency, refuse_promotion,
        require_observed_coverage,
    };
    use crate::PredictionContradictionError;
    use temporal_core::{
        AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
        TemporalPrecision,
    };

    fn event_at(second: u8) -> EventTime {
        EventTime::parse_rfc3339(&format!("2026-01-01T00:00:{second:02}Z")).expect("event time")
    }

    fn closed(start: u8, end: u8) -> TemporalInterval<EventTime> {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_at(start)),
            TemporalBoundary::Included(event_at(end)),
            TemporalPrecision::Second,
        )
        .expect("closed interval")
    }

    fn clocks() -> (AvailableTime, KnowledgeCutoff) {
        (
            AvailableTime::parse_rfc3339("2026-01-02T00:00:00Z").expect("available"),
            KnowledgeCutoff::parse_rfc3339("2026-01-03T00:00:00Z").expect("cutoff"),
        )
    }

    #[test]
    fn intervals_contradict_only_before_and_after() {
        let predicted = closed(0, 10);
        assert!(intervals_contradict(&predicted, &closed(20, 30)).expect("before"));
        assert!(intervals_contradict(&closed(40, 50), &predicted).expect("after"));
        assert!(!intervals_contradict(&predicted, &closed(10, 20)).expect("meets"));
        assert!(!intervals_contradict(&closed(10, 20), &predicted).expect("met_by"));
        assert!(!intervals_contradict(&predicted, &closed(5, 15)).expect("overlaps"));
        assert!(!intervals_contradict(&closed(5, 15), &predicted).expect("overlapped_by"));
        assert!(!intervals_contradict(&predicted, &closed(0, 8)).expect("started_by"));
        assert!(!intervals_contradict(&closed(0, 8), &predicted).expect("starts"));
        assert!(!intervals_contradict(&predicted, &closed(2, 8)).expect("contains"));
        assert!(!intervals_contradict(&closed(2, 8), &predicted).expect("during"));
        assert!(!intervals_contradict(&predicted, &closed(2, 10)).expect("finished_by"));
        assert!(!intervals_contradict(&closed(2, 10), &predicted).expect("finishes"));
        assert!(!intervals_contradict(&predicted, &closed(0, 10)).expect("equals"));
    }

    #[test]
    fn refuse_contradiction_or_adjacency_accepts_overlap_family_and_refuses_gaps() {
        let (available, cutoff) = clocks();
        let predicted = closed(0, 10);
        refuse_contradiction_or_adjacency(&predicted, &closed(5, 15), available, cutoff)
            .expect("overlap");
        refuse_contradiction_or_adjacency(&closed(5, 15), &predicted, available, cutoff)
            .expect("overlapped_by");
        refuse_contradiction_or_adjacency(&predicted, &closed(0, 8), available, cutoff)
            .expect("started_by");
        refuse_contradiction_or_adjacency(&closed(0, 8), &predicted, available, cutoff)
            .expect("starts");
        refuse_contradiction_or_adjacency(&predicted, &closed(2, 8), available, cutoff)
            .expect("contains");
        refuse_contradiction_or_adjacency(&closed(2, 8), &predicted, available, cutoff)
            .expect("during");
        refuse_contradiction_or_adjacency(&predicted, &closed(2, 10), available, cutoff)
            .expect("finished_by");
        refuse_contradiction_or_adjacency(&closed(2, 10), &predicted, available, cutoff)
            .expect("finishes");
        refuse_contradiction_or_adjacency(&predicted, &closed(0, 10), available, cutoff)
            .expect("equals");
        assert_eq!(
            refuse_contradiction_or_adjacency(&predicted, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        assert_eq!(
            refuse_contradiction_or_adjacency(&closed(40, 50), &predicted, available, cutoff),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        assert_eq!(
            refuse_contradiction_or_adjacency(&predicted, &closed(10, 20), available, cutoff),
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        );
        assert_eq!(
            refuse_contradiction_or_adjacency(&closed(10, 20), &predicted, available, cutoff),
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        );
    }

    #[test]
    fn classify_promotion_support_labels_all_thirteen_relations() {
        let predicted = closed(0, 10);
        assert_eq!(
            classify_promotion_support(&predicted, &closed(20, 30)).expect("before"),
            PromotionSupport::ContradictoryDisjoint
        );
        assert_eq!(
            classify_promotion_support(&closed(40, 50), &predicted).expect("after"),
            PromotionSupport::ContradictoryDisjoint
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(10, 20)).expect("meets"),
            PromotionSupport::AdjacentWithoutOverlap
        );
        assert_eq!(
            classify_promotion_support(&closed(10, 20), &predicted).expect("met_by"),
            PromotionSupport::AdjacentWithoutOverlap
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(5, 15)).expect("overlaps"),
            PromotionSupport::PartialOverlap
        );
        assert_eq!(
            classify_promotion_support(&closed(5, 15), &predicted).expect("overlapped_by"),
            PromotionSupport::PartialOverlap
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(0, 8)).expect("started_by"),
            PromotionSupport::PartialOverlap
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(2, 8)).expect("contains"),
            PromotionSupport::PartialOverlap
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(2, 10)).expect("finished_by"),
            PromotionSupport::PartialOverlap
        );
        assert_eq!(
            classify_promotion_support(&closed(0, 8), &predicted).expect("starts"),
            PromotionSupport::ObservedCoversPrediction
        );
        assert_eq!(
            classify_promotion_support(&closed(2, 8), &predicted).expect("during"),
            PromotionSupport::ObservedCoversPrediction
        );
        assert_eq!(
            classify_promotion_support(&closed(2, 10), &predicted).expect("finishes"),
            PromotionSupport::ObservedCoversPrediction
        );
        assert_eq!(
            classify_promotion_support(&predicted, &closed(0, 10)).expect("equals"),
            PromotionSupport::ObservedCoversPrediction
        );
    }

    #[test]
    fn refuse_promotion_matches_require_observed_coverage() {
        let (available, cutoff) = clocks();
        let predicted = closed(0, 10);
        refuse_promotion(&closed(0, 8), &predicted, available, cutoff).expect("starts");
        refuse_promotion(&predicted, &closed(0, 10), available, cutoff).expect("equals");
        assert_eq!(
            refuse_promotion(&predicted, &closed(5, 15), available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            refuse_promotion(&predicted, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        assert_eq!(
            refuse_promotion(&predicted, &closed(10, 20), available, cutoff),
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        );
    }

    #[test]
    fn require_observed_coverage_accepts_only_full_coverage() {
        let (available, cutoff) = clocks();
        let predicted = closed(0, 10);
        require_observed_coverage(&closed(0, 8), &predicted, available, cutoff).expect("starts");
        require_observed_coverage(&closed(2, 8), &predicted, available, cutoff).expect("during");
        require_observed_coverage(&closed(2, 10), &predicted, available, cutoff).expect("finishes");
        require_observed_coverage(&predicted, &closed(0, 10), available, cutoff).expect("equals");
        assert_eq!(
            require_observed_coverage(&predicted, &closed(5, 15), available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            require_observed_coverage(&closed(5, 15), &predicted, available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(0, 8), available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(2, 8), available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(2, 10), available, cutoff),
            Err(PredictionContradictionError::PredictionNotCoveredByObservation)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(10, 20), available, cutoff),
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        );
    }

    #[test]
    fn cutoff_and_half_open_payloads_fail_closed() {
        let (available, cutoff) = clocks();
        let predicted = closed(0, 10);
        let on_cutoff = (
            AvailableTime::parse_rfc3339("2026-01-03T00:00:00Z").expect("on cutoff"),
            KnowledgeCutoff::parse_rfc3339("2026-01-03T00:00:00Z").expect("cutoff"),
        );
        require_observed_coverage(&predicted, &closed(0, 10), on_cutoff.0, on_cutoff.1)
            .expect("available == cutoff");
        refuse_promotion(&predicted, &closed(0, 10), on_cutoff.0, on_cutoff.1)
            .expect("available == cutoff on promotion");
        let late = AvailableTime::parse_rfc3339("2026-01-04T00:00:00Z").expect("late");
        assert_eq!(
            refuse_contradiction_or_adjacency(&predicted, &closed(5, 15), late, cutoff),
            Err(PredictionContradictionError::EvidenceAfterCutoff)
        );
        assert_eq!(
            refuse_promotion(&predicted, &closed(5, 15), late, cutoff),
            Err(PredictionContradictionError::EvidenceAfterCutoff)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &closed(0, 10), late, cutoff),
            Err(PredictionContradictionError::EvidenceAfterCutoff)
        );
        let half_open = TemporalInterval::bounded(
            TemporalBoundary::Included(event_at(0)),
            TemporalBoundary::Excluded(event_at(10)),
            TemporalPrecision::Second,
        )
        .expect("half-open");
        assert_eq!(
            intervals_contradict(&half_open, &closed(20, 30)),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        assert_eq!(
            refuse_contradiction_or_adjacency(&half_open, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        assert_eq!(
            refuse_promotion(&half_open, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        assert_eq!(
            classify_promotion_support(&predicted, &half_open),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        assert_eq!(
            require_observed_coverage(&predicted, &half_open, available, cutoff),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
    }

    #[test]
    fn agreement_rate_matches_or_fails_closed() {
        let matched = contradiction_agreement_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            contradiction_agreement_rate(&[], &[]),
            Err(PredictionContradictionError::AgreementSliceMismatch)
        );
        assert_eq!(
            contradiction_agreement_rate(&[true], &[]),
            Err(PredictionContradictionError::AgreementSliceMismatch)
        );
    }
}
