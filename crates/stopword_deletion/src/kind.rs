//! Deletion methods that cannot silently erase repeated report language.

use crate::StopwordDeletionError;

/// Closed vocabulary of deletion versus explicit method-source treatments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeletionKind {
    /// A default or global stopword list applied as deletion.
    DefaultStopwordList,
    /// Repeated language kept as explicit method/background structure.
    ExplicitMethodSource,
}

impl DeletionKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::DefaultStopwordList => "default_stopword_list",
            Self::ExplicitMethodSource => "explicit_method_source",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`StopwordDeletionError::InvalidDeletionPayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, StopwordDeletionError> {
        match name {
            "default_stopword_list" => Ok(Self::DefaultStopwordList),
            "explicit_method_source" => Ok(Self::ExplicitMethodSource),
            _ => Err(StopwordDeletionError::InvalidDeletionPayload),
        }
    }
}

/// Refuse to treat a default stopword list as a valid deletion method.
///
/// # Errors
///
/// Returns [`StopwordDeletionError::DefaultStopwordDeletion`] when `kind` is
/// [`DeletionKind::DefaultStopwordList`].
pub fn refuse_default_stopword_deletion(kind: DeletionKind) -> Result<(), StopwordDeletionError> {
    match kind {
        DeletionKind::DefaultStopwordList => Err(StopwordDeletionError::DefaultStopwordDeletion),
        DeletionKind::ExplicitMethodSource => Ok(()),
    }
}

/// Fraction of recovered deletion kinds that match known truth.
///
/// # Errors
///
/// Returns [`StopwordDeletionError::InvalidDeletionPayload`] when either slice
/// is empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[DeletionKind],
    decided: &[DeletionKind],
) -> Result<f64, StopwordDeletionError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(StopwordDeletionError::InvalidDeletionPayload);
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
    use super::{DeletionKind, identity_recovery_rate, refuse_default_stopword_deletion};
    use crate::StopwordDeletionError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_default_stopword_deletion(DeletionKind::DefaultStopwordList),
            Err(StopwordDeletionError::DefaultStopwordDeletion)
        );
        refuse_default_stopword_deletion(DeletionKind::ExplicitMethodSource).expect("source");
        for kind in [
            DeletionKind::DefaultStopwordList,
            DeletionKind::ExplicitMethodSource,
        ] {
            assert_eq!(
                DeletionKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            DeletionKind::from_wire_name("tfidf_weight"),
            Err(StopwordDeletionError::InvalidDeletionPayload)
        );
        let matched = identity_recovery_rate(
            &[DeletionKind::ExplicitMethodSource],
            &[DeletionKind::ExplicitMethodSource],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(StopwordDeletionError::InvalidDeletionPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[DeletionKind::DefaultStopwordList], &[]),
            Err(StopwordDeletionError::InvalidDeletionPayload)
        );
    }
}
