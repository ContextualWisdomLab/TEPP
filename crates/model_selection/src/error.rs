//! Fail-closed model-selection errors.

use std::fmt;

/// A fail-closed model-selection error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ModelSelectionError {
    /// Candidate `K` was less than two.
    NonPositiveCandidateK,
    /// A diagnostic was non-finite or otherwise unusable.
    InvalidDiagnostic,
    /// No candidates were supplied.
    EmptyCandidateSet,
    /// An LLM vote was asked to define the numerical optimum.
    LlmVoteIsNotStatisticalAuthority,
    /// TF-IDF, BM25, stopword deletion, or LLM labels were offered as coordinates.
    LexicalWeightForbidden,
    /// Every fitted candidate failed to converge or produced a typed numeric failure.
    NoSuccessfulFit,
    /// Fewer than two successful recovery replications remain after failures.
    InsufficientRecoveryReplications,
    /// A rolling-origin numerical input did not match the admitted split identities.
    PartitionInputMismatch,
    /// More than one predictive candidate represented the same fitted topic dimension.
    DuplicateCandidateK,
    /// Rolling-origin windows were not a contiguous chronological sequence.
    RollingOriginWindowMismatch,
    /// One document identity appeared in more than one evaluation window.
    RepeatedEvaluationDocument,
    /// Rolling-origin windows did not expose the same unique candidate-K set.
    PredictiveCandidateSetMismatch,
}

impl fmt::Display for ModelSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonPositiveCandidateK => "candidate k must be at least two",
            Self::InvalidDiagnostic => "invalid model-selection diagnostic",
            Self::EmptyCandidateSet => "empty model-selection candidate set",
            Self::LlmVoteIsNotStatisticalAuthority => "llm vote is not statistical authority",
            Self::LexicalWeightForbidden => "lexical inferential weights are forbidden",
            Self::NoSuccessfulFit => "no fitted candidate produced a finite diagnostic",
            Self::InsufficientRecoveryReplications => {
                "at least two successful recovery replications are required"
            }
            Self::PartitionInputMismatch => "rolling-origin partition input mismatch",
            Self::DuplicateCandidateK => "duplicate predictive candidate k",
            Self::RollingOriginWindowMismatch => "rolling-origin window sequence mismatch",
            Self::RepeatedEvaluationDocument => "repeated rolling-origin evaluation document",
            Self::PredictiveCandidateSetMismatch => {
                "rolling-origin predictive candidate set mismatch"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ModelSelectionError {}

#[cfg(test)]
mod tests {
    use super::ModelSelectionError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                ModelSelectionError::NonPositiveCandidateK,
                "candidate k must be at least two",
            ),
            (
                ModelSelectionError::InvalidDiagnostic,
                "invalid model-selection diagnostic",
            ),
            (
                ModelSelectionError::EmptyCandidateSet,
                "empty model-selection candidate set",
            ),
            (
                ModelSelectionError::LlmVoteIsNotStatisticalAuthority,
                "llm vote is not statistical authority",
            ),
            (
                ModelSelectionError::LexicalWeightForbidden,
                "lexical inferential weights are forbidden",
            ),
            (
                ModelSelectionError::NoSuccessfulFit,
                "no fitted candidate produced a finite diagnostic",
            ),
            (
                ModelSelectionError::InsufficientRecoveryReplications,
                "at least two successful recovery replications are required",
            ),
            (
                ModelSelectionError::PartitionInputMismatch,
                "rolling-origin partition input mismatch",
            ),
            (
                ModelSelectionError::DuplicateCandidateK,
                "duplicate predictive candidate k",
            ),
            (
                ModelSelectionError::RollingOriginWindowMismatch,
                "rolling-origin window sequence mismatch",
            ),
            (
                ModelSelectionError::RepeatedEvaluationDocument,
                "repeated rolling-origin evaluation document",
            ),
            (
                ModelSelectionError::PredictiveCandidateSetMismatch,
                "rolling-origin predictive candidate set mismatch",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
