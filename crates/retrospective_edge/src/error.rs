//! Fail-closed retrospective-edge errors.

use std::fmt;

/// A fail-closed retrospective-edge error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetrospectiveEdgeError {
    /// A retrospective report was treated as a state transition.
    RetrospectiveIsNotTransition,
    /// A retrospective report was treated as a translation.
    RetrospectiveIsNotTranslation,
    /// A recovery slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for RetrospectiveEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RetrospectiveIsNotTransition => {
                "retrospective reporting is not a state transition"
            }
            Self::RetrospectiveIsNotTranslation => "retrospective reporting is not a translation",
            Self::InvalidEdgePayload => "invalid retrospective-edge payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RetrospectiveEdgeError {}

#[cfg(test)]
mod tests {
    use super::RetrospectiveEdgeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                RetrospectiveEdgeError::RetrospectiveIsNotTransition,
                "retrospective reporting is not a state transition",
            ),
            (
                RetrospectiveEdgeError::RetrospectiveIsNotTranslation,
                "retrospective reporting is not a translation",
            ),
            (
                RetrospectiveEdgeError::InvalidEdgePayload,
                "invalid retrospective-edge payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
