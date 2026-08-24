//! Corpus-background wording versus unique latent content.

use crate::CorpusBackgroundError;

/// Closed vocabulary of corpus-background token treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusBackgroundKind {
    /// Corpus-level background language, not unique document meaning.
    CorpusBackground,
    /// Token treatment reserved for unique latent content.
    UniqueContent,
}

impl CorpusBackgroundKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CorpusBackground => "corpus_background",
            Self::UniqueContent => "unique_content",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`CorpusBackgroundError::InvalidCorpusBackgroundPayload`] for
    /// unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, CorpusBackgroundError> {
        match name {
            "corpus_background" => Ok(Self::CorpusBackground),
            "unique_content" => Ok(Self::UniqueContent),
            _ => Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload),
        }
    }
}

/// Refuse to treat corpus-background wording as unique latent content.
///
/// # Errors
///
/// Returns [`CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent`] when
/// `kind` is [`CorpusBackgroundKind::CorpusBackground`].
pub fn refuse_corpus_background_as_unique_content(
    kind: CorpusBackgroundKind,
) -> Result<(), CorpusBackgroundError> {
    match kind {
        CorpusBackgroundKind::CorpusBackground => {
            Err(CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent)
        }
        CorpusBackgroundKind::UniqueContent => Ok(()),
    }
}

/// Refuse to treat corpus-background wording as stopword deletion.
///
/// # Errors
///
/// Returns [`CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion`]
/// when `kind` is [`CorpusBackgroundKind::CorpusBackground`].
pub fn refuse_corpus_background_as_stopword_deletion(
    kind: CorpusBackgroundKind,
) -> Result<(), CorpusBackgroundError> {
    match kind {
        CorpusBackgroundKind::CorpusBackground => {
            Err(CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion)
        }
        CorpusBackgroundKind::UniqueContent => Ok(()),
    }
}

/// Fraction of recovered corpus-background kinds that match known truth.
///
/// # Errors
///
/// Returns [`CorpusBackgroundError::InvalidCorpusBackgroundPayload`] when
/// either slice is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[CorpusBackgroundKind],
    decided: &[CorpusBackgroundKind],
) -> Result<f64, CorpusBackgroundError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload);
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
        CorpusBackgroundKind, identity_recovery_rate,
        refuse_corpus_background_as_stopword_deletion, refuse_corpus_background_as_unique_content,
    };
    use crate::CorpusBackgroundError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_corpus_background_as_unique_content(CorpusBackgroundKind::CorpusBackground),
            Err(CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent)
        );
        assert_eq!(
            refuse_corpus_background_as_stopword_deletion(CorpusBackgroundKind::CorpusBackground),
            Err(CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion)
        );
        refuse_corpus_background_as_unique_content(CorpusBackgroundKind::UniqueContent)
            .expect("unique");
        refuse_corpus_background_as_stopword_deletion(CorpusBackgroundKind::UniqueContent)
            .expect("unique");
        for kind in [
            CorpusBackgroundKind::CorpusBackground,
            CorpusBackgroundKind::UniqueContent,
        ] {
            assert_eq!(
                CorpusBackgroundKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            CorpusBackgroundKind::from_wire_name("stopword"),
            Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
        );
        let matched = identity_recovery_rate(
            &[CorpusBackgroundKind::CorpusBackground],
            &[CorpusBackgroundKind::CorpusBackground],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[CorpusBackgroundKind::CorpusBackground], &[]),
            Err(CorpusBackgroundError::InvalidCorpusBackgroundPayload)
        );
    }
}
