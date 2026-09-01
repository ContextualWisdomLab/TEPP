//! Contract tests for `tepp-export-cancel cancel`.

use tepp_api::{
    compose_export_cancel_cli_http, dispatch_export_cancel_cli, execute_export_cancel_cli,
    render_export_cancel_cli_stdout, AnalysisRunLiveService, AnalyticalPurpose, ApiError,
    ExportAuthorizationRequest, ExportCancelCliInvocation, ExportCancelled, NARUON_CONSUMER_CODE,
    NARUON_EXPORT_PATH, NaruonLiveResponse,
};

const ORIGIN: &str = "https://tepp.example.test";

fn authorize_body() -> String {
    let request = ExportAuthorizationRequest {
        tenant_workspace_id: "export-cancel-cli-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-cancel-cli-1".into(),
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

fn cancel_invocation(export_id: &str) -> ExportCancelCliInvocation {
    ExportCancelCliInvocation::from_args(
        [
            "cancel",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            NARUON_CONSUMER_CODE,
            "--export-id",
            export_id,
        ],
        "",
    )
    .expect("cancel")
}

#[test]
fn dispatch_cancels_one_metric_free_identity() {
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&authorize_http("export-cancel-cli-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval: serde_json::Value = serde_json::from_str(&posted.body).expect("posted");
    let export_id = retrieval["export_id"].as_str().expect("id");
    let cancelled =
        dispatch_export_cancel_cli(&mut service, &cancel_invocation(export_id)).expect("cancel");
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let stdout =
        render_export_cancel_cli_stdout(&cancel_invocation(export_id), &cancelled).expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("principal_id"));
    let parsed = ExportCancelled::from_json(&stdout).expect("parsed");
    assert!(parsed.cancelled);
    assert_eq!(parsed.export_id, export_id);
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let cancel = cancel_invocation("export-1");
    assert_eq!(
        render_export_cancel_cli_stdout(
            &cancel,
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
        render_export_cancel_cli_stdout(
            &cancel,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"export_id":"e","artifact_id":"a","decision_code":"purpose_bound_export_allowed","purpose":"modular_service_consumer","idempotency_key":"k","cancelled":true,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let http = compose_export_cancel_cli_http(&cancel).expect("http");
    assert!(http.starts_with("POST /v1/exports/export-1/cancel HTTP/1.1"));
}

#[test]
fn execute_over_tcp_cancels_authorized_export() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let posted = service.handle_http_request(&authorize_http("export-cancel-tcp-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval: serde_json::Value = serde_json::from_str(&posted.body).expect("posted");
    let export_id = retrieval["export_id"].as_str().expect("id").to_owned();
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let mut invocation = cancel_invocation(&export_id);
    invocation.host = addr.to_string();
    let response = execute_export_cancel_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_export_cancel_cli_stdout(&invocation, &response).expect("out");
    let parsed = ExportCancelled::from_json(&stdout).expect("parsed");
    assert!(parsed.cancelled);
    handle.join().expect("join");
}
