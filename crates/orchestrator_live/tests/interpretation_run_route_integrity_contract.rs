//! Interpretation-run route identities fail closed before they become unreachable.

use orchestrator_live::{
    INTERPRETATION_RUN_CONTRACT_VERSION, INTERPRETATION_RUN_LOOKUP_ID_MAX_LEN,
    InterpretationRunRequest, OrchestrationMode, OrchestratorLiveError, OrchestratorLiveService,
};

#[test]
fn reserved_lookup_segment_cannot_be_an_idempotency_key() {
    let result = InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "by-run-id",
        "tenant-a",
        "snapshot-a",
        "2026-09-01T00:00:00Z",
        OrchestrationMode::Direct,
        128,
        vec!["span-a".into()],
        false,
    );

    assert_eq!(result, Err(OrchestratorLiveError::InvalidWirePayload));
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
