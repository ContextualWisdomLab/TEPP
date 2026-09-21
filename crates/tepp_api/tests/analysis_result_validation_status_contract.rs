//! Public contract for release-safe analysis validation status semantics.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted,
    AnalysisRunRequest, AnalysisRunTerminalResult, ApiError,
};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-release-1".into(),
        tenant_workspace_id: "workspace-opaque".into(),
        snapshot_id: "snapshot-immutable".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "temporal-model-v1".into(),
        output_profile: "validation-report".into(),
    }
}

fn accepted() -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-release-1", "accepted", "idem-release-1")
        .expect("accepted receipt")
}

#[test]
fn validated_success_round_trips_with_canonical_status() {
    let summary =
        AnalysisResultSummary::new("temporal_topic_measurement", 120, 42, "validated")
            .expect("validated summary");
    let result = AnalysisRunTerminalResult::succeeded(
        &request(),
        &accepted(),
        "artifact-release-1",
        DIGEST,
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        summary,
    )
    .expect("validated success");

    let json = result.to_json().expect("json");
    assert!(json.contains("\"validation_status\":\"validated\""));
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&json).expect("decoded"),
        result
    );
}

#[test]
fn non_verifiable_and_non_converged_cannot_deserialize_as_success() {
    let validated =
        AnalysisResultSummary::new("temporal_topic_measurement", 120, 42, "validated")
            .expect("validated summary");
    let success = AnalysisRunTerminalResult::succeeded(
        &request(),
        &accepted(),
        "artifact-release-1",
        DIGEST,
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        validated,
    )
    .expect("validated success");
    let json = success.to_json().expect("json");

    for refused in ["not_verifiable", "non_converged", "insufficient_evidence", "failed"] {
        assert_eq!(
            AnalysisResultSummary::new("temporal_topic_measurement", 120, 42, refused),
            Err(ApiError::InvalidWirePayload),
            "{refused} must not construct a successful summary"
        );
        let hostile = json.replace(
            "\"validation_status\":\"validated\"",
            &format!("\"validation_status\":\"{refused}\""),
        );
        assert_eq!(
            AnalysisRunTerminalResult::from_json(&hostile),
            Err(ApiError::InvalidWirePayload),
            "{refused} must not enter the succeeded wire shape"
        );
    }
}

#[test]
fn non_success_scientific_states_use_terminal_failure_semantics() {
    for code in ["not_verifiable", "non_converged", "insufficient_evidence"] {
        let failed = AnalysisRunTerminalResult::failed(
            &request(),
            &accepted(),
            "2026-08-02T03:04:05Z",
            code,
        )
        .expect("typed terminal failure");
        assert_eq!(failed.failure_code.as_deref(), Some(code));
        assert!(failed.summary.is_none());
        assert!(failed.result_artifact_id.is_none());
    }
}
