//! Contract tests for the `LineageWeave` temporal-context loopback CLI.

use tepp_api::{
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, TemporalContextCliInvocation, TemporalContextCliVerb,
    TemporalContextEvent, TemporalContextRequest, compose_temporal_context_cli_http,
};

fn query_body() -> String {
    TemporalContextRequest {
        contract_version: 1,
        consumer_code: LINEAGEWEAVE_CONSUMER_CODE.into(),
        knowledge_cutoff: "2026-08-20T00:00:00Z".into(),
        subject_post_id: None,
        events: vec![TemporalContextEvent {
            event_id: "event-1".into(),
            source_post_id: "post-1".into(),
            event_type_code: "order_awarded".into(),
            event_label: "Order awarded".into(),
            event_time: "2026-08-01T09:00:00Z".into(),
            available_time: "2026-08-01T10:00:00Z".into(),
            project_reference: None,
            actor_references: vec!["actor-1".into()],
        }],
    }
    .to_json()
    .expect("json")
}

#[test]
fn temporal_context_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        TemporalContextCliVerb::parse("query").expect("verb"),
        TemporalContextCliVerb::Query
    );
    let invocation = TemporalContextCliInvocation::from_args(
        ["query", "--host", "127.0.0.1:18081"],
        query_body(),
    )
    .expect("invocation");
    let http = compose_temporal_context_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/temporal-context HTTP/1.1"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/analysis-runs"));
}

#[test]
fn temporal_context_cli_refuses_non_loopback_unknown_verbs_and_metrics() {
    assert_eq!(
        TemporalContextCliInvocation::from_args(["query", "--host", "8.8.8.8:80"], query_body()),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        TemporalContextCliVerb::parse("cancel"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        TemporalContextCliInvocation::from_args(
            ["query", "--host", "127.0.0.1:18081"],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
