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
    /// A temporal transform input or intermediate value was not representable.
    InvalidTemporalTransformInput,
    /// At least one marginal variance was not strictly positive.
    NonPositiveMarginalVariance,
    /// The event-time interval was not strictly positive.
    NonPositiveEventInterval,
    /// The covariance violates the Cauchy-Schwarz bound implied by the two marginals.
    CovarianceBoundViolation,
    /// A stationary within-person variance was requested without stable negative drift.
    StationaryVarianceRequiresStableDrift,
    /// `discreteDRIFTstd` was requested without positive stationary within-person variance.
    StandardisedDriftRequiresPositiveWithinVariance,
    /// Unstandardised `discreteDRIFT` was substituted for `discreteDRIFTstd`.
    UnstandardisedDriftIsNotStandardisedDrift,
    /// A trait-plus-state association was substituted for `discreteDRIFTstd`.
    TraitStateAssociationIsNotStandardisedDrift,
    /// Between-unit trait variance was used as the drift standardisation variance.
    TraitVarianceIsNotDriftStandardisationVariance,
}

impl fmt::Display for LongitudinalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::BetweenIsNotWithinChange => "between component is not within-unit change",
            Self::UnknownComponentLevel => "unknown component level",
            Self::InvalidComponentPayload => "invalid longitudinal component payload",
            Self::InvalidObservationPayload => "invalid longitudinal observation payload",
            Self::InvalidTemporalAssociationInput => "invalid temporal association input",
            Self::InvalidTemporalTransformInput => "invalid temporal transform input",
            Self::NonPositiveMarginalVariance => {
                "temporal correlation requires strictly positive marginal variances"
            }
            Self::NonPositiveEventInterval => {
                "temporal composition requires a strictly positive event-time interval"
            }
            Self::CovarianceBoundViolation => {
                "lagged covariance is incompatible with the supplied marginal variances"
            }
            Self::StationaryVarianceRequiresStableDrift => {
                "stationary within-person variance requires strictly negative drift"
            }
            Self::StandardisedDriftRequiresPositiveWithinVariance => {
                "standardised discrete drift requires positive stationary within-person variance"
            }
            Self::UnstandardisedDriftIsNotStandardisedDrift => {
                "unstandardised discrete drift is not standardised discrete drift"
            }
            Self::TraitStateAssociationIsNotStandardisedDrift => {
                "trait-plus-state association is not standardised discrete drift"
            }
            Self::TraitVarianceIsNotDriftStandardisationVariance => {
                "trait variance is not the drift standardisation variance"
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
                LongitudinalError::InvalidTemporalTransformInput,
                "invalid temporal transform input",
            ),
            (
                LongitudinalError::NonPositiveMarginalVariance,
                "temporal correlation requires strictly positive marginal variances",
            ),
            (
                LongitudinalError::NonPositiveEventInterval,
                "temporal composition requires a strictly positive event-time interval",
            ),
            (
                LongitudinalError::CovarianceBoundViolation,
                "lagged covariance is incompatible with the supplied marginal variances",
            ),
            (
                LongitudinalError::StationaryVarianceRequiresStableDrift,
                "stationary within-person variance requires strictly negative drift",
            ),
            (
                LongitudinalError::StandardisedDriftRequiresPositiveWithinVariance,
                "standardised discrete drift requires positive stationary within-person variance",
            ),
            (
                LongitudinalError::UnstandardisedDriftIsNotStandardisedDrift,
                "unstandardised discrete drift is not standardised discrete drift",
            ),
            (
                LongitudinalError::TraitStateAssociationIsNotStandardisedDrift,
                "trait-plus-state association is not standardised discrete drift",
            ),
            (
                LongitudinalError::TraitVarianceIsNotDriftStandardisationVariance,
                "trait variance is not the drift standardisation variance",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
