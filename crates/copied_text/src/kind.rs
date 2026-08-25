//! Copied-text residue versus unique latent content.

use crate::CopiedTextError;

/// Closed vocabulary of copied-text token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopiedKind {
    /// Copied or boilerplate residue, not unique document meaning.
    CopiedText,
    /// Token treatment reserved for unique latent content.
    UniqueContent,
}

impl CopiedKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CopiedText => "copied_text",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`CopiedTextError::InvalidCopiedPayload`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, CopiedTextError> {
        match name {
            "copied_text" => Ok(Self::CopiedText),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(CopiedTextError::InvalidCopiedPayload),
        }
    }
}

/// Refuse to treat copied-text residue as unique latent content.
///
/// # Errors
///
/// Returns [`CopiedTextError::CopiedTextIsNotUniqueContent`] when `kind` is
/// [`CopiedKind::CopiedText`].
pub fn refuse_copied_text_as_unique_content(kind: CopiedKind) -> Result<(), CopiedTextError> {
    match kind {
        CopiedKind::CopiedText => Err(CopiedTextError::CopiedTextIsNotUniqueContent),
        CopiedKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat copied-text residue as stopword deletion.
///
/// # Errors
///
/// Returns [`CopiedTextError::CopiedTextIsNotStopwordDeletion`] when `kind` is
/// [`CopiedKind::CopiedText`].
pub fn refuse_copied_text_as_stopword_deletion(kind: CopiedKind) -> Result<(), CopiedTextError> {
    match kind {
        CopiedKind::CopiedText => Err(CopiedTextError::CopiedTextIsNotStopwordDeletion),
        CopiedKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered copied-text kinds that match known truth.
///
/// # Errors
///
/// Returns [`CopiedTextError::InvalidCopiedPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[CopiedKind],
    decided: &[CopiedKind],
) -> Result<f64, CopiedTextError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CopiedTextError::InvalidCopiedPayload);
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
        CopiedKind, identity_recovery_rate, refuse_copied_text_as_stopword_deletion,
        refuse_copied_text_as_unique_content,
    };
    use crate::CopiedTextError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_copied_text_as_unique_content(CopiedKind::CopiedText),
            Err(CopiedTextError::CopiedTextIsNotUniqueContent)
        );
        assert_eq!(
            refuse_copied_text_as_stopword_deletion(CopiedKind::CopiedText),
            Err(CopiedTextError::CopiedTextIsNotStopwordDeletion)
        );
        refuse_copied_text_as_unique_content(CopiedKind::UniqueContent).expect("unique");
        refuse_copied_text_as_stopword_deletion(CopiedKind::UniqueContent).expect("unique");
        for kind in [CopiedKind::CopiedText, CopiedKind::UniqueContent] {
            assert_eq!(
                CopiedKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            CopiedKind::from_wire_name("stopword"),
            Err(CopiedTextError::InvalidCopiedPayload)
        );
        let matched = identity_recovery_rate(&[CopiedKind::CopiedText], &[CopiedKind::CopiedText])
            .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(CopiedTextError::InvalidCopiedPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[CopiedKind::CopiedText], &[]),
            Err(CopiedTextError::InvalidCopiedPayload)
        );
    }
}
