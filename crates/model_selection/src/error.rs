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
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
