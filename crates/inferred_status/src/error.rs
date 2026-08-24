//! Fail-closed inferred-status errors.

use std::fmt;

/// A fail-closed inferred-status error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InferredStatusError {
    /// An inferred relation was treated as observed evidence.
    InferredIsNotObserved,
    /// An inferred relation was treated as a state transition.
    InferredIsNotTransition,
    /// A recovery slice was empty or length-mismatched.
    InvalidStatusPayload,
}

impl fmt::Display for InferredStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InferredIsNotObserved => "inferred relation is not observed evidence",
            Self::InferredIsNotTransition => "inferred relation is not a state transition",
            Self::InvalidStatusPayload => "invalid inferred-status payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InferredStatusError {}

#[cfg(test)]
mod tests {
    use super::InferredStatusError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                InferredStatusError::InferredIsNotObserved,
                "inferred relation is not observed evidence",
            ),
            (
                InferredStatusError::InferredIsNotTransition,
                "inferred relation is not a state transition",
            ),
            (
                InferredStatusError::InvalidStatusPayload,
                "invalid inferred-status payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
