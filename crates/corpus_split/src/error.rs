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
}

impl fmt::Display for CorpusSplitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RelationLeakage => "relation-aware split leakage",
            Self::UnavailableAtCutoff => "document unavailable at knowledge cutoff",
            Self::DuplicateDocumentIdentity => "duplicate document identity",
            Self::InvalidSplitConfiguration => "invalid split configuration",
            Self::EmptyCanonicalText => "empty canonical text",
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
    }
}
