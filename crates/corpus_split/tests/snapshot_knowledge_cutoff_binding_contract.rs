//! Contract for one knowledge horizon per cutoff-qualified corpus snapshot.

use corpus_split::{CorpusDocument, CorpusSnapshot, CorpusSplitError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

fn available(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("fixture availability must parse")
}

fn cutoff(value: &str) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(value).expect("fixture cutoff must parse")
}

#[test]
fn one_snapshot_cannot_mix_distinct_knowledge_cutoffs() {
    let first_cutoff = cutoff("2026-02-01T00:00:00Z");
    let later_cutoff = cutoff("2026-04-01T00:00:00Z");
    let mut snapshot = CorpusSnapshot::new();
    assert_eq!(snapshot.knowledge_cutoff(), None);

    snapshot
        .insert_if_eligible(
            CorpusDocument::new(
                Uuid::from_u128(1),
                available("2026-01-01T00:00:00Z"),
            ),
            &first_cutoff,
        )
        .expect("first eligible document binds the snapshot horizon");
    assert_eq!(snapshot.knowledge_cutoff(), Some(first_cutoff));

    let mixed_horizon = snapshot.insert_if_eligible(
        CorpusDocument::new(
            Uuid::from_u128(2),
            available("2026-03-01T00:00:00Z"),
        ),
        &later_cutoff,
    );
    assert_eq!(
        mixed_horizon,
        Err(CorpusSplitError::KnowledgeCutoffMismatch),
        "one snapshot must not combine documents admitted under different knowledge cutoffs"
    );
    assert_eq!(snapshot.knowledge_cutoff(), Some(first_cutoff));

    snapshot
        .insert_if_eligible(
            CorpusDocument::new(
                Uuid::from_u128(3),
                available("2026-01-15T00:00:00Z"),
            ),
            &first_cutoff,
        )
        .expect("the bound knowledge cutoff remains reusable");
    assert_eq!(snapshot.knowledge_cutoff(), Some(first_cutoff));
}

#[test]
fn failed_first_admission_does_not_bind_the_snapshot_horizon() {
    let early_cutoff = cutoff("2026-02-01T00:00:00Z");
    let later_cutoff = cutoff("2026-04-01T00:00:00Z");
    let document = CorpusDocument::new(
        Uuid::from_u128(4),
        available("2026-03-01T00:00:00Z"),
    );
    let mut snapshot = CorpusSnapshot::new();

    assert_eq!(
        snapshot.insert_if_eligible(document.clone(), &early_cutoff),
        Err(CorpusSplitError::UnavailableAtCutoff)
    );
    assert_eq!(snapshot.knowledge_cutoff(), None);

    snapshot
        .insert_if_eligible(document, &later_cutoff)
        .expect("a failed admission must not claim the snapshot horizon");
    assert_eq!(snapshot.knowledge_cutoff(), Some(later_cutoff));
}
