//! Fail-closed topic-coordinate errors.

use std::fmt;

/// A fail-closed topic-measurement error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TopicMeasurementError {
    /// Composition is empty, has fewer than two parts, is non-positive,
    /// non-finite, or does not sum to one.
    InvalidComposition,
    /// Log-ratio vector is empty, non-finite, or not representable as a strictly positive `f64` simplex.
    InvalidLogRatioDimension,
    /// TF-IDF, BM25, or keyword scores were offered as inferential coordinates.
    LexicalWeightForbidden,
    /// A sparse matrix violated its compressed-storage contract.
    InvalidSparseMatrix,
    /// A reference-estimator input or configuration violated its scientific contract.
    InvalidModelInput,
    /// The estimator produced a non-finite intermediate and failed closed.
    NonFiniteEstimate,
    /// No seeded initialization converged within the bounded iteration budget.
    DidNotConverge,
}

impl fmt::Display for TopicMeasurementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidComposition => "invalid compositional topic vector",
            Self::InvalidLogRatioDimension => "invalid log-ratio dimension",
            Self::LexicalWeightForbidden => "lexical inferential weights are forbidden",
            Self::InvalidSparseMatrix => "invalid sparse matrix",
            Self::InvalidModelInput => "invalid topic model input",
            Self::NonFiniteEstimate => "non-finite topic estimate",
            Self::DidNotConverge => "topic estimator did not converge",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TopicMeasurementError {}

#[cfg(test)]
mod tests {
    use super::TopicMeasurementError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            TopicMeasurementError::InvalidComposition.to_string(),
            "invalid compositional topic vector"
        );
        assert_eq!(
            TopicMeasurementError::InvalidLogRatioDimension.to_string(),
            "invalid log-ratio dimension"
        );
        assert_eq!(
            TopicMeasurementError::LexicalWeightForbidden.to_string(),
            "lexical inferential weights are forbidden"
        );
        assert_eq!(
            TopicMeasurementError::InvalidSparseMatrix.to_string(),
            "invalid sparse matrix"
        );
        assert_eq!(
            TopicMeasurementError::InvalidModelInput.to_string(),
            "invalid topic model input"
        );
        assert_eq!(
            TopicMeasurementError::NonFiniteEstimate.to_string(),
            "non-finite topic estimate"
        );
        assert_eq!(
            TopicMeasurementError::DidNotConverge.to_string(),
            "topic estimator did not converge"
        );
    }
}
