//! Contract tests for the analysis-run collection loopback CLI.

use tepp_api::{
    ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION, AnalysisRunCollection,
    AnalysisRunCollectionCliInvocation, AnalysisRunCollectionCliVerb, ApiError,
    NARUON_CONSUMER_CODE, compose_analysis_run_collection_cli_http,
};

#[test]
fn collection_cli_list_is_metric_free_get_without_credentials() {
    assert_eq!(
        AnalysisRunCollectionCliVerb::parse("list").expect("list"),
        AnalysisRunCollectionCliVerb::List
    );
    let invocation =
        AnalysisRunCollectionCliInvocation::from_args(["list", "--host", "127.0.0.1:18081"], "")
            .expect("invocation");
    assert_eq!(invocation.consumer, NARUON_CONSUMER_CODE);
    let http = compose_analysis_run_collection_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/analysis-runs HTTP/1.1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("idempotency-key"));
    assert!(!http.contains("copilot"));
    assert_eq!(
        ANALYSIS_RUN_COLLECTION_CONTRACT_VERSION,
        AnalysisRunCollection::new(Vec::new(), None)
            .expect("empty")
            .contract_version
    );
}

#[test]
fn collection_cli_refuses_non_loopback_unknown_verbs_and_metric_bodies() {
    assert_eq!(
        AnalysisRunCollectionCliInvocation::from_args(["list", "--host", "8.8.8.8:80"], ""),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        AnalysisRunCollectionCliVerb::parse("create"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunCollectionCliInvocation::from_args(
            ["list", "--host", "127.0.0.1:18081"],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
