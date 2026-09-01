//! Regression tests for export idempotency-lookup review findings.

use tepp_api::{
    AnalysisRunLiveService, ApiError, ExportIdempotencyLookup, naruon_export_retrieval_exchange,
    refuse_metrics_on_export_idempotency_lookup_payload,
};

const EXPORT_REQUEST_JSON: &str = r#"{"tenant_workspace_id":"tenant-a","principal_id":"principal-a","purpose":"modular_service_consumer","artifact_id":"artifact-a","includes_source_text":false}"#;

fn export_post_http(idempotency_key: &str) -> String {
    format!(
        "POST /v1/exports HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{EXPORT_REQUEST_JSON}",
        EXPORT_REQUEST_JSON.len()
    )
}

fn export_lookup_http(encoded_idempotency_key: &str) -> String {
    format!(
        "GET /v1/exports/by-idempotency/{encoded_idempotency_key} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    )
}

#[test]
fn slash_containing_idempotency_key_round_trips_through_post_then_lookup() {
    let mut service = AnalysisRunLiveService::new();
    let posted = service.handle_http_request(&export_post_http("scope/key"));
    assert_eq!(posted.status_code, 200, "POST must preserve an already-valid opaque key");

    let looked_up = service.handle_http_request(&export_lookup_http("scope%2Fkey"));
    assert_eq!(
        looked_up.status_code, 200,
        "one percent-encoded path segment must recover the opaque slash-containing key"
    );
    let lookup = ExportIdempotencyLookup::from_json(&looked_up.body).expect("lookup payload");
    assert_eq!(lookup.idempotency_key, "scope/key");
}

#[test]
fn reserved_lookup_prefix_cannot_build_an_unroutable_retrieval_exchange() {
    assert_eq!(
        naruon_export_retrieval_exchange("https://tepp.example.test", "by-idempotency"),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn lookup_metric_refusal_walks_nested_objects_and_arrays() {
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(
            r#"{"safe":{"nested":{"rmse":1.0}}}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(
            r#"{"safe":[{"deeper":{"scientific_acceptance":{}}}]}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_export_idempotency_lookup_payload(r#"{"safe":[{"value":1}]}"#),
        Ok(())
    );
}
