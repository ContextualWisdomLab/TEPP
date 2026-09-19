//! Contract tests for the analysis-run create loopback CLI.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunCreateCliInvocation, AnalysisRunCreateCliVerb,
    AnalysisRunRequest, ApiError, NARUON_CONSUMER_CODE, compose_analysis_run_create_cli_http,
};

fn request_json(idempotency_key: &str) -> String {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "cli-create-contract-tenant".into(),
        snapshot_id: "cli-create-contract-snapshot".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "tepp-analysis-run-v1".into(),
        output_profile: "calibrated_event_measurement".into(),
    }
    .to_json()
    .expect("json")
}

#[test]
fn create_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        AnalysisRunCreateCliVerb::parse("create").expect("create"),
        AnalysisRunCreateCliVerb::Create
    );
    let invocation = AnalysisRunCreateCliInvocation::from_args(
        [
            "create",
            "--host",
            "127.0.0.1:18081",
            "--idempotency-key",
            "idem-1",
        ],
        request_json("idem-1"),
    )
    .expect("invocation");
    assert_eq!(invocation.consumer, NARUON_CONSUMER_CODE);
    let http = compose_analysis_run_create_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/analysis-runs HTTP/1.1"));
    assert!(http.contains("idempotency-key: idem-1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("copilot"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/cancel"));
}

#[test]
fn create_cli_refuses_non_loopback_unknown_verbs_and_metric_bodies() {
    assert_eq!(
        AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "8.8.8.8:80",
                "--idempotency-key",
                "idem-1"
            ],
            request_json("idem-1")
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        AnalysisRunCreateCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunCreateCliVerb::parse("cancel"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--idempotency-key",
                "idem-1"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunCreateCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18081",
                "--idempotency-key",
                "idem-1"
            ],
            ""
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
