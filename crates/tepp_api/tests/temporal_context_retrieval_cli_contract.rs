//! Contract tests for `tepp-temporal-context-get get`.

use tepp_api::{
    compose_temporal_context_retrieval_cli_http, dispatch_temporal_context_retrieval_cli,
    execute_temporal_context_retrieval_cli, render_temporal_context_retrieval_cli_stdout,
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    NaruonLiveResponse, TEMPORAL_CONTEXT_PATH, TemporalContextRetrieved,
    TemporalContextRetrievalCliInvocation,
};

const ORIGIN: &str = "https://tepp.example.test";
const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

fn post_http(idempotency_key: &str) -> String {
    format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{TEMPORAL_BODY}",
        TEMPORAL_BODY.len()
    )
}

fn get_invocation(host: &str, key: &str) -> TemporalContextRetrievalCliInvocation {
    TemporalContextRetrievalCliInvocation::from_args(
        [
            "get",
            "--host",
            host,
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
            "--idempotency-key",
            key,
        ],
        "",
    )
    .expect("get")
}

#[test]
fn dispatch_gets_one_metric_free_identity() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(
        service.handle_http_request(&post_http("idem-cli")).status_code,
        200
    );
    let listed = dispatch_temporal_context_retrieval_cli(
        &mut service,
        &get_invocation("127.0.0.1:18081", "idem-cli"),
    )
    .expect("get");
    assert_eq!(listed.status_code, 200, "{}", listed.body);
    let stdout = render_temporal_context_retrieval_cli_stdout(
        &get_invocation("127.0.0.1:18081", "idem-cli"),
        &listed,
    )
    .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("event_label"));
    assert!(!stdout.contains("actor_references"));
    let row = TemporalContextRetrieved::from_json(&stdout).expect("row");
    assert_eq!(row.idempotency_key, "idem-cli");
    assert_eq!(row.inference_status, "temporal_association_only");
}

#[test]
fn render_refuses_metrics_naruon_and_empty_bodies() {
    let get = get_invocation("127.0.0.1:18081", "idem-cli");
    assert_eq!(
        render_temporal_context_retrieval_cli_stdout(
            &get,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: String::new(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        TemporalContextRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--consumer",
                NARUON_CONSUMER_CODE,
                "--idempotency-key",
                "idem-cli"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let http = compose_temporal_context_retrieval_cli_http(&get).expect("http");
    assert!(http.starts_with("GET /v1/temporal-context/idem-cli HTTP/1.1"));
}

#[test]
fn execute_over_tcp_retrieves_authorized_identity() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    assert_eq!(
        service
            .handle_http_request(&post_http("idem-tcp"))
            .status_code,
        200
    );
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = get_invocation(&addr.to_string(), "idem-tcp");
    let response = execute_temporal_context_retrieval_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_temporal_context_retrieval_cli_stdout(&invocation, &response).expect("out");
    let row = TemporalContextRetrieved::from_json(&stdout).expect("row");
    assert_eq!(row.idempotency_key, "idem-tcp");
    handle.join().expect("join");
}
