//! Contract tests for the export retrieval GET exchange.

use tepp_api::{
    ApiError, EXPORT_RETRIEVAL_CONTRACT_VERSION, EXPORT_RETRIEVAL_ID_MAX_LEN, ExportRetrieval,
    naruon_export_retrieval_exchange, refuse_metrics_on_export_retrieval_payload,
};

#[test]
fn export_retrieval_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange = naruon_export_retrieval_exchange("https://tepp.example.test", "export-9")
        .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/exports/export-9"
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
    let payload = ExportRetrieval::new(
        "export-9",
        "artifact-9",
        "purpose_bound_export_allowed",
        "modular_service_consumer",
        "export-idem-9",
    )
    .expect("payload");
    assert_eq!(payload.contract_version, EXPORT_RETRIEVAL_CONTRACT_VERSION);
    let json = payload.to_json().expect("json");
    assert_eq!(refuse_metrics_on_export_retrieval_payload(&json), Ok(()));
    assert!(!json.contains("scientific_acceptance"));
    assert!(!json.contains("tenant_workspace_id"));
    assert!(!json.contains("principal_id"));
    assert!(!json.contains("includes_source_text"));
    assert!(!json.contains("terminal_result"));
}

#[test]
fn export_retrieval_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_export_retrieval_exchange(origin, "export-9"),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        naruon_export_retrieval_exchange(
            "https://tepp.example.test",
            &"a".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1)
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_export_retrieval_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_retrieval_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_retrieval_payload(r#"{"includes_source_text":true}"#),
        Err(ApiError::InvalidWirePayload)
    );
}
