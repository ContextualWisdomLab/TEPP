//! Regression coverage for literal immutable snapshot IDs that resemble mutable locator prefixes.

use evidence_core::{
    SourceArtifact, SourceAvailability, SourceObservation, SourceSnapshotReceiptV1,
};

fn source_availability() -> SourceAvailability {
    let artifact =
        SourceArtifact::from_bytes(b"canonical snapshot").expect("artifact must be valid");
    let observation = SourceObservation::observe(&artifact).expect("observation must be valid");
    SourceAvailability::make_available(&observation).expect("availability must be valid")
}

#[test]
fn bare_locator_prefixes_without_numeric_targets_remain_literal_snapshot_ids() {
    let availability = source_availability();

    for snapshot_id in ["pr-", "pull-", "issue-"] {
        let receipt = SourceSnapshotReceiptV1::from_source_availability(snapshot_id, &availability)
            .expect("prefix without a numeric locator target is a literal immutable identifier");
        assert_eq!(receipt.snapshot_id(), snapshot_id);
    }
}

#[test]
fn immutable_identifier_that_only_shares_a_prefix_remains_valid() {
    let availability = source_availability();
    let receipt = SourceSnapshotReceiptV1::from_source_availability("pr-release-v1", &availability)
        .expect("descriptive immutable identifier must remain valid");

    assert_eq!(receipt.snapshot_id(), "pr-release-v1");
}
