//! Fail-closed multilingual concept and language-profile errors.

use std::fmt;

/// A fail-closed concept-dictionary error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConceptError {
    /// Language profile is not validated or calibrated for interpretation.
    ProfileNotValidated,
    /// Machine translation is not measurement equivalence.
    TranslationNotEquivalence,
    /// A semantic unit was offered without an exact source span.
    MissingSourceSpan,
    /// A source span was empty or reversed.
    InvalidSourceSpan,
    /// Unknown meaning was forced into a known concept.
    ForcedConceptAssignment,
    /// TF-IDF or BM25 was offered as an inferential weight.
    InferentialWeightForbidden,
    /// Default stopword deletion was requested.
    StopwordDeletionForbidden,
    /// Empty, unequal-length, or non-finite numeric input.
    InvalidNumericInput,
    /// Cross-language comparison lacked invariance evidence.
    InvarianceRequired,
    /// A language-specific lexical form was treated as concept identity.
    LexicalFormNotConcept,
}

impl fmt::Display for ConceptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ProfileNotValidated => {
                "language profile is not validated for comparative interpretation"
            }
            Self::TranslationNotEquivalence => "machine translation is not measurement equivalence",
            Self::MissingSourceSpan => "semantic units require an exact source span",
            Self::InvalidSourceSpan => "source span is empty or reversed",
            Self::ForcedConceptAssignment => {
                "unknown meaning cannot be forced into a known concept"
            }
            Self::InferentialWeightForbidden => {
                "TF-IDF and BM25 are not inferential estimator weights"
            }
            Self::StopwordDeletionForbidden => "default stopword deletion is forbidden",
            Self::InvalidNumericInput => "invalid concept-coordinate numeric input",
            Self::InvarianceRequired => {
                "cross-language mean comparison requires invariance evidence"
            }
            Self::LexicalFormNotConcept => "language-specific lexical form is not concept identity",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ConceptError {}

#[cfg(test)]
mod tests {
    use super::ConceptError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            ConceptError::ProfileNotValidated.to_string(),
            "language profile is not validated for comparative interpretation"
        );
        assert_eq!(
            ConceptError::TranslationNotEquivalence.to_string(),
            "machine translation is not measurement equivalence"
        );
        assert_eq!(
            ConceptError::MissingSourceSpan.to_string(),
            "semantic units require an exact source span"
        );
        assert_eq!(
            ConceptError::InvalidSourceSpan.to_string(),
            "source span is empty or reversed"
        );
        assert_eq!(
            ConceptError::ForcedConceptAssignment.to_string(),
            "unknown meaning cannot be forced into a known concept"
        );
        assert_eq!(
            ConceptError::InferentialWeightForbidden.to_string(),
            "TF-IDF and BM25 are not inferential estimator weights"
        );
        assert_eq!(
            ConceptError::StopwordDeletionForbidden.to_string(),
            "default stopword deletion is forbidden"
        );
        assert_eq!(
            ConceptError::InvalidNumericInput.to_string(),
            "invalid concept-coordinate numeric input"
        );
        assert_eq!(
            ConceptError::InvarianceRequired.to_string(),
            "cross-language mean comparison requires invariance evidence"
        );
        assert_eq!(
            ConceptError::LexicalFormNotConcept.to_string(),
            "language-specific lexical form is not concept identity"
        );
    }
}
