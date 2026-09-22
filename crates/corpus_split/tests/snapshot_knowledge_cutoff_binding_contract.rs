//! Contract for one knowledge horizon per cutoff-qualified corpus snapshot.

use corpus_split::{CorpusDocument, CorpusSnapshot};
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

    snapshot
        .insert_if_eligible(
            CorpusDocument::new(
                Uuid::from_u128(1),
                available("2026-01-01T00:00:00Z"),
            ),
            &first_cutoff,
        )
        .expect("first eligible document binds the snapshot horizon");

    let mixed_horizon = snapshot.insert_if_eligible(
        CorpusDocument::new(
            Uuid::from_u128(2),
            available("2026-03-01T00:00:00Z"),
        ),
        &later_cutoff,
    );
    assert!(
        mixed_horizon.is_err(),
        "one snapshot must not combine documents admitted under different knowledge cutoffs"
    );

    snapshot
        .insert_if_eligible(
            CorpusDocument::new(
                Uuid::from_u128(3),
                available("2026-01-15T00:00:00Z"),
            ),
            &first_cutoff,
        )
        .expect("the bound knowledge cutoff remains reusable");
}
