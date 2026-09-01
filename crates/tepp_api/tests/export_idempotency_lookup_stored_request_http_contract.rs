//! Contract tests for the quarantined export lookup stored-request GET.

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ApiError, ExportAuthorizationRequest,
    NaruonLiveService, export_idempotency_lookup_stored_request_path_key,
    naruon_export_idempotency_lookup_stored_request_exchange,
    refuse_metrics_on_export_lookup_stored_request_payload,
};

fn sample_request() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "export-live-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-live-1".into(),
        includes_source_text: false,
    }
}

fn export_post_http(body: &str) -> String {
    format!(
        "POST /v1/exports HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: export-idem-1\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn lookup_stored_request_exchange_is_quarantined_without_tenant_principal_binding() {
    assert_eq!(
        naruon_export_idempotency_lookup_stored_request_exchange(
            "https://tepp.example.test",
            "export-idem-1",
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        export_idempotency_lookup_stored_request_path_key(
            "/v1/exports/by-idempotency/export%2Fidem/request"
        ),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn live_get_without_tenant_principal_scope_fails_closed() {
    let request = sample_request();
    let body = serde_json::to_string(&request).expect("json");
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&export_post_http(&body));
    assert_eq!(posted.status_code, 200, "{}", posted.body);

    let got = service.handle_http_request(
        "GET /v1/exports/by-idempotency/export-idem-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(got.status_code, 400, "{}", got.body);
    assert!(!got.body.contains("export-live-tenant"));
    assert!(!got.body.contains("principal-analyst-1"));
    assert!(!got.body.contains("artifact-live-1"));
    assert_eq!(
        refuse_metrics_on_export_lookup_stored_request_payload(&body),
        Err(ApiError::InvalidWirePayload)
    );

    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/exports/by-idempotency/export-idem-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/exports/by-idempotency/missing/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "GET /v1/exports/by-idempotency/export-idem-1/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
}

#[test]
fn naruon_live_service_stays_post_only_for_lookup_stored_request() {
    let mut service = NaruonLiveService::new();
    let response = service.handle_http_request(
        "GET /v1/exports/by-idempotency/export-idem-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(response.status_code, 400);
    let _ = ApiError::InvalidWirePayload;
}
