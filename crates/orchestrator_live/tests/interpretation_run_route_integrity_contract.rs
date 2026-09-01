//! Interpretation-run route identities fail closed before they become unreachable.

use orchestrator_live::{
    INTERPRETATION_RUN_CONTRACT_VERSION, INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN,
    INTERPRETATION_RUN_PATH, InterpretationRunRequest, OrchestrationMode, OrchestratorLiveService,
};

fn post_request(request: &InterpretationRunRequest) -> String {
    let body = request.to_json().expect("request JSON");
    format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key(),
        body.len()
    )
}

#[test]
fn service_refuses_reserved_lookup_segment_before_acceptance() {
    let request = InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "by-run-id",
        "tenant-a",
        "snapshot-a",
        "2026-09-01T00:00:00Z",
        OrchestrationMode::Direct,
        128,
        vec!["span-a".into()],
        false,
    )
    .expect("wire-valid request reaches service identity policy");

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
