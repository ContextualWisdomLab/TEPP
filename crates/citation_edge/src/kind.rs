//! Provenance kinds that may point to the past.

use crate::CitationEdgeError;

/// Closed vocabulary of provenance edges that are not state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceKind {
    /// A citation of earlier evidence.
    Citation,
    /// A translation of an earlier document.
    Translation,
    /// A revision of an earlier document.
    Revision,
    /// A retrospective report about an earlier event.
    RetrospectiveReport,
}

/// Refuse to treat a provenance edge as a forward state transition.
///
/// # Errors
///
/// Always returns [`CitationEdgeError::ProvenanceIsNotTransition`].
pub fn refuse_provenance_as_transition(_kind: ProvenanceKind) -> Result<(), CitationEdgeError> {
    Err(CitationEdgeError::ProvenanceIsNotTransition)
}

/// Fraction of recovered provenance kinds that match known truth.
///
/// # Errors
///
/// Returns [`CitationEdgeError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn edge_kind_recovery_rate(
    truth: &[ProvenanceKind],
    decided: &[ProvenanceKind],
) -> Result<f64, CitationEdgeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(CitationEdgeError::InvalidEdgePayload);
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
    use super::{ProvenanceKind, edge_kind_recovery_rate, refuse_provenance_as_transition};
    use crate::CitationEdgeError;

    #[test]
    fn local_branches_cover_all_kinds_and_payloads() {
        for kind in [
            ProvenanceKind::Citation,
            ProvenanceKind::Translation,
            ProvenanceKind::Revision,
            ProvenanceKind::RetrospectiveReport,
        ] {
            assert_eq!(
                refuse_provenance_as_transition(kind),
                Err(CitationEdgeError::ProvenanceIsNotTransition)
            );
        }
        let truth = [ProvenanceKind::Citation, ProvenanceKind::Revision];
        let matched = edge_kind_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            edge_kind_recovery_rate(&[], &[]),
            Err(CitationEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            edge_kind_recovery_rate(&truth, &[]),
            Err(CitationEdgeError::InvalidEdgePayload)
        );
    }
}
