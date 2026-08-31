//! Contract tests for the analysis-run stored-request GET exchange.

use tepp_api::{
    ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION, ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN,
    AnalysisRunStatusState, AnalysisRunStoredRequest, ApiError,
    naruon_analysis_run_stored_request_exchange, refuse_metrics_on_stored_request_payload,
};

#[test]
fn stored_request_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange =
        naruon_analysis_run_stored_request_exchange("https://tepp.example.test", "tepp-run-9")
            .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs/tepp-run-9/request"
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
    let stored = AnalysisRunStoredRequest::new(
        "tepp-run-9",
        AnalysisRunStatusState::Failed,
        "idem-9",
        "snapshot-9",
        "2026-08-01T00:00:00Z",
        "tepp-analysis-run-v1",
        "calibrated_event_measurement",
    )
    .expect("stored");
    assert_eq!(
        stored.contract_version,
        ANALYSIS_RUN_STORED_REQUEST_CONTRACT_VERSION
    );
    let json = stored.to_json().expect("json");
    assert_eq!(refuse_metrics_on_stored_request_payload(&json), Ok(()));
    assert!(!json.contains("scientific_acceptance"));
    assert!(!json.contains("tenant_workspace_id"));
}

#[test]
fn stored_request_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_analysis_run_stored_request_exchange(origin, "tepp-run-9"),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        naruon_analysis_run_stored_request_exchange(
            "https://tepp.example.test",
            &"a".repeat(ANALYSIS_RUN_STORED_REQUEST_ID_MAX_LEN + 1)
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_stored_request_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_stored_request_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
}
