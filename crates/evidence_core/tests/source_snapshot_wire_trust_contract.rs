//! Contract separating validated receipt wire from owner-issued Evidence state.

use evidence_core::{
    SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
    ValidatedSourceSnapshotReceiptWireV1,
};

#[test]
fn canonical_wire_reconstruction_remains_untrusted_until_owner_authentication() {
    let artifact = SourceArtifact::from_bytes(b"owner-issued source").expect("artifact");
    let observation = SourceObservation::observe(&artifact).expect("observation");
    let availability = SourceAvailability::make_available(&observation).expect("availability");
    let owner_receipt = SourceSnapshotReceiptV1::from_source_availability(
        "snapshot-rubin-loading-v1",
        &availability,
    )
    .expect("owner receipt");
    let payload = owner_receipt.to_json().expect("canonical json");

    let parsed = ValidatedSourceSnapshotReceiptWireV1::from_json(&payload)
        .expect("validated untrusted wire");
    assert_eq!(parsed.receipt_id(), owner_receipt.receipt_id());
    assert_eq!(
        parsed.source_artifact_id(),
        owner_receipt.source_artifact_id()
    );
    assert_eq!(parsed.available_at(), owner_receipt.available_at());
}
