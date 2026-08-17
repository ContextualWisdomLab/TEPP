//! Summary provenance versus the summarized source document.

use crate::SummarizesEdgeError;

/// Closed vocabulary of summary-related document identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SummarizesKind {
    /// A summary of an earlier source (provenance; may point backward).
    Summary,
    /// The earlier source document being summarized.
    SourceDocument,
}

impl SummarizesKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Summary => "summarizes",
            Self::SourceDocument => "source_document",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`SummarizesEdgeError::InvalidEdgePayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, SummarizesEdgeError> {
        match name {
            "summarizes" => Ok(Self::Summary),
            "source_document" => Ok(Self::SourceDocument),
            _ => Err(SummarizesEdgeError::InvalidEdgePayload),
        }
    }
}

/// Refuse to treat a summary as a forward state transition.
///
/// # Errors
///
/// Returns [`SummarizesEdgeError::SummaryIsNotTransition`] when `kind` is
/// [`SummarizesKind::Summary`].
pub fn refuse_summary_as_transition(kind: SummarizesKind) -> Result<(), SummarizesEdgeError> {
    match kind {
        SummarizesKind::Summary => Err(SummarizesEdgeError::SummaryIsNotTransition),
        SummarizesKind::SourceDocument => Ok(()),
    }
}

/// Refuse to treat a summary as the source document identity.
///
/// # Errors
///
/// Returns [`SummarizesEdgeError::SummaryIsNotSourceIdentity`] when `kind` is
/// [`SummarizesKind::Summary`].
pub fn refuse_summary_as_source_identity(kind: SummarizesKind) -> Result<(), SummarizesEdgeError> {
    match kind {
        SummarizesKind::Summary => Err(SummarizesEdgeError::SummaryIsNotSourceIdentity),
        SummarizesKind::SourceDocument => Ok(()),
    }
}

/// Fraction of recovered summary kinds that match known truth.
///
/// # Errors
///
/// Returns [`SummarizesEdgeError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[SummarizesKind],
    decided: &[SummarizesKind],
) -> Result<f64, SummarizesEdgeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SummarizesEdgeError::InvalidEdgePayload);
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
        identity_recovery_rate, refuse_summary_as_source_identity, refuse_summary_as_transition,
        SummarizesKind,
    };
    use crate::SummarizesEdgeError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_summary_as_transition(SummarizesKind::Summary),
            Err(SummarizesEdgeError::SummaryIsNotTransition)
        );
        assert_eq!(
            refuse_summary_as_source_identity(SummarizesKind::Summary),
            Err(SummarizesEdgeError::SummaryIsNotSourceIdentity)
        );
        refuse_summary_as_transition(SummarizesKind::SourceDocument).expect("source");
        refuse_summary_as_source_identity(SummarizesKind::SourceDocument).expect("source");
        for kind in [SummarizesKind::Summary, SummarizesKind::SourceDocument] {
            assert_eq!(
                SummarizesKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            SummarizesKind::from_wire_name("references"),
            Err(SummarizesEdgeError::InvalidEdgePayload)
        );
        let matched =
            identity_recovery_rate(&[SummarizesKind::Summary], &[SummarizesKind::Summary])
                .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(SummarizesEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[SummarizesKind::Summary], &[]),
            Err(SummarizesEdgeError::InvalidEdgePayload)
        );
    }
}
