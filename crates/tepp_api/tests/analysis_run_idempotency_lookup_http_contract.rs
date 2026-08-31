//! Contract tests for the analysis-run idempotency-key lookup GET exchange.

use tepp_api::{
    ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION, ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN,
    AnalysisRunIdempotencyLookup, AnalysisRunStatusState, ApiError,
    naruon_analysis_run_idempotency_lookup_exchange, refuse_metrics_on_idempotency_lookup_payload,
};

#[test]
fn idempotency_lookup_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange =
        naruon_analysis_run_idempotency_lookup_exchange("https://tepp.example.test", "idem-9")
            .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs/by-idempotency/idem-9"
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
    let lookup =
        AnalysisRunIdempotencyLookup::new("tepp-run-9", AnalysisRunStatusState::Failed, "idem-9")
            .expect("lookup");
    assert_eq!(
        lookup.contract_version,
        ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_CONTRACT_VERSION
    );
    let json = lookup.to_json().expect("json");
    assert_eq!(refuse_metrics_on_idempotency_lookup_payload(&json), Ok(()));
    assert!(!json.contains("scientific_acceptance"));
    assert!(!json.contains("tenant_workspace_id"));
    assert!(!json.contains("snapshot_id"));
}

#[test]
fn idempotency_lookup_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_analysis_run_idempotency_lookup_exchange(origin, "idem-9"),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        naruon_analysis_run_idempotency_lookup_exchange(
            "https://tepp.example.test",
            &"a".repeat(ANALYSIS_RUN_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1)
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_idempotency_lookup_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_idempotency_lookup_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
}
