//! Language profiles are metadata, not unit identity.

use crate::error::SemanticError;

/// A language profile that may select tailoring without becoming identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LanguageProfile {
    /// No language metadata; span bounds stay the supplied exact coordinates.
    Unresolved,
    /// A canonical primary ISO 639 subtag with optional ISO 3166-1 region.
    Tagged {
        /// Lowercase `language` or `language-region` label.
        tag: String,
    },
}

impl LanguageProfile {
    /// Return the unresolved profile.
    #[must_use]
    pub const fn unresolved() -> Self {
        Self::Unresolved
    }

    /// Parse a primary language subtag with an optional region.
    ///
    /// Accepts a two- or three-letter primary language subtag, optionally
    /// followed by a hyphen and either an ISO 3166-1 alpha-2 region or a
    /// three-digit UN M.49 numeric region (Phillips & Davis, 2009, section
    /// 2.2.4; for example `es-419`). The stored form is lowercase. Empty tags
    /// and any other shape fail closed. The tag never becomes a
    /// [`crate::SemanticIdentity`].
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyLanguageTag`] or
    /// [`SemanticError::InvalidLanguageTag`].
    pub fn parse_bcp47(tag: &str) -> Result<Self, SemanticError> {
        if tag.is_empty() {
            return Err(SemanticError::EmptyLanguageTag);
        }
        let canonical = tag.to_ascii_lowercase();
        if !is_primary_language_tag(&canonical) {
            return Err(SemanticError::InvalidLanguageTag);
        }
        Ok(Self::Tagged { tag: canonical })
    }

    /// Return whether this profile is unresolved.
    #[must_use]
    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved)
    }

    /// Return the stable profile label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Unresolved => "unresolved",
            Self::Tagged { tag } => tag,
        }
    }
}

/// Validate a primary language tag per RFC 5646 sections 2.2.1 and 2.2.4.
///
/// A region may be an ISO 3166-1 alpha-2 code or a UN M.49 three-digit
/// numeric subtag; both are accepted here so regional variants such as
/// Latin-American Spanish resolve instead of failing closed.
fn is_primary_language_tag(tag: &str) -> bool {
    match tag.split_once('-') {
        None => is_letter_run(tag, 2, 3),
        Some((language, region)) => {
            is_letter_run(language, 2, 3)
                && (is_letter_run(region, 2, 2) || is_numeric_region(region))
        }
    }
}

/// Return whether `region` is exactly three ASCII digits.
fn is_numeric_region(region: &str) -> bool {
    region.len() == 3 && region.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_letter_run(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::LanguageProfile;
    use crate::error::SemanticError;

    #[test]
    fn parse_accepts_primary_and_region_and_rejects_noise() {
        let korean = LanguageProfile::parse_bcp47("KO").expect("ko");
        assert_eq!(korean.as_str(), "ko");
        assert!(!korean.is_unresolved());
        let tagged = LanguageProfile::parse_bcp47("en-US").expect("en-us");
        assert_eq!(tagged.as_str(), "en-us");
        let numeric = LanguageProfile::parse_bcp47("es-419").expect("es-419");
        assert_eq!(numeric.as_str(), "es-419");
        assert_eq!(
            LanguageProfile::parse_bcp47("ES-419")
                .expect("upper")
                .as_str(),
            "es-419"
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("yue").expect("yue").as_str(),
            "yue"
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("").unwrap_err(),
            SemanticError::EmptyLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("english").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-US-x-private").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en_us").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e1").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-u1").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-u").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e-us").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-41a").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-41").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-4199").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        let unresolved = LanguageProfile::unresolved();
        assert!(unresolved.is_unresolved());
        assert_eq!(unresolved.as_str(), "unresolved");
    }
}
