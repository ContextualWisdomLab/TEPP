//! Contract tests for the analysis-run status loopback CLI.

use tepp_api::{
    AnalysisRunStatusCliInvocation, AnalysisRunStatusCliVerb, ApiError, NARUON_CONSUMER_CODE,
    compose_analysis_run_status_cli_http,
};

#[test]
fn status_cli_is_metric_free_get_without_credentials() {
    assert_eq!(
        AnalysisRunStatusCliVerb::parse("status").expect("status"),
        AnalysisRunStatusCliVerb::Status
    );
    let invocation = AnalysisRunStatusCliInvocation::from_args(
        [
            "status",
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
    let http = compose_analysis_run_status_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/analysis-runs/tepp-run-1 HTTP/1.1"));
    assert!(http.contains("idempotency-key: idem-1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("copilot"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/cancel"));
}

#[test]
fn status_cli_refuses_non_loopback_unknown_verbs_and_bodies() {
    assert_eq!(
        AnalysisRunStatusCliInvocation::from_args(
            [
                "status",
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
        AnalysisRunStatusCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunStatusCliVerb::parse("create"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunStatusCliInvocation::from_args(
            [
                "status",
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
