//! Association and temporal precedence are not causal identification.

use relation_graph::{RelationError, RelationKind, refuse_association_as_cause};

#[test]
fn identified_causal_vocabulary_is_allowed_and_associations_are_not() {
    refuse_association_as_cause(RelationKind::Causes).expect("causes");
    refuse_association_as_cause(RelationKind::IntervenesOn).expect("intervention");

    for kind in [
        RelationKind::LeadsTo,
        RelationKind::Enables,
        RelationKind::References,
        RelationKind::Summarizes,
        RelationKind::Revises,
        RelationKind::Translates,
        RelationKind::RetrospectivelyReports,
        RelationKind::Supports,
        RelationKind::Contradicts,
        RelationKind::OutcomeOf,
        RelationKind::InputTo,
        RelationKind::ProcessTo,
        RelationKind::Produces,
        RelationKind::TransitionsTo,
    ] {
        assert_eq!(
            refuse_association_as_cause(kind),
            Err(RelationError::CausalClaimNotIdentified),
            "{kind:?} must not be treated as identified causation"
        );
    }
}
