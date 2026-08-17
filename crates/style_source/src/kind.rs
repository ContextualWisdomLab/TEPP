//! Style residue versus unique latent content.

use crate::StyleSourceError;

/// Closed vocabulary of style-related token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleKind {
    /// House-voice or style residue, not unique document meaning.
    StyleResidue,
    /// Token treatment reserved for unique latent content.
    UniqueContent,
}

impl StyleKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::StyleResidue => "style",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`StyleSourceError::InvalidStylePayload`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, StyleSourceError> {
        match name {
            "style" => Ok(Self::StyleResidue),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(StyleSourceError::InvalidStylePayload),
        }
    }
}

/// Refuse to treat style residue as unique latent content.
///
/// # Errors
///
/// Returns [`StyleSourceError::StyleIsNotUniqueContent`] when `kind` is
/// [`StyleKind::StyleResidue`].
pub fn refuse_style_as_unique_content(kind: StyleKind) -> Result<(), StyleSourceError> {
    match kind {
        StyleKind::StyleResidue => Err(StyleSourceError::StyleIsNotUniqueContent),
        StyleKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat style residue as stopword deletion.
///
/// # Errors
///
/// Returns [`StyleSourceError::StyleIsNotStopwordDeletion`] when `kind` is
/// [`StyleKind::StyleResidue`].
pub fn refuse_style_as_stopword_deletion(kind: StyleKind) -> Result<(), StyleSourceError> {
    match kind {
        StyleKind::StyleResidue => Err(StyleSourceError::StyleIsNotStopwordDeletion),
        StyleKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered style kinds that match known truth.
///
/// # Errors
///
/// Returns [`StyleSourceError::InvalidStylePayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[StyleKind],
    decided: &[StyleKind],
) -> Result<f64, StyleSourceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(StyleSourceError::InvalidStylePayload);
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
        identity_recovery_rate, refuse_style_as_stopword_deletion, refuse_style_as_unique_content,
        StyleKind,
    };
    use crate::StyleSourceError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_style_as_unique_content(StyleKind::StyleResidue),
            Err(StyleSourceError::StyleIsNotUniqueContent)
        );
        assert_eq!(
            refuse_style_as_stopword_deletion(StyleKind::StyleResidue),
            Err(StyleSourceError::StyleIsNotStopwordDeletion)
        );
        refuse_style_as_unique_content(StyleKind::UniqueContent).expect("unique");
        refuse_style_as_stopword_deletion(StyleKind::UniqueContent).expect("unique");
        for kind in [StyleKind::StyleResidue, StyleKind::UniqueContent] {
            assert_eq!(
                StyleKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            StyleKind::from_wire_name("stopword"),
            Err(StyleSourceError::InvalidStylePayload)
        );
        let matched =
            identity_recovery_rate(&[StyleKind::StyleResidue], &[StyleKind::StyleResidue])
                .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(StyleSourceError::InvalidStylePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[StyleKind::StyleResidue], &[]),
            Err(StyleSourceError::InvalidStylePayload)
        );
    }
}
