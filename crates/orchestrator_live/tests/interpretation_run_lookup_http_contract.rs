//! Contract tests for contextual-orchestrator interpretation-run lookup GET.

use std::io::{Read, Write};

use orchestrator_live::{
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, INTERPRETATION_RUN_CONTRACT_VERSION,
    INTERPRETATION_RUN_LOOKUP_PREFIX, INTERPRETATION_RUN_PATH, InterpretationRunRequest,
    OrchestrationMode, OrchestratorLiveError, OrchestratorLiveService,
    contextual_orchestrator_interpretation_run_lookup_exchange, interpretation_run_lookup_path_id,
    interpretation_run_retrieval_path_id, is_interpretation_run_lookup_path,
};

fn sample_request() -> InterpretationRunRequest {
    InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "orch-live-idem-001",
        "orch-tenant-workspace-demo",
        "tepp-snapshot-demo-001",
        "2026-08-01T00:00:00Z",
        OrchestrationMode::Direct,
        2048,
        vec!["span-001".into()],
        false,
    )
    .expect("sample")
}

fn post_http(request: &InterpretationRunRequest) -> String {
    let body = request.to_json().expect("json");
    format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key(),
        body.len()
    )
}

#[test]
fn lookup_exchange_is_metric_free_get_without_credentials() {
    let exchange = contextual_orchestrator_interpretation_run_lookup_exchange(
        "https://tepp.example.test",
        "orch-run-1",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/interpretation-runs/by-run-id/orch-run-1")
    );
    assert!(exchange.body.is_empty());
    assert_eq!(
        interpretation_run_lookup_path_id("/v1/interpretation-runs/by-run-id/orch-run-1")
            .expect("id"),
        "orch-run-1"
    );
    assert!(!is_interpretation_run_lookup_path(
        "/v1/interpretation-runs/orch-run-1"
    ));
    assert_eq!(INTERPRETATION_RUN_LOOKUP_PREFIX, "by-run-id");
}

#[test]
fn live_get_returns_metric_free_identity_for_unique_run_id() {
    let request = sample_request();
    let mut service = OrchestratorLiveService::new();
    let accepted = service.handle_http_request(&post_http(&request));
    assert_eq!(accepted.status_code, 202, "{}", accepted.body);
    assert!(
        accepted
            .body
            .contains("\"interpretation_run_id\":\"orch-run-1\"")
    );
    let got = service.handle_http_request(
        "GET /v1/interpretation-runs/by-run-id/orch-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(got.status_code, 200, "{}", got.body);
    assert!(
        got.body
            .contains("\"interpretation_run_id\":\"orch-run-1\"")
    );
    assert!(
        got.body
            .contains("\"idempotency_key\":\"orch-live-idem-001\"")
    );
    assert!(got.body.contains("\"claim_status\":\"hypothetical\""));
    assert!(got.body.contains("\"scientific_authority\":false"));
    assert!(!got.body.contains("tepp.scientific_acceptance.v1"));
    assert!(!got.body.contains("rmse"));
    assert!(!got.body.contains("evidence_span_ids"));
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/orch-run-missing HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/orch-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/orch-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 1\r\n\r\nx"
            )
            .status_code,
        400
    );
    assert_eq!(
        interpretation_run_retrieval_path_id("/v1/interpretation-runs/by-run-id"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}

#[test]
fn lookup_serves_over_tcp() {
    let request = sample_request();
    let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
    assert_eq!(
        service
            .handle_http_request(&post_http(&request))
            .status_code,
        202
    );
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let http = "GET /v1/interpretation-runs/by-run-id/orch-run-1 HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n";
    let mut stream = std::net::TcpStream::connect(addr).expect("connect");
    stream.write_all(http.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read");
    let text = String::from_utf8(bytes).expect("utf8");
    assert!(text.contains("HTTP/1.1 200"), "{text}");
    assert!(text.contains("orch-run-1"), "{text}");
    assert!(text.contains("\"scientific_authority\":false"), "{text}");
    assert!(text.contains("\"claim_status\":\"hypothetical\""), "{text}");
    handle.join().expect("join");
}
