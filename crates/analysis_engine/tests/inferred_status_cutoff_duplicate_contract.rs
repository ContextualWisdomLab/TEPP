//! Historical replay contract for inferred-status evidence admission.

use analysis_engine::{
    INFERRED_STATUS_MODEL_CONTRACT_VERSION, INFERRED_STATUS_OUTPUT_PROFILE, InferredStatusEvidence,
    execute_inferred_status_run,
};
use inferred_status::EvidenceStatus;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("valid availability")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("valid cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "inferred-status-cutoff-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-inferred-status-cutoff".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: INFERRED_STATUS_MODEL_CONTRACT_VERSION.into(),
        output_profile: INFERRED_STATUS_OUTPUT_PROFILE.into(),
    }
}

fn evidence(
    evidence_id: &str,
    status: EvidenceStatus,
    available_at: &str,
) -> InferredStatusEvidence {
    InferredStatusEvidence::new(evidence_id, status, available(available_at)).expect("valid evidence")
}

fn execute(evidence: &[InferredStatusEvidence]) -> analysis_engine::InferredStatusExecution {
    let request = request();
    let accepted = AnalysisRunAccepted::new(
        "run-inferred-status-cutoff",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted run");
    execute_inferred_status_run(
        &request,
        &accepted,
        "snapshot-inferred-status-cutoff",
        cutoff(),
        evidence,
        "2026-08-02T00:00:00Z",
    )
    .expect("historical execution")
}

#[test]
fn future_duplicate_identity_cannot_change_historical_result() {
    let visible = vec![
        evidence(
            "observed-a",
            EvidenceStatus::Observed,
            "2026-07-01T00:00:00Z",
        ),
        evidence(
            "inferred-b",
            EvidenceStatus::Inferred,
            "2026-07-02T00:00:00Z",
        ),
    ];
    let baseline = execute(&visible);

    let mut with_future_duplicate = vec![evidence(
        "inferred-b",
        EvidenceStatus::Inferred,
        "2026-08-02T00:00:00Z",
    )];
    with_future_duplicate.extend(visible);
    let replay = execute(&with_future_duplicate);

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}
