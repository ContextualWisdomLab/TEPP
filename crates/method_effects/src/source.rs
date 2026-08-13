//! Explicit method-source vocabulary and recovery.

use crate::MethodEffectsError;

/// Closed method-source vocabulary for report and corpus background structure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodSourceKind {
    /// Repeated report template language.
    Template,
    /// Section heading or boilerplate.
    Section,
    /// Copied or reused text.
    CopiedText,
    /// Style or house-voice residue.
    Style,
    /// Non-lexical modality channel.
    Modality,
    /// Corpus-background prevalence, not document meaning.
    CorpusBackground,
}

impl MethodSourceKind {
    /// Stable wire name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Template => "template",
            Self::Section => "section",
            Self::CopiedText => "copied_text",
            Self::Style => "style",
            Self::Modality => "modality",
            Self::CorpusBackground => "corpus_background",
        }
    }

    /// Parse a stable wire method-source name.
    ///
    /// # Errors
    ///
    /// Returns [`MethodEffectsError::UnknownMethodSource`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, MethodEffectsError> {
        match name {
            "template" => Ok(Self::Template),
            "section" => Ok(Self::Section),
            "copied_text" => Ok(Self::CopiedText),
            "style" => Ok(Self::Style),
            "modality" => Ok(Self::Modality),
            "corpus_background" => Ok(Self::CorpusBackground),
            _ => Err(MethodEffectsError::UnknownMethodSource),
        }
    }
}

/// Explicit refusal to treat a method source as an inferential topic weight.
///
/// # Errors
///
/// Always returns [`MethodEffectsError::MethodSourceIsNotInferentialWeight`].
pub fn refuse_method_source_as_inferential_weight(
    _kind: MethodSourceKind,
) -> Result<(), MethodEffectsError> {
    Err(MethodEffectsError::MethodSourceIsNotInferentialWeight)
}

/// Fraction of recovered method sources that match known truth.
///
/// # Errors
///
/// Returns [`MethodEffectsError::InvalidSourcePayload`] when either slice is
/// empty or the lengths differ.
pub fn source_recovery_rate(
    truth: &[MethodSourceKind],
    decided: &[MethodSourceKind],
) -> Result<f64, MethodEffectsError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(MethodEffectsError::InvalidSourcePayload);
    }
    let mut matches = 0_u32;
    for (truth_kind, decided_kind) in truth.iter().zip(decided) {
        if truth_kind == decided_kind {
            matches += 1;
        }
    }
    Ok(f64::from(matches) / truth.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{MethodSourceKind, source_recovery_rate};
    use crate::MethodEffectsError;

    #[test]
    fn wire_names_round_trip_and_unknown_fails() {
        for kind in [
            MethodSourceKind::Template,
            MethodSourceKind::Section,
            MethodSourceKind::CopiedText,
            MethodSourceKind::Style,
            MethodSourceKind::Modality,
            MethodSourceKind::CorpusBackground,
        ] {
            assert_eq!(
                MethodSourceKind::from_wire_name(kind.wire_name()).expect("round trip"),
                kind
            );
        }
        assert_eq!(
            MethodSourceKind::from_wire_name("tfidf"),
            Err(MethodEffectsError::UnknownMethodSource)
        );
        assert_eq!(
            source_recovery_rate(&[MethodSourceKind::Style], &[]),
            Err(MethodEffectsError::InvalidSourcePayload)
        );
    }
}
