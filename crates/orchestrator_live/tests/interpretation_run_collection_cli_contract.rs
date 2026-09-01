//! Contract tests for the contextual-orchestrator interpretation-run collection CLI.

use orchestrator_live::{
    compose_interpretation_run_collection_cli_http, InterpretationRunCollectionCliInvocation,
    InterpretationRunCollectionCliVerb, OrchestratorLiveError,
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
};

#[test]
fn interpretation_run_collection_cli_is_metric_free_get_without_credentials() {
    assert_eq!(
        InterpretationRunCollectionCliVerb::parse("list").expect("verb"),
        InterpretationRunCollectionCliVerb::List
    );
    let invocation = InterpretationRunCollectionCliInvocation::from_args(
        [
            "list",
            "--host",
            "127.0.0.1:18082",
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
        ],
        "",
    )
    .expect("invocation");
    let http = compose_interpretation_run_collection_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/interpretation-runs HTTP/1.1"));
    assert!(http.contains("tepp-consumer: contextual-orchestrator"));
    assert!(http.contains("content-length: 0"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("idempotency-key"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/analysis-runs"));
    assert!(!http.contains("/v1/exports"));
    assert!(!http.contains("/v1/project-histories"));
}

#[test]
fn interpretation_run_collection_cli_refuses_non_loopback_unknown_verbs_and_foreign_consumers() {
    assert_eq!(
        InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "8.8.8.8:80",
                "--origin",
                "https://tepp.example.test"
            ],
            ""
        ),
        Err(OrchestratorLiveError::AuthorizationDenied)
    );
    assert_eq!(
        InterpretationRunCollectionCliVerb::parse("create"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                "https://tepp.example.test",
                "--consumer",
                "naruon"
            ],
            ""
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        InterpretationRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18082",
                "--origin",
                "https://tepp.example.test"
            ],
            "{}"
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}
