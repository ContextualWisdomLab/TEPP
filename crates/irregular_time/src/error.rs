//! Fail-closed irregular-time errors.

use std::fmt;

/// A fail-closed irregular-time error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IrregularTimeError {
    /// Equal system-time spacing was treated as event-time spacing.
    SystemSpacingIsNotEventSpacing,
    /// Event time did not strictly increase.
    NonIncreasingEventTime,
    /// Observation or lag slices were empty or length-mismatched.
    InvalidObservationPayload,
}

impl fmt::Display for IrregularTimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SystemSpacingIsNotEventSpacing => {
                "equal system spacing is not event-time spacing"
            }
            Self::NonIncreasingEventTime => "event time is not strictly increasing",
            Self::InvalidObservationPayload => "invalid irregular-time payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for IrregularTimeError {}

#[cfg(test)]
mod tests {
    use super::IrregularTimeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                IrregularTimeError::SystemSpacingIsNotEventSpacing,
                "equal system spacing is not event-time spacing",
            ),
            (
                IrregularTimeError::NonIncreasingEventTime,
                "event time is not strictly increasing",
            ),
            (
                IrregularTimeError::InvalidObservationPayload,
                "invalid irregular-time payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
