//! Fail-closed corpus-split errors.

use std::fmt;

/// A fail-closed corpus-split domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CorpusSplitError {
    /// A partition assignment would separate linked records.
    RelationLeakage,
    /// A document was unavailable at the requested knowledge cutoff.
    UnavailableAtCutoff,
    /// A duplicate document identity was rejected.
    DuplicateDocumentIdentity,
    /// Split proportions or seeds were invalid.
    InvalidSplitConfiguration,
    /// A document body was empty and has no Unicode identity.
    EmptyCanonicalText,
    /// A retrieval ranking score was treated as an inferential estimator weight.
    InferentialRetrievalWeight,
    /// Global stopword deletion was proposed as the default preprocessing rule.
    DefaultStopwordDeletion,
}

impl fmt::Display for CorpusSplitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RelationLeakage => "relation-aware split leakage",
            Self::UnavailableAtCutoff => "document unavailable at knowledge cutoff",
            Self::DuplicateDocumentIdentity => "duplicate document identity",
            Self::InvalidSplitConfiguration => "invalid split configuration",
            Self::EmptyCanonicalText => "empty canonical text",
            Self::InferentialRetrievalWeight => "retrieval score is not an inferential weight",
            Self::DefaultStopwordDeletion => "global stopword deletion is not the default rule",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CorpusSplitError {}

#[cfg(test)]
mod tests {
    use super::CorpusSplitError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            CorpusSplitError::RelationLeakage.to_string(),
            "relation-aware split leakage"
        );
        assert_eq!(
            CorpusSplitError::UnavailableAtCutoff.to_string(),
            "document unavailable at knowledge cutoff"
        );
        assert_eq!(
            CorpusSplitError::DuplicateDocumentIdentity.to_string(),
            "duplicate document identity"
        );
        assert_eq!(
            CorpusSplitError::InvalidSplitConfiguration.to_string(),
            "invalid split configuration"
        );
        assert_eq!(
            CorpusSplitError::EmptyCanonicalText.to_string(),
            "empty canonical text"
        );
        assert_eq!(
            CorpusSplitError::InferentialRetrievalWeight.to_string(),
            "retrieval score is not an inferential weight"
        );
        assert_eq!(
            CorpusSplitError::DefaultStopwordDeletion.to_string(),
            "global stopword deletion is not the default rule"
        );
    }
}
