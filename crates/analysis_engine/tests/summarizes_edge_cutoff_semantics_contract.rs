//! Regression contract for semantic knowledge-cutoff equality in summarizes-edge runs.

use analysis_engine::{
    SUMMARIZES_EDGE_MODEL_CONTRACT_VERSION, SUMMARIZES_EDGE_OUTPUT_PROFILE,
    SummarizesEdgeAssignment, execute_summarizes_edge_run,
};
use summarizes_edge::SummarizesKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available() -> AvailableTime {
    AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available time")
}

fn assignment(assignment_id: &str, kind: SummarizesKind) -> SummarizesEdgeAssignment {
    SummarizesEdgeAssignment::new(assignment_id, kind, available()).expect("assignment")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "summarizes-edge-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-summarizes-edge".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: SUMMARIZES_EDGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: SUMMARIZES_EDGE_OUTPUT_PROFILE.into(),
    };
    let accepted =
        AnalysisRunAccepted::new("run-summarizes-edge", "accepted", &request.idempotency_key)
            .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let assignments = vec![
        assignment("summary-a", SummarizesKind::Summary),
        assignment("source-b", SummarizesKind::SourceDocument),
        assignment("summary-c", SummarizesKind::Summary),
    ];

    let execution = execute_summarizes_edge_run(
        &request,
        &accepted,
        "snapshot-summarizes-edge",
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
