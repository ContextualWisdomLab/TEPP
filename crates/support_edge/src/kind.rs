//! Evidential kinds that may point to the past.

use crate::SupportEdgeError;

/// Closed vocabulary of evidential edges that are not state transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceKind {
    /// Supportive evidence for an earlier claim or event.
    Support,
    /// Contradicting evidence against an earlier claim or event.
    Contradiction,
    /// A summary of an earlier source.
    Summarizes,
    /// An outcome pointing back to its producer (inverse of production).
    OutcomeOf,
}

impl EvidenceKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Support => "supports",
            Self::Contradiction => "contradicts",
            Self::Summarizes => "summarizes",
            Self::OutcomeOf => "outcome_of",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`SupportEdgeError::InvalidEdgePayload`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, SupportEdgeError> {
        match name {
            "supports" => Ok(Self::Support),
            "contradicts" => Ok(Self::Contradiction),
            "summarizes" => Ok(Self::Summarizes),
            "outcome_of" => Ok(Self::OutcomeOf),
            _ => Err(SupportEdgeError::InvalidEdgePayload),
        }
    }

    /// Return whether this kind is a forward state-transition edge.
    ///
    /// Evidential kinds are never transitions.
    #[must_use]
    pub const fn is_transition_edge(self) -> bool {
        match self {
            Self::Support | Self::Contradiction | Self::Summarizes | Self::OutcomeOf => false,
        }
    }
}

/// Refuse to treat an evidential edge as a forward state transition.
///
/// # Errors
///
/// Always returns [`SupportEdgeError::EvidenceIsNotTransition`].
pub fn refuse_evidence_as_transition(_kind: EvidenceKind) -> Result<(), SupportEdgeError> {
    Err(SupportEdgeError::EvidenceIsNotTransition)
}

/// Fraction of recovered evidential kinds that match known truth.
///
/// # Errors
///
/// Returns [`SupportEdgeError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn edge_kind_recovery_rate(
    truth: &[EvidenceKind],
    decided: &[EvidenceKind],
) -> Result<f64, SupportEdgeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(SupportEdgeError::InvalidEdgePayload);
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
    use super::{EvidenceKind, edge_kind_recovery_rate, refuse_evidence_as_transition};
    use crate::SupportEdgeError;

    #[test]
    fn local_branches_cover_all_kinds_and_payloads() {
        for kind in [
            EvidenceKind::Support,
            EvidenceKind::Contradiction,
            EvidenceKind::Summarizes,
            EvidenceKind::OutcomeOf,
        ] {
            assert!(!kind.is_transition_edge());
            assert_eq!(
                EvidenceKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
            assert_eq!(
                refuse_evidence_as_transition(kind),
                Err(SupportEdgeError::EvidenceIsNotTransition)
            );
        }
        assert_eq!(
            EvidenceKind::from_wire_name("causes"),
            Err(SupportEdgeError::InvalidEdgePayload)
        );
        let truth = [EvidenceKind::Support, EvidenceKind::Contradiction];
        let matched = edge_kind_recovery_rate(&truth, &truth).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            edge_kind_recovery_rate(&[], &[]),
            Err(SupportEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            edge_kind_recovery_rate(&truth, &[]),
            Err(SupportEdgeError::InvalidEdgePayload)
        );
    }
}
