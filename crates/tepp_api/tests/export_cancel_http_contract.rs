//! Contract tests for loopback `POST /v1/exports/{export_id}/cancel`.

use tepp_api::{
    naruon_export_cancel_exchange, AnalysisRunLiveService, AnalyticalPurpose, ApiError,
    ExportAuthorizationRequest, ExportCancelled, ExportCollection, NARUON_CONSUMER_CODE,
    NARUON_EXPORT_PATH, NaruonLiveService,
};

fn authorize_body() -> String {
    let request = ExportAuthorizationRequest {
        tenant_workspace_id: "export-cancel-contract-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-cancel-contract-1".into(),
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

#[test]
fn live_service_cancels_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&authorize_http("export-cancel-contract-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval: serde_json::Value = serde_json::from_str(&posted.body).expect("posted");
    let export_id = retrieval["export_id"].as_str().expect("id");
    let cancel = format!(
        "POST {NARUON_EXPORT_PATH}/{export_id}/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    );
    let cancelled = service.handle_http_request(&cancel);
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let parsed = ExportCancelled::from_json(&cancelled.body).expect("cancelled");
    assert!(parsed.cancelled);
    assert_eq!(parsed.export_id, export_id);
    let listed = service.handle_http_request(&format!(
        "GET {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    ));
    let page: ExportCollection = serde_json::from_str(&listed.body).expect("page");
    assert!(page.items.is_empty());
    let mut naruon = NaruonLiveService::new();
    assert_ne!(naruon.handle_http_request(&cancel).status_code, 200);
}

#[test]
fn cancel_exchange_refuses_http_origin() {
    assert_eq!(
        naruon_export_cancel_exchange("http://tepp.example.test", "export-1"),
        Err(ApiError::InvalidWirePayload)
    );
}
