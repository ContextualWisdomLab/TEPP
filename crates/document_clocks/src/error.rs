//! Fail-closed document-clock errors.

use std::fmt;

/// A fail-closed document-clock error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DocumentClockError {
    /// Assertion time or document time was omitted from a document row.
    OmittedAssertionOrDocumentTime,
    /// A required clock used the wrong family label.
    ClockFamilyMismatch,
    /// A recovery slice was empty or mismatched.
    InvalidClockPayload,
}

impl fmt::Display for DocumentClockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::OmittedAssertionOrDocumentTime => {
                "document rows must carry assertion time and document time"
            }
            Self::ClockFamilyMismatch => "document clock family does not match the named field",
            Self::InvalidClockPayload => "invalid document-clock payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for DocumentClockError {}

#[cfg(test)]
mod tests {
    use super::DocumentClockError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                DocumentClockError::OmittedAssertionOrDocumentTime,
                "document rows must carry assertion time and document time",
            ),
            (
                DocumentClockError::ClockFamilyMismatch,
                "document clock family does not match the named field",
            ),
            (
                DocumentClockError::InvalidClockPayload,
                "invalid document-clock payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
