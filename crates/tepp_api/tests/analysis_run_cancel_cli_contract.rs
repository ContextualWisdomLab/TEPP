//! Contract tests for the analysis-run cancel loopback CLI.

use tepp_api::{
    ANALYSIS_RUN_CANCEL_CONTRACT_VERSION, AnalysisRunCancelCliInvocation, AnalysisRunCancelCliVerb,
    AnalysisRunCancelRequest, ApiError, NARUON_CONSUMER_CODE, compose_analysis_run_cancel_cli_http,
};

#[test]
fn cancel_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        AnalysisRunCancelCliVerb::parse("cancel").expect("cancel"),
        AnalysisRunCancelCliVerb::Cancel
    );
    let invocation = AnalysisRunCancelCliInvocation::from_args(
        [
            "cancel",
            "--host",
            "127.0.0.1:18081",
            "--run-id",
            "tepp-run-1",
            "--idempotency-key",
            "idem-1",
        ],
        "",
    )
    .expect("invocation");
    assert_eq!(invocation.consumer, NARUON_CONSUMER_CODE);
    let http = compose_analysis_run_cancel_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/analysis-runs/tepp-run-1/cancel HTTP/1.1"));
    assert!(http.contains("idempotency-key: idem-1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("copilot"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert_eq!(
        ANALYSIS_RUN_CANCEL_CONTRACT_VERSION,
        AnalysisRunCancelRequest::new("tepp-run-1", "idem-1")
            .expect("request")
            .contract_version
    );
}

#[test]
fn cancel_cli_refuses_non_loopback_unknown_verbs_and_metric_bodies() {
    assert_eq!(
        AnalysisRunCancelCliInvocation::from_args(
            [
                "cancel",
                "--host",
                "8.8.8.8:80",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1"
            ],
            ""
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        AnalysisRunCancelCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunCancelCliInvocation::from_args(
            [
                "cancel",
                "--host",
                "127.0.0.1:18081",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
