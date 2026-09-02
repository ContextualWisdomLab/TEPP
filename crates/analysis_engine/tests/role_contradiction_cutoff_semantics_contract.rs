//! Regression contract for semantic knowledge-cutoff equality in role-contradiction runs.

use analysis_engine::{
    ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION, ROLE_CONTRADICTION_OUTPUT_PROFILE,
    RoleContradictionAssignment, execute_role_contradiction_run,
};
use role_contradiction::ContextualRole;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn assignment(assignment_id: &str, role: ContextualRole) -> RoleContradictionAssignment {
    RoleContradictionAssignment::new(assignment_id, "group-mixed", role, available())
        .expect("assignment")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "role-contradiction-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-role-contradiction".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION.into(),
        output_profile: ROLE_CONTRADICTION_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-role-contradiction",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let assignments = vec![
        assignment("customer-mixed", ContextualRole::Customer),
        assignment("competitor-mixed", ContextualRole::Competitor),
        assignment("partner-mixed", ContextualRole::Partner),
    ];

    let execution = execute_role_contradiction_run(
        &request,
        &accepted,
        "snapshot-role-contradiction",
        execution_cutoff,
        &assignments,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent RFC 3339 spellings denote the same cutoff instant");

    assert_eq!(
        execution.artifact.knowledge_cutoff,
        execution_cutoff.to_rfc3339()
    );
}
