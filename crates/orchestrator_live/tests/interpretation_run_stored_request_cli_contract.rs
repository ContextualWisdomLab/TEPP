//! Contract tests for `tepp-interpretation-run-request get`.

use orchestrator_live::{
    compose_interpretation_run_cli_http, dispatch_interpretation_run_stored_request_cli,
    execute_interpretation_run_stored_request_cli, render_interpretation_run_stored_request_cli_stdout,
    InterpretationRunCliInvocation, InterpretationRunRequest,
    InterpretationRunStoredRequestCliInvocation, OrchestrationMode, OrchestratorLiveError,
    OrchestratorLiveResponse, OrchestratorLiveService, CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
    INTERPRETATION_RUN_CONTRACT_VERSION,
};

const ORIGIN: &str = "https://tepp.example.test";

fn query_body(idem: &str) -> String {
    InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        idem,
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

fn create_http(idem: &str) -> String {
    let invocation = InterpretationRunCliInvocation::from_args(
        [
            "create",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
        ],
        query_body(idem),
    )
    .expect("create");
    compose_interpretation_run_cli_http(&invocation).expect("post")
}

fn get_invocation(idem: &str) -> InterpretationRunStoredRequestCliInvocation {
    InterpretationRunStoredRequestCliInvocation::from_args(
        [
            "get",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--idempotency-key",
            idem,
        ],
        "",
    )
    .expect("get")
}

#[test]
fn dispatch_retrieves_stored_request_without_scientific_authority() {
    let mut service = OrchestratorLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&create_http("idem-a"))
            .status_code,
        202
    );
    let got = dispatch_interpretation_run_stored_request_cli(&mut service, &get_invocation("idem-a"))
        .expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout =
        render_interpretation_run_stored_request_cli_stdout(&get_invocation("idem-a"), &got)
            .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("causal_score"));
    let stored = InterpretationRunRequest::from_json(&stdout).expect("stored");
    assert_eq!(stored.idempotency_key(), "idem-a");
    assert!(stdout.contains("\"scientific_authority\":false"));
    assert_eq!(
        dispatch_interpretation_run_stored_request_cli(&mut service, &get_invocation("missing"))
            .expect("missing")
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let get = get_invocation("idem-a");
    assert_eq!(
        render_interpretation_run_stored_request_cli_stdout(
            &get,
            &OrchestratorLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: String::new(),
            }
        )
        .unwrap_err(),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        render_interpretation_run_stored_request_cli_stdout(
            &get,
            &OrchestratorLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"idempotency_key":"idem-a","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","orchestration_mode":"direct","compute_budget_tokens":8,"evidence_span_ids":["span-001"],"scientific_authority":false,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        render_interpretation_run_stored_request_cli_stdout(
            &get,
            &OrchestratorLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            }
        )
        .unwrap_err(),
        OrchestratorLiveError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_returns_missing_identity_as_invalid_wire() {
    let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let mut invocation = get_invocation("idem-a");
    invocation.host = addr.to_string();
    let response = execute_interpretation_run_stored_request_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 400, "{}", response.body);
    handle.join().expect("join");
}
