//! Contract tests for `tepp-project-history-cancel cancel`.

use tepp_api::{
    compose_project_history_cancel_cli_http, dispatch_project_history_cancel_cli,
    execute_project_history_cancel_cli, render_project_history_cancel_cli_stdout,
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, PROJECT_HISTORY_PATH,
    ProjectHistoryCancelCliInvocation, ProjectHistoryCancelled, ProjectHistoryEvent,
    ProjectHistoryRequest, NaruonLiveResponse,
};

const ORIGIN: &str = "https://tepp.example.test";

fn sample_request() -> ProjectHistoryRequest {
    ProjectHistoryRequest {
        contract_version: 1,
        idempotency_key: "idem-cancel-cli".into(),
        tenant_workspace_id: "history-tenant".into(),
        project_key: "project-cancel-cli".into(),
        project_name: "Project".into(),
        knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
        focus_event_id: "focus".into(),
        events: vec![ProjectHistoryEvent {
            event_id: "focus".into(),
            event_type_code: "voc_received".into(),
            event_title: "VOC".into(),
            occurred_at: "2026-08-19T09:00:00Z".into(),
            available_at: "2026-08-19T10:00:00Z".into(),
            source_post_id: "post".into(),
            evidence_text: "explicit evidence".into(),
            actor_ids: Vec::new(),
        }],
    }
}

fn post_http(request: &ProjectHistoryRequest) -> String {
    let body = request.to_json().expect("json");
    format!(
        "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key,
        body.len()
    )
}

fn cancel_invocation(idempotency_key: &str) -> ProjectHistoryCancelCliInvocation {
    ProjectHistoryCancelCliInvocation::from_args(
        [
            "cancel",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
            "--tenant-workspace-id",
            "history-tenant",
            "--idempotency-key",
            idempotency_key,
        ],
        "",
    )
    .expect("cancel")
}

#[test]
fn dispatch_cancels_one_metric_free_identity() {
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&post_http(&sample_request()));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let cancelled =
        dispatch_project_history_cancel_cli(&mut service, &cancel_invocation("idem-cancel-cli"))
            .expect("cancel");
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let stdout = render_project_history_cancel_cli_stdout(
        &cancel_invocation("idem-cancel-cli"),
        &cancelled,
    )
    .expect("out");
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("evidence_text"));
    assert!(!stdout.contains("findings"));
    let parsed = ProjectHistoryCancelled::from_json(&stdout).expect("parsed");
    assert!(parsed.cancelled);
    assert_eq!(parsed.idempotency_key, "idem-cancel-cli");
    assert_eq!(parsed.inference_status, "temporal_association_only");
}

#[test]
fn render_refuses_metrics_schema_and_empty_bodies() {
    let cancel = cancel_invocation("idem-a");
    assert_eq!(
        render_project_history_cancel_cli_stdout(
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
        render_project_history_cancel_cli_stdout(
            &cancel,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"project_key":"p","idempotency_key":"idem-a","knowledge_cutoff":"2026-08-19T23:59:59Z","inference_status":"temporal_association_only","cancelled":true,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let http = compose_project_history_cancel_cli_http(&cancel).expect("http");
    assert!(http.starts_with("POST /v1/project-histories/idem-a/cancel HTTP/1.1"));
}

#[test]
fn execute_over_tcp_cancels_authorized_project_history() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let posted = service.handle_http_request(&post_http(&sample_request()));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let mut invocation = cancel_invocation("idem-cancel-cli");
    invocation.host = addr.to_string();
    let response = execute_project_history_cancel_cli(&invocation).expect("tcp");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_project_history_cancel_cli_stdout(&invocation, &response).expect("out");
    let parsed = ProjectHistoryCancelled::from_json(&stdout).expect("parsed");
    assert!(parsed.cancelled);
    handle.join().expect("join");
}
