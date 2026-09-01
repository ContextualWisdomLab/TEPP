//! Contract tests for loopback `GET /v1/temporal-context`.

use std::io::{Read, Write};

use tepp_api::{
    AnalysisRunLiveService, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    TEMPORAL_CONTEXT_PATH, TemporalContextCollection,
};

const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

fn post_http(idempotency_key: &str) -> String {
    format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{TEMPORAL_BODY}",
        TEMPORAL_BODY.len()
    )
}

#[test]
fn collection_get_pages_metric_free_identities() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(service.handle_http_request(&post_http("idem-b")).status_code, 200);
    assert_eq!(service.handle_http_request(&post_http("idem-a")).status_code, 200);
    let listed = service.handle_http_request(
        &format!(
            "GET {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ntepp-page-limit: 1\r\ncontent-length: 0\r\n\r\n"
        ),
    );
    assert_eq!(listed.status_code, 200, "{}", listed.body);
    let page = TemporalContextCollection::from_json(&listed.body).expect("page");
    assert_eq!(page.contexts.len(), 1);
    assert_eq!(page.contexts[0].idempotency_key, "idem-a");
    assert_eq!(page.next_cursor.as_deref(), Some("idem-a"));
    assert!(!listed.body.contains("event_label"));
    assert!(!listed.body.contains("rmse"));
}

#[test]
fn collection_get_refuses_naruon_and_serves_over_tcp() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    assert_eq!(service.handle_http_request(&post_http("idem-tcp")).status_code, 200);
    assert_eq!(
        service
            .handle_http_request(
                &format!(
                    "GET {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
                )
            )
            .status_code,
        400
    );
    let addr = service.local_addr().expect("addr");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let request = format!(
        "GET {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: {addr}\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    );
    let mut stream = std::net::TcpStream::connect(addr).expect("connect");
    stream.write_all(request.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("read");
    let text = String::from_utf8(bytes).expect("utf8");
    assert!(text.contains("HTTP/1.1 200"), "{text}");
    assert!(text.contains("idem-tcp"), "{text}");
    handle.join().expect("join");
}
