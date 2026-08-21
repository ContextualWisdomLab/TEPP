//! Fail-closed validation metric errors.

use std::fmt;

/// A fail-closed validation-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Empty, unequal-length, or non-finite input vectors.
    InvalidInput,
    /// Acceptance thresholds or Monte Carlo settings were inconsistent.
    InvalidConfiguration,
    /// Candidate and protected heads are not the same exact commit.
    ClaimHeadMismatch,
    /// A required exact-head gate is absent or failed.
    ClaimEvidenceMissing,
    /// A queued or in-progress check was treated as passing evidence.
    ClaimQueuedEvidence,
    /// Predecessor-head or stale evidence was treated as current-head proof.
    ClaimPredecessorHead,
    /// An LLM judgment was treated as scientific or implementation authority.
    ClaimLlmJudgment,
    /// A skipped required test was treated as passing evidence.
    ClaimSkippedRequired,
    /// Computed recovery did not fall within the configured SE gate.
    ClaimRecoveryRejected,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidInput => "invalid validation input",
            Self::InvalidConfiguration => "invalid validation configuration",
            Self::ClaimHeadMismatch => "claim candidate head is not the protected head",
            Self::ClaimEvidenceMissing => "required claim evidence is missing",
            Self::ClaimQueuedEvidence => "queued checks cannot promote a claim",
            Self::ClaimPredecessorHead => "predecessor-head evidence cannot promote a claim",
            Self::ClaimLlmJudgment => "llm judgment cannot promote a claim",
            Self::ClaimSkippedRequired => "skipped required tests cannot promote a claim",
            Self::ClaimRecoveryRejected => "computed recovery does not support the claim",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::ValidationError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            ValidationError::InvalidInput.to_string(),
            "invalid validation input"
        );
        assert_eq!(
            ValidationError::InvalidConfiguration.to_string(),
            "invalid validation configuration"
        );
        assert_eq!(
            ValidationError::ClaimHeadMismatch.to_string(),
            "claim candidate head is not the protected head"
        );
        assert_eq!(
            ValidationError::ClaimEvidenceMissing.to_string(),
            "required claim evidence is missing"
        );
        assert_eq!(
            ValidationError::ClaimQueuedEvidence.to_string(),
            "queued checks cannot promote a claim"
        );
        assert_eq!(
            ValidationError::ClaimPredecessorHead.to_string(),
            "predecessor-head evidence cannot promote a claim"
        );
        assert_eq!(
            ValidationError::ClaimLlmJudgment.to_string(),
            "llm judgment cannot promote a claim"
        );
        assert_eq!(
            ValidationError::ClaimSkippedRequired.to_string(),
            "skipped required tests cannot promote a claim"
        );
        assert_eq!(
            ValidationError::ClaimRecoveryRejected.to_string(),
            "computed recovery does not support the claim"
        );
    }
}
