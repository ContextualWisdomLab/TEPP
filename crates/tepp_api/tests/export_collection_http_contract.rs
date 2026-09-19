//! Contract tests for naruon export collection GET.

use tepp_api::{
    is_export_collection_path, naruon_export_collection_exchange, ApiError, NARUON_CONSUMER_CODE,
};

#[test]
fn export_collection_is_metric_free_get_without_credentials() {
    let exchange =
        naruon_export_collection_exchange("https://tepp.example.test").expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange.target_url.ends_with("/v1/exports"));
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
    assert!(is_export_collection_path("/v1/exports"));
    assert!(!is_export_collection_path("/v1/exports/export-1"));
}

#[test]
fn export_collection_refuses_insecure_origins() {
    assert_eq!(
        naruon_export_collection_exchange("http://tepp.example.test"),
        Err(ApiError::InvalidWirePayload)
    );
}
