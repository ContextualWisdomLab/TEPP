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
}

impl fmt::Display for TopicMeasurementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidComposition => "invalid compositional topic vector",
            Self::InvalidLogRatioDimension => "invalid log-ratio dimension",
            Self::LexicalWeightForbidden => "lexical inferential weights are forbidden",
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
    }
}
