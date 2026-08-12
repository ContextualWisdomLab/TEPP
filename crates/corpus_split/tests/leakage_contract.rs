//! Integration contracts for cutoff snapshots and relation-aware splits.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, CorpusSplitError, LeakageLink, LeakageLinkKind,
    assert_no_group_leakage, build_connected_groups, rolling_origin_windows,
};
use std::collections::BTreeMap;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

#[test]
fn retrospective_late_documents_cannot_enter_earlier_cutoffs() {
    let mut snapshot = CorpusSnapshot::new();
    let cutoff = KnowledgeCutoff::parse_rfc3339("2025-12-31T00:00:00Z").expect("cutoff");
    let retrospective = CorpusDocument::new(
        Uuid::now_v7(),
        AvailableTime::parse_rfc3339("2026-06-01T00:00:00Z").expect("available"),
    );
    assert_eq!(
        snapshot.insert_if_eligible(retrospective, &cutoff),
        Err(CorpusSplitError::UnavailableAtCutoff)
    );
    assert!(snapshot.is_empty());
}

#[test]
fn linked_variants_never_cross_partitions() {
    let original = Uuid::now_v7();
    let revision = Uuid::now_v7();
    let translation = Uuid::now_v7();
    let copy = Uuid::now_v7();
    let episode_peer = Uuid::now_v7();
    let groups = build_connected_groups(
        &[original, revision, translation, copy, episode_peer],
        &[
            LeakageLink {
                left: original,
                right: revision,
                kind: LeakageLinkKind::Revision,
            },
            LeakageLink {
                left: original,
                right: translation,
                kind: LeakageLinkKind::Translation,
            },
            LeakageLink {
                left: original,
                right: copy,
                kind: LeakageLinkKind::CopiedVariant,
            },
            LeakageLink {
                left: original,
                right: episode_peer,
                kind: LeakageLinkKind::SameEpisode,
            },
        ],
    );
    assert_eq!(groups.len(), 1);
    let mut map = BTreeMap::new();
    for member in groups[0].members() {
        map.insert(*member, 0);
    }
    assert_no_group_leakage(&groups, &map).expect("co-located");
    map.insert(translation, 1);
    assert_eq!(
        assert_no_group_leakage(&groups, &map),
        Err(CorpusSplitError::RelationLeakage)
    );
}

#[test]
fn rolling_origin_uses_ordered_cutoffs() {
    let cutoffs = [
        KnowledgeCutoff::parse_rfc3339("2026-01-01T00:00:00Z").expect("1"),
        KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("2"),
        KnowledgeCutoff::parse_rfc3339("2026-03-01T00:00:00Z").expect("3"),
    ];
    let windows = rolling_origin_windows(&cutoffs).expect("windows");
    assert_eq!(windows.len(), 2);
}
