//! Contract tests for loopback `POST /v1/project-histories/{key}/cancel`.

use tepp_api::{
    lineageweave_project_history_cancel_exchange, AnalysisRunLiveService, ApiError,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NaruonLiveService, PROJECT_HISTORY_PATH,
    ProjectHistoryCancelled, ProjectHistoryEvent, ProjectHistoryRequest,
};

fn sample_request() -> ProjectHistoryRequest {
    ProjectHistoryRequest {
        contract_version: 1,
        idempotency_key: "idem-cancel-contract".into(),
        tenant_workspace_id: "history-tenant".into(),
        project_key: "project-cancel".into(),
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

#[test]
fn live_service_cancels_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&post_http(&sample_request()));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let cancel = format!(
        "POST {PROJECT_HISTORY_PATH}/idem-cancel-contract/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ntepp-tenant-workspace-id: history-tenant\r\ncontent-length: 0\r\n\r\n"
    );
    let cancelled = service.handle_http_request(&cancel);
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let parsed = ProjectHistoryCancelled::from_json(&cancelled.body).expect("cancelled");
    assert!(parsed.cancelled);
    assert_eq!(parsed.inference_status, "temporal_association_only");
    let mut naruon = NaruonLiveService::new();
    assert_ne!(naruon.handle_http_request(&cancel).status_code, 200);
    let naruon_consumer = format!(
        "POST {PROJECT_HISTORY_PATH}/idem-cancel-contract/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ntepp-tenant-workspace-id: history-tenant\r\ncontent-length: 0\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&naruon_consumer).status_code,
        400
    );
}

#[test]
fn cancel_exchange_refuses_http_origin() {
    assert_eq!(
        lineageweave_project_history_cancel_exchange(
            "http://tepp.example.test",
            "history-tenant",
            "idem-a"
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
