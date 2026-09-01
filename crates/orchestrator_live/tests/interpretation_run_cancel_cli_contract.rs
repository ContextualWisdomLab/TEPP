//! Contract tests for `tepp-interpretation-run-cancel cancel`.

use orchestrator_live::{
    compose_interpretation_run_cli_http, dispatch_interpretation_run_cancel_cli,
    execute_interpretation_run_cancel_cli, render_interpretation_run_cancel_cli_stdout,
    InterpretationRunCancelCliInvocation, InterpretationRunCancelled,
    InterpretationRunCliInvocation, InterpretationRunRequest, OrchestrationMode,
    OrchestratorLiveError, OrchestratorLiveResponse, OrchestratorLiveService,
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, HYPOTHETICAL_CLAIM_STATUS,
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

fn cancel_invocation(idem: &str) -> InterpretationRunCancelCliInvocation {
    InterpretationRunCancelCliInvocation::from_args(
        [
            "cancel",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--idempotency-key",
            idem,
        ],
        "",
    )
    .expect("cancel")
}

#[test]
fn dispatch_cancels_one_hypothetical_identity_without_metrics() {
    let mut service = OrchestratorLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&create_http("idem-a"))
            .status_code,
        202
    );
    let cancelled =
        dispatch_interpretation_run_cancel_cli(&mut service, &cancel_invocation("idem-a"))
            .expect("cancel");
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let stdout =
        render_interpretation_run_cancel_cli_stdout(&cancel_invocation("idem-a"), &cancelled)
            .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("evidence_span_ids"));
    assert!(!stdout.contains("causal_score"));
    let item: InterpretationRunCancelled = serde_json::from_str(&stdout).expect("item");
    assert_eq!(item.idempotency_key, "idem-a");
    assert_eq!(item.claim_status, HYPOTHETICAL_CLAIM_STATUS);
    assert!(!item.scientific_authority);
    assert!(item.cancelled);
    assert_eq!(
        dispatch_interpretation_run_cancel_cli(&mut service, &cancel_invocation("idem-a"))
            .expect("second")
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let cancel = cancel_invocation("idem-a");
    assert_eq!(
        render_interpretation_run_cancel_cli_stdout(
            &cancel,
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
        render_interpretation_run_cancel_cli_stdout(
            &cancel,
            &OrchestratorLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"interpretation_run_id":"r","idempotency_key":"idem-a","orchestration_mode":"direct","claim_status":"hypothetical","scientific_authority":false,"cancelled":true,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        render_interpretation_run_cancel_cli_stdout(
            &cancel,
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
    let mut invocation = cancel_invocation("idem-a");
    invocation.host = addr.to_string();
    let response = execute_interpretation_run_cancel_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 400, "{}", response.body);
    handle.join().expect("join");
}
