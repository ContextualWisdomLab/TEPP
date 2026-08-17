//! Retrospective reporting versus contemporaneous forward reporting.

use crate::RetrospectiveEdgeError;

/// Closed vocabulary of reporting edges that may point at earlier event time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrospectiveKind {
    /// A later report about an earlier event (provenance; may point backward).
    RetrospectiveReport,
    /// A contemporaneous report that is not a translation or a transition.
    ForwardReport,
}

impl RetrospectiveKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::RetrospectiveReport => "retrospectively_reports",
            Self::ForwardReport => "forward_report",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`RetrospectiveEdgeError::InvalidEdgePayload`] for unrecognized
    /// names.
    pub fn from_wire_name(name: &str) -> Result<Self, RetrospectiveEdgeError> {
        match name {
            "retrospectively_reports" => Ok(Self::RetrospectiveReport),
            "forward_report" => Ok(Self::ForwardReport),
            _ => Err(RetrospectiveEdgeError::InvalidEdgePayload),
        }
    }
}

/// Refuse to treat a retrospective report as a forward state transition.
///
/// # Errors
///
/// Returns [`RetrospectiveEdgeError::RetrospectiveIsNotTransition`] when
/// `kind` is [`RetrospectiveKind::RetrospectiveReport`].
pub fn refuse_retrospective_as_transition(
    kind: RetrospectiveKind,
) -> Result<(), RetrospectiveEdgeError> {
    match kind {
        RetrospectiveKind::RetrospectiveReport => {
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTransition)
        }
        RetrospectiveKind::ForwardReport => Ok(()),
    }
}

/// Refuse to treat a retrospective report as a translation.
///
/// # Errors
///
/// Returns [`RetrospectiveEdgeError::RetrospectiveIsNotTranslation`] when
/// `kind` is [`RetrospectiveKind::RetrospectiveReport`].
pub fn refuse_retrospective_as_translation(
    kind: RetrospectiveKind,
) -> Result<(), RetrospectiveEdgeError> {
    match kind {
        RetrospectiveKind::RetrospectiveReport => {
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTranslation)
        }
        RetrospectiveKind::ForwardReport => Ok(()),
    }
}

/// Fraction of recovered reporting kinds that match known truth.
///
/// # Errors
///
/// Returns [`RetrospectiveEdgeError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn identity_recovery_rate(
    truth: &[RetrospectiveKind],
    decided: &[RetrospectiveKind],
) -> Result<f64, RetrospectiveEdgeError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(RetrospectiveEdgeError::InvalidEdgePayload);
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
        identity_recovery_rate, refuse_retrospective_as_transition,
        refuse_retrospective_as_translation, RetrospectiveKind,
    };
    use crate::RetrospectiveEdgeError;

    #[test]
    fn local_branches_cover_kinds_payloads_and_wire_names() {
        assert_eq!(
            refuse_retrospective_as_transition(RetrospectiveKind::RetrospectiveReport),
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTransition)
        );
        assert_eq!(
            refuse_retrospective_as_translation(RetrospectiveKind::RetrospectiveReport),
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTranslation)
        );
        refuse_retrospective_as_transition(RetrospectiveKind::ForwardReport).expect("forward");
        refuse_retrospective_as_translation(RetrospectiveKind::ForwardReport).expect("forward");
        for kind in [
            RetrospectiveKind::RetrospectiveReport,
            RetrospectiveKind::ForwardReport,
        ] {
            assert_eq!(
                RetrospectiveKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert_eq!(
            RetrospectiveKind::from_wire_name("translates"),
            Err(RetrospectiveEdgeError::InvalidEdgePayload)
        );
        let matched = identity_recovery_rate(
            &[RetrospectiveKind::RetrospectiveReport],
            &[RetrospectiveKind::RetrospectiveReport],
        )
        .expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        assert_eq!(
            identity_recovery_rate(&[], &[]),
            Err(RetrospectiveEdgeError::InvalidEdgePayload)
        );
        assert_eq!(
            identity_recovery_rate(&[RetrospectiveKind::RetrospectiveReport], &[]),
            Err(RetrospectiveEdgeError::InvalidEdgePayload)
        );
    }
}
