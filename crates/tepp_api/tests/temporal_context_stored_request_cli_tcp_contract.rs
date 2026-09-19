//! Production-path coverage for the temporal-context stored-request loopback CLI.
//!
//! This test crosses a real loopback TCP socket so the public execution path is
//! not proven only by the in-process dispatcher.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use tepp_api::{
    LINEAGEWEAVE_CONSUMER_CODE, TemporalContextRequest, TemporalContextStoredRequestCliInvocation,
    execute_temporal_context_stored_request_cli, render_temporal_context_stored_request_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

#[test]
fn execute_traverses_loopback_tcp_and_parses_the_stored_request_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let address = listener.local_addr().expect("listener address");

    let server = thread::spawn(move || {
        let (mut stream, peer) = listener.accept().expect("accept loopback client");
        assert!(peer.ip().is_loopback());

        let mut request_bytes = [0_u8; 4096];
        let received = stream.read(&mut request_bytes).expect("read request");
        let request = std::str::from_utf8(&request_bytes[..received]).expect("utf8 request");
        assert!(request.starts_with("GET /v1/temporal-context/idem-tcp/request HTTP/1.1\r\n"));
        assert!(request.contains("tepp-consumer: lineageweave\r\n"));
        assert!(request.contains("content-length: 0\r\n\r\n"));
        assert!(!request.to_ascii_lowercase().contains("authorization"));

        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            TEMPORAL_BODY.len(),
            TEMPORAL_BODY
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        stream.flush().expect("flush response");
    });

    let host = address.to_string();
    let invocation = TemporalContextStoredRequestCliInvocation::from_args(
        [
            "get",
            "--host",
            host.as_str(),
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
            "--idempotency-key",
            "idem-tcp",
        ],
        "",
    )
    .expect("valid invocation");

    let response = execute_temporal_context_stored_request_cli(&invocation)
        .expect("execute over loopback TCP");
    server.join().expect("join loopback server");

    assert_eq!(response.status_code, 200);
    assert_eq!(response.reason_phrase, "OK");
    let stdout = render_temporal_context_stored_request_cli_stdout(&invocation, &response)
        .expect("render stored request");
    assert_eq!(
        TemporalContextRequest::from_json(&stdout).expect("stored request"),
        TemporalContextRequest::from_json(TEMPORAL_BODY).expect("expected request")
    );
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
}
