//! Regression contract for canonical source-snapshot receipt wire bytes.

use evidence_core::{
    EvidenceError, SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
    ValidatedSourceSnapshotReceiptWireV1,
};

#[test]
fn semantically_equivalent_noncanonical_json_fails_closed() {
    let source_artifact =
        SourceArtifact::from_bytes(b"canonical source snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");
    let receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot-rubin-loading-v1",
        &availability,
    )
    .expect("receipt");
    let canonical = receipt.to_json().expect("canonical receipt json");

    let leading_whitespace = format!(" {canonical}");
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&leading_whitespace),
        Err(EvidenceError::InvalidWirePayload)
    );

    let trailing_newline = format!("{canonical}\n");
    assert_eq!(
        ValidatedSourceSnapshotReceiptWireV1::from_json(&trailing_newline),
        Err(EvidenceError::InvalidWirePayload)
    );
}
