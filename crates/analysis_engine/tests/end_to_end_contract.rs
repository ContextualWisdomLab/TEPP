//! Realistic cutoff-safe end-to-end analysis execution.

use analysis_engine::{
    AnalysisCorpus, AnalysisEngineError, AnalysisEvidenceUnit, execute_analysis_run,
};
use temporal_core::{AvailableTime, EventTime};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn evidence(id: &str, available: &str, memberships: u32) -> AnalysisEvidenceUnit {
    AnalysisEvidenceUnit::new(
        id,
        EventTime::parse_rfc3339("2026-07-10T12:00:00Z").expect("event time"),
        AvailableTime::parse_rfc3339(available).expect("available time"),
        memberships,
    )
    .expect("evidence")
}

#[test]
fn production_shape_run_excludes_future_available_evidence() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "customer-run-2026-08-01".into(),
        tenant_workspace_id: "workspace-opaque-1".into(),
        snapshot_id: "snapshot-customer-2026-08-01".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "temporal-evidence-v1".into(),
        output_profile: "validation-report".into(),
    };
    let accepted =
        AnalysisRunAccepted::new("run-customer-1", "accepted", "customer-run-2026-08-01")
            .expect("accepted");
    let corpus = AnalysisCorpus::new(
        "snapshot-customer-2026-08-01",
        vec![
            evidence("invoice-renewal", "2026-07-31T23:59:59Z", 2),
            evidence("later-correction", "2026-08-01T00:00:01Z", 4),
        ],
    )
    .expect("snapshot");
    let execution = execute_analysis_run(&request, &accepted, &corpus, "2026-08-01T00:01:00Z")
        .expect("execute");
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution
            .artifact
            .expect("artifact")
            .eligible_evidence_count,
        1
    );
}

#[test]
fn snapshot_identity_is_not_inferred_from_customer_payload() {
    let request = AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "run".into(),
        tenant_workspace_id: "workspace".into(),
        snapshot_id: "request-snapshot".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "model-v1".into(),
        output_profile: "report".into(),
    };
    let accepted = AnalysisRunAccepted::new("run", "accepted", "run").expect("accepted");
    let corpus = AnalysisCorpus::new(
        "other-snapshot",
        vec![evidence("evidence", "2026-07-01T00:00:00Z", 1)],
    )
    .expect("snapshot");
    assert_eq!(
        execute_analysis_run(&request, &accepted, &corpus, "2026-08-01T00:01:00Z"),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
}
