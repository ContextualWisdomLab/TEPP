//! Regression contract for canonical source-snapshot receipt wire bytes.

use evidence_core::{EvidenceError, SourceArtifact, SourceSnapshotReceiptV1};
use temporal_core::AvailableTime;

#[test]
fn semantically_equivalent_noncanonical_json_fails_closed() {
    let source_artifact = SourceArtifact::from_bytes(b"canonical source snapshot").expect("artifact");
    let receipt = SourceSnapshotReceiptV1::from_source_artifact(
        "snapshot-rubin-loading-v1",
        &source_artifact,
        AvailableTime::parse_rfc3339("2026-09-15T01:00:00Z").expect("availability"),
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
