//! Regression tests for opaque export idempotency-key lookup compatibility.

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ExportAuthorizationRequest,
    ExportIdempotencyLookupCliInvocation, NARUON_CONSUMER_CODE,
    dispatch_export_idempotency_lookup_cli, naruon_export_idempotency_lookup_exchange,
};

const ORIGIN: &str = "https://tepp.example.test";

fn request() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "lookup-key-compat-tenant".into(),
        principal_id: "lookup-key-compat-principal".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "lookup-key-compat-artifact".into(),
        includes_source_text: false,
    }
}

fn post(service: &mut AnalysisRunLiveService, key: &str) {
    let body = serde_json::to_string(&request()).expect("request json");
    let raw = format!(
        "POST /v1/exports HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    );
    let response = service.handle_http_request(&raw);
    assert_eq!(response.status_code, 200, "{}", response.body);
}

fn invocation(key: &str) -> ExportIdempotencyLookupCliInvocation {
    ExportIdempotencyLookupCliInvocation::from_args(
        [
            "lookup",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            NARUON_CONSUMER_CODE,
            "--idempotency-key",
            key,
        ],
        "",
    )
    .expect("opaque accepted export key must remain lookup-addressable")
}

#[test]
fn slash_key_remains_addressable_through_encoded_lookup_and_cli() {
    let exchange = naruon_export_idempotency_lookup_exchange(ORIGIN, "scope/key")
        .expect("encoded slash exchange");
    assert!(exchange.target_url.ends_with("/by-idempotency/scope%2Fkey"));

    let mut service = AnalysisRunLiveService::new();
    post(&mut service, "scope/key");
    let response = dispatch_export_idempotency_lookup_cli(&mut service, &invocation("scope/key"))
        .expect("dispatch");
    assert_eq!(response.status_code, 200, "{}", response.body);
    assert!(response.body.contains("\"idempotency_key\":\"scope/key\""));
}

#[test]
fn route_prefix_key_remains_addressable_as_nested_opaque_value() {
    let exchange = naruon_export_idempotency_lookup_exchange(ORIGIN, "by-idempotency")
        .expect("reserved-looking value is data after the route prefix");
    assert!(
        exchange
            .target_url
            .ends_with("/by-idempotency/by-idempotency")
    );

    let mut service = AnalysisRunLiveService::new();
    post(&mut service, "by-idempotency");
    let response = dispatch_export_idempotency_lookup_cli(
        &mut service,
        &invocation("by-idempotency"),
    )
    .expect("dispatch");
    assert_eq!(response.status_code, 200, "{}", response.body);
    assert!(
        response
            .body
            .contains("\"idempotency_key\":\"by-idempotency\"")
    );
}
