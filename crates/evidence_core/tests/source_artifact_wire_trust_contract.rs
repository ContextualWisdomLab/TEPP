//! Trust-state contract for source-artifact wire input.

use evidence_core::{
    ContentDigest, EvidenceId, SourceArtifact, SourceAvailability, SourceObservation,
    SourceSnapshotReceiptV1, WIRE_SCHEMA_VERSION,
};

#[test]
fn caller_selected_wire_identity_must_not_become_receipt_owner_identity() {
    let content = b"caller-supplied source artifact";
    let caller_selected_id = EvidenceId::new();
    let digest = ContentDigest::sha256(content);
    let content_bytes = serde_json::to_string(content).expect("content bytes must serialize");
    let payload = format!(
        "{{\"schema_version\":{WIRE_SCHEMA_VERSION},\"artifact_id\":\"{caller_selected_id}\",\"content_sha256\":\"{digest}\",\"content_bytes\":{content_bytes}}}"
    );

    let artifact = SourceArtifact::from_wire_json(&payload)
        .expect("current public wire path accepts structurally valid source artifact JSON");
    let observation = SourceObservation::observe(&artifact).expect("observation must succeed");
    let availability =
        SourceAvailability::make_available(&observation).expect("availability must succeed");
    let receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot:v1_2026.09-15",
        &availability,
    )
    .expect("receipt must succeed");

    assert_ne!(
        receipt.source_artifact_id(),
        caller_selected_id.to_string(),
        "untrusted JSON must not be able to select the owner source-artifact identity"
    );
}
