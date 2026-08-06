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
        .assert_relation(
            first,
            second,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate");
    let second_constraint = reasoner
        .assert_relation(
            second,
            third,
            RelationSet::singleton(AllenRelation::Meets),
        )
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
        .assert_relation(
            first,
            second,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate");
    let second_constraint = reasoner
        .assert_relation(
            second,
            third,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate");
    let third_constraint = reasoner
        .assert_relation(
            third,
            first,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate before closure");

    let TemporalReasonerError::Contradiction(contradiction) =
        reasoner.close().expect_err("cycle must contradict before")
    else {
        panic!("expected contradiction evidence");
    };

    assert!(contradiction.support().contains(&first_constraint));
    assert!(contradiction.support().contains(&second_constraint));
    assert!(contradiction.support().contains(&third_constraint));
    assert_ne!(contradiction.left(), contradiction.right());
    assert_eq!(
        contradiction.to_string(),
        "temporal relation network is contradictory"
    );
}

#[test]
fn reasoner_rejects_invalid_limits_unknown_variables_empty_relations_and_capacity_overflow() {
    assert_eq!(
        TemporalReasonerLimits::new(0, 1, 1),
        Err(TemporalReasonerError::InvalidLimits)
    );

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
        .assert_relation(
            first,
            second,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("first constraint must fit");
    assert_eq!(
        bounded.assert_relation(
            first,
            second,
            RelationSet::singleton(AllenRelation::Meets),
        ),
        Err(TemporalReasonerError::LimitExceeded(
            ReasonerLimitKind::Constraints
        ))
    );

    let mut other = TemporalReasoner::with_limits(limits(3, 3, 10));
    other.add_variable().expect("variable must fit");
    other.add_variable().expect("variable must fit");
    let foreign = other.add_variable().expect("variable must fit");
    assert_eq!(
        bounded.relation(first, foreign),
        Err(TemporalReasonerError::UnknownVariable)
    );

    let mut empty_guard = TemporalReasoner::with_limits(limits(2, 2, 10));
    let left = empty_guard.add_variable().expect("variable must fit");
    let right = empty_guard.add_variable().expect("variable must fit");
    assert_eq!(
        empty_guard.assert_relation(left, right, RelationSet::empty()),
        Err(TemporalReasonerError::EmptyRelationSet)
    );
}

#[test]
fn propagation_work_is_bounded_and_fails_closed() {
    let mut reasoner = TemporalReasoner::with_limits(limits(3, 3, 1));
    let first = reasoner.add_variable().expect("variable must fit");
    let second = reasoner.add_variable().expect("variable must fit");
    let third = reasoner.add_variable().expect("variable must fit");

    reasoner
        .assert_relation(
            first,
            second,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate");
    reasoner
        .assert_relation(
            second,
            third,
            RelationSet::singleton(AllenRelation::Before),
        )
        .expect("constraint must validate");

    assert_eq!(
        reasoner.close(),
        Err(TemporalReasonerError::LimitExceeded(
            ReasonerLimitKind::PropagationSteps
        ))
    );
}
