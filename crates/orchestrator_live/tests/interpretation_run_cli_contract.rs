//! Contract tests for the contextual-orchestrator interpretation-run loopback CLI.

use orchestrator_live::{
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, INTERPRETATION_RUN_CONTRACT_VERSION,
    InterpretationRunCliInvocation, InterpretationRunCliVerb, InterpretationRunRequest,
    OrchestrationMode, OrchestratorLiveError, compose_interpretation_run_cli_http,
};

fn query_body() -> String {
    InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "orch-cli-contract-1",
        "orch-tenant-demo",
        "tepp-snapshot-demo-001",
        "2026-08-01T00:00:00Z",
        OrchestrationMode::Direct,
        2048,
        vec!["span-001".into()],
        false,
    )
    .expect("request")
    .to_json()
    .expect("json")
}

#[test]
fn interpretation_run_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        InterpretationRunCliVerb::parse("create").expect("verb"),
        InterpretationRunCliVerb::Create
    );
    let invocation = InterpretationRunCliInvocation::from_args(
        [
            "create",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
        ],
        query_body(),
    )
    .expect("invocation");
    let http = compose_interpretation_run_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/interpretation-runs HTTP/1.1"));
    assert!(http.contains("tepp-consumer: contextual-orchestrator"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/analysis-runs"));
    assert!(!http.contains("/v1/exports"));
}

#[test]
fn interpretation_run_cli_refuses_non_loopback_unknown_verbs_and_metrics() {
    assert_eq!(
        InterpretationRunCliInvocation::from_args(
            [
                "create",
                "--host",
                "8.8.8.8:80",
                "--origin",
                "https://tepp.example.test"
            ],
            query_body()
        ),
        Err(OrchestratorLiveError::AuthorizationDenied)
    );
    assert_eq!(
        InterpretationRunCliVerb::parse("cancel"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        InterpretationRunCliInvocation::from_args(
            [
                "create",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                "https://tepp.example.test"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}
