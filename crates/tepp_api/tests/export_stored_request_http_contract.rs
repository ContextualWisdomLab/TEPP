//! Contract tests for naruon export stored-request GET.

use tepp_api::{
    export_stored_request_path_id, naruon_export_stored_request_exchange,
    refuse_metrics_on_export_stored_request_payload, ApiError, NARUON_CONSUMER_CODE,
};

#[test]
fn stored_request_exchange_is_metric_free_get_without_credentials() {
    let exchange = naruon_export_stored_request_exchange("https://tepp.example.test", "export-1")
        .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange
        .target_url
        .ends_with("/v1/exports/export-1/request"));
    assert!(exchange.body.is_empty());
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "tepp-consumer" && value == NARUON_CONSUMER_CODE));
    assert!(!exchange
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
            || name.eq_ignore_ascii_case("idempotency-key")));
    assert_eq!(
        export_stored_request_path_id("/v1/exports/export-1/request").expect("id"),
        "export-1"
    );
    assert_eq!(
        export_stored_request_path_id("/v1/exports/export-1"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        export_stored_request_path_id("/v1/exports/export-1/cancel"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_stored_request_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(refuse_metrics_on_export_stored_request_payload(""), Ok(()));
}
