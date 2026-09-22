//! Rolling-origin partitions bind cutoff snapshots before scientific evaluation.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, CorpusSplitError, LeakageLink, LeakageLinkKind,
    admit_rolling_origin_partition,
};
use std::collections::BTreeSet;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

fn cutoff(day: u8) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("cutoff")
}

fn available(day: u8) -> AvailableTime {
    AvailableTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("available")
}

fn document(id: u128, available_day: u8) -> CorpusDocument {
    CorpusDocument::new(Uuid::from_u128(id), available(available_day))
}

fn snapshots() -> (CorpusSnapshot, CorpusSnapshot, [Uuid; 4]) {
    let ids = [
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(3),
        Uuid::from_u128(4),
    ];
    let mut train = CorpusSnapshot::new();
    for id in [1_u128, 2] {
        train
            .insert_if_eligible(document(id, 1), &cutoff(10))
            .expect("train document");
    }
    let mut evaluation = CorpusSnapshot::new();
    for (id, day) in [(1_u128, 1_u8), (2, 1), (3, 15), (4, 15)] {
        evaluation
            .insert_if_eligible(document(id, day), &cutoff(20))
            .expect("evaluation snapshot document");
    }
    (train, evaluation, ids)
}

#[test]
fn rolling_origin_partition_binds_newly_available_rows_to_canonical_window() {
    let (train, evaluation, ids) = snapshots();
    let partition = admit_rolling_origin_partition(
        &[cutoff(10), cutoff(20)],
        0,
        &train,
        &evaluation,
        &[ids[0], ids[1]],
        &[ids[2], ids[3]],
        &[],
    )
    .expect("rolling-origin partition");

    assert_eq!(partition.window().train_cutoff, cutoff(10));
    assert_eq!(partition.window().test_cutoff, cutoff(20));
    assert_eq!(
        partition.training_document_ids(),
        &BTreeSet::from([ids[0], ids[1]])
    );
    assert_eq!(
        partition.evaluation_document_ids(),
        &BTreeSet::from([ids[2], ids[3]])
    );
}

#[test]
fn rolling_origin_partition_rejects_temporal_and_connected_group_leakage() {
    let (train, evaluation, ids) = snapshots();
    let cutoffs = [cutoff(10), cutoff(20)];

    assert_eq!(
        admit_rolling_origin_partition(
            &cutoffs,
            0,
            &train,
            &evaluation,
            &[ids[0], ids[1]],
            &[ids[0], ids[2]],
            &[],
        ),
        Err(CorpusSplitError::InvalidSplitConfiguration)
    );
    assert_eq!(
        admit_rolling_origin_partition(
            &cutoffs,
            0,
            &train,
            &evaluation,
            &[ids[0], ids[1]],
            &[ids[2], ids[3]],
            &[LeakageLink {
                left: ids[1],
                right: ids[2],
                kind: LeakageLinkKind::CopiedVariant,
            }],
        ),
        Err(CorpusSplitError::RelationLeakage)
    );
    assert_eq!(
        admit_rolling_origin_partition(
            &cutoffs,
            0,
            &train,
            &evaluation,
            &[ids[0], ids[2]],
            &[ids[3]],
            &[],
        ),
        Err(CorpusSplitError::UnavailableAtCutoff)
    );
}

#[test]
fn rolling_origin_partition_rejects_wrong_window_and_snapshot_horizon() {
    let (train, evaluation, ids) = snapshots();
    assert_eq!(
        admit_rolling_origin_partition(
            &[cutoff(10), cutoff(20)],
            1,
            &train,
            &evaluation,
            &[ids[0], ids[1]],
            &[ids[2], ids[3]],
            &[],
        ),
        Err(CorpusSplitError::InvalidSplitConfiguration)
    );

    let mut wrong_evaluation = CorpusSnapshot::new();
    wrong_evaluation
        .insert_if_eligible(document(3, 15), &cutoff(25))
        .expect("wrong-horizon evaluation document");
    assert_eq!(
        admit_rolling_origin_partition(
            &[cutoff(10), cutoff(20)],
            0,
            &train,
            &wrong_evaluation,
            &[ids[0], ids[1]],
            &[ids[2]],
            &[],
        ),
        Err(CorpusSplitError::KnowledgeCutoffMismatch)
    );
}
