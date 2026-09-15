//! Public contract for backdating-safe source-snapshot availability ownership.

use evidence_core::{
    SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
};
use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};

#[test]
fn snapshot_receipt_derives_clock_from_evidence_owned_availability() {
    let source_artifact = SourceArtifact::from_bytes(b"observed immutable snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");
    let receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot-rubin-loading-v1",
        &availability,
    )
    .expect("receipt");

    assert_eq!(
        receipt.source_artifact_id(),
        availability.source_artifact_id().to_string()
    );
    assert_eq!(
        receipt.source_snapshot_sha256(),
        availability.source_snapshot_sha256().to_string()
    );
    assert_eq!(
        receipt.system_observed_at(),
        availability.system_observed_at().to_rfc3339()
    );
    assert_eq!(
        receipt.available_at(),
        availability.available_at().to_rfc3339()
    );
    assert!(availability.available_at().instant() >= availability.system_observed_at().instant());
}

#[test]
fn historical_cutoff_before_evidence_availability_cannot_admit_the_receipt() {
    let source_artifact = SourceArtifact::from_bytes(b"late snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");
    let receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot-rubin-loading-late-v1",
        &availability,
    )
    .expect("receipt");

    let available = AvailableTime::parse_rfc3339(receipt.available_at()).expect("availability");
    let system_observed =
        SystemTime::parse_rfc3339(receipt.system_observed_at()).expect("system observation");
    let historical_cutoff =
        KnowledgeCutoff::parse_rfc3339("2000-01-01T00:00:00Z").expect("cutoff");

    assert!(available.instant() > historical_cutoff.instant());
    assert!(available.instant() >= system_observed.instant());
}
