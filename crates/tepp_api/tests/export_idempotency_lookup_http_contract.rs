//! Contract tests for the export idempotency-key lookup GET exchange.

use tepp_api::{
    ApiError, EXPORT_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION, EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN,
    ExportIdempotencyLookup, NaruonLiveService, naruon_export_idempotency_lookup_exchange,
    refuse_metrics_on_export_idempotency_lookup_payload,
};

#[test]
fn export_idempotency_lookup_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange = naruon_export_idempotency_lookup_exchange("https://tepp.example.test", "idem-9")
        .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/exports/by-idempotency/idem-9"
    );
    assert!(exchange.body.is_empty());
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "tepp-consumer" && value == "naruon")
    );
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name.contains("authorization")
                || name.contains("token")
                || name.contains("copilot")
                || name.contains("idempotency"))
    );
    let lookup = ExportIdempotencyLookup::new("export-9", "purpose_bound_export_allowed", "idem-9")
        .expect("lookup");
    assert_eq!(
        lookup.contract_version,
        EXPORT_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION
    );
    let json = lookup.to_json().expect("json");
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(&json),
        Ok(())
    );
    assert!(!json.contains("scientific_acceptance"));
    assert!(!json.contains("tenant_workspace_id"));
    assert!(!json.contains("principal_id"));
    assert!(!json.contains("includes_source_text"));
    assert!(!json.contains("terminal_result"));
}

#[test]
fn export_idempotency_lookup_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_export_idempotency_lookup_exchange(origin, "idem-9"),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        naruon_export_idempotency_lookup_exchange(
            "https://tepp.example.test",
            &"a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        naruon_export_idempotency_lookup_exchange("https://tepp.example.test", "by-idempotency"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn naruon_live_service_stays_post_only_for_export_lookup() {
    let mut service = NaruonLiveService::new();
    let response = service.handle_http_request(
        "GET /v1/exports/by-idempotency/idem-a HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
    );
    assert_eq!(response.status_code, 400);
}
