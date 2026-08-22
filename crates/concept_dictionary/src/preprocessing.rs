//! Refuse stopword deletion and lexical-retrieval inferential weights.

use crate::error::ConceptError;

/// Inferential weight kind offered to the statistical estimator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InferentialWeightKind {
    /// Statistical / posterior weight from the topic or concept model.
    StatisticalPosterior,
    /// Term frequency–inverse document frequency.
    TfIdf,
    /// BM25 lexical retrieval weight.
    Bm25,
}

impl InferentialWeightKind {
    /// Stable wire name for the weight kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StatisticalPosterior => "statistical_posterior",
            Self::TfIdf => "tfidf",
            Self::Bm25 => "bm25",
        }
    }
}

/// Admit only statistical/posterior inferential weights.
///
/// # Errors
///
/// Returns [`ConceptError::InferentialWeightForbidden`] for TF-IDF or BM25.
pub const fn admit_inferential_weight(kind: InferentialWeightKind) -> Result<(), ConceptError> {
    match kind {
        InferentialWeightKind::StatisticalPosterior => Ok(()),
        InferentialWeightKind::TfIdf | InferentialWeightKind::Bm25 => {
            Err(ConceptError::InferentialWeightForbidden)
        }
    }
}

/// Refuse default stopword deletion as a preprocessing rule.
///
/// # Errors
///
/// Always returns [`ConceptError::StopwordDeletionForbidden`].
pub const fn apply_default_stopword_deletion() -> Result<(), ConceptError> {
    Err(ConceptError::StopwordDeletionForbidden)
}

#[cfg(test)]
mod tests {
    use super::{InferentialWeightKind, admit_inferential_weight, apply_default_stopword_deletion};
    use crate::error::ConceptError;

    #[test]
    fn posterior_is_admitted_and_lexical_weights_are_refused() {
        assert_eq!(
            InferentialWeightKind::StatisticalPosterior.as_str(),
            "statistical_posterior"
        );
        assert_eq!(InferentialWeightKind::TfIdf.as_str(), "tfidf");
        assert_eq!(InferentialWeightKind::Bm25.as_str(), "bm25");
        admit_inferential_weight(InferentialWeightKind::StatisticalPosterior).expect("posterior");
        assert_eq!(
            admit_inferential_weight(InferentialWeightKind::TfIdf),
            Err(ConceptError::InferentialWeightForbidden)
        );
        assert_eq!(
            apply_default_stopword_deletion(),
            Err(ConceptError::StopwordDeletionForbidden)
        );
    }
}
