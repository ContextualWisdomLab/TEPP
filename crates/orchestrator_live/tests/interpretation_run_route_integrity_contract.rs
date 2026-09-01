//! Interpretation-run route identities fail closed before they become unreachable.

use orchestrator_live::{
    INTERPRETATION_RUN_CONTRACT_VERSION, INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN,
    INTERPRETATION_RUN_PATH, InterpretationRunAccepted, InterpretationRunLookupStoredRequestCliInvocation,
    InterpretationRunRequest, OrchestrationMode, OrchestratorLiveError, OrchestratorLiveService,
    dispatch_interpretation_run_lookup_stored_request_cli,
    render_interpretation_run_lookup_stored_request_cli_stdout,
};

fn interpretation_request(idempotency_key: &str, snapshot_id: &str) -> InterpretationRunRequest {
    InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        idempotency_key,
        "tenant-a",
        snapshot_id,
        "2026-09-01T00:00:00Z",
        OrchestrationMode::Direct,
        128,
        vec!["span-a".into()],
        false,
    )
    .expect("valid request")
}

fn post_request(request: &InterpretationRunRequest) -> String {
    let body = request.to_json().expect("request JSON");
    format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key(),
        body.len()
    )
}

fn accept(
    service: &mut OrchestratorLiveService,
    request: &InterpretationRunRequest,
) -> InterpretationRunAccepted {
    let response = service.handle_http_request(&post_request(request));
    assert_eq!(response.status_code, 202);
    InterpretationRunAccepted::from_json(&response.body).expect("accepted")
}

fn lookup_request_invocation(run_id: &str) -> InterpretationRunLookupStoredRequestCliInvocation {
    InterpretationRunLookupStoredRequestCliInvocation::from_args(
        [
            "get",
            "--host",
            "127.0.0.1:41414",
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            "contextual-orchestrator",
            "--interpretation-run-id",
            run_id,
        ],
        "",
    )
    .expect("lookup invocation")
}

#[test]
fn service_refuses_reserved_lookup_segment_before_acceptance() {
    let request = interpretation_request("by-run-id", "snapshot-reserved");
    let response = OrchestratorLiveService::new().handle_http_request(&post_request(&request));
    assert_eq!(response.status_code, 400);
    assert!(response.body.contains("invalid_wire_payload"));
}

#[test]
fn oversized_lookup_stored_request_identity_preserves_limit_status() {
    let oversized = "a".repeat(INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN + 1);
    let request = format!(
        "GET /v1/interpretation-runs/by-run-id/{oversized}/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    );

    let response = OrchestratorLiveService::new().handle_http_request(&request);
    assert_eq!(response.status_code, 413);
    assert!(response.body.contains("limit_exceeded"));
}

#[test]
fn lookup_request_cli_rejects_response_bound_to_another_run() {
    let mut service = OrchestratorLiveService::new();
    let first_request = interpretation_request("idem-first", "snapshot-first");
    let second_request = interpretation_request("idem-second", "snapshot-second");
    let first = accept(&mut service, &first_request);
    let second = accept(&mut service, &second_request);

    let first_invocation = lookup_request_invocation(first.interpretation_run_id());
    let second_invocation = lookup_request_invocation(second.interpretation_run_id());
    let second_response =
        dispatch_interpretation_run_lookup_stored_request_cli(&mut service, &second_invocation)
            .expect("dispatch");
    assert_eq!(second_response.status_code, 200);

    assert_eq!(
        render_interpretation_run_lookup_stored_request_cli_stdout(
            &first_invocation,
            &second_response,
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}
