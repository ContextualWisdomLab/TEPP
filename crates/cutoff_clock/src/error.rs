//! Fail-closed cutoff-clock errors.

use std::fmt;

/// A fail-closed cutoff-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CutoffClockError {
    /// Event time was treated as knowledge cutoff.
    EventTimeIsNotKnowledgeCutoff,
    /// System time was treated as knowledge cutoff.
    SystemTimeIsNotKnowledgeCutoff,
    /// Availability time was treated as knowledge cutoff.
    AvailableTimeIsNotKnowledgeCutoff,
    /// A recovery slice was empty or length-mismatched.
    InvalidCutoffPayload,
}

impl fmt::Display for CutoffClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EventTimeIsNotKnowledgeCutoff => "event time is not knowledge cutoff",
            Self::SystemTimeIsNotKnowledgeCutoff => "system time is not knowledge cutoff",
            Self::AvailableTimeIsNotKnowledgeCutoff => "availability time is not knowledge cutoff",
            Self::InvalidCutoffPayload => "invalid cutoff-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CutoffClockError {}

#[cfg(test)]
mod tests {
    use super::CutoffClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CutoffClockError::EventTimeIsNotKnowledgeCutoff,
                "event time is not knowledge cutoff",
            ),
            (
                CutoffClockError::SystemTimeIsNotKnowledgeCutoff,
                "system time is not knowledge cutoff",
            ),
            (
                CutoffClockError::AvailableTimeIsNotKnowledgeCutoff,
                "availability time is not knowledge cutoff",
            ),
            (
                CutoffClockError::InvalidCutoffPayload,
                "invalid cutoff-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
