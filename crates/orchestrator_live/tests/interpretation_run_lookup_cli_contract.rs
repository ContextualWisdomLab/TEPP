//! Contract tests for `tepp-interpretation-run-lookup lookup`.

use orchestrator_live::{
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, INTERPRETATION_RUN_CONTRACT_VERSION,
    InterpretationRunAccepted, InterpretationRunCliInvocation,
    InterpretationRunLookupCliInvocation, InterpretationRunRequest, OrchestrationMode,
    OrchestratorLiveError, OrchestratorLiveResponse, OrchestratorLiveService,
    compose_interpretation_run_cli_http, dispatch_interpretation_run_lookup_cli,
    execute_interpretation_run_lookup_cli, render_interpretation_run_lookup_cli_stdout,
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

fn lookup_invocation(run_id: &str) -> InterpretationRunLookupCliInvocation {
    InterpretationRunLookupCliInvocation::from_args(
        [
            "lookup",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            ORIGIN,
            "--interpretation-run-id",
            run_id,
        ],
        "",
    )
    .expect("lookup")
}

#[test]
fn dispatch_retrieves_metric_free_identity_without_scientific_authority() {
    let mut service = OrchestratorLiveService::new();
    let created = service.handle_http_request(&create_http("idem-a"));
    assert_eq!(created.status_code, 202, "{}", created.body);
    let accepted = InterpretationRunAccepted::from_json(&created.body).expect("accepted");
    let run_id = accepted.interpretation_run_id().to_owned();
    let got = dispatch_interpretation_run_lookup_cli(&mut service, &lookup_invocation(&run_id))
        .expect("lookup");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_interpretation_run_lookup_cli_stdout(&lookup_invocation(&run_id), &got)
        .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("causal_score"));
    assert!(!stdout.contains("evidence_span_ids"));
    assert!(stdout.contains(&format!("\"interpretation_run_id\":\"{run_id}\"")));
    assert!(stdout.contains("\"idempotency_key\":\"idem-a\""));
    assert!(stdout.contains("\"claim_status\":\"hypothetical\""));
    assert!(stdout.contains("\"scientific_authority\":false"));
    assert_eq!(
        dispatch_interpretation_run_lookup_cli(&mut service, &lookup_invocation("missing"))
            .expect("missing")
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let lookup = lookup_invocation("orch-run-1");
    assert_eq!(
        render_interpretation_run_lookup_cli_stdout(
            &lookup,
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
        render_interpretation_run_lookup_cli_stdout(
            &lookup,
            &OrchestratorLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"interpretation_run_id":"orch-run-1","idempotency_key":"idem-a","orchestration_mode":"direct","claim_status":"hypothetical","scientific_authority":false,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        render_interpretation_run_lookup_cli_stdout(
            &lookup,
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
    let mut invocation = lookup_invocation("orch-run-1");
    invocation.host = addr.to_string();
    let response = execute_interpretation_run_lookup_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 400, "{}", response.body);
    handle.join().expect("join");
}
