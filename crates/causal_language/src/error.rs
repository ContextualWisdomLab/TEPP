//! Fail-closed causal-language errors.

use std::fmt;

/// A fail-closed causal-language error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CausalLanguageError {
    /// Association, precedence, or a document link was treated as causation.
    UnidentifiedIsNotCausal,
    /// A recovery slice was empty or length-mismatched.
    InvalidClaimPayload,
}

impl fmt::Display for CausalLanguageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnidentifiedIsNotCausal => {
                "association, temporal precedence, and document links are not causal language"
            }
            Self::InvalidClaimPayload => "invalid causal-language payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CausalLanguageError {}

#[cfg(test)]
mod tests {
    use super::CausalLanguageError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CausalLanguageError::UnidentifiedIsNotCausal,
                "association, temporal precedence, and document links are not causal language",
            ),
            (
                CausalLanguageError::InvalidClaimPayload,
                "invalid causal-language payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
