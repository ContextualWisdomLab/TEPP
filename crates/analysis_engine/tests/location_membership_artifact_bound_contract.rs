//! Serialization bound for the location-membership analysis artifact.

use analysis_engine::{
    AnalysisEngineError, LocationMembershipArtifact, MAX_EVIDENCE_UNITS,
    LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
};

#[test]
fn compact_oversized_location_artifact_fails_closed() {
    let document_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let location_count = document_count - 2;
    let artifact = LocationMembershipArtifact {
        schema_version: LOCATION_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        document_count,
        location_count,
        entity_identity_count: 1,
        language_channel_count: 1,
        refused_as_entity_identity_count: location_count,
        refused_as_language_channel_count: location_count,
        inference_status: "location_is_not_entity_identity_not_language_channel".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");

    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidLocationMembershipArtifact)
    );
    assert_eq!(
        LocationMembershipArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidLocationMembershipArtifact)
    );
}
