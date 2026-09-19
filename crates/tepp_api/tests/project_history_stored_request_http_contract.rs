//! Contract tests for `LineageWeave` project-history stored-request GET.

use tepp_api::{
    lineageweave_project_history_stored_request_exchange, project_history_stored_request_path_id,
    ApiError, LINEAGEWEAVE_CONSUMER_CODE,
};

#[test]
fn stored_request_exchange_is_metric_free_get_without_credentials() {
    let exchange = lineageweave_project_history_stored_request_exchange(
        "https://tepp.example.test",
        "history-tenant",
        "idem-a",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange
        .target_url
        .ends_with("/v1/project-histories/idem-a/request"));
    assert!(exchange.body.is_empty());
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "tepp-consumer" && value == LINEAGEWEAVE_CONSUMER_CODE));
    assert!(!exchange
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
            || name.eq_ignore_ascii_case("idempotency-key")));
    assert_eq!(
        project_history_stored_request_path_id("/v1/project-histories/idem-a/request").expect("id"),
        "idem-a"
    );
    assert_eq!(
        project_history_stored_request_path_id("/v1/project-histories/idem-a"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        project_history_stored_request_path_id("/v1/project-histories/idem-a/cancel"),
        Err(ApiError::InvalidWirePayload)
    );
}
