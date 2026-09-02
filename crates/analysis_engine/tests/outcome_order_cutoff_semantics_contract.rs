//! Regression contract for semantic knowledge-cutoff equality in outcome-order runs.

use analysis_engine::{
    OUTCOME_ORDER_MODEL_CONTRACT_VERSION, OUTCOME_ORDER_OUTPUT_PROFILE, OutcomeOrderEdge,
    execute_outcome_order_run,
};
use outcome_order::OutcomeKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available time")
}

fn edge(
    edge_id: &str,
    kind: OutcomeKind,
    source_rank: u64,
    target_rank: u64,
) -> OutcomeOrderEdge {
    OutcomeOrderEdge::new(
        edge_id,
        kind,
        source_rank,
        target_rank,
        available("2026-07-01T00:00:00Z"),
    )
    .expect("edge")
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_admitted() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "outcome-order-equivalent-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-outcome-order".into(),
        knowledge_cutoff: "2026-08-01T09:00:00+09:00".into(),
        model_contract_version: OUTCOME_ORDER_MODEL_CONTRACT_VERSION.into(),
        output_profile: OUTCOME_ORDER_OUTPUT_PROFILE.into(),
    };
    let accepted = AnalysisRunAccepted::new("run-outcome-order", "accepted", &request.idempotency_key)
        .expect("accepted");
    let execution_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("execution cutoff");
    let edges = vec![
        edge("input-a", OutcomeKind::InputTo, 1, 2),
        edge("process-b", OutcomeKind::ProcessTo, 2, 3),
        edge("outcome-c", OutcomeKind::OutcomeOf, 9, 1),
    ];

    let execution = execute_outcome_order_run(
        &request,
        &accepted,
        "snapshot-outcome-order",
        execution_cutoff,
        &edges,
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent RFC 3339 spellings denote the same cutoff instant");

    assert_eq!(
        execution.artifact.knowledge_cutoff,
        execution_cutoff.to_rfc3339()
    );
}
