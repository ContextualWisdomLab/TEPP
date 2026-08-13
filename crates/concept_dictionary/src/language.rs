//! Language tags, profile status, and comparative-interpretation gates.

use crate::error::ConceptError;

/// BCP 47 language tag used by a TEPP language profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LanguageTag {
    /// English.
    Eng,
    /// Korean.
    Kor,
    /// Japanese.
    Jpn,
    /// Chinese.
    Zho,
    /// Vietnamese.
    Vie,
    /// Indonesian.
    Ind,
    /// French.
    Fra,
    /// German.
    Deu,
    /// Turkish.
    Tur,
}

impl LanguageTag {
    /// Stable BCP 47 primary language subtag.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eng => "en",
            Self::Kor => "ko",
            Self::Jpn => "ja",
            Self::Zho => "zh",
            Self::Vie => "vi",
            Self::Ind => "id",
            Self::Fra => "fr",
            Self::Deu => "de",
            Self::Tur => "tr",
        }
    }
}

/// Validity status of a language profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProfileStatus {
    /// Alignment, reliability, fairness, and invariance evidence support interpretation.
    Validated,
    /// Calibration evidence exists but full validation is incomplete.
    Calibrated,
    /// The profile may be used operationally without comparative claims.
    Provisional,
    /// The profile cannot support interpretation.
    Unresolved,
}

impl ProfileStatus {
    /// Stable wire name for the profile status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validated => "validated",
            Self::Calibrated => "calibrated",
            Self::Provisional => "provisional",
            Self::Unresolved => "unresolved",
        }
    }

    /// Return whether comparative interpretation is admissible.
    #[must_use]
    pub const fn admits_comparative_interpretation(self) -> bool {
        matches!(self, Self::Validated | Self::Calibrated)
    }
}

/// Language profile with explicit validity status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LanguageProfile {
    tag: LanguageTag,
    status: ProfileStatus,
    architecture_supported: bool,
}

impl LanguageProfile {
    /// Record a language profile. Architecture support is not validation.
    #[must_use]
    pub const fn new(
        tag: LanguageTag,
        status: ProfileStatus,
        architecture_supported: bool,
    ) -> Self {
        Self {
            tag,
            status,
            architecture_supported,
        }
    }

    /// Language tag.
    #[must_use]
    pub const fn tag(self) -> LanguageTag {
        self.tag
    }

    /// Validity status.
    #[must_use]
    pub const fn status(self) -> ProfileStatus {
        self.status
    }

    /// Whether the architecture can accept the language.
    #[must_use]
    pub const fn architecture_supported(self) -> bool {
        self.architecture_supported
    }
}

/// Permit comparative interpretation only for validated or calibrated profiles.
///
/// Architecture support is recorded but is not authority for interpretation
/// (ADR 0004).
///
/// # Errors
///
/// Returns [`ConceptError::ProfileNotValidated`] for provisional or unresolved
/// profiles, including those the architecture can already accept.
pub fn claim_comparative_interpretation(profile: &LanguageProfile) -> Result<(), ConceptError> {
    let _architecture_is_not_validation = profile.architecture_supported();
    if profile.status().admits_comparative_interpretation() {
        Ok(())
    } else {
        Err(ConceptError::ProfileNotValidated)
    }
}

#[cfg(test)]
mod tests {
    use super::{LanguageProfile, LanguageTag, ProfileStatus, claim_comparative_interpretation};
    use crate::error::ConceptError;

    #[test]
    fn tags_and_statuses_have_stable_wire_names() {
        assert_eq!(LanguageTag::Eng.as_str(), "en");
        assert_eq!(LanguageTag::Kor.as_str(), "ko");
        assert_eq!(LanguageTag::Jpn.as_str(), "ja");
        assert_eq!(LanguageTag::Zho.as_str(), "zh");
        assert_eq!(LanguageTag::Vie.as_str(), "vi");
        assert_eq!(LanguageTag::Ind.as_str(), "id");
        assert_eq!(LanguageTag::Fra.as_str(), "fr");
        assert_eq!(LanguageTag::Deu.as_str(), "de");
        assert_eq!(LanguageTag::Tur.as_str(), "tr");
        assert_eq!(ProfileStatus::Validated.as_str(), "validated");
        assert_eq!(ProfileStatus::Calibrated.as_str(), "calibrated");
        assert_eq!(ProfileStatus::Provisional.as_str(), "provisional");
        assert_eq!(ProfileStatus::Unresolved.as_str(), "unresolved");
        let profile = LanguageProfile::new(LanguageTag::Kor, ProfileStatus::Validated, true);
        assert_eq!(profile.tag(), LanguageTag::Kor);
        assert!(profile.architecture_supported());
        claim_comparative_interpretation(&profile).expect("validated");
        assert_eq!(
            claim_comparative_interpretation(&LanguageProfile::new(
                LanguageTag::Eng,
                ProfileStatus::Unresolved,
                false,
            )),
            Err(ConceptError::ProfileNotValidated)
        );
    }
}
