//! Contract tests for contextual-orchestrator lookup stored-request GET.

use std::io::{Read, Write};

use orchestrator_live::{
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE, INTERPRETATION_RUN_CONTRACT_VERSION,
    INTERPRETATION_RUN_PATH, InterpretationRunRequest, OrchestrationMode, OrchestratorLiveService,
    contextual_orchestrator_interpretation_run_lookup_stored_request_exchange,
    interpretation_run_lookup_stored_request_path_id, is_interpretation_run_lookup_path,
    is_interpretation_run_lookup_stored_request_path, is_interpretation_run_stored_request_path,
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
fn lookup_stored_request_exchange_is_metric_free_get_without_credentials() {
    let exchange = contextual_orchestrator_interpretation_run_lookup_stored_request_exchange(
        "https://tepp.example.test",
        "orch-run-1",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/interpretation-runs/by-run-id/orch-run-1/request")
    );
    assert!(exchange.body.is_empty());
    assert_eq!(
        interpretation_run_lookup_stored_request_path_id(
            "/v1/interpretation-runs/by-run-id/orch-run-1/request"
        )
        .expect("id"),
        "orch-run-1"
    );
    assert!(is_interpretation_run_lookup_stored_request_path(
        "/v1/interpretation-runs/by-run-id/orch-run-1/request"
    ));
    assert!(!is_interpretation_run_lookup_path(
        "/v1/interpretation-runs/by-run-id/orch-run-1/request"
    ));
    assert!(!is_interpretation_run_stored_request_path(
        "/v1/interpretation-runs/by-run-id/orch-run-1/request"
    ));
}

#[test]
fn live_get_returns_stored_request_without_scientific_authority() {
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
        "GET /v1/interpretation-runs/by-run-id/orch-run-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stored = InterpretationRunRequest::from_json(&got.body).expect("stored");
    assert_eq!(stored, request);
    assert!(!got.body.contains("tepp.scientific_acceptance.v1"));
    assert!(!got.body.contains("rmse"));
    assert!(got.body.contains("\"scientific_authority\":false"));
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/orch-run-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/missing/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/interpretation-runs/by-run-id/orch-run-1/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
}

#[test]
fn lookup_stored_request_serves_over_tcp() {
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
    let http = "GET /v1/interpretation-runs/by-run-id/orch-run-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n";
    let mut stream = std::net::TcpStream::connect(addr).expect("connect");
    stream.write_all(http.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read");
    let text = String::from_utf8(bytes).expect("utf8");
    assert!(text.contains("HTTP/1.1 200"), "{text}");
    assert!(text.contains("orch-live-idem-001"), "{text}");
    assert!(text.contains("\"scientific_authority\":false"), "{text}");
    handle.join().expect("join");
}
