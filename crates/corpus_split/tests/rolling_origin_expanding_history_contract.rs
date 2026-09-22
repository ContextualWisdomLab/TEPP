//! Expanding rolling-origin history is derived by the split owner and excludes current evaluation leakage groups.

use std::collections::BTreeSet;

use corpus_split::{
    CorpusDocument, CorpusSnapshot, LeakageLink, LeakageLinkKind,
    admit_expanding_rolling_origin_partition,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

fn cutoff(day: u8) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("cutoff")
}

fn available(day: u8) -> AvailableTime {
    AvailableTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("available")
}

fn insert(snapshot: &mut CorpusSnapshot, document_id: Uuid, day: u8, cutoff: &KnowledgeCutoff) {
    snapshot
        .insert_if_eligible(CorpusDocument::new(document_id, available(day)), cutoff)
        .expect("eligible document");
}

#[test]
fn expanding_history_retains_unrelated_history_and_audits_evaluation_connected_exclusions() {
    let cutoffs = [cutoff(10), cutoff(20), cutoff(30)];
    let unrelated_old = Uuid::from_u128(1);
    let prior_evaluation_now_historical = Uuid::from_u128(2);
    let linked_old = Uuid::from_u128(3);
    let linked_old_transitive = Uuid::from_u128(4);
    let current_evaluation = Uuid::from_u128(5);

    let mut training_snapshot = CorpusSnapshot::new();
    insert(&mut training_snapshot, unrelated_old, 1, &cutoffs[1]);
    insert(
        &mut training_snapshot,
        prior_evaluation_now_historical,
        15,
        &cutoffs[1],
    );
    insert(&mut training_snapshot, linked_old, 5, &cutoffs[1]);
    insert(
        &mut training_snapshot,
        linked_old_transitive,
        6,
        &cutoffs[1],
    );

    let mut evaluation_snapshot = CorpusSnapshot::new();
    for (document_id, day) in [
        (unrelated_old, 1),
        (prior_evaluation_now_historical, 15),
        (linked_old, 5),
        (linked_old_transitive, 6),
        (current_evaluation, 25),
    ] {
        insert(&mut evaluation_snapshot, document_id, day, &cutoffs[2]);
    }

    let links = [
        LeakageLink {
            left: current_evaluation,
            right: linked_old,
            kind: LeakageLinkKind::Revision,
        },
        LeakageLink {
            left: linked_old,
            right: linked_old_transitive,
            kind: LeakageLinkKind::Translation,
        },
    ];

    let admitted = admit_expanding_rolling_origin_partition(
        &cutoffs,
        1,
        &training_snapshot,
        &evaluation_snapshot,
        &[current_evaluation],
        &links,
    )
    .expect("leakage-safe expanding partition");

    assert_eq!(
        admitted.partition().training_document_ids(),
        &BTreeSet::from([unrelated_old, prior_evaluation_now_historical])
    );
    assert_eq!(
        admitted.partition().evaluation_document_ids(),
        &BTreeSet::from([current_evaluation])
    );
    assert_eq!(
        admitted.leakage_excluded_training_document_ids(),
        &BTreeSet::from([linked_old, linked_old_transitive])
    );
}
