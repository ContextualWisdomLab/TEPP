//! Contract tests for quarantined `tepp-export-lookup-request get`.

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ApiError, ExportAuthorizationRequest,
    ExportIdempotencyLookupStoredRequestCliInvocation, ExportIdempotencyLookupStoredRequestCliVerb,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH,
    compose_export_idempotency_lookup_stored_request_cli_http,
    dispatch_export_idempotency_lookup_stored_request_cli,
};

const ORIGIN: &str = "https://tepp.example.test";

fn sample_request() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "export-lookup-sr-cli-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-lookup-sr-cli-1".into(),
        includes_source_text: false,
    }
}

fn export_post(request: &ExportAuthorizationRequest, idempotency_key: &str) -> String {
    let body = serde_json::to_string(request).expect("request json");
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn get_args<'a>(host: &'a str, key: &'a str, consumer: &'a str) -> [&'a str; 9] {
    [
        "get",
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--idempotency-key",
        key,
    ]
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliVerb::parse("get").expect("get"),
        ExportIdempotencyLookupStoredRequestCliVerb::Get
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliVerb::Get.as_str(),
        "get"
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliVerb::parse("lookup"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
            get_args("8.8.8.8:80", "idem-1", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
            get_args("localhost:18081", "idem-1", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-1", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "by-idempotency", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem/slash", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_stays_quarantined_and_never_discloses_stored_create() {
    let invocation = ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        compose_export_idempotency_lookup_stored_request_cli_http(&invocation),
        Err(ApiError::AuthorizationDenied)
    );
    let mut service = AnalysisRunLiveService::new();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-lookup-sr-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let scoped = ExportIdempotencyLookupStoredRequestCliInvocation::from_args(
        get_args(
            "127.0.0.1:18081",
            "export-lookup-sr-1",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("scoped");
    assert_eq!(
        dispatch_export_idempotency_lookup_stored_request_cli(&mut service, &scoped),
        Err(ApiError::AuthorizationDenied)
    );
}
