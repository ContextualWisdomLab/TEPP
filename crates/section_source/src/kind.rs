//! Section boilerplate versus unique latent content.

use crate::SectionSourceError;

/// Closed vocabulary of section-related token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionKind {
    /// Repeated report section heading or boilerplate.
    SectionBoilerplate,
    /// Unique document content that is not section structure.
    UniqueContent,
}

impl SectionKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::SectionBoilerplate => "section_boilerplate",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`SectionSourceError::InvalidSectionPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, SectionSourceError> {
        match name {
            "section_boilerplate" => Ok(Self::SectionBoilerplate),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(SectionSourceError::InvalidSectionPayload),
        }
    }
}

/// Refuse to treat section boilerplate as unique latent content.
///
/// # Errors
///
/// Returns [`SectionSourceError::SectionIsNotUniqueContent`] when `kind` is
/// [`SectionKind::SectionBoilerplate`].
pub fn refuse_section_as_unique_content(kind: SectionKind) -> Result<(), SectionSourceError> {
    match kind {
        SectionKind::SectionBoilerplate => Err(SectionSourceError::SectionIsNotUniqueContent),
        SectionKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat section boilerplate as stopword deletion.
///
/// # Errors
///
/// Returns [`SectionSourceError::SectionIsNotStopwordDeletion`] when `kind` is
/// [`SectionKind::SectionBoilerplate`].
pub fn refuse_section_as_stopword_deletion(kind: SectionKind) -> Result<(), SectionSourceError> {
    match kind {
        SectionKind::SectionBoilerplate => Err(SectionSourceError::SectionIsNotStopwordDeletion),
        SectionKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered section kinds that match known truth.
///
/// # Errors
///
/// Returns [`SectionSourceError::InvalidSectionPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[SectionKind],
    decided: &[SectionKind],
) -> Result<f64, SectionSourceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SectionSourceError::InvalidSectionPayload);
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
        SectionKind, identity_recovery_rate, refuse_section_as_stopword_deletion,
        refuse_section_as_unique_content,
    };
    use crate::SectionSourceError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_section_as_unique_content(SectionKind::SectionBoilerplate),
            Err(SectionSourceError::SectionIsNotUniqueContent)
        );
        assert_eq!(
            refuse_section_as_stopword_deletion(SectionKind::SectionBoilerplate),
            Err(SectionSourceError::SectionIsNotStopwordDeletion)
        );
        refuse_section_as_unique_content(SectionKind::UniqueContent).expect("unique");
        refuse_section_as_stopword_deletion(SectionKind::UniqueContent).expect("unique");
        for kind in [SectionKind::SectionBoilerplate, SectionKind::UniqueContent] {
            assert_eq!(
                SectionKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            SectionKind::from_wire_name("template_source"),
            Err(SectionSourceError::InvalidSectionPayload)
        );
        let matched = identity_recovery_rate(
            &[SectionKind::SectionBoilerplate],
            &[SectionKind::SectionBoilerplate],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(SectionSourceError::InvalidSectionPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[SectionKind::SectionBoilerplate], &[]),
            Err(SectionSourceError::InvalidSectionPayload)
        );
    }
}
