//! Contract tests for `tepp-temporal-contexts list`.

use tepp_api::{
    compose_temporal_context_collection_cli_http, dispatch_temporal_context_collection_cli,
    execute_temporal_context_collection_cli, render_temporal_context_collection_cli_stdout,
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, TEMPORAL_CONTEXT_PATH,
    TemporalContextCollection, TemporalContextCollectionCliInvocation, NaruonLiveResponse,
};

const ORIGIN: &str = "https://tepp.example.test";
const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

fn post_http(idempotency_key: &str) -> String {
    format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{TEMPORAL_BODY}",
        TEMPORAL_BODY.len()
    )
}

fn list_invocation(host: &str) -> TemporalContextCollectionCliInvocation {
    TemporalContextCollectionCliInvocation::from_args(
        [
            "list",
            "--host",
            host,
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ],
        "",
    )
    .expect("list")
}

#[test]
fn dispatch_lists_one_metric_free_page() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(
        service.handle_http_request(&post_http("idem-cli")).status_code,
        200
    );
    let listed =
        dispatch_temporal_context_collection_cli(&mut service, &list_invocation("127.0.0.1:18081"))
            .expect("list");
    assert_eq!(listed.status_code, 200, "{}", listed.body);
    let stdout = render_temporal_context_collection_cli_stdout(
        &list_invocation("127.0.0.1:18081"),
        &listed,
    )
    .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("event_label"));
    assert!(!stdout.contains("actor_references"));
    let page = TemporalContextCollection::from_json(&stdout).expect("page");
    assert_eq!(page.contexts.len(), 1);
    assert_eq!(page.contexts[0].idempotency_key, "idem-cli");
    assert_eq!(page.contexts[0].inference_status, "temporal_association_only");
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let list = list_invocation("127.0.0.1:18081");
    assert_eq!(
        render_temporal_context_collection_cli_stdout(
            &list,
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
        render_temporal_context_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"contexts":[],"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let http = compose_temporal_context_collection_cli_http(&list).expect("http");
    assert!(http.starts_with("GET /v1/temporal-context HTTP/1.1"));
}

#[test]
fn execute_over_tcp_lists_authorized_identities() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    assert_eq!(
        service.handle_http_request(&post_http("idem-tcp")).status_code,
        200
    );
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = list_invocation(&addr.to_string());
    let response = execute_temporal_context_collection_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_temporal_context_collection_cli_stdout(&invocation, &response).expect("out");
    let page = TemporalContextCollection::from_json(&stdout).expect("page");
    assert_eq!(page.contexts[0].idempotency_key, "idem-tcp");
    handle.join().expect("join");
}
