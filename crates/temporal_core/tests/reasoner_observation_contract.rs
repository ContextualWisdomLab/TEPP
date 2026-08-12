//! Direct-observation provenance contracts for inverse temporal relations.

use temporal_core::{AllenRelation, RelationSet, TemporalReasoner, TemporalReasonerLimits};

fn reasoner() -> TemporalReasoner {
    let limits = TemporalReasonerLimits::new(8, 16, 1_000).expect("limits must validate");
    TemporalReasoner::with_limits(limits)
}

#[test]
fn inverse_propagation_does_not_fabricate_direct_observation() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left variable must fit");
    let right = reasoner.add_variable().expect("right variable must fit");

    reasoner
        .assert_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("direct assertion must validate");

    assert!(
        reasoner
            .relation(left, right)
            .expect("forward relation must exist")
            .is_observed()
    );
    assert!(
        !reasoner
            .relation(right, left)
            .expect("inverse relation must exist")
            .is_observed()
    );
}

#[test]
fn direct_assertions_in_both_directions_remain_observed() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left variable must fit");
    let right = reasoner.add_variable().expect("right variable must fit");

    reasoner
        .assert_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("forward assertion must validate");
    reasoner
        .assert_relation(right, left, RelationSet::singleton(AllenRelation::After))
        .expect("reverse assertion must validate");

    assert!(
        reasoner
            .relation(left, right)
            .expect("forward relation must exist")
            .is_observed()
    );
    assert!(
        reasoner
            .relation(right, left)
            .expect("reverse relation must exist")
            .is_observed()
    );
}

#[test]
fn closure_preserves_observation_on_the_direction_actually_asserted() {
    let mut reasoner = reasoner();
    let first = reasoner.add_variable().expect("first variable must fit");
    let middle = reasoner.add_variable().expect("middle variable must fit");
    let last = reasoner.add_variable().expect("last variable must fit");

    reasoner
        .assert_relation(
            last,
            first,
            RelationSet::from_relations(&[AllenRelation::After, AllenRelation::MetBy]),
        )
        .expect("reverse-direction assertion must validate");
    reasoner
        .assert_relation(first, middle, RelationSet::singleton(AllenRelation::Before))
        .expect("first path assertion must validate");
    reasoner
        .assert_relation(middle, last, RelationSet::singleton(AllenRelation::Before))
        .expect("second path assertion must validate");

    reasoner.close().expect("network must close consistently");

    let derived_forward = reasoner
        .relation(first, last)
        .expect("derived forward relation must exist");
    assert_eq!(
        derived_forward.relations(),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert!(!derived_forward.is_observed());

    let observed_reverse = reasoner
        .relation(last, first)
        .expect("observed reverse relation must exist");
    assert_eq!(
        observed_reverse.relations(),
        RelationSet::singleton(AllenRelation::After)
    );
    assert!(observed_reverse.is_observed());
}

#[test]
fn direct_identity_assertion_remains_observed() {
    let mut reasoner = reasoner();
    let variable = reasoner.add_variable().expect("variable must fit");

    reasoner
        .assert_relation(
            variable,
            variable,
            RelationSet::singleton(AllenRelation::Equals),
        )
        .expect("identity assertion must validate");

    assert!(
        reasoner
            .relation(variable, variable)
            .expect("identity relation must exist")
            .is_observed()
    );
}
