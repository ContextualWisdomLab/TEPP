//! Allen interval-relation and composition contracts.

use temporal_core::{
    AllenRelation, EventTime, RelationSet, TemporalBoundary, TemporalError, TemporalInterval,
    TemporalPrecision, classify_interval_relation,
};

fn time(second: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-01T00:00:{second:02}Z"))
        .expect("test instant must parse")
}

fn proper_interval(start: u8, end: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(time(start)),
        TemporalBoundary::Included(time(end)),
        TemporalPrecision::Second,
    )
    .expect("proper test interval must validate")
}

#[test]
fn every_elementary_relation_has_the_expected_inverse() {
    let inverse_pairs = [
        (AllenRelation::Before, AllenRelation::After),
        (AllenRelation::Meets, AllenRelation::MetBy),
        (AllenRelation::Overlaps, AllenRelation::OverlappedBy),
        (AllenRelation::Starts, AllenRelation::StartedBy),
        (AllenRelation::During, AllenRelation::Contains),
        (AllenRelation::Finishes, AllenRelation::FinishedBy),
        (AllenRelation::Equals, AllenRelation::Equals),
    ];

    for (relation, inverse) in inverse_pairs {
        assert_eq!(relation.inverse(), inverse);
        assert_eq!(inverse.inverse(), relation);
    }
    assert_eq!(AllenRelation::ALL.len(), 13);
}

#[test]
fn concrete_proper_intervals_classify_all_thirteen_relations() {
    let examples = [
        ((1, 2), (3, 4), AllenRelation::Before),
        ((3, 4), (1, 2), AllenRelation::After),
        ((1, 2), (2, 4), AllenRelation::Meets),
        ((2, 4), (1, 2), AllenRelation::MetBy),
        ((1, 3), (2, 4), AllenRelation::Overlaps),
        ((2, 4), (1, 3), AllenRelation::OverlappedBy),
        ((1, 3), (1, 4), AllenRelation::Starts),
        ((1, 4), (1, 3), AllenRelation::StartedBy),
        ((2, 3), (1, 4), AllenRelation::During),
        ((1, 4), (2, 3), AllenRelation::Contains),
        ((2, 4), (1, 4), AllenRelation::Finishes),
        ((1, 4), (2, 4), AllenRelation::FinishedBy),
        ((1, 4), (1, 4), AllenRelation::Equals),
    ];

    for ((left_start, left_end), (right_start, right_end), expected) in examples {
        let left = proper_interval(left_start, left_end);
        let right = proper_interval(right_start, right_end);
        assert_eq!(
            classify_interval_relation(&left, &right).expect("relation must classify"),
            expected
        );
    }

    let excluded = TemporalInterval::bounded(
        TemporalBoundary::Excluded(time(1)),
        TemporalBoundary::Excluded(time(2)),
        TemporalPrecision::Second,
    )
    .expect("excluded proper interval must validate");
    assert_eq!(
        classify_interval_relation(&excluded, &proper_interval(3, 4))
            .expect("boundary inclusion must not change endpoint classification"),
        AllenRelation::Before
    );
}

#[test]
fn qualitative_classification_rejects_exact_open_and_unknown_intervals() {
    let exact = TemporalInterval::exact(time(1), TemporalPrecision::Second)
        .expect("exact interval must validate");
    let upper_open = TemporalInterval::bounded(
        TemporalBoundary::Included(time(1)),
        TemporalBoundary::Unbounded,
        TemporalPrecision::Second,
    )
    .expect("upper-open interval must validate");
    let lower_open = TemporalInterval::bounded(
        TemporalBoundary::Unbounded,
        TemporalBoundary::Included(time(2)),
        TemporalPrecision::Second,
    )
    .expect("lower-open interval must validate");
    let unknown = TemporalInterval::<EventTime>::unknown();
    let proper = proper_interval(1, 2);

    for invalid in [exact, upper_open, lower_open, unknown] {
        assert_eq!(
            classify_interval_relation(&invalid, &proper),
            Err(TemporalError::RelationRequiresProperBoundedInterval)
        );
        assert_eq!(
            classify_interval_relation(&proper, &invalid),
            Err(TemporalError::RelationRequiresProperBoundedInterval)
        );
    }
}

