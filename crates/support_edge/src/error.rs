//! Fail-closed support-edge errors.

use std::fmt;

/// A fail-closed support-edge error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SupportEdgeError {
    /// An evidential or inverse-production edge was treated as a state transition.
    EvidenceIsNotTransition,
    /// A kind slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for SupportEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EvidenceIsNotTransition => {
                "support, contradiction, summary, and outcome_of edges are not state transitions"
            }
            Self::InvalidEdgePayload => "invalid support-edge payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SupportEdgeError {}

#[cfg(test)]
mod tests {
    use super::SupportEdgeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SupportEdgeError::EvidenceIsNotTransition,
                "support, contradiction, summary, and outcome_of edges are not state transitions",
            ),
            (
                SupportEdgeError::InvalidEdgePayload,
                "invalid support-edge payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
