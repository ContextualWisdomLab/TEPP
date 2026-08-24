//! Fail-closed citation-edge errors.

use std::fmt;

/// A fail-closed citation-edge error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CitationEdgeError {
    /// A provenance edge was treated as a state transition.
    ProvenanceIsNotTransition,
    /// A kind slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for CitationEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ProvenanceIsNotTransition => {
                "citation or retrospective edges are not state transitions"
            }
            Self::InvalidEdgePayload => "invalid citation-edge payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CitationEdgeError {}

#[cfg(test)]
mod tests {
    use super::CitationEdgeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CitationEdgeError::ProvenanceIsNotTransition,
                "citation or retrospective edges are not state transitions",
            ),
            (
                CitationEdgeError::InvalidEdgePayload,
                "invalid citation-edge payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
