//! Contract tests for the analysis-run retry-lineage GET exchange.

use tepp_api::{
    ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION, ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN,
    AnalysisRunRetryLineage, AnalysisRunRetryLineageItem, AnalysisRunStatusState, ApiError,
    naruon_analysis_run_retry_lineage_exchange, refuse_metrics_on_retry_lineage_payload,
};

#[test]
fn retry_lineage_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange =
        naruon_analysis_run_retry_lineage_exchange("https://tepp.example.test", "tepp-run-9")
            .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs/tepp-run-9/retries"
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
    let lineage = AnalysisRunRetryLineage::new(
        "tepp-run-9",
        AnalysisRunStatusState::Failed,
        "idem-9",
        vec![
            AnalysisRunRetryLineageItem::new(
                "tepp-run-10",
                AnalysisRunStatusState::Accepted,
                "idem-retry-9",
            )
            .expect("child"),
        ],
    )
    .expect("lineage");
    assert_eq!(
        lineage.contract_version,
        ANALYSIS_RUN_RETRY_LINEAGE_CONTRACT_VERSION
    );
    let json = lineage.to_json().expect("json");
    assert_eq!(refuse_metrics_on_retry_lineage_payload(&json), Ok(()));
    assert!(!json.contains("scientific_acceptance"));
    assert!(!json.contains("tenant_workspace_id"));
    assert!(!json.contains("snapshot_id"));
}

#[test]
fn retry_lineage_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_analysis_run_retry_lineage_exchange(origin, "tepp-run-9"),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        naruon_analysis_run_retry_lineage_exchange(
            "https://tepp.example.test",
            &"a".repeat(ANALYSIS_RUN_RETRY_LINEAGE_ID_MAX_LEN + 1)
        ),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_retry_lineage_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_retry_lineage_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
}
