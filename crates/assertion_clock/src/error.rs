//! Fail-closed assertion-clock errors.

use std::fmt;

/// A fail-closed assertion-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AssertionClockError {
    /// Event time was treated as assertion time.
    EventTimeIsNotAssertionTime,
    /// System time was treated as assertion time.
    SystemTimeIsNotAssertionTime,
    /// Document time was treated as assertion time.
    DocumentTimeIsNotAssertionTime,
    /// Availability time was treated as assertion time.
    AvailableTimeIsNotAssertionTime,
    /// A recovery slice was empty or length-mismatched.
    InvalidAssertionPayload,
}

impl fmt::Display for AssertionClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EventTimeIsNotAssertionTime => "event time is not assertion time",
            Self::SystemTimeIsNotAssertionTime => "system time is not assertion time",
            Self::DocumentTimeIsNotAssertionTime => "document time is not assertion time",
            Self::AvailableTimeIsNotAssertionTime => "availability time is not assertion time",
            Self::InvalidAssertionPayload => "invalid assertion-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AssertionClockError {}

#[cfg(test)]
mod tests {
    use super::AssertionClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                AssertionClockError::EventTimeIsNotAssertionTime,
                "event time is not assertion time",
            ),
            (
                AssertionClockError::SystemTimeIsNotAssertionTime,
                "system time is not assertion time",
            ),
            (
                AssertionClockError::DocumentTimeIsNotAssertionTime,
                "document time is not assertion time",
            ),
            (
                AssertionClockError::AvailableTimeIsNotAssertionTime,
                "availability time is not assertion time",
            ),
            (
                AssertionClockError::InvalidAssertionPayload,
                "invalid assertion-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
