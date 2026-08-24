//! Fail-closed stopword-deletion errors.

use std::fmt;

/// A fail-closed stopword-deletion error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StopwordDeletionError {
    /// A default or global stopword list was used as deletion.
    DefaultStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidDeletionPayload,
}

impl fmt::Display for StopwordDeletionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DefaultStopwordDeletion => {
                "default stopword deletion is not a valid method for repeated report language"
            }
            Self::InvalidDeletionPayload => "invalid stopword-deletion payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StopwordDeletionError {}

#[cfg(test)]
mod tests {
    use super::StopwordDeletionError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                StopwordDeletionError::DefaultStopwordDeletion,
                "default stopword deletion is not a valid method for repeated report language",
            ),
            (
                StopwordDeletionError::InvalidDeletionPayload,
                "invalid stopword-deletion payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
