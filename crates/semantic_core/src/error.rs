//! Fail-closed semantic-unit validation errors.

use std::fmt;

/// A fail-closed semantic-unit error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SemanticError {
    /// A language tag was offered as the unit identity.
    LanguageIsNotIdentity,
    /// A language tag was empty.
    EmptyLanguageTag,
    /// A language tag was not a primary ISO 639 subtag with optional region.
    InvalidLanguageTag,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::LanguageIsNotIdentity => "language tag is not semantic-unit identity",
            Self::EmptyLanguageTag => "empty language tag",
            Self::InvalidLanguageTag => "invalid language tag",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SemanticError {}

#[cfg(test)]
mod tests {
    use super::SemanticError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SemanticError::LanguageIsNotIdentity,
                "language tag is not semantic-unit identity",
            ),
            (SemanticError::EmptyLanguageTag, "empty language tag"),
            (SemanticError::InvalidLanguageTag, "invalid language tag"),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
