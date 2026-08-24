//! Fail-closed revision-order errors.

use std::fmt;

/// A fail-closed revision-order error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RevisionOrderError {
    /// A later revision did not have a later system time.
    SystemTimeDidNotIncrease,
    /// A revision number or recovery slice was empty, zero, or mismatched.
    InvalidRevisionPayload,
}

impl fmt::Display for RevisionOrderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SystemTimeDidNotIncrease => {
                "later document revisions must have later system time"
            }
            Self::InvalidRevisionPayload => "invalid revision-order payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RevisionOrderError {}

#[cfg(test)]
mod tests {
    use super::RevisionOrderError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                RevisionOrderError::SystemTimeDidNotIncrease,
                "later document revisions must have later system time",
            ),
            (
                RevisionOrderError::InvalidRevisionPayload,
                "invalid revision-order payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
