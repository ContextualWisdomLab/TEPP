//! Canonical completion-time contract for immutable analysis results.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest,
    AnalysisRunTerminalResult, ApiError,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-completed-at".into(),
        tenant_workspace_id: "workspace-opaque".into(),
        snapshot_id: "snapshot-immutable".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "temporal-model-v1".into(),
        output_profile: "validation-report".into(),
    }
}

fn accepted() -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-completed-at", "accepted", "idem-completed-at")
        .expect("accepted receipt")
}

fn summary() -> AnalysisResultSummary {
    AnalysisResultSummary::new("temporal_topic_measurement", 12, 4, "validated")
        .expect("validated summary")
}

#[test]
fn terminal_completion_time_has_one_canonical_utc_wire_identity() {
    let canonical = AnalysisRunTerminalResult::succeeded(
        &request(),
        &accepted(),
        "artifact-completed-at",
        DIGEST,
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        summary(),
    )
    .expect("canonical UTC completion time");
    let canonical_json = canonical.to_json().expect("canonical json");
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&canonical_json),
        Ok(canonical)
    );

    assert_eq!(
        AnalysisRunTerminalResult::succeeded(
            &request(),
            &accepted(),
            "artifact-completed-at",
            DIGEST,
            "tepp-result-v1",
            "2026-08-02T12:04:05+09:00",
            summary(),
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunTerminalResult::failed(
            &request(),
            &accepted(),
            "2026-08-02T12:04:05+09:00",
            "estimation_failed",
        ),
        Err(ApiError::InvalidWirePayload)
    );

    let alias_json = canonical_json.replace(
        "2026-08-02T03:04:05Z",
        "2026-08-02T12:04:05+09:00",
    );
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&alias_json),
        Err(ApiError::InvalidWirePayload)
    );
}
