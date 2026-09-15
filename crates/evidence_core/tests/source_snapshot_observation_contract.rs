//! Public contract for backdating-safe source-snapshot availability ownership.

use evidence_core::{SourceArtifact, SourceObservation, SourceSnapshotReceiptV1};
use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};

#[test]
fn snapshot_receipt_derives_clock_from_evidence_owned_observation() {
    let source_artifact = SourceArtifact::from_bytes(b"observed immutable snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let receipt = SourceSnapshotReceiptV1::from_source_observation(
        "snapshot-rubin-loading-v1",
        &observation,
    )
    .expect("receipt");

    assert_eq!(receipt.source_artifact_id(), observation.source_artifact_id().to_string());
    assert_eq!(
        receipt.source_snapshot_sha256(),
        observation.source_snapshot_sha256().to_string()
    );
    assert_eq!(receipt.system_observed_at(), observation.system_observed_at().to_rfc3339());
    assert_eq!(receipt.available_at(), observation.available_at().to_rfc3339());
    assert_eq!(observation.system_observed_at().instant(), observation.available_at().instant());
}

#[test]
fn historical_cutoff_before_evidence_observation_cannot_admit_the_receipt() {
    let source_artifact = SourceArtifact::from_bytes(b"late snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let receipt = SourceSnapshotReceiptV1::from_source_observation(
        "snapshot-rubin-loading-late-v1",
        &observation,
    )
    .expect("receipt");

    let available = AvailableTime::parse_rfc3339(receipt.available_at()).expect("availability");
    let system_observed =
        SystemTime::parse_rfc3339(receipt.system_observed_at()).expect("system observation");
    let historical_cutoff =
        KnowledgeCutoff::parse_rfc3339("2000-01-01T00:00:00Z").expect("cutoff");

    assert!(available.instant() > historical_cutoff.instant());
    assert_eq!(available.instant(), system_observed.instant());
}
