//! Serialization bound contract for the membership-target profile.

use analysis_engine::{
    MAX_EVIDENCE_UNITS, MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT,
    MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION, MembershipTargetArtifact,
};

#[test]
fn maximal_valid_membership_target_artifact_fits_wire_limit() {
    let language_count = MAX_EVIDENCE_UNITS as u64 - 1;
    let artifact = MembershipTargetArtifact {
        schema_version: MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "r".repeat(256),
        snapshot_id: "s".repeat(256),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        document_count: MAX_EVIDENCE_UNITS as u64,
        language_count,
        episode_count: 0,
        template_count: 0,
        department_count: 0,
        opportunity_pool_count: 0,
        entity_count: 1,
        project_count: 0,
        refused_as_entity_count: language_count,
        refused_as_project_count: language_count,
        inference_status: "language_episode_template_department_opportunity_pool_are_not_entities"
            .into(),
    };

    let payload = artifact.to_json().expect("maximal valid artifact");
    assert!(payload.len() < MEMBERSHIP_TARGET_ARTIFACT_BYTE_LIMIT);
}
