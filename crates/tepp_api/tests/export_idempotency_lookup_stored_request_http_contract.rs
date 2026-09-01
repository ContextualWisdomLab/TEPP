//! Contract tests for export lookup stored-request GET.

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ApiError, ExportAuthorizationRequest,
    NaruonLiveService, naruon_export_idempotency_lookup_stored_request_exchange,
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
fn lookup_stored_request_exchange_is_https_get_without_credentials() {
    let exchange = naruon_export_idempotency_lookup_stored_request_exchange(
        "https://tepp.example.test",
        "export-idem-1",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/exports/by-idempotency/export-idem-1/request"
    );
    assert!(exchange.body.is_empty());
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "tepp-consumer" && value == "naruon")
    );
}

#[test]
fn live_get_returns_stored_authorization_request() {
    let request = sample_request();
    let body = serde_json::to_string(&request).expect("json");
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&export_post_http(&body));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let got = service.handle_http_request(
        "GET /v1/exports/by-idempotency/export-idem-1/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(got.status_code, 200, "{}", got.body);
    assert_eq!(
        refuse_metrics_on_export_lookup_stored_request_payload(&got.body),
        Ok(())
    );
    assert!(!got.body.contains("tepp.scientific_acceptance.v1"));
    assert!(!got.body.contains("rmse"));
    assert!(got.body.contains("\"artifact_id\":\"artifact-live-1\""));
    assert!(
        got.body
            .contains("\"tenant_workspace_id\":\"export-live-tenant\"")
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
