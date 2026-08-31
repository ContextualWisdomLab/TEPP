//! Contract tests for the analysis-run retry HTTP exchange.

use tepp_api::{
    ANALYSIS_RUN_RETRY_CONTRACT_VERSION, ANALYSIS_RUN_RETRY_ID_MAX_LEN, AnalysisRunRetryRequest,
    ApiError, naruon_analysis_run_retry_exchange, refuse_metrics_on_retry_payload,
};

#[test]
fn retry_exchange_is_https_post_without_credentials_or_metrics() {
    let request = AnalysisRunRetryRequest::new("tepp-run-9", "idem-retry-9").expect("request");
    assert_eq!(
        request.contract_version,
        ANALYSIS_RUN_RETRY_CONTRACT_VERSION
    );
    let exchange = naruon_analysis_run_retry_exchange("https://tepp.example.test", &request)
        .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs/tepp-run-9/retry"
    );
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "idempotency-key" && value == "idem-retry-9")
    );
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name.contains("authorization")
                || name.contains("token")
                || name.contains("copilot"))
    );
    let decoded = AnalysisRunRetryRequest::from_json(&exchange.body).expect("body");
    assert_eq!(decoded, request);
    assert_eq!(refuse_metrics_on_retry_payload(&exchange.body), Ok(()));
}

#[test]
fn retry_contract_refuses_table_access_and_metric_keys() {
    let request = AnalysisRunRetryRequest::new("tepp-run-9", "idem-retry-9").expect("request");
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_analysis_run_retry_exchange(origin, &request),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        AnalysisRunRetryRequest::new(
            "a".repeat(ANALYSIS_RUN_RETRY_ID_MAX_LEN + 1),
            "idem-retry-9"
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_retry_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_retry_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
}
