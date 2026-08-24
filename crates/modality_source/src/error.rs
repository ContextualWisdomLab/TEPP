//! Fail-closed modality-source errors.

use std::fmt;

/// A fail-closed modality-source error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ModalitySourceError {
    /// Non-lexical modality was treated as unique latent content.
    ModalityIsNotUniqueContent,
    /// Non-lexical modality was treated as stopword deletion.
    ModalityIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidModalityPayload,
}

impl fmt::Display for ModalitySourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ModalityIsNotUniqueContent => "non-lexical modality is not unique latent content",
            Self::ModalityIsNotStopwordDeletion => "non-lexical modality is not stopword deletion",
            Self::InvalidModalityPayload => "invalid modality-source payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ModalitySourceError {}

#[cfg(test)]
mod tests {
    use super::ModalitySourceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                ModalitySourceError::ModalityIsNotUniqueContent,
                "non-lexical modality is not unique latent content",
            ),
            (
                ModalitySourceError::ModalityIsNotStopwordDeletion,
                "non-lexical modality is not stopword deletion",
            ),
            (
                ModalitySourceError::InvalidModalityPayload,
                "invalid modality-source payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
