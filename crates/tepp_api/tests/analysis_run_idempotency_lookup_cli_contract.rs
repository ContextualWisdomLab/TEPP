//! Contract tests for the analysis-run idempotency-lookup loopback CLI.

use tepp_api::{
    AnalysisRunIdempotencyLookupCliInvocation, AnalysisRunIdempotencyLookupCliVerb, ApiError,
    NARUON_CONSUMER_CODE, compose_analysis_run_idempotency_lookup_cli_http,
};

#[test]
fn lookup_cli_is_metric_free_get_without_credentials() {
    assert_eq!(
        AnalysisRunIdempotencyLookupCliVerb::parse("lookup").expect("verb"),
        AnalysisRunIdempotencyLookupCliVerb::Lookup
    );
    let invocation = AnalysisRunIdempotencyLookupCliInvocation::from_args(
        [
            "lookup",
            "--host",
            "127.0.0.1:18081",
            "--idempotency-key",
            "idem-1",
        ],
        "",
    )
    .expect("invocation");
    assert_eq!(invocation.consumer, NARUON_CONSUMER_CODE);
    let http = compose_analysis_run_idempotency_lookup_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/analysis-runs/by-idempotency/idem-1 HTTP/1.1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("idempotency-key:"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/cancel"));
    assert!(!http.contains("/retry"));
}

#[test]
fn lookup_cli_refuses_non_loopback_unknown_verbs_and_bodies() {
    assert_eq!(
        AnalysisRunIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "8.8.8.8:80",
                "--idempotency-key",
                "idem-1"
            ],
            ""
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        AnalysisRunIdempotencyLookupCliVerb::parse("stored-request"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "127.0.0.1:18081",
                "--idempotency-key",
                "idem-1"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
