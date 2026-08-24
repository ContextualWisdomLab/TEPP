//! Fail-closed summarizes-edge errors.

use std::fmt;

/// A fail-closed summarizes-edge error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SummarizesEdgeError {
    /// A summary was treated as a state transition.
    SummaryIsNotTransition,
    /// A summary was treated as the source document identity.
    SummaryIsNotSourceIdentity,
    /// A recovery slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for SummarizesEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SummaryIsNotTransition => "a summary is not a state transition",
            Self::SummaryIsNotSourceIdentity => "a summary is not the source document identity",
            Self::InvalidEdgePayload => "invalid summarizes-edge payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SummarizesEdgeError {}

#[cfg(test)]
mod tests {
    use super::SummarizesEdgeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SummarizesEdgeError::SummaryIsNotTransition,
                "a summary is not a state transition",
            ),
            (
                SummarizesEdgeError::SummaryIsNotSourceIdentity,
                "a summary is not the source document identity",
            ),
            (
                SummarizesEdgeError::InvalidEdgePayload,
                "invalid summarizes-edge payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
