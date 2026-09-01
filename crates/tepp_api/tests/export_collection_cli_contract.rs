//! Contract tests for `tepp-export-list list`.

use tepp_api::{
    compose_export_collection_cli_http, dispatch_export_collection_cli,
    execute_export_collection_cli, render_export_collection_cli_stdout, AnalysisRunLiveService,
    AnalyticalPurpose, ApiError, ExportAuthorizationRequest, ExportCollection,
    ExportCollectionCliInvocation, NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH, NaruonLiveResponse,
};

const ORIGIN: &str = "https://tepp.example.test";

fn authorize_body() -> String {
    let request = ExportAuthorizationRequest {
        tenant_workspace_id: "export-cli-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-cli-1".into(),
        includes_source_text: false,
    };
    serde_json::to_string(&request).expect("json")
}

fn authorize_http(idem: &str) -> String {
    let body = authorize_body();
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idem}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn list_invocation() -> ExportCollectionCliInvocation {
    ExportCollectionCliInvocation::from_args(
        [
            "list",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            NARUON_CONSUMER_CODE,
        ],
        "",
    )
    .expect("list")
}

#[test]
fn dispatch_lists_one_metric_free_identity() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&authorize_http("export-idem-1"))
            .status_code,
        200
    );
    let listed = dispatch_export_collection_cli(&mut service, &list_invocation()).expect("list");
    assert_eq!(listed.status_code, 200, "{}", listed.body);
    let stdout = render_export_collection_cli_stdout(&list_invocation(), &listed).expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("principal_id"));
    assert!(!stdout.contains("includes_source_text"));
    let page: ExportCollection = serde_json::from_str(&stdout).expect("page");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].idempotency_key, "export-idem-1");
    assert_eq!(page.items[0].decision_code, "purpose_bound_export_allowed");
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let list = list_invocation();
    assert_eq!(
        render_export_collection_cli_stdout(
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
        render_export_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"items":[{"contract_version":1,"export_id":"e","artifact_id":"a","decision_code":"purpose_bound_export_allowed","purpose":"modular_service_consumer","idempotency_key":"k","rmse":1.0}]}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let http = compose_export_collection_cli_http(&list).expect("http");
    assert!(http.starts_with("GET /v1/exports HTTP/1.1"));
}

#[test]
fn execute_over_tcp_returns_empty_collection() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let mut invocation = list_invocation();
    invocation.host = addr.to_string();
    let response = execute_export_collection_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_export_collection_cli_stdout(&invocation, &response).expect("out");
    let page: ExportCollection = serde_json::from_str(&stdout).expect("page");
    assert!(page.items.is_empty());
    handle.join().expect("join");
}
