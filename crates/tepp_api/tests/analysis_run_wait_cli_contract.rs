//! Contract tests for the analysis-run wait loopback CLI.

use tepp_api::{
    AnalysisRunWaitCliInvocation, AnalysisRunWaitCliVerb, ApiError, NARUON_CONSUMER_CODE,
};

#[test]
fn wait_cli_is_status_poll_without_credentials() {
    assert_eq!(
        AnalysisRunWaitCliVerb::parse("wait").expect("verb"),
        AnalysisRunWaitCliVerb::Wait
    );
    let invocation = AnalysisRunWaitCliInvocation::from_args(
        [
            "wait",
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
    assert_eq!(invocation.status.consumer, NARUON_CONSUMER_CODE);
    assert!(!invocation.status.host.contains("authorization"));
}

#[test]
fn wait_cli_refuses_non_loopback_unknown_verbs_and_bodies() {
    assert_eq!(
        AnalysisRunWaitCliInvocation::from_args(
            [
                "wait",
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
        AnalysisRunWaitCliVerb::parse("lookup"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunWaitCliInvocation::from_args(
            [
                "wait",
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
