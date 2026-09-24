//! Rolling-origin evaluation cannot relabel evidence already present at the prior cutoff.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, CorpusSplitError, admit_rolling_origin_partition,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use uuid::Uuid;

fn cutoff(day: u8) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("cutoff")
}

fn available(day: u8) -> AvailableTime {
    AvailableTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("available")
}

fn document(document_id: Uuid, available_day: u8) -> CorpusDocument {
    CorpusDocument::new(document_id, available(available_day))
}

#[test]
fn evaluation_rejects_identity_already_present_in_training_snapshot() {
    let previously_available = Uuid::from_u128(1);
    let retained_training = Uuid::from_u128(2);
    let mut training_snapshot = CorpusSnapshot::new();
    training_snapshot
        .insert_if_eligible(document(previously_available, 1), &cutoff(10))
        .expect("prior evidence");
    training_snapshot
        .insert_if_eligible(document(retained_training, 1), &cutoff(10))
        .expect("retained training evidence");

    // Rebinding the same UUID to a later AvailableTime must not turn evidence
    // proven present at the prior cutoff into newly available held-out evidence.
    let mut evaluation_snapshot = CorpusSnapshot::new();
    evaluation_snapshot
        .insert_if_eligible(document(previously_available, 15), &cutoff(20))
        .expect("later snapshot representation");

    assert_eq!(
        admit_rolling_origin_partition(
            &[cutoff(10), cutoff(20)],
            0,
            &training_snapshot,
            &evaluation_snapshot,
            &[retained_training],
            &[previously_available],
            &[],
        ),
        Err(CorpusSplitError::InvalidSplitConfiguration)
    );
}
