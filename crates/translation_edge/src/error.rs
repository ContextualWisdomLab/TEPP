//! Fail-closed translation-edge errors.

use std::fmt;

/// A fail-closed translation-edge error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TranslationEdgeError {
    /// A translation, copy, or revision edge was treated as a state transition.
    TranslationIsNotTransition,
    /// A same-language pair was classified as a translation.
    SameLanguageIsNotTranslation,
    /// A language tag was empty or lacked a primary subtag.
    InvalidLanguageTag,
    /// A kind slice was empty or length-mismatched.
    InvalidEdgePayload,
}

impl fmt::Display for TranslationEdgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TranslationIsNotTransition => {
                "translation, same-language copy, and revision edges are not state transitions"
            }
            Self::SameLanguageIsNotTranslation => "same primary language is not a translation",
            Self::InvalidLanguageTag => "invalid language tag",
            Self::InvalidEdgePayload => "invalid translation-edge payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TranslationEdgeError {}

#[cfg(test)]
mod tests {
    use super::TranslationEdgeError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                TranslationEdgeError::TranslationIsNotTransition,
                "translation, same-language copy, and revision edges are not state transitions",
            ),
            (
                TranslationEdgeError::SameLanguageIsNotTranslation,
                "same primary language is not a translation",
            ),
            (
                TranslationEdgeError::InvalidLanguageTag,
                "invalid language tag",
            ),
            (
                TranslationEdgeError::InvalidEdgePayload,
                "invalid translation-edge payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
