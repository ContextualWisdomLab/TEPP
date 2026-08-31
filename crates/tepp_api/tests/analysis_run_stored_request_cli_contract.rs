//! Contract tests for the analysis-run stored-request loopback CLI.

use tepp_api::{
    AnalysisRunStoredRequestCliInvocation, AnalysisRunStoredRequestCliVerb, ApiError,
    NARUON_CONSUMER_CODE, compose_analysis_run_stored_request_cli_http,
};

#[test]
fn stored_request_cli_is_metric_free_get_without_credentials() {
    assert_eq!(
        AnalysisRunStoredRequestCliVerb::parse("stored-request").expect("verb"),
        AnalysisRunStoredRequestCliVerb::StoredRequest
    );
    let invocation = AnalysisRunStoredRequestCliInvocation::from_args(
        [
            "stored-request",
            "--host",
            "127.0.0.1:18081",
            "--run-id",
            "tepp-run-1",
        ],
        "",
    )
    .expect("invocation");
    assert_eq!(invocation.consumer, NARUON_CONSUMER_CODE);
    let http = compose_analysis_run_stored_request_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/analysis-runs/tepp-run-1/request HTTP/1.1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("copilot"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/cancel"));
    assert!(!http.contains("/retry"));
}

#[test]
fn stored_request_cli_refuses_non_loopback_unknown_verbs_and_bodies() {
    assert_eq!(
        AnalysisRunStoredRequestCliInvocation::from_args(
            [
                "stored-request",
                "--host",
                "8.8.8.8:80",
                "--run-id",
                "tepp-run-1"
            ],
            ""
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        AnalysisRunStoredRequestCliVerb::parse("retry"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunStoredRequestCliInvocation::from_args(
            [
                "stored-request",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                "tepp-run-1"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
