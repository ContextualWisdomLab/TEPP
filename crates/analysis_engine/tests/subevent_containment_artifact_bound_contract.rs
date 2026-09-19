//! Serialization bound contract for the subevent-containment profile.

use analysis_engine::{
    MAX_EVIDENCE_UNITS, SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT,
    SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION, SubeventContainmentArtifact,
};

#[test]
fn maximal_valid_subevent_containment_artifact_fits_wire_limit() {
    let escaped_count = MAX_EVIDENCE_UNITS as u64 - 1;
    let artifact = SubeventContainmentArtifact {
        schema_version: SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "r".repeat(256),
        snapshot_id: "s".repeat(256),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count: MAX_EVIDENCE_UNITS as u64,
        contained_count: 1,
        escaped_count,
        refused_as_escape_count: escaped_count,
        inference_status: "subevent_interval_cannot_escape_parent_interval".into(),
    };

    let payload = artifact.to_json().expect("maximal valid artifact");
    assert!(payload.len() < SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT);
}
