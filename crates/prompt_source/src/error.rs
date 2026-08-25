//! Fail-closed prompt-source errors.

use std::fmt;

/// A fail-closed prompt-source error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PromptSourceError {
    /// Prompt boilerplate was treated as unique latent content.
    PromptIsNotUniqueContent,
    /// Prompt boilerplate was treated as stopword deletion.
    PromptIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidPromptPayload,
}

impl fmt::Display for PromptSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PromptIsNotUniqueContent => "prompt boilerplate is not unique latent content",
            Self::PromptIsNotStopwordDeletion => "prompt boilerplate is not stopword deletion",
            Self::InvalidPromptPayload => "invalid prompt-source payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PromptSourceError {}

#[cfg(test)]
mod tests {
    use super::PromptSourceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PromptSourceError::PromptIsNotUniqueContent,
                "prompt boilerplate is not unique latent content",
            ),
            (
                PromptSourceError::PromptIsNotStopwordDeletion,
                "prompt boilerplate is not stopword deletion",
            ),
            (
                PromptSourceError::InvalidPromptPayload,
                "invalid prompt-source payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
