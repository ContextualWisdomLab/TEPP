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

/// Refuse promotion when evidence is ineligible, disjoint, or only adjacent.
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
pub fn refuse_promotion(
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
    use super::{contradiction_agreement_rate, intervals_contradict, refuse_promotion};
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
    fn local_branches_cover_relations_cutoff_and_agreement() {
        let (available, cutoff) = clocks();
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
        refuse_promotion(&predicted, &closed(5, 15), available, cutoff).expect("overlap");
        refuse_promotion(&predicted, &closed(0, 8), available, cutoff).expect("started_by");
        refuse_promotion(&closed(0, 8), &predicted, available, cutoff).expect("starts");
        refuse_promotion(&predicted, &closed(2, 8), available, cutoff).expect("contains");
        refuse_promotion(&closed(2, 8), &predicted, available, cutoff).expect("during");
        refuse_promotion(&predicted, &closed(2, 10), available, cutoff).expect("finished_by");
        refuse_promotion(&closed(2, 10), &predicted, available, cutoff).expect("finishes");
        refuse_promotion(&predicted, &closed(0, 10), available, cutoff).expect("equals");
        assert_eq!(
            refuse_promotion(&predicted, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        assert_eq!(
            refuse_promotion(&predicted, &closed(10, 20), available, cutoff),
            Err(PredictionContradictionError::PredictionLacksOverlappingSupport)
        );
        let late = AvailableTime::parse_rfc3339("2026-01-04T00:00:00Z").expect("late");
        assert_eq!(
            refuse_promotion(&predicted, &closed(5, 15), late, cutoff),
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
            refuse_promotion(&half_open, &closed(20, 30), available, cutoff),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
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
