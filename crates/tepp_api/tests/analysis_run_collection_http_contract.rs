//! Contract tests for the analysis-run collection GET exchange.

use tepp_api::{
    ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION, ANALYSIS_RUN_COLLECTION_MAX_LIMIT,
    AnalysisRunCollection, AnalysisRunCollectionItem, AnalysisRunStatusState, ApiError,
    is_analysis_run_collection_path, naruon_analysis_run_collection_exchange,
    parse_collection_page_limit, refuse_metrics_on_collection_payload,
};

#[test]
fn collection_exchange_is_https_get_without_credentials_or_metrics() {
    let exchange = naruon_analysis_run_collection_exchange("https://tepp.example.test", None, None)
        .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs"
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
    assert_eq!(
        ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION,
        AnalysisRunCollection::new(Vec::new(), None)
            .expect("empty")
            .contract_version
    );
    assert!(is_analysis_run_collection_path("/v1/analysis-runs"));
    assert!(!is_analysis_run_collection_path(
        "/v1/analysis-runs/tepp-run-1"
    ));
}

#[test]
fn collection_contract_refuses_table_access_and_metric_keys() {
    for origin in [
        "http://tepp.example.test",
        "https://db.postgres.example",
        "https://jdbc.example",
    ] {
        assert_eq!(
            naruon_analysis_run_collection_exchange(origin, None, None),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
    assert_eq!(
        parse_collection_page_limit(Some(&(ANALYSIS_RUN_COLLECTION_MAX_LIMIT + 1).to_string())),
        Err(ApiError::LimitExceeded)
    );
    assert_eq!(
        refuse_metrics_on_collection_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_collection_payload(r#"{"scientific_acceptance":{}}"#),
        Err(ApiError::InvalidWirePayload)
    );
    let item =
        AnalysisRunCollectionItem::new("tepp-run-9", AnalysisRunStatusState::Cancelled, "idem-9")
            .expect("item");
    let json = AnalysisRunCollection::new(vec![item], None)
        .expect("page")
        .to_json()
        .expect("json");
    assert!(!json.contains("terminal_result"));
    assert!(!json.contains("scientific_acceptance"));
}
