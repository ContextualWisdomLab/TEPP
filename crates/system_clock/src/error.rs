//! Fail-closed system-clock errors.

use std::fmt;

/// A fail-closed system-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SystemClockError {
    /// Event time was treated as system time.
    EventTimeIsNotSystemTime,
    /// Assertion time was treated as system time.
    AssertionTimeIsNotSystemTime,
    /// Document time was treated as system time.
    DocumentTimeIsNotSystemTime,
    /// Availability time was treated as system time.
    AvailableTimeIsNotSystemTime,
    /// Knowledge-cutoff time was treated as system time.
    CutoffTimeIsNotSystemTime,
    /// A recovery slice was empty or length-mismatched.
    InvalidSystemPayload,
}

impl fmt::Display for SystemClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EventTimeIsNotSystemTime => "event time is not system time",
            Self::AssertionTimeIsNotSystemTime => "assertion time is not system time",
            Self::DocumentTimeIsNotSystemTime => "document time is not system time",
            Self::AvailableTimeIsNotSystemTime => "availability time is not system time",
            Self::CutoffTimeIsNotSystemTime => "knowledge cutoff is not system time",
            Self::InvalidSystemPayload => "invalid system-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SystemClockError {}

#[cfg(test)]
mod tests {
    use super::SystemClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SystemClockError::EventTimeIsNotSystemTime,
                "event time is not system time",
            ),
            (
                SystemClockError::AssertionTimeIsNotSystemTime,
                "assertion time is not system time",
            ),
            (
                SystemClockError::DocumentTimeIsNotSystemTime,
                "document time is not system time",
            ),
            (
                SystemClockError::AvailableTimeIsNotSystemTime,
                "availability time is not system time",
            ),
            (
                SystemClockError::CutoffTimeIsNotSystemTime,
                "knowledge cutoff is not system time",
            ),
            (
                SystemClockError::InvalidSystemPayload,
                "invalid system-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
