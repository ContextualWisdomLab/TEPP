//! Half-open event-time intervals and contradiction checks.

use crate::PredictionContradictionError;

/// One half-open event-time interval `[start, end)` in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClosedEventInterval {
    start_seconds: i64,
    end_seconds: i64,
}

impl ClosedEventInterval {
    /// Construct a half-open interval with a strictly positive length.
    ///
    /// # Errors
    ///
    /// Returns [`PredictionContradictionError::InvalidIntervalPayload`] when
    /// `end_seconds` is not greater than `start_seconds`.
    pub const fn new(
        start_seconds: i64,
        end_seconds: i64,
    ) -> Result<Self, PredictionContradictionError> {
        if end_seconds <= start_seconds {
            return Err(PredictionContradictionError::InvalidIntervalPayload);
        }
        Ok(Self {
            start_seconds,
            end_seconds,
        })
    }

    /// Inclusive start bound in seconds.
    #[must_use]
    pub const fn start_seconds(self) -> i64 {
        self.start_seconds
    }

    /// Exclusive end bound in seconds.
    #[must_use]
    pub const fn end_seconds(self) -> i64 {
        self.end_seconds
    }
}

/// Return whether two half-open intervals are disjoint.
///
/// # Errors
///
/// This function is infallible for validated intervals and exists to keep the
/// public comparison surface explicit.
#[allow(clippy::unnecessary_wraps)]
pub fn intervals_contradict(
    predicted: ClosedEventInterval,
    observed: ClosedEventInterval,
) -> Result<bool, PredictionContradictionError> {
    Ok(predicted.end_seconds <= observed.start_seconds
        || observed.end_seconds <= predicted.start_seconds)
}

/// Refuse to promote a contradicting prediction to observed fact.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::PredictionContradictsObservation`]
/// when the intervals are disjoint.
pub fn refuse_promotion_when_contradict(
    predicted: ClosedEventInterval,
    observed: ClosedEventInterval,
) -> Result<(), PredictionContradictionError> {
    if intervals_contradict(predicted, observed)? {
        return Err(PredictionContradictionError::PredictionContradictsObservation);
    }
    Ok(())
}

/// Fraction of recovered contradiction flags that match known truth.
///
/// # Errors
///
/// Returns [`PredictionContradictionError::InvalidIntervalPayload`] when
/// either slice is empty or the lengths differ.
pub fn contradiction_recovery_rate(
    truth: &[bool],
    decided: &[bool],
) -> Result<f64, PredictionContradictionError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PredictionContradictionError::InvalidIntervalPayload);
    }
    let mut matches = 0_u32;
    for (truth_flag, decided_flag) in truth.iter().zip(decided) {
        if truth_flag == decided_flag {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        ClosedEventInterval, contradiction_recovery_rate, intervals_contradict,
        refuse_promotion_when_contradict,
    };
    use crate::PredictionContradictionError;

    #[test]
    fn local_branches_cover_overlap_disjoint_and_payloads() {
        let predicted = ClosedEventInterval::new(0, 10).expect("predicted");
        let observed = ClosedEventInterval::new(20, 30).expect("observed");
        assert_eq!(predicted.start_seconds(), 0);
        assert_eq!(predicted.end_seconds(), 10);
        assert!(intervals_contradict(predicted, observed).expect("disjoint"));
        assert_eq!(
            refuse_promotion_when_contradict(predicted, observed),
            Err(PredictionContradictionError::PredictionContradictsObservation)
        );
        let overlap = ClosedEventInterval::new(5, 15).expect("overlap");
        refuse_promotion_when_contradict(predicted, overlap).expect("consistent");
        assert!(!intervals_contradict(predicted, overlap).expect("overlap"));
        assert_eq!(
            ClosedEventInterval::new(4, 4),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        let matched = contradiction_recovery_rate(&[true], &[true]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            contradiction_recovery_rate(&[], &[]),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
        assert_eq!(
            contradiction_recovery_rate(&[true], &[]),
            Err(PredictionContradictionError::InvalidIntervalPayload)
        );
    }
}
