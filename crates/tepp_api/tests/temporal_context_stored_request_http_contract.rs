//! Contract tests for loopback `GET /v1/temporal-context/{key}/request`.

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    TEMPORAL_CONTEXT_PATH, TemporalContextRequest,
    lineageweave_temporal_context_stored_request_exchange,
    refuse_metrics_on_temporal_context_stored_request_payload,
    temporal_context_stored_request_path_id,
};

const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

fn post_http(idempotency_key: &str) -> String {
    format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{TEMPORAL_BODY}",
        TEMPORAL_BODY.len()
    )
}

#[test]
fn stored_request_get_returns_create_request_and_fails_closed() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&post_http("idem-a"))
            .status_code,
        200
    );
    let got = service.handle_http_request(&format!(
        "GET {TEMPORAL_CONTEXT_PATH}/idem-a/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    ));
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stored = TemporalContextRequest::from_json(&got.body).expect("stored");
    let original = TemporalContextRequest::from_json(TEMPORAL_BODY).expect("original");
    assert_eq!(stored, original);
    assert!(!got.body.contains("rmse"));
    assert!(!got.body.contains("tepp.scientific_acceptance.v1"));
    assert!(!got.body.contains("causal_score"));
    assert_eq!(
        service
            .handle_http_request(&format!(
                "GET {TEMPORAL_CONTEXT_PATH}/idem-a/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&format!(
                "GET {TEMPORAL_CONTEXT_PATH}/idem-a/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&format!(
                "GET {TEMPORAL_CONTEXT_PATH}/missing/request HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
            ))
            .status_code,
        400
    );
    let exchange = lineageweave_temporal_context_stored_request_exchange(
        "https://tepp.example.test",
        "idem-a",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/temporal-context/idem-a/request")
    );
    assert!(exchange.body.is_empty());
    assert_eq!(
        temporal_context_stored_request_path_id("/v1/temporal-context/idem-a/request").expect("id"),
        "idem-a"
    );
    assert_eq!(
        temporal_context_stored_request_path_id("/v1/temporal-context/idem-a"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_temporal_context_stored_request_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_temporal_context_stored_request_payload(""),
        Ok(())
    );
}
