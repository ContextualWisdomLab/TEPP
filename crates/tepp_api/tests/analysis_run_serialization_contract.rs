//! Release-contract tests separating deterministic wire serialization from live admission time.

use tepp_api::{ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError};

fn future_request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-release-clock-1".into(),
        tenant_workspace_id: "workspace-opaque".into(),
        snapshot_id: "snapshot-immutable".into(),
        knowledge_cutoff: "2099-01-01T00:00:00Z".into(),
        model_contract_version: "temporal-model-v1".into(),
        output_profile: "validation-report".into(),
    }
}

#[test]
fn canonical_request_serialization_does_not_depend_on_the_current_wall_clock() {
    let request = future_request();
    let json = request
        .to_json()
        .expect("canonical wire serialization must validate syntax, not current-time admission");

    assert!(json.contains("\"knowledge_cutoff\":\"2099-01-01T00:00:00Z\""));
    assert_eq!(
        AnalysisRunRequest::from_json(&json),
        Err(ApiError::InvalidWirePayload),
        "live admission must still refuse evidence cutoffs that have not happened yet"
    );
}

#[test]
fn canonical_request_serialization_still_rejects_malformed_cutoff_syntax() {
    let mut request = future_request();
    request.knowledge_cutoff = "not-a-time".into();

    assert_eq!(request.to_json(), Err(ApiError::InvalidWirePayload));
}
