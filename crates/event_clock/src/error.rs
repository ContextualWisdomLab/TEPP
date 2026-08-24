//! Fail-closed event-clock errors.

use std::fmt;

/// A fail-closed event-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EventClockError {
    /// Assertion time was treated as event time.
    AssertionTimeIsNotEventTime,
    /// System time was treated as event time.
    SystemTimeIsNotEventTime,
    /// Document time was treated as event time.
    DocumentTimeIsNotEventTime,
    /// Availability time was treated as event time.
    AvailableTimeIsNotEventTime,
    /// A recovery slice was empty or length-mismatched.
    InvalidEventPayload,
}

impl fmt::Display for EventClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AssertionTimeIsNotEventTime => "assertion time is not event time",
            Self::SystemTimeIsNotEventTime => "system time is not event time",
            Self::DocumentTimeIsNotEventTime => "document time is not event time",
            Self::AvailableTimeIsNotEventTime => "availability time is not event time",
            Self::InvalidEventPayload => "invalid event-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EventClockError {}

#[cfg(test)]
mod tests {
    use super::EventClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                EventClockError::AssertionTimeIsNotEventTime,
                "assertion time is not event time",
            ),
            (
                EventClockError::SystemTimeIsNotEventTime,
                "system time is not event time",
            ),
            (
                EventClockError::DocumentTimeIsNotEventTime,
                "document time is not event time",
            ),
            (
                EventClockError::AvailableTimeIsNotEventTime,
                "availability time is not event time",
            ),
            (
                EventClockError::InvalidEventPayload,
                "invalid event-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
