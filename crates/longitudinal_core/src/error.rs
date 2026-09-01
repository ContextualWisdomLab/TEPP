//! Fail-closed longitudinal modeling errors.

use std::fmt;

/// A fail-closed longitudinal-modeling error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LongitudinalError {
    /// A between-unit component was treated as within-unit change.
    BetweenIsNotWithinChange,
    /// An unknown component-level wire name was supplied.
    UnknownComponentLevel,
    /// Component slices were empty, length-mismatched, or non-finite.
    InvalidComponentPayload,
    /// Observations were empty, sparse, duplicated, or non-finite.
    InvalidObservationPayload,
    /// A lagged covariance, marginal variance, or event interval was non-finite.
    InvalidTemporalAssociationInput,
    /// At least one marginal variance was not strictly positive.
    NonPositiveMarginalVariance,
    /// The event-time interval was not strictly positive.
    NonPositiveEventInterval,
    /// The covariance violates the Cauchy-Schwarz bound implied by the two marginals.
    CovarianceBoundViolation,
}

impl fmt::Display for LongitudinalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::BetweenIsNotWithinChange => "between component is not within-unit change",
            Self::UnknownComponentLevel => "unknown component level",
            Self::InvalidComponentPayload => "invalid longitudinal component payload",
            Self::InvalidObservationPayload => "invalid longitudinal observation payload",
            Self::InvalidTemporalAssociationInput => "invalid temporal association input",
            Self::NonPositiveMarginalVariance => {
                "temporal correlation requires strictly positive marginal variances"
            }
            Self::NonPositiveEventInterval => {
                "temporal correlation requires a strictly positive event-time interval"
            }
            Self::CovarianceBoundViolation => {
                "lagged covariance is incompatible with the supplied marginal variances"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for LongitudinalError {}

#[cfg(test)]
mod tests {
    use super::LongitudinalError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                LongitudinalError::BetweenIsNotWithinChange,
                "between component is not within-unit change",
            ),
            (
                LongitudinalError::UnknownComponentLevel,
                "unknown component level",
            ),
            (
                LongitudinalError::InvalidComponentPayload,
                "invalid longitudinal component payload",
            ),
            (
                LongitudinalError::InvalidObservationPayload,
                "invalid longitudinal observation payload",
            ),
            (
                LongitudinalError::InvalidTemporalAssociationInput,
                "invalid temporal association input",
            ),
            (
                LongitudinalError::NonPositiveMarginalVariance,
                "temporal correlation requires strictly positive marginal variances",
            ),
            (
                LongitudinalError::NonPositiveEventInterval,
                "temporal correlation requires a strictly positive event-time interval",
            ),
            (
                LongitudinalError::CovarianceBoundViolation,
                "lagged covariance is incompatible with the supplied marginal variances",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
