//! Regression contract for semantic knowledge-cutoff equality in support-edge runs.

use analysis_engine::{
    execute_support_edge_run, SupportEdgeAssignment, SUPPORT_EDGE_MODEL_CONTRACT_VERSION,
    SUPPORT_EDGE_OUTPUT_PROFILE,
};
use support_edge::EvidenceKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn assignment(assignment_id: &str, kind: EvidenceKind) -> SupportEdgeAssignment {
    SupportEdgeAssignment::new(assignment_id, kind, available()).expect("assignment")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "support-edge-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-support-edge".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: SUPPORT_EDGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: SUPPORT_EDGE_OUTPUT_PROFILE.into(),
    };
    let accepted =
        AnalysisRunAccepted::new("run-support-edge", "accepted", &request.idempotency_key)
            .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let assignments = vec![
        assignment("support-a", EvidenceKind::Support),
        assignment("contradiction-b", EvidenceKind::Contradiction),
        assignment("summarizes-c", EvidenceKind::Summarizes),
        assignment("outcome-d", EvidenceKind::OutcomeOf),
    ];

    let execution = execute_support_edge_run(
        &request,
        &accepted,
        "snapshot-support-edge",
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
