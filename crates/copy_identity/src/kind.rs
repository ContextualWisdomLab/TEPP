//! Template-copy identity versus the copied source document.

use crate::CopyIdentityError;

/// Closed vocabulary of copy-related document identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CopyKind {
    /// A template or pasted copy of an earlier source.
    TemplateCopy,
    /// The earlier source document being copied.
    SourceDocument,
}

impl CopyKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::TemplateCopy => "template_copy_of",
            Self::SourceDocument => "source_document",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`CopyIdentityError::InvalidCopyPayload`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, CopyIdentityError> {
        match name {
            "template_copy_of" => Ok(Self::TemplateCopy),
            "source_document" => Ok(Self::SourceDocument),
            _ => Err(CopyIdentityError::InvalidCopyPayload),
        }
    }
}

/// Refuse to treat a template copy as the source document identity.
///
/// # Errors
///
/// Returns [`CopyIdentityError::CopyIsNotSourceIdentity`] when `kind` is
/// [`CopyKind::TemplateCopy`].
pub fn refuse_copy_as_source_identity(kind: CopyKind) -> Result<(), CopyIdentityError> {
    match kind {
        CopyKind::TemplateCopy => Err(CopyIdentityError::CopyIsNotSourceIdentity),
        CopyKind::SourceDocument => Ok(()),
    }
}

/// Refuse to treat a template copy as a forward state transition.
///
/// # Errors
///
/// Returns [`CopyIdentityError::CopyIsNotTransition`] when `kind` is
/// [`CopyKind::TemplateCopy`].
pub fn refuse_copy_as_transition(kind: CopyKind) -> Result<(), CopyIdentityError> {
    match kind {
        CopyKind::TemplateCopy => Err(CopyIdentityError::CopyIsNotTransition),
        CopyKind::SourceDocument => Ok(()),
    }
}

/// Fraction of recovered copy kinds that match known truth.
///
/// # Errors
///
/// Returns [`CopyIdentityError::InvalidCopyPayload`] when either slice is empty
/// or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[CopyKind],
    decided: &[CopyKind],
) -> Result<f64, CopyIdentityError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CopyIdentityError::InvalidCopyPayload);
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
        identity_recovery_rate, refuse_copy_as_source_identity, refuse_copy_as_transition, CopyKind,
    };
    use crate::CopyIdentityError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_copy_as_source_identity(CopyKind::TemplateCopy),
            Err(CopyIdentityError::CopyIsNotSourceIdentity)
        );
        assert_eq!(
            refuse_copy_as_transition(CopyKind::TemplateCopy),
            Err(CopyIdentityError::CopyIsNotTransition)
        );
        refuse_copy_as_source_identity(CopyKind::SourceDocument).expect("source");
        refuse_copy_as_transition(CopyKind::SourceDocument).expect("source");
        for kind in [CopyKind::TemplateCopy, CopyKind::SourceDocument] {
            assert_eq!(
                CopyKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            CopyKind::from_wire_name("summarizes"),
            Err(CopyIdentityError::InvalidCopyPayload)
        );
        let matched = identity_recovery_rate(&[CopyKind::TemplateCopy], &[CopyKind::TemplateCopy])
            .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(CopyIdentityError::InvalidCopyPayload)
        );
        assert_eq!(
            identity_recovery_rate(&[CopyKind::TemplateCopy], &[]),
            Err(CopyIdentityError::InvalidCopyPayload)
        );
    }
}
