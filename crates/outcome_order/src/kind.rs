//! Input, process, and outcome-of kinds with event-time order gates.

use crate::OutcomeOrderError;
use std::cmp::Ordering;

/// Closed vocabulary of input-process-outcome edges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// Input feeding a later process (forward transition).
    InputTo,
    /// Process feeding a later process or outcome (forward transition).
    ProcessTo,
    /// Outcome pointing back at its producer (provenance; may look backward).
    OutcomeOf,
}

impl OutcomeKind {
    /// Return the stable wire kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::InputTo => "input_to",
            Self::ProcessTo => "process_to",
            Self::OutcomeOf => "outcome_of",
        }
    }

    /// Parse a stable wire kind name.
    ///
    /// # Errors
    ///
    /// Returns [`OutcomeOrderError::InvalidEdgePayload`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, OutcomeOrderError> {
        match name {
            "input_to" => Ok(Self::InputTo),
            "process_to" => Ok(Self::ProcessTo),
            "outcome_of" => Ok(Self::OutcomeOf),
            _ => Err(OutcomeOrderError::InvalidEdgePayload),
        }
    }

    /// Return whether this kind is a forward state-transition edge.
    ///
    /// `outcome_of` is provenance (the inverse of `produces`).
    #[must_use]
    pub const fn is_transition_edge(self) -> bool {
        match self {
            Self::InputTo | Self::ProcessTo => true,
            Self::OutcomeOf => false,
        }
    }
}

/// Refuse reverse event-time order on input and process transitions.
///
/// `source_rank` and `target_rank` are opaque event-time ordinals, not clock
/// identities. Transition kinds require `source_rank < target_rank`.
/// [`OutcomeKind::OutcomeOf`] may point at an earlier producer.
///
/// # Errors
///
/// Returns [`OutcomeOrderError::ReverseIpoOrder`] when a transition moves
/// backward and [`OutcomeOrderError::UncertainIpoOrder`] when a transition
/// uses equal ranks.
pub fn refuse_reverse_ipo_order(
    kind: OutcomeKind,
    source_rank: u64,
    target_rank: u64,
) -> Result<(), OutcomeOrderError> {
    match kind {
        OutcomeKind::InputTo | OutcomeKind::ProcessTo => match source_rank.cmp(&target_rank) {
            Ordering::Less => Ok(()),
            Ordering::Greater => Err(OutcomeOrderError::ReverseIpoOrder),
            Ordering::Equal => Err(OutcomeOrderError::UncertainIpoOrder),
        },
        OutcomeKind::OutcomeOf => Ok(()),
    }
}

/// Refuse to treat `outcome_of` as a forward state transition.
///
/// # Errors
///
/// Returns [`OutcomeOrderError::OutcomeOfIsNotTransition`] when `kind` is
/// [`OutcomeKind::OutcomeOf`].
pub fn refuse_outcome_of_as_transition(kind: OutcomeKind) -> Result<(), OutcomeOrderError> {
    match kind {
        OutcomeKind::OutcomeOf => Err(OutcomeOrderError::OutcomeOfIsNotTransition),
        OutcomeKind::InputTo | OutcomeKind::ProcessTo => Ok(()),
    }
}

/// Fraction of recovered IPO kinds that match known truth.
///
/// # Errors
///
/// Returns [`OutcomeOrderError::InvalidEdgePayload`] when either slice is
/// empty or the lengths differ.
pub fn kind_recovery_rate(
    truth: &[OutcomeKind],
    decided: &[OutcomeKind],
) -> Result<f64, OutcomeOrderError> {
    if truth.is_empty() || truth.len() != decided.len() {
        return Err(OutcomeOrderError::InvalidEdgePayload);
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
        OutcomeKind, kind_recovery_rate, refuse_outcome_of_as_transition, refuse_reverse_ipo_order,
    };
    use crate::OutcomeOrderError;

    #[test]
    fn local_branches_cover_kinds_order_and_payloads() {
        for kind in [
            OutcomeKind::InputTo,
            OutcomeKind::ProcessTo,
            OutcomeKind::OutcomeOf,
        ] {
            assert_eq!(
                OutcomeKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
        assert!(OutcomeKind::InputTo.is_transition_edge());
        assert!(OutcomeKind::ProcessTo.is_transition_edge());
        assert!(!OutcomeKind::OutcomeOf.is_transition_edge());
        assert_eq!(
            OutcomeKind::from_wire_name("causes"),
            Err(OutcomeOrderError::InvalidEdgePayload)
        );
        refuse_reverse_ipo_order(OutcomeKind::InputTo, 1, 2).expect("forward");
        refuse_reverse_ipo_order(OutcomeKind::ProcessTo, 2, 3).expect("forward");
        refuse_reverse_ipo_order(OutcomeKind::OutcomeOf, 9, 1).expect("look-back");
        assert_eq!(
            refuse_reverse_ipo_order(OutcomeKind::InputTo, 4, 1),
            Err(OutcomeOrderError::ReverseIpoOrder)
        );
        assert_eq!(
            refuse_reverse_ipo_order(OutcomeKind::ProcessTo, 8, 8),
            Err(OutcomeOrderError::UncertainIpoOrder)
        );
        assert_eq!(
            refuse_outcome_of_as_transition(OutcomeKind::OutcomeOf),
            Err(OutcomeOrderError::OutcomeOfIsNotTransition)
        );
        refuse_outcome_of_as_transition(OutcomeKind::InputTo).expect("transition");
        refuse_outcome_of_as_transition(OutcomeKind::ProcessTo).expect("transition");
        let matched =
            kind_recovery_rate(&[OutcomeKind::InputTo], &[OutcomeKind::InputTo]).expect("rate");
        assert!((matched - 1.0).abs() < f64::EPSILON);
        let partial = kind_recovery_rate(
            &[OutcomeKind::InputTo, OutcomeKind::OutcomeOf],
            &[OutcomeKind::InputTo, OutcomeKind::InputTo],
        )
        .expect("partial");
        assert!((partial - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            kind_recovery_rate(&[], &[]),
            Err(OutcomeOrderError::InvalidEdgePayload)
        );
        assert_eq!(
            kind_recovery_rate(&[OutcomeKind::InputTo], &[]),
            Err(OutcomeOrderError::InvalidEdgePayload)
        );
    }
}
