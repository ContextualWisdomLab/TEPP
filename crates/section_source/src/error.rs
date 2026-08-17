//! Fail-closed section-source errors.

use std::fmt;

/// A fail-closed section-source error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SectionSourceError {
    /// Section boilerplate was treated as unique latent content.
    SectionIsNotUniqueContent,
    /// Section boilerplate was treated as stopword deletion.
    SectionIsNotStopwordDeletion,
    /// A recovery slice was empty or length-mismatched.
    InvalidSectionPayload,
}

impl fmt::Display for SectionSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SectionIsNotUniqueContent => "section boilerplate is not unique latent content",
            Self::SectionIsNotStopwordDeletion => "section boilerplate is not stopword deletion",
            Self::InvalidSectionPayload => "invalid section-source payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SectionSourceError {}

#[cfg(test)]
mod tests {
    use super::SectionSourceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                SectionSourceError::SectionIsNotUniqueContent,
                "section boilerplate is not unique latent content",
            ),
            (
                SectionSourceError::SectionIsNotStopwordDeletion,
                "section boilerplate is not stopword deletion",
            ),
            (
                SectionSourceError::InvalidSectionPayload,
                "invalid section-source payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