#[test]
fn relation_sets_support_inverse_intersection_and_complete_composition() {
    let overlaps_twice = RelationSet::singleton(AllenRelation::Overlaps)
        .compose(RelationSet::singleton(AllenRelation::Overlaps));
    assert_eq!(
        overlaps_twice,
        RelationSet::from_relations(&[
            AllenRelation::Before,
            AllenRelation::Meets,
            AllenRelation::Overlaps,
        ])
    );

    assert_eq!(
        RelationSet::singleton(AllenRelation::Before)
            .compose(RelationSet::singleton(AllenRelation::Before)),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert_eq!(
        RelationSet::singleton(AllenRelation::Meets)
            .compose(RelationSet::singleton(AllenRelation::Meets)),
        RelationSet::singleton(AllenRelation::Before)
    );
    assert_eq!(
        RelationSet::singleton(AllenRelation::Starts)
            .compose(RelationSet::singleton(AllenRelation::Finishes)),
        RelationSet::singleton(AllenRelation::During)
    );

    let selected = RelationSet::from_relations(&[AllenRelation::Before, AllenRelation::Meets]);
    assert_eq!(
        selected.inverse(),
        RelationSet::from_relations(&[AllenRelation::After, AllenRelation::MetBy])
    );
    assert_eq!(
        selected.intersection(RelationSet::singleton(AllenRelation::Meets)),
        RelationSet::singleton(AllenRelation::Meets)
    );
    assert_eq!(
        RelationSet::singleton(AllenRelation::Before)
            .union(RelationSet::singleton(AllenRelation::Meets)),
        selected
    );
    assert!(selected.contains(AllenRelation::Before));
    assert!(!selected.contains(AllenRelation::After));
    assert_eq!(selected.len(), 2);
    assert!(!selected.is_empty());
    assert!(RelationSet::empty().is_empty());
    assert_eq!(RelationSet::from_relations(&[]), RelationSet::empty());
    assert_eq!(RelationSet::all().len(), 13);
}

#[test]
fn composition_and_inverse_obey_the_converse_law_for_every_relation_pair() {
    for left in AllenRelation::ALL {
        for right in AllenRelation::ALL {
            let composed = RelationSet::singleton(left)
                .compose(RelationSet::singleton(right))
                .inverse();
            let reversed = RelationSet::singleton(right.inverse())
                .compose(RelationSet::singleton(left.inverse()));
            assert_eq!(composed, reversed);
            assert!(!composed.is_empty());
        }
    }
}

#[test]
fn composition_matches_an_exhaustive_endpoint_oracle_for_every_relation_pair() {
    let intervals: Vec<_> = (0_u8..8)
        .flat_map(|start| ((start + 1)..8).map(move |end| proper_interval(start, end)))
        .collect();
    let mut expected = [[RelationSet::empty(); 13]; 13];

    for left_interval in &intervals {
        for middle_interval in &intervals {
            let left_relation = classify_interval_relation(left_interval, middle_interval)
                .expect("proper intervals must classify");
            for right_interval in &intervals {
                let right_relation = classify_interval_relation(middle_interval, right_interval)
                    .expect("proper intervals must classify");
                let composed_relation = classify_interval_relation(left_interval, right_interval)
                    .expect("proper intervals must classify");
                expected[left_relation as usize][right_relation as usize] = expected
                    [left_relation as usize][right_relation as usize]
                    .union(RelationSet::singleton(composed_relation));
            }
        }
    }

    for left_relation in AllenRelation::ALL {
        for right_relation in AllenRelation::ALL {
            assert_eq!(
                RelationSet::singleton(left_relation)
                    .compose(RelationSet::singleton(right_relation)),
                expected[left_relation as usize][right_relation as usize]
            );
        }
    }
}
