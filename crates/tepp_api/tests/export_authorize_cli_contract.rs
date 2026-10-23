//! Contract tests for the purpose-bound export-authorize loopback CLI.

use tepp_api::{
    AnalyticalPurpose, ApiError, ExportAuthorizationRequest, ExportAuthorizeCliInvocation,
    ExportAuthorizeCliVerb, compose_export_authorize_cli_http,
};

fn allowed_body() -> String {
    serde_json::to_string(&ExportAuthorizationRequest {
        tenant_workspace_id: "tenant-a".into(),
        principal_id: "naruon-service".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-a".into(),
        includes_source_text: false,
    })
    .expect("json")
}

#[test]
fn export_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        ExportAuthorizeCliVerb::parse("authorize").expect("verb"),
        ExportAuthorizeCliVerb::Authorize
    );
    let invocation = ExportAuthorizeCliInvocation::from_args(
        [
            "authorize",
            "--host",
            "127.0.0.1:18082",
            "--idempotency-key",
            "export-idem-1",
        ],
        allowed_body(),
    )
    .expect("invocation");
    let http = compose_export_authorize_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/exports HTTP/1.1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/analysis-runs"));
}

#[test]
fn export_cli_refuses_non_loopback_unknown_verbs_and_metrics() {
    assert_eq!(
        ExportAuthorizeCliInvocation::from_args(
            [
                "authorize",
                "--host",
                "8.8.8.8:80",
                "--idempotency-key",
                "export-idem-1"
            ],
            allowed_body()
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        ExportAuthorizeCliVerb::parse("create"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportAuthorizeCliInvocation::from_args(
            [
                "authorize",
                "--host",
                "127.0.0.1:18082",
                "--idempotency-key",
                "export-idem-1"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
