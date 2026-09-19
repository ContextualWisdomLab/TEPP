//! Public contract for immutable Evidence-owned source-snapshot receipts.

use evidence_core::{
    EvidenceError, EvidenceId, SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT, SourceArtifact,
    SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
    ValidatedSourceSnapshotReceiptWireV1,
};
use std::str::FromStr;
use temporal_core::{AvailableTime, SystemTime};

const SNAPSHOT_ID: &str = "snapshot-rubin-loading";
const SOURCE_A_BYTES: &[u8] = b"canonical snapshot a";
const SOURCE_B_BYTES: &[u8] = b"canonical snapshot b";

fn receipt(source_bytes: &[u8]) -> SourceSnapshotReceiptV1 {
    let source_artifact = SourceArtifact::from_bytes(source_bytes).expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");
    SourceSnapshotReceiptV1::from_source_availability(SNAPSHOT_ID, &availability).expect("receipt")
}

#[test]
fn creation_mints_distinct_receipt_identity_inside_evidence() {
    let source_artifact = SourceArtifact::from_bytes(b"same source record").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");

    let first = SourceSnapshotReceiptV1::from_source_availability(SNAPSHOT_ID, &availability)
        .expect("first receipt");
    let second = SourceSnapshotReceiptV1::from_source_availability(SNAPSHOT_ID, &availability)
        .expect("second receipt");

    assert_ne!(first.receipt_id(), second.receipt_id());
    assert!(EvidenceId::from_str(first.receipt_id()).is_ok());
    assert!(EvidenceId::from_str(second.receipt_id()).is_ok());
}

#[test]
fn source_artifact_identity_is_bound_separately_from_equal_content() {
    let first_artifact = SourceArtifact::from_bytes(b"same canonical snapshot").expect("artifact");
    let second_artifact = SourceArtifact::from_bytes(b"same canonical snapshot").expect("artifact");
    assert_eq!(
        first_artifact.content_digest(),
        second_artifact.content_digest()
    );
    assert_ne!(first_artifact.id(), second_artifact.id());

    let first_observation = SourceObservation::observe(&first_artifact).expect("first observation");
    let second_observation =
        SourceObservation::observe(&second_artifact).expect("second observation");
    let first_availability =
        SourceAvailability::make_available(&first_observation).expect("first availability");
    let second_availability =
        SourceAvailability::make_available(&second_observation).expect("second availability");
    let first = SourceSnapshotReceiptV1::from_source_availability(SNAPSHOT_ID, &first_availability)
        .expect("first receipt");
    let second =
        SourceSnapshotReceiptV1::from_source_availability(SNAPSHOT_ID, &second_availability)
            .expect("second receipt");

    assert_eq!(
        first.source_snapshot_sha256(),
        second.source_snapshot_sha256()
    );
    assert_eq!(
        EvidenceId::from_str(first.source_artifact_id()).expect("first source identity"),
        first_artifact.id()
    );
    assert_eq!(
        EvidenceId::from_str(second.source_artifact_id()).expect("second source identity"),
        second_artifact.id()
    );
    assert_ne!(first.receipt_id(), second.receipt_id());
    assert_ne!(
        first.binding_sha256().expect("first binding"),
        second.binding_sha256().expect("second binding")
    );
}

#[test]
fn same_logical_snapshot_with_different_source_bytes_has_different_binding() {
    let first = receipt(SOURCE_A_BYTES);
    let second = receipt(SOURCE_B_BYTES);

    assert_eq!(first.snapshot_id(), second.snapshot_id());
    assert_ne!(
        first.source_snapshot_sha256(),
        second.source_snapshot_sha256()
    );
    assert_ne!(
        first.binding_sha256().expect("first binding"),
        second.binding_sha256().expect("second binding")
    );
}

#[test]
fn availability_and_system_observation_remain_distinct_nominal_clocks() {
    let observed = receipt(SOURCE_A_BYTES);
    let system_observed =
        SystemTime::parse_rfc3339(observed.system_observed_at()).expect("system time");
    let available = AvailableTime::parse_rfc3339(observed.available_at()).expect("availability");

    assert!(available.instant() >= system_observed.instant());
    assert!(EvidenceId::from_str(observed.receipt_id()).is_ok());
}

#[test]
fn git_head_alias_cannot_be_minted_as_immutable_snapshot_identity() {
    let source_artifact = SourceArtifact::from_bytes(SOURCE_A_BYTES).expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");

    for alias in ["HEAD", "Head"] {
        assert_eq!(
            SourceSnapshotReceiptV1::from_source_availability(alias, &availability),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    assert!(
        SourceSnapshotReceiptV1::from_source_availability("snapshot:v1_2026.09-15", &availability,)
            .is_ok()
    );
}

#[test]
fn wire_refuses_noncanonical_digest_unknown_duplicate_and_oversized_payloads() {
    let canonical = receipt(SOURCE_A_BYTES).to_json().expect("json");
    let wire: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    let source_digest = wire["source_snapshot_sha256"]
        .as_str()
        .expect("source digest");
    let uppercase_source_digest = source_digest.to_ascii_uppercase();

    let uppercase = canonical.replace(source_digest, &uppercase_source_digest);
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&uppercase),
        Err(EvidenceError::InvalidContentDigest)
    );

    let unknown = canonical.replacen('{', "{\"unexpected\":true,", 1);
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&unknown),
        Err(EvidenceError::InvalidWirePayload)
    );

    let duplicate = canonical.replacen(
        "\"snapshot_id\":",
        "\"snapshot_id\":\"other-snapshot\",\"snapshot_id\":",
        1,
    );
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&duplicate),
        Err(EvidenceError::InvalidWirePayload)
    );

    let oversized = "x".repeat(SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT + 1);
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&oversized),
        Err(EvidenceError::InvalidWirePayload)
    );
}

#[test]
fn noncanonical_uuid_and_clock_wire_values_fail_closed() {
    let canonical = receipt(SOURCE_A_BYTES).to_json().expect("json");
    let wire: serde_json::Value = serde_json::from_str(&canonical).expect("json");
    let receipt_id = wire["receipt_id"].as_str().expect("receipt id");
    let uppercase_receipt_id = receipt_id.to_ascii_uppercase();

    let uppercase_id = canonical.replace(receipt_id, &uppercase_receipt_id);
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&uppercase_id),
        Err(EvidenceError::InvalidWirePayload)
    );

    let available_at = wire["available_at"].as_str().expect("available time");
    let noncanonical_time = canonical.replace(available_at, "2026-08-01T08:59:59+09:00");
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&noncanonical_time),
        Err(EvidenceError::InvalidWirePayload)
    );
}
