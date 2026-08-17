//! Non-lexical modality versus unique latent content.

use crate::ModalitySourceError;

/// Closed vocabulary of modality-related token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModalityKind {
    /// Non-lexical modality channel, not unique document meaning.
    NonLexicalModality,
    /// Token treatment reserved for unique latent content.
    UniqueContent,
}

impl ModalityKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::NonLexicalModality => "modality",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`ModalitySourceError::InvalidModalityPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, ModalitySourceError> {
        match name {
            "modality" => Ok(Self::NonLexicalModality),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(ModalitySourceError::InvalidModalityPayload),
        }
    }
}

/// Refuse to treat non-lexical modality as unique latent content.
///
/// # Errors
///
/// Returns [`ModalitySourceError::ModalityIsNotUniqueContent`] when `kind` is
/// [`ModalityKind::NonLexicalModality`].
pub fn refuse_modality_as_unique_content(kind: ModalityKind) -> Result<(), ModalitySourceError> {
    match kind {
        ModalityKind::NonLexicalModality => Err(ModalitySourceError::ModalityIsNotUniqueContent),
        ModalityKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat non-lexical modality as stopword deletion.
///
/// # Errors
///
/// Returns [`ModalitySourceError::ModalityIsNotStopwordDeletion`] when `kind`
/// is [`ModalityKind::NonLexicalModality`].
pub fn refuse_modality_as_stopword_deletion(kind: ModalityKind) -> Result<(), ModalitySourceError> {
    match kind {
        ModalityKind::NonLexicalModality => Err(ModalitySourceError::ModalityIsNotStopwordDeletion),
        ModalityKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered modality kinds that match known truth.
///
/// # Errors
///
/// Returns [`ModalitySourceError::InvalidModalityPayload`] when either slice
/// is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[ModalityKind],
    decided: &[ModalityKind],
) -> Result<f64, ModalitySourceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(ModalitySourceError::InvalidModalityPayload);
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
    use super::{
        identity_recovery_rate, refuse_modality_as_stopword_deletion,
        refuse_modality_as_unique_content, ModalityKind,
    };
    use crate::ModalitySourceError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_modality_as_unique_content(ModalityKind::NonLexicalModality),
            Err(ModalitySourceError::ModalityIsNotUniqueContent)
        );
        assert_eq!(
            refuse_modality_as_stopword_deletion(ModalityKind::NonLexicalModality),
            Err(ModalitySourceError::ModalityIsNotStopwordDeletion)
        );
        refuse_modality_as_unique_content(ModalityKind::UniqueContent).expect("unique");
        refuse_modality_as_stopword_deletion(ModalityKind::UniqueContent).expect("unique");
        for kind in [
            ModalityKind::NonLexicalModality,
            ModalityKind::UniqueContent,
        ] {
            assert_eq!(
                ModalityKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            ModalityKind::from_wire_name("stopword"),
            Err(ModalitySourceError::InvalidModalityPayload)
        );
        let matched = identity_recovery_rate(
            &[ModalityKind::NonLexicalModality],
            &[ModalityKind::NonLexicalModality],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(ModalitySourceError::InvalidModalityPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[ModalityKind::NonLexicalModality], &[]),
            Err(ModalitySourceError::InvalidModalityPayload)
        );
    }
}
