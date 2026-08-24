//! Fail-closed copied-text errors.

use std::fmt;

/// A fail-closed copied-text error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CopiedTextError {
    /// Copied-text residue was treated as unique latent content.
    CopiedTextIsNotUniqueContent,
    /// Copied-text residue was treated as stopword deletion.
    CopiedTextIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidCopiedPayload,
}

impl fmt::Display for CopiedTextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CopiedTextIsNotUniqueContent => {
                "copied-text residue is not unique latent content"
            }
            Self::CopiedTextIsNotStopwordDeletion => "copied-text residue is not stopword deletion",
            Self::InvalidCopiedPayload => "invalid copied-text payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CopiedTextError {}

#[cfg(test)]
mod tests {
    use super::CopiedTextError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CopiedTextError::CopiedTextIsNotUniqueContent,
                "copied-text residue is not unique latent content",
            ),
            (
                CopiedTextError::CopiedTextIsNotStopwordDeletion,
                "copied-text residue is not stopword deletion",
            ),
            (
                CopiedTextError::InvalidCopiedPayload,
                "invalid copied-text payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
