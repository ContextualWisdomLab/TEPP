//! Public contract for immutable Evidence-owned source-snapshot receipts.

use evidence_core::{
    ContentDigest, EvidenceError, EvidenceId, SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT, SourceArtifact,
    SourceSnapshotReceiptV1,
};
use std::str::FromStr;
use temporal_core::AvailableTime;

const RECEIPT_ID: &str = "018f1f6b-7c2a-7abc-8def-0123456789ab";
const SNAPSHOT_ID: &str = "snapshot-rubin-loading";
const SOURCE_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SOURCE_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn receipt(source_sha256: &str, available_at: &str) -> SourceSnapshotReceiptV1 {
    SourceSnapshotReceiptV1::new(
        EvidenceId::from_str(RECEIPT_ID).expect("uuidv7"),
        SNAPSHOT_ID,
        ContentDigest::from_str(source_sha256).expect("digest"),
        AvailableTime::parse_rfc3339(available_at).expect("availability"),
    )
    .expect("receipt")
}

#[test]
fn source_artifact_identity_is_bound_separately_from_equal_content() {
    let first_artifact = SourceArtifact::from_bytes(b"same canonical snapshot").expect("artifact");
    let second_artifact = SourceArtifact::from_bytes(b"same canonical snapshot").expect("artifact");
    assert_eq!(first_artifact.content_digest(), second_artifact.content_digest());
    assert_ne!(first_artifact.id(), second_artifact.id());

    let receipt_id = EvidenceId::from_str(RECEIPT_ID).expect("uuidv7");
    let available = AvailableTime::parse_rfc3339("2026-07-31T23:59:59Z").expect("availability");
    let first = SourceSnapshotReceiptV1::from_source_artifact(
        receipt_id,
        SNAPSHOT_ID,
        &first_artifact,
        available,
    )
    .expect("first receipt");
    let second = SourceSnapshotReceiptV1::from_source_artifact(
        receipt_id,
        SNAPSHOT_ID,
        &second_artifact,
        available,
    )
    .expect("second receipt");

    assert_eq!(first.source_snapshot_sha256(), second.source_snapshot_sha256());
    assert_eq!(first.source_artifact_id(), first_artifact.id().to_string());
    assert_eq!(second.source_artifact_id(), second_artifact.id().to_string());
    assert_ne!(first.binding_sha256(), second.binding_sha256());
}

#[test]
fn same_logical_snapshot_with_different_source_bytes_has_different_binding() {
    let first = receipt(SOURCE_A, "2026-07-31T23:59:59Z");
    let second = receipt(SOURCE_B, "2026-07-31T23:59:59Z");

    assert_eq!(first.snapshot_id(), second.snapshot_id());
    assert_ne!(first.source_snapshot_sha256(), second.source_snapshot_sha256());
    assert_ne!(
        first.binding_sha256().expect("first binding"),
        second.binding_sha256().expect("second binding")
    );
}

#[test]
fn availability_is_authoritative_receipt_data_not_a_constructor_cutoff() {
    let future = receipt(SOURCE_A, "2026-08-02T00:00:00Z");
    assert_eq!(future.available_at(), "2026-08-02T00:00:00Z");
    assert_eq!(future.receipt_id(), RECEIPT_ID);
}

#[test]
fn wire_refuses_noncanonical_digest_unknown_duplicate_and_oversized_payloads() {
    let canonical = receipt(SOURCE_A, "2026-07-31T23:59:59Z")
        .to_json()
        .expect("json");

    let uppercase = canonical.replace(SOURCE_A, &SOURCE_A.to_ascii_uppercase());
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&uppercase),
        Err(EvidenceError::InvalidContentDigest)
    );

    let unknown = canonical.replacen('{', "{\"unexpected\":true,", 1);
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&unknown),
        Err(EvidenceError::InvalidWirePayload)
    );

    let duplicate = canonical.replacen(
        "\"snapshot_id\":",
        "\"snapshot_id\":\"other-snapshot\",\"snapshot_id\":",
        1,
    );
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&duplicate),
        Err(EvidenceError::InvalidWirePayload)
    );

    let oversized = "x".repeat(SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT + 1);
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&oversized),
        Err(EvidenceError::InvalidWirePayload)
    );
}

#[test]
fn noncanonical_uuid_and_availability_wire_values_fail_closed() {
    let canonical = receipt(SOURCE_A, "2026-07-31T23:59:59Z")
        .to_json()
        .expect("json");

    let uppercase_id = canonical.replace(RECEIPT_ID, &RECEIPT_ID.to_ascii_uppercase());
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&uppercase_id),
        Err(EvidenceError::InvalidWirePayload)
    );

    let noncanonical_time = canonical.replace(
        "2026-07-31T23:59:59Z",
        "2026-08-01T08:59:59+09:00",
    );
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&noncanonical_time),
        Err(EvidenceError::InvalidWirePayload)
    );
}
