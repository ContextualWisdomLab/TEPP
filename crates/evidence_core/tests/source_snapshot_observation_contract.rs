//! Public contract for backdating-safe source-snapshot availability ownership.

use evidence_core::{
    SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
};
use temporal_core::{AvailableTime, KnowledgeCutoff, SystemTime};

#[test]
fn snapshot_receipt_derives_clock_from_evidence_owned_availability() {
    let source_artifact =
        SourceArtifact::from_bytes(b"observed immutable snapshot").expect("artifact");
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
    let historical_cutoff = KnowledgeCutoff::parse_rfc3339("2000-01-01T00:00:00Z").expect("cutoff");

    assert!(available.instant() > historical_cutoff.instant());
    assert!(available.instant() >= system_observed.instant());
}

#[test]
fn owner_records_have_distinct_identities_and_receipt_preserves_exact_lineage() {
    let source_artifact = SourceArtifact::from_bytes(b"same immutable snapshot").expect("artifact");
    let first_observation =
        SourceObservation::observe(&source_artifact).expect("first observation");
    let second_observation =
        SourceObservation::observe(&source_artifact).expect("second observation");

    assert_ne!(
        first_observation.observation_id(),
        second_observation.observation_id()
    );
    assert_eq!(
        first_observation.source_artifact_id(),
        second_observation.source_artifact_id()
    );
    assert_eq!(
        first_observation.source_snapshot_sha256(),
        second_observation.source_snapshot_sha256()
    );

    let first_availability =
        SourceAvailability::make_available(&first_observation).expect("first availability");
    let second_availability =
        SourceAvailability::make_available(&second_observation).expect("second availability");

    assert_ne!(
        first_availability.availability_id(),
        second_availability.availability_id()
    );
    assert_eq!(
        first_availability.source_observation_id(),
        first_observation.observation_id()
    );
    assert_eq!(
        second_availability.source_observation_id(),
        second_observation.observation_id()
    );

    let receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot-rubin-loading-owner-lineage-v1",
        &first_availability,
    )
    .expect("receipt");
    assert_eq!(
        receipt.source_observation_id(),
        first_observation.observation_id().to_string()
    );
    assert_eq!(
        receipt.source_availability_id(),
        first_availability.availability_id().to_string()
    );

    let canonical: serde_json::Value =
        serde_json::from_str(&receipt.to_json().expect("canonical receipt")).expect("json");
    assert_eq!(
        canonical["source_observation_id"].as_str(),
        Some(receipt.source_observation_id())
    );
    assert_eq!(
        canonical["source_availability_id"].as_str(),
        Some(receipt.source_availability_id())
    );
}
