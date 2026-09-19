//! Serialization bound contract for the episode-membership profile.

use analysis_engine::{
    EPISODE_MEMBERSHIP_ARTIFACT_BYTE_LIMIT, EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
    EpisodeMembershipArtifact, MAX_EVIDENCE_UNITS,
};

#[test]
fn maximal_valid_episode_membership_artifact_fits_wire_limit() {
    let escaped_count = MAX_EVIDENCE_UNITS as u64 - 1;
    let artifact = EpisodeMembershipArtifact {
        schema_version: EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "r".repeat(256),
        snapshot_id: "s".repeat(256),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count: MAX_EVIDENCE_UNITS as u64,
        contained_count: 1,
        escaped_count,
        refused_as_escape_count: escaped_count,
        inference_status: "membership_window_cannot_escape_episode_interval".into(),
    };

    let payload = artifact.to_json().expect("maximal valid artifact");
    assert!(payload.len() < EPISODE_MEMBERSHIP_ARTIFACT_BYTE_LIMIT);
}
