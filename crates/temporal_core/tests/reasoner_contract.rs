//! Bounded temporal-reasoner closure and provenance contracts.

use temporal_core::{
    AllenRelation, ReasonerLimitKind, RelationSet, TemporalReasoner, TemporalReasonerError,
    TemporalReasonerLimits,
};

fn limits(
    maximum_variables: usize,
    maximum_constraints: usize,
    maximum_propagation_steps: usize,
) -> TemporalReasonerLimits {
    TemporalReasonerLimits::new(
        maximum_variables,
        maximum_constraints,
        maximum_propagation_steps,
    )
    .expect("test limits must validate")
}

#[test]
fn closure_derives_inverse_relations_with_conservative_provenance() {
    let mut reasoner = TemporalReasoner::with_limits(limits(8, 16, 1_000));
    let first = reasoner.add_variable().expect("variable must fit");
    let second = reasoner.add_variable().expect("variable must fit");
    let third = reasoner.add_variable().expect("variable must fit");

    let first_constraint = reasoner
        .assert_relation(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate");
    let second_constraint = reasoner
        .assert_relation(second, third, RelationSet::singleton(AllenRelation::Meets))
        .expect("constraint must validate");

    let report = reasoner.close().expect("network must be consistent");
    assert!(report.changed());
    assert!(report.revisions() > 0);
    assert!(report.propagation_steps() >= report.revisions());

    let observed = reasoner
        .relation(first, second)
        .expect("observed relation must exist");
    assert_eq!(
        observed.relations(),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert!(observed.is_observed());
    assert_eq!(observed.support(), &[first_constraint]);

    let derived = reasoner
        .relation(first, third)
        .expect("derived relation must exist");
    assert_eq!(
        derived.relations(),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert!(!derived.is_observed());
    assert!(derived.support().contains(&first_constraint));
    assert!(derived.support().contains(&second_constraint));

    let inverse = reasoner
        .relation(third, first)
        .expect("inverse relation must exist");
    assert_eq!(
        inverse.relations(),
        RelationSet::singleton(AllenRelation::After)
    );
    assert_eq!(inverse.support(), derived.support());

    let second_report = reasoner.close().expect("closed network must remain valid");
    assert!(!second_report.changed());
    assert_eq!(second_report.revisions(), 0);
}

#[test]
fn contradictory_cycles_return_the_supporting_assertions() {
    let mut reasoner = TemporalReasoner::with_limits(limits(8, 16, 1_000));
    let first = reasoner.add_variable().expect("variable must fit");
    let second = reasoner.add_variable().expect("variable must fit");
    let third = reasoner.add_variable().expect("variable must fit");

    let first_constraint = reasoner
        .assert_relation(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate");
    let second_constraint = reasoner
        .assert_relation(second, third, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate");
    let third_constraint = reasoner
        .assert_relation(third, first, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate before closure");

    let TemporalReasonerError::Contradiction(contradiction) =
        reasoner.close().expect_err("cycle must contradict before")
    else {
        panic!("expected contradiction evidence");
    };

    assert!(contradiction.support().contains(&first_constraint));
    assert!(contradiction.support().contains(&second_constraint));
    assert!(contradiction.support().contains(&third_constraint));
    assert_eq!(contradiction.attempted_relations(), None);
    assert_ne!(contradiction.left(), contradiction.right());
    assert_eq!(
        contradiction.to_string(),
        "temporal relation network is contradictory"
    );
}

#[test]
fn direct_contradiction_is_atomic_and_does_not_fabricate_an_accepted_identifier() {
    let mut reasoner = TemporalReasoner::with_limits(limits(2, 3, 100));
    let left = reasoner.add_variable().expect("left variable must fit");
    let right = reasoner.add_variable().expect("right variable must fit");
    let before = RelationSet::singleton(AllenRelation::Before);
    let after = RelationSet::singleton(AllenRelation::After);

    let first_constraint = reasoner
        .assert_relation(left, right, before)
        .expect("first assertion must validate");
    let TemporalReasonerError::Contradiction(contradiction) = reasoner
        .assert_relation(left, right, after)
        .expect_err("incompatible direct assertion must fail")
    else {
        panic!("expected direct contradiction evidence");
    };

    assert_eq!(contradiction.left(), left);
    assert_eq!(contradiction.right(), right);
    assert_eq!(contradiction.support(), &[first_constraint]);
    assert_eq!(contradiction.attempted_relations(), Some(after));

    let unchanged = reasoner
        .relation(left, right)
        .expect("rejected assertion must leave relation intact");
    assert_eq!(unchanged.relations(), before);
    assert_eq!(unchanged.support(), &[first_constraint]);

    let second_constraint = reasoner
        .assert_relation(left, right, before)
        .expect("a later compatible assertion must remain admissible");
    assert_ne!(first_constraint, second_constraint);
    let repeated = reasoner
        .relation(left, right)
        .expect("accepted repeated assertion must be observable");
    assert_eq!(repeated.support(), &[first_constraint, second_constraint]);

    let mut other = TemporalReasoner::with_limits(limits(2, 1, 100));
    let other_left = other.add_variable().expect("other left must fit");
    let other_right = other.add_variable().expect("other right must fit");
    let other_constraint = other
        .assert_relation(other_left, other_right, before)
        .expect("other assertion must validate");
    assert_ne!(first_constraint, other_constraint);
}

#[test]
fn reasoner_rejects_invalid_limits_foreign_variables_empty_relations_and_capacity_overflow() {
    for invalid_limits in [(0, 1, 1), (1, 0, 1), (1, 1, 0)] {
        assert_eq!(
            TemporalReasonerLimits::new(invalid_limits.0, invalid_limits.1, invalid_limits.2),
            Err(TemporalReasonerError::InvalidLimits)
        );
    }

    let mut bounded = TemporalReasoner::with_limits(limits(2, 1, 10));
    let first = bounded.add_variable().expect("first variable must fit");
    let second = bounded.add_variable().expect("second variable must fit");
    assert_eq!(
        bounded.add_variable(),
        Err(TemporalReasonerError::LimitExceeded(
            ReasonerLimitKind::Variables
        ))
    );

    bounded
        .assert_relation(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("first constraint must fit");
    assert_eq!(
        bounded.assert_relation(first, second, RelationSet::empty()),
        Err(TemporalReasonerError::EmptyRelationSet)
    );
    assert_eq!(
        bounded.assert_relation(first, second, RelationSet::singleton(AllenRelation::Meets)),
        Err(TemporalReasonerError::LimitExceeded(
            ReasonerLimitKind::Constraints
        ))
    );

    let mut other = TemporalReasoner::with_limits(limits(2, 2, 10));
    let foreign_same_index = other.add_variable().expect("foreign variable must fit");
    assert_eq!(
        bounded.relation(first, foreign_same_index),
        Err(TemporalReasonerError::UnknownVariable)
    );
    assert_eq!(
        bounded.assert_relation(
            first,
            foreign_same_index,
            RelationSet::singleton(AllenRelation::Before),
        ),
        Err(TemporalReasonerError::UnknownVariable)
    );
}

#[test]
fn propagation_work_is_bounded_and_failure_restores_the_preclosure_network() {
    let mut reasoner = TemporalReasoner::with_limits(limits(3, 3, 1));
    let first = reasoner.add_variable().expect("variable must fit");
    let second = reasoner.add_variable().expect("variable must fit");
    let third = reasoner.add_variable().expect("variable must fit");

    reasoner
        .assert_relation(first, second, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate");
    reasoner
        .assert_relation(second, third, RelationSet::singleton(AllenRelation::Before))
        .expect("constraint must validate");

    assert_eq!(
        reasoner.close(),
        Err(TemporalReasonerError::LimitExceeded(
            ReasonerLimitKind::PropagationSteps
        ))
    );
    let restored = reasoner
        .relation(first, third)
        .expect("rollback must preserve an unconstrained pair");
    assert_eq!(restored.relations(), RelationSet::all());
    assert!(!restored.is_observed());
    assert!(restored.support().is_empty());
}

#[test]
fn reasoner_error_messages_are_stable_and_content_redacting() {
    let errors = [
        (
            TemporalReasonerError::InvalidLimits,
            "invalid temporal reasoner limits",
        ),
        (
            TemporalReasonerError::UnknownVariable,
            "unknown temporal reasoner variable",
        ),
        (
            TemporalReasonerError::EmptyRelationSet,
            "temporal relation set is empty",
        ),
        (
            TemporalReasonerError::LimitExceeded(ReasonerLimitKind::Variables),
            "temporal reasoner resource limit exceeded",
        ),
    ];

    for (error, message) in errors {
        assert_eq!(error.to_string(), message);
    }

    let mut reasoner = TemporalReasoner::with_limits(limits(1, 1, 10));
    let variable = reasoner.add_variable().expect("variable must fit");
    let contradiction = reasoner
        .assert_relation(
            variable,
            variable,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect_err("self-before must contradict identity");
    assert_eq!(
        contradiction.to_string(),
        "temporal relation network is contradictory"
    );
}
