//! Fail-closed payload-semantics errors.

use std::fmt;

/// A fail-closed payload-semantics error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadSemanticsError {
    /// An untrusted payload was treated as estimator or posterior authority.
    UntrustedPayloadIsNotEstimator,
    /// An LLM output was treated as source evidence.
    LlmOutputIsNotEvidence,
    /// A document, metadata, or serialized record was treated as interpretation.
    EvidenceIsNotInterpretation,
    /// Identity, size, or authorization bounds were treated as semantics.
    BoundsAreNotSemantics,
    /// A recovery slice was empty or length-mismatched.
    InvalidSemanticsPayload,
}

impl fmt::Display for PayloadSemanticsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UntrustedPayloadIsNotEstimator => {
                "an untrusted payload is not estimator or posterior authority"
            }
            Self::LlmOutputIsNotEvidence => "an llm output is not source evidence",
            Self::EvidenceIsNotInterpretation => {
                "a document, metadata, or serialized record is not interpretation"
            }
            Self::BoundsAreNotSemantics => {
                "identity, size, and authorization bounds are not scientific semantics"
            }
            Self::InvalidSemanticsPayload => "invalid payload-semantics payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PayloadSemanticsError {}

#[cfg(test)]
mod tests {
    use super::PayloadSemanticsError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PayloadSemanticsError::UntrustedPayloadIsNotEstimator,
                "an untrusted payload is not estimator or posterior authority",
            ),
            (
                PayloadSemanticsError::LlmOutputIsNotEvidence,
                "an llm output is not source evidence",
            ),
            (
                PayloadSemanticsError::EvidenceIsNotInterpretation,
                "a document, metadata, or serialized record is not interpretation",
            ),
            (
                PayloadSemanticsError::BoundsAreNotSemantics,
                "identity, size, and authorization bounds are not scientific semantics",
            ),
            (
                PayloadSemanticsError::InvalidSemanticsPayload,
                "invalid payload-semantics payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
