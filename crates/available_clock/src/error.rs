//! Fail-closed available-clock errors.

use std::fmt;

/// A fail-closed available-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AvailableClockError {
    /// Event time was treated as availability time.
    EventTimeIsNotAvailableTime,
    /// System time was treated as availability time.
    SystemTimeIsNotAvailableTime,
    /// A recovery slice was empty or length-mismatched.
    InvalidAvailabilityPayload,
}

impl fmt::Display for AvailableClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EventTimeIsNotAvailableTime => "event time is not availability time",
            Self::SystemTimeIsNotAvailableTime => "system time is not availability time",
            Self::InvalidAvailabilityPayload => "invalid available-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AvailableClockError {}

#[cfg(test)]
mod tests {
    use super::AvailableClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                AvailableClockError::EventTimeIsNotAvailableTime,
                "event time is not availability time",
            ),
            (
                AvailableClockError::SystemTimeIsNotAvailableTime,
                "system time is not availability time",
            ),
            (
                AvailableClockError::InvalidAvailabilityPayload,
                "invalid available-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
