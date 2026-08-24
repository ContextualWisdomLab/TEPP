//! Fail-closed longitudinal within/between errors.

use std::fmt;

/// A fail-closed longitudinal-decomposition error.
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
}

impl fmt::Display for LongitudinalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::BetweenIsNotWithinChange => "between component is not within-unit change",
            Self::UnknownComponentLevel => "unknown component level",
            Self::InvalidComponentPayload => "invalid longitudinal component payload",
            Self::InvalidObservationPayload => "invalid longitudinal observation payload",
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
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
