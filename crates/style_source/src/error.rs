//! Fail-closed style-source errors.

use std::fmt;

/// A fail-closed style-source error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StyleSourceError {
    /// Style residue was treated as unique latent content.
    StyleIsNotUniqueContent,
    /// Style residue was treated as stopword deletion.
    StyleIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidStylePayload,
}

impl fmt::Display for StyleSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::StyleIsNotUniqueContent => {
                "house-voice style residue is not unique latent content"
            }
            Self::StyleIsNotStopwordDeletion => {
                "house-voice style residue is not stopword deletion"
            }
            Self::InvalidStylePayload => "invalid style-source payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StyleSourceError {}

#[cfg(test)]
mod tests {
    use super::StyleSourceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                StyleSourceError::StyleIsNotUniqueContent,
                "house-voice style residue is not unique latent content",
            ),
            (
                StyleSourceError::StyleIsNotStopwordDeletion,
                "house-voice style residue is not stopword deletion",
            ),
            (
                StyleSourceError::InvalidStylePayload,
                "invalid style-source payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
