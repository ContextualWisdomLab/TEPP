use evidence_core::{EvidenceError, SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1};

fn source_availability() -> SourceAvailability {
    let artifact = SourceArtifact::from_bytes(b"canonical snapshot").expect("artifact must be valid");
    let observation = SourceObservation::observe(&artifact).expect("observation must be valid");
    SourceAvailability::make_available(&observation).expect("availability must be valid")
}

#[test]
fn bare_mutable_locator_prefixes_fail_closed() {
    let availability = source_availability();

    for alias in ["pr-", "pull-", "issue-"] {
        assert_eq!(
            SourceSnapshotReceiptV1::from_source_availability(alias, &availability),
            Err(EvidenceError::InvalidWirePayload),
            "bare mutable locator prefix must not become an immutable snapshot identity: {alias}"
        );
    }
}

#[test]
fn immutable_identifier_that_only_shares_a_prefix_remains_valid() {
    let availability = source_availability();
    let receipt = SourceSnapshotReceiptV1::from_source_availability("pr-release-v1", &availability)
        .expect("descriptive immutable identifier must remain valid");

    assert_eq!(receipt.snapshot_id(), "pr-release-v1");
}
