//! Fail-closed corpus-background errors.

use std::fmt;

/// A fail-closed corpus-background error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CorpusBackgroundError {
    /// Corpus-background wording was treated as unique latent content.
    CorpusBackgroundIsNotUniqueContent,
    /// Corpus-background wording was treated as stopword deletion.
    CorpusBackgroundIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidCorpusBackgroundPayload,
}

impl fmt::Display for CorpusBackgroundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CorpusBackgroundIsNotUniqueContent => {
                "corpus-background wording is not unique latent content"
            }
            Self::CorpusBackgroundIsNotStopwordDeletion => {
                "corpus-background wording is not stopword deletion"
            }
            Self::InvalidCorpusBackgroundPayload => "invalid corpus-background payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CorpusBackgroundError {}

#[cfg(test)]
mod tests {
    use super::CorpusBackgroundError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent,
                "corpus-background wording is not unique latent content",
            ),
            (
                CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion,
                "corpus-background wording is not stopword deletion",
            ),
            (
                CorpusBackgroundError::InvalidCorpusBackgroundPayload,
                "invalid corpus-background payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
