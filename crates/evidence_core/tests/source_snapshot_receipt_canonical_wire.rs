//! Regression contract for canonical source-snapshot receipt wire bytes.

use evidence_core::{
    EvidenceError, SourceArtifact, SourceObservation, SourceSnapshotReceiptV1,
};

#[test]
fn semantically_equivalent_noncanonical_json_fails_closed() {
    let source_artifact = SourceArtifact::from_bytes(b"canonical source snapshot").expect("artifact");
    let observation = SourceObservation::observe(&source_artifact).expect("observation");
    let receipt = SourceSnapshotReceiptV1::from_source_observation(
        "snapshot-rubin-loading-v1",
        &observation,
    )
    .expect("receipt");
    let canonical = receipt.to_json().expect("canonical receipt json");

    let leading_whitespace = format!(" {canonical}");
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&leading_whitespace),
        Err(EvidenceError::InvalidWirePayload)
    );

    let trailing_newline = format!("{canonical}\n");
    assert_eq!(
        SourceSnapshotReceiptV1::from_json(&trailing_newline),
        Err(EvidenceError::InvalidWirePayload)
    );
}
