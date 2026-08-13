//! Predicted CHRONOS/TDT relations stay hypothetical and cannot rewrite observations.

use temporal_core::{
    AllenRelation, RelationSet, TemporalReasoner, TemporalReasonerError, TemporalReasonerLimits,
};

fn reasoner() -> TemporalReasoner {
    let limits = TemporalReasonerLimits::new(8, 16, 1_000).expect("limits must validate");
    TemporalReasoner::with_limits(limits)
}

#[test]
fn predicted_assertion_is_hypothetical_not_observed() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left");
    let right = reasoner.add_variable().expect("right");

    reasoner
        .assert_predicted_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("predicted assertion must validate");

    let forward = reasoner.relation(left, right).expect("forward");
    assert_eq!(
        forward.relations(),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert!(!forward.is_observed());
    assert!(forward.is_predicted());
    assert!(
        !reasoner
            .relation(right, left)
            .expect("inverse")
            .is_observed()
    );
}

#[test]
fn predicted_contradiction_against_observation_rejects_prediction_only() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left");
    let right = reasoner.add_variable().expect("right");

    reasoner
        .assert_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("observed assertion");

    let rejected = reasoner
        .assert_predicted_relation(left, right, RelationSet::singleton(AllenRelation::After))
        .expect_err("prediction must not rewrite observed Before");
    assert!(matches!(
        rejected,
        TemporalReasonerError::PredictedRelationRejected(_)
    ));

    let forward = reasoner.relation(left, right).expect("forward");
    assert_eq!(
        forward.relations(),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert!(forward.is_observed());
    assert!(!forward.is_predicted());
}

#[test]
fn predicted_assertion_cannot_narrow_an_observed_relation_set() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left");
    let right = reasoner.add_variable().expect("right");
    let observed = RelationSet::from_relations(&[AllenRelation::Before, AllenRelation::Meets]);

    reasoner
        .assert_relation(left, right, observed)
        .expect("observed disjunction");
    reasoner
        .assert_predicted_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("compatible prediction must be accepted without tightening observation");

    let forward = reasoner.relation(left, right).expect("forward");
    assert_eq!(forward.relations(), observed);
    assert!(forward.is_observed());
    assert!(!forward.is_predicted());
}

#[test]
fn predicted_only_empty_intersection_is_a_network_contradiction() {
    let mut reasoner = reasoner();
    let left = reasoner.add_variable().expect("left");
    let right = reasoner.add_variable().expect("right");

    reasoner
        .assert_predicted_relation(left, right, RelationSet::singleton(AllenRelation::Before))
        .expect("first prediction");
    let error = reasoner
        .assert_predicted_relation(left, right, RelationSet::singleton(AllenRelation::After))
        .expect_err("incompatible predictions contradict");
    assert!(matches!(error, TemporalReasonerError::Contradiction(_)));
}
