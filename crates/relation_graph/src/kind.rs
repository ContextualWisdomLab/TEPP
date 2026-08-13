//! Closed relation vocabulary with derived transition classification.

use crate::RelationError;
use serde::{Deserialize, Serialize};

/// Closed TEPP relation vocabulary for documents, events, and transitions.
///
/// `transition_edge` is not a free-form flag: it is derived from this vocabulary
/// so provenance edges cannot be promoted into reverse state transitions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    /// Causal production of a later state or outcome.
    Causes,
    /// Enabling condition for a later state without asserting sole causation.
    Enables,
    /// Directed intervention on a later target process.
    IntervenesOn,
    /// Leads-to successor without full causal identification.
    LeadsTo,
    /// Produces a later product, artifact, or outcome event.
    Produces,
    /// Explicit state transition into a later event instance.
    TransitionsTo,
    /// Input feeding a later process event.
    InputTo,
    /// Process stage feeding a later process or outcome.
    ProcessTo,
    /// Citation or hyperlink reference (provenance; may point backward).
    References,
    /// Summary of an earlier source (provenance).
    Summarizes,
    /// Revision of an earlier document or claim (provenance).
    Revises,
    /// Translation of an earlier document or claim (provenance).
    Translates,
    /// Retrospective reporting of an earlier event (provenance).
    RetrospectivelyReports,
    /// Supportive evidence relation (provenance).
    Supports,
    /// Contradicting evidence relation (provenance).
    Contradicts,
    /// Outcome pointing back to its producer (provenance; inverse of `Produces`).
    OutcomeOf,
}

impl RelationKind {
    /// Return whether this kind may carry an identified causal claim.
    ///
    /// `Causes` and `IntervenesOn` are the only vocabulary members that may be
    /// described as causal. Temporal precedence, enabling, production, and
    /// provenance remain non-causal until a later identified design.
    #[must_use]
    pub const fn is_identified_causal_claim(self) -> bool {
        matches!(self, Self::Causes | Self::IntervenesOn)
    }

    /// Return whether this kind is a forward state-transition edge.
    #[must_use]
    pub const fn is_transition_edge(self) -> bool {
        matches!(
            self,
            Self::Causes
                | Self::Enables
                | Self::IntervenesOn
                | Self::LeadsTo
                | Self::Produces
                | Self::TransitionsTo
                | Self::InputTo
                | Self::ProcessTo
        )
    }

    /// Parse a stable wire relation-kind name.
    ///
    /// # Errors
    ///
    /// Returns [`RelationError::UnknownRelationKind`] for unrecognized names.
    pub fn from_wire_name(name: &str) -> Result<Self, RelationError> {
        match name {
            "causes" => Ok(Self::Causes),
            "enables" => Ok(Self::Enables),
            "intervenes_on" => Ok(Self::IntervenesOn),
            "leads_to" => Ok(Self::LeadsTo),
            "produces" => Ok(Self::Produces),
            "transitions_to" => Ok(Self::TransitionsTo),
            "input_to" => Ok(Self::InputTo),
            "process_to" => Ok(Self::ProcessTo),
            "references" => Ok(Self::References),
            "summarizes" => Ok(Self::Summarizes),
            "revises" => Ok(Self::Revises),
            "translates" => Ok(Self::Translates),
            "retrospectively_reports" => Ok(Self::RetrospectivelyReports),
            "supports" => Ok(Self::Supports),
            "contradicts" => Ok(Self::Contradicts),
            "outcome_of" => Ok(Self::OutcomeOf),
            _ => Err(RelationError::UnknownRelationKind),
        }
    }

    /// Return the stable wire relation-kind name.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Causes => "causes",
            Self::Enables => "enables",
            Self::IntervenesOn => "intervenes_on",
            Self::LeadsTo => "leads_to",
            Self::Produces => "produces",
            Self::TransitionsTo => "transitions_to",
            Self::InputTo => "input_to",
            Self::ProcessTo => "process_to",
            Self::References => "references",
            Self::Summarizes => "summarizes",
            Self::Revises => "revises",
            Self::Translates => "translates",
            Self::RetrospectivelyReports => "retrospectively_reports",
            Self::Supports => "supports",
            Self::Contradicts => "contradicts",
            Self::OutcomeOf => "outcome_of",
        }
    }
}

/// Refuse treating association, precedence, or provenance as causation.
///
/// # Errors
///
/// Returns [`RelationError::CausalClaimNotIdentified`] unless `kind` is
/// [`RelationKind::Causes`] or [`RelationKind::IntervenesOn`].
pub fn refuse_association_as_cause(kind: RelationKind) -> Result<(), RelationError> {
    if kind.is_identified_causal_claim() {
        Ok(())
    } else {
        Err(RelationError::CausalClaimNotIdentified)
    }
}

#[cfg(test)]
mod tests {
    use super::RelationKind;
    use crate::RelationError;

    #[test]
    fn identified_causal_kinds_are_only_causes_and_intervention() {
        assert!(RelationKind::Causes.is_identified_causal_claim());
        assert!(RelationKind::IntervenesOn.is_identified_causal_claim());
        assert!(!RelationKind::LeadsTo.is_identified_causal_claim());
        super::refuse_association_as_cause(RelationKind::Causes).expect("causes");
    }

    #[test]
    fn transition_vocabulary_matches_erd_contract() {
        for kind in [
            RelationKind::Causes,
            RelationKind::Enables,
            RelationKind::IntervenesOn,
            RelationKind::LeadsTo,
            RelationKind::Produces,
            RelationKind::TransitionsTo,
            RelationKind::InputTo,
            RelationKind::ProcessTo,
        ] {
            assert!(kind.is_transition_edge());
            assert_eq!(
                RelationKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
    }

    #[test]
    fn provenance_vocabulary_is_never_a_transition() {
        for kind in [
            RelationKind::References,
            RelationKind::Summarizes,
            RelationKind::Revises,
            RelationKind::Translates,
            RelationKind::RetrospectivelyReports,
            RelationKind::Supports,
            RelationKind::Contradicts,
            RelationKind::OutcomeOf,
        ] {
            assert!(!kind.is_transition_edge());
            assert_eq!(
                RelationKind::from_wire_name(kind.wire_name()).expect("round-trip"),
                kind
            );
        }
    }

    #[test]
    fn unknown_wire_name_fails_closed() {
        assert_eq!(
            RelationKind::from_wire_name("causes_maybe"),
            Err(RelationError::UnknownRelationKind)
        );
    }
}
