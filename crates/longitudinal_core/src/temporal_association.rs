//! Typed public boundary for event-time lagged association.

use crate::{EventTimeInterval, LongitudinalError, association};

/// Recover a Pearson correlation for a substantive event-time lag.
///
/// The caller supplies the lagged covariance and both occasion-specific
/// marginal variances. [`EventTimeInterval`] makes the public clock ownership
/// explicit; assertion-, document-, system-, availability-, or method-occasion
/// intervals cannot enter this API as bare numeric durations.
///
/// # Errors
///
/// Returns [`LongitudinalError::InvalidTemporalAssociationInput`] for invalid
/// covariance or marginal inputs, or when a nonzero exact correlation is too
/// small to be represented as binary64,
/// [`LongitudinalError::NonPositiveMarginalVariance`] when either marginal
/// variance is non-positive, and [`LongitudinalError::CovarianceBoundViolation`]
/// when the covariance exceeds the exact binary64 Cauchy–Schwarz bound.
pub fn recover_event_time_lagged_correlation(
    lagged_covariance: f64,
    earlier_total_variance: f64,
    later_total_variance: f64,
    event_interval: EventTimeInterval,
) -> Result<f64, LongitudinalError> {
    association::recover_event_time_lagged_correlation(
        lagged_covariance,
        earlier_total_variance,
        later_total_variance,
        event_interval,
    )
}

#[cfg(test)]
mod tests {
    use super::recover_event_time_lagged_correlation;
    use crate::{EventTimeInterval, LongitudinalError};

    #[test]
    fn public_boundary_requires_admitted_event_time() {
        let interval = EventTimeInterval::new(1.0).expect("valid event time");
        assert_eq!(
            recover_event_time_lagged_correlation(2.0, 1.0, 4.0, interval),
            Ok(1.0)
        );
    }

    #[test]
    fn public_boundary_does_not_report_underflowed_nonzero_correlation_as_zero() {
        let interval = EventTimeInterval::new(1.0).expect("valid event time");
        assert_eq!(
            recover_event_time_lagged_correlation(
                f64::from_bits(1),
                f64::MAX,
                f64::MAX,
                interval,
            ),
            Err(LongitudinalError::InvalidTemporalAssociationInput)
        );
    }

    #[test]
    fn wrong_clock_shaped_numeric_values_fail_at_value_object_admission() {
        assert_eq!(
            EventTimeInterval::new(0.0),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
        assert_eq!(
            EventTimeInterval::new(f64::NAN),
            Err(LongitudinalError::NonPositiveEventInterval)
        );
    }
}
