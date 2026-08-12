//! Stable iteration contracts for qualitative relation sets.

use temporal_core::{AllenRelation, RelationSet};

#[test]
fn relation_sets_iterate_once_in_stable_elementary_order() {
    let selected = RelationSet::from_relations(&[
        AllenRelation::Meets,
        AllenRelation::Before,
        AllenRelation::Meets,
        AllenRelation::Equals,
    ]);

    assert_eq!(
        selected.iter().collect::<Vec<_>>(),
        vec![
            AllenRelation::Before,
            AllenRelation::Meets,
            AllenRelation::Equals,
        ]
    );
    assert_eq!(RelationSet::empty().iter().next(), None);
    assert_eq!(RelationSet::all().iter().count(), AllenRelation::ALL.len());
}
