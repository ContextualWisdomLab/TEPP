//! Prompt boilerplate versus unique latent content.

use crate::PromptSourceError;

/// Closed vocabulary of prompt-related token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptKind {
    /// Instruction or prompt boilerplate, not unique document meaning.
    PromptBoilerplate,
    /// Token treatment reserved for unique latent content.
    UniqueContent,
}

impl PromptKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::PromptBoilerplate => "prompt_boilerplate",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`PromptSourceError::InvalidPromptPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, PromptSourceError> {
        match name {
            "prompt_boilerplate" => Ok(Self::PromptBoilerplate),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(PromptSourceError::InvalidPromptPayload),
        }
    }
}

/// Refuse to treat prompt boilerplate as unique latent content.
///
/// # Errors
///
/// Returns [`PromptSourceError::PromptIsNotUniqueContent`] when `kind` is
/// [`PromptKind::PromptBoilerplate`].
pub fn refuse_prompt_as_unique_content(kind: PromptKind) -> Result<(), PromptSourceError> {
    match kind {
        PromptKind::PromptBoilerplate => Err(PromptSourceError::PromptIsNotUniqueContent),
        PromptKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat prompt boilerplate as stopword deletion.
///
/// # Errors
///
/// Returns [`PromptSourceError::PromptIsNotStopwordDeletion`] when `kind` is
/// [`PromptKind::PromptBoilerplate`].
pub fn refuse_prompt_as_stopword_deletion(kind: PromptKind) -> Result<(), PromptSourceError> {
    match kind {
        PromptKind::PromptBoilerplate => Err(PromptSourceError::PromptIsNotStopwordDeletion),
        PromptKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered prompt kinds that match known truth.
///
/// # Errors
///
/// Returns [`PromptSourceError::InvalidPromptPayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[PromptKind],
    decided: &[PromptKind],
) -> Result<f64, PromptSourceError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(PromptSourceError::InvalidPromptPayload);
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
        identity_recovery_rate, refuse_prompt_as_stopword_deletion,
        refuse_prompt_as_unique_content, PromptKind,
    };
    use crate::PromptSourceError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_prompt_as_unique_content(PromptKind::PromptBoilerplate),
            Err(PromptSourceError::PromptIsNotUniqueContent)
        );
        assert_eq!(
            refuse_prompt_as_stopword_deletion(PromptKind::PromptBoilerplate),
            Err(PromptSourceError::PromptIsNotStopwordDeletion)
        );
        refuse_prompt_as_unique_content(PromptKind::UniqueContent).expect("unique");
        refuse_prompt_as_stopword_deletion(PromptKind::UniqueContent).expect("unique");
        for kind in [PromptKind::PromptBoilerplate, PromptKind::UniqueContent] {
            assert_eq!(
                PromptKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            PromptKind::from_wire_name("template"),
            Err(PromptSourceError::InvalidPromptPayload)
        );
        let matched = identity_recovery_rate(
            &[PromptKind::PromptBoilerplate],
            &[PromptKind::PromptBoilerplate],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(PromptSourceError::InvalidPromptPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[PromptKind::PromptBoilerplate], &[]),
            Err(PromptSourceError::InvalidPromptPayload)
        );
    }
}
