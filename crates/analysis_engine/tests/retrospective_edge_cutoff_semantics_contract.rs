//! Regression contract for semantic knowledge-cutoff equality in retrospective-edge runs.

use analysis_engine::{
    RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION, RETROSPECTIVE_EDGE_OUTPUT_PROFILE,
    RetrospectiveEdgeAssignment, execute_retrospective_edge_run,
};
use retrospective_edge::RetrospectiveKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn assignment(assignment_id: &str, kind: RetrospectiveKind) -> RetrospectiveEdgeAssignment {
    RetrospectiveEdgeAssignment::new(assignment_id, kind, available()).expect("assignment")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "retrospective-edge-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-retrospective-edge".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: RETROSPECTIVE_EDGE_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new(
        "run-retrospective-edge",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let assignments = vec![
        assignment("retrospective-a", RetrospectiveKind::RetrospectiveReport),
        assignment("forward-b", RetrospectiveKind::ForwardReport),
        assignment("retrospective-c", RetrospectiveKind::RetrospectiveReport),
    ];

    let execution = execute_retrospective_edge_run(
        &request,
        &accepted,
        "snapshot-retrospective-edge",
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
