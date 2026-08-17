//! Translation-related provenance kinds that may point to the past.

use crate::TranslationEdgeError;

/// Closed vocabulary of translation-related edges that are not state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationKind {
    /// A translation of an earlier document into a different language.
    Translation,
    /// A same-language template or copied variant.
    SameLanguageCopy,
    /// A same-language revision of an earlier document.
    Revision,
}

impl TranslationKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Translation => "translates",
            Self::SameLanguageCopy => "template_copy_of",
            Self::Revision => "revises",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`TranslationEdgeError::InvalidEdgePayload`] for unrecognized
    /// names, including transition names such as `causes`.
    pub fn from_wire_name(name: &str) -> Result<Self, TranslationEdgeError> {
        match name {
            "translates" => Ok(Self::Translation),
            "template_copy_of" => Ok(Self::SameLanguageCopy),
            "revises" => Ok(Self::Revision),
            _ => Err(TranslationEdgeError::InvalidEdgePayload),
        }
    }

    /// Return whether this kind is a forward state-transition edge.
    ///
    /// Translation-related provenance kinds are never transitions.
    #[must_use]
    pub const fn is_transition_edge(self) -> bool {
        match self {
            Self::Translation | Self::SameLanguageCopy | Self::Revision => false,
        }
    }
}

/// Refuse to treat a translation-related edge as a forward state transition.
///
/// # Errors
///
/// Always returns [`TranslationEdgeError::TranslationIsNotTransition`].
pub fn refuse_translation_as_transition(
    _kind: TranslationKind,
) -> Result<(), TranslationEdgeError> {
    Err(TranslationEdgeError::TranslationIsNotTransition)
}

/// Refuse to classify a same primary-language pair as a translation.
///
/// Primary subtags are compared case-insensitively. `en` and `en-US` share
/// a primary language and therefore cannot be a translation.
///
/// # Errors
///
/// Returns [`TranslationEdgeError::InvalidLanguageTag`] when either tag is
/// empty or lacks a primary subtag, and
/// [`TranslationEdgeError::SameLanguageIsNotTranslation`] when the primary
/// subtags match.
pub fn refuse_same_language_as_translation(
    source_language: &str,
    target_language: &str,
) -> Result<(), TranslationEdgeError> {
    let source = primary_language_subtag(source_language)?;
    let target = primary_language_subtag(target_language)?;
    if source.eq_ignore_ascii_case(target) {
        return Err(TranslationEdgeError::SameLanguageIsNotTranslation);
    }
    Ok(())
}

/// Fraction of recovered provenance kinds that match known truth.
///
/// # Errors
///
/// Returns [`TranslationEdgeError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn edge_kind_recovery_rate(
    truth: &[TranslationKind],
    decided: &[TranslationKind],
) -> Result<f64, TranslationEdgeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(TranslationEdgeError::InvalidEdgePayload);
    }
    let mut matches = 0_u32;
    for (truth_kind, decided_kind) in truth.iter().zip(decided) {
        if truth_kind == decided_kind {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

fn primary_language_subtag(tag: &str) -> Result<&str, TranslationEdgeError> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return Err(TranslationEdgeError::InvalidLanguageTag);
    }
    match trimmed.split_once('-') {
        Some(("", _)) => Err(TranslationEdgeError::InvalidLanguageTag),
        Some((primary, _)) => Ok(primary),
        None => Ok(trimmed),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TranslationKind, edge_kind_recovery_rate, primary_language_subtag,
        refuse_same_language_as_translation, refuse_translation_as_transition,
    };
    use crate::TranslationEdgeError;

    #[test]
    fn local_branches_cover_kinds_languages_and_payloads() {
        for kind in [
            TranslationKind::Translation,
            TranslationKind::SameLanguageCopy,
            TranslationKind::Revision,
        ] {
            assert!(!kind.is_transition_edge());
            assert_eq!(
                TranslationKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
            assert_eq!(
                refuse_translation_as_transition(kind),
                Err(TranslationEdgeError::TranslationIsNotTransition)
            );
        }
        assert_eq!(
            TranslationKind::from_wire_name("causes"),
            Err(TranslationEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            refuse_same_language_as_translation("en-US", "EN"),
            Err(TranslationEdgeError::SameLanguageIsNotTranslation)
        );
        refuse_same_language_as_translation("en", "fr").expect("cross-language");
        assert_eq!(
            primary_language_subtag(""),
            Err(TranslationEdgeError::InvalidLanguageTag)
        );
        assert_eq!(
            primary_language_subtag("-"),
            Err(TranslationEdgeError::InvalidLanguageTag)
        );
        let truth = [TranslationKind::Translation, TranslationKind::Revision];
        let matched = edge_kind_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        let partial = edge_kind_recovery_rate(
            &truth,
            &[TranslationKind::Translation, TranslationKind::Translation],
        )
        .expect("partial");
        assert!((partial - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            edge_kind_recovery_rate(&[], &[]),
            Err(TranslationEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            edge_kind_recovery_rate(&truth, &[]),
            Err(TranslationEdgeError::InvalidEdgePayload)
        );
    }
}
