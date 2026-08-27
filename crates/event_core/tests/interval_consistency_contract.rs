//! #170 interval-consistency contract against Allen (1983) / CHRONOS (2013).

use event_core::{
    EventError, IntervalConsistencyNetwork,
    refuse_interval_consistency_as_unrestricted_satisfiability,
    refuse_interval_contradiction_as_instance,
};
use temporal_core::{
    AllenRelation, EventTime, RelationSet, TemporalBoundary, TemporalInterval, TemporalPrecision,
};

fn closed(start: &str, end: &str) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(EventTime::parse_rfc3339(start).expect("start")),
        TemporalBoundary::Included(EventTime::parse_rfc3339(end).expect("end")),
        TemporalPrecision::Second,
    )
    .expect("proper closed interval")
}

#[test]
fn quantitative_chain_recovers_composed_before_with_zero_rmse_vs_truth() {
    let mut network = IntervalConsistencyNetwork::with_limits(8, 16, 256).expect("limits");
    let a = network.add_variable().expect("a");
    let b = network.add_variable().expect("b");
    let c = network.add_variable().expect("c");
    let early = closed("2026-08-01T00:00:00Z", "2026-08-02T00:00:00Z");
    let mid = closed("2026-08-03T00:00:00Z", "2026-08-04T00:00:00Z");
    let late = closed("2026-08-05T00:00:00Z", "2026-08-06T00:00:00Z");
    network
        .assert_quantitative_allen_relation(a, b, &early, &mid)
        .expect("a before b");
    network
        .assert_quantitative_allen_relation(b, c, &mid, &late)
        .expect("b before c");
    let report = network.close().expect("consistent");
    let ac = network.relation_set(a, c).expect("a to c");
    let truth_contains_before = ac.contains(AllenRelation::Before);
    let truth_contains_after = ac.contains(AllenRelation::After);
    let before_error = f64::from(u8::from(!truth_contains_before));
    let after_error = f64::from(u8::from(truth_contains_after));
    assert!(
        before_error < 1e-15,
        "Allen/CHRONOS: composed before must remain; RMSE {before_error}"
    );
    assert!(
        after_error < 1e-15,
        "Allen/CHRONOS: composed after must be excluded; RMSE {after_error}"
    );
    assert_eq!(
        refuse_interval_consistency_as_unrestricted_satisfiability(&report),
        Err(EventError::IntervalConsistencyIsNotUnrestrictedSatisfiability)
    );
}

#[test]
fn composition_contradiction_fails_closed_without_instance_promotion() {
    let mut network = IntervalConsistencyNetwork::with_limits(8, 16, 512).expect("limits");
    let a = network.add_variable().expect("a");
    let b = network.add_variable().expect("b");
    let c = network.add_variable().expect("c");
    network
        .assert_qualitative_relations(a, b, RelationSet::singleton(AllenRelation::Before))
        .expect("a before b");
    network
        .assert_qualitative_relations(b, c, RelationSet::singleton(AllenRelation::Before))
        .expect("b before c");
    network
        .assert_qualitative_relations(a, c, RelationSet::singleton(AllenRelation::After))
        .expect("assert after until close");
    let closed = network.close();
    assert_eq!(
        closed.map(|_| ()),
        Err(EventError::IntervalConsistencyContradiction)
    );
    assert_eq!(
        refuse_interval_contradiction_as_instance(EventError::IntervalConsistencyContradiction),
        Err(EventError::IntervalContradictionIsNotEventInstance)
    );
}
