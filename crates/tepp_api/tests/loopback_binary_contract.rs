//! The packaged loopback binary serves the published temporal-context wire.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

#[test]
fn binary_serves_one_bounded_temporal_context_request() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", "1"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback service");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let body = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":"post-1","events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"health_probe","event_label":"Health probe","event_time":"2026-08-20T00:00:00Z","available_time":"2026-08-20T00:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;
    let request = format!(
        "POST /v1/temporal-context HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: {}\r\n\r\n{body}",
        address.trim(),
        body.len()
    );
    let mut stream = TcpStream::connect(address.trim()).expect("connect");
    stream.write_all(request.as_bytes()).expect("request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("association_not_causal"));
    assert!(child.wait().expect("wait").success());
}

#[test]
fn binary_retries_a_cancelled_lineageweave_analysis_run_over_tcp() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", "3"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback service");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let host = address.trim();
    let body = r#"{"contract_version":1,"idempotency_key":"loopback-retry-parent","tenant_workspace_id":"loopback-retry-tenant","snapshot_id":"loopback-retry-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#;
    let create = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retry-parent\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect create");
    stream.write_all(create.as_bytes()).expect("create");
    let mut accepted = String::new();
    stream.read_to_string(&mut accepted).expect("accepted");
    assert!(accepted.starts_with("HTTP/1.1 202 Accepted"));
    let run_id = accepted
        .split("\"run_id\":\"")
        .nth(1)
        .expect("run_id")
        .split('"')
        .next()
        .expect("id");

    let cancel = format!(
        "POST /v1/analysis-runs/{run_id}/cancel HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retry-parent\r\ncontent-length: 0\r\n\r\n"
    );
    let mut stream = TcpStream::connect(host).expect("connect cancel");
    stream.write_all(cancel.as_bytes()).expect("cancel");
    let mut cancelled = String::new();
    stream.read_to_string(&mut cancelled).expect("cancelled");
    assert!(cancelled.starts_with("HTTP/1.1 200 OK"));
    assert!(cancelled.contains("\"run_state\":\"cancelled\""));
    assert!(!cancelled.contains("rmse"));

    let retry_body = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","idempotency_key":"loopback-retry-child"}}"#
    );
    let retry = format!(
        "POST /v1/analysis-runs/{run_id}/retry HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retry-child\r\ncontent-length: {}\r\n\r\n{retry_body}",
        retry_body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect retry");
    stream.write_all(retry.as_bytes()).expect("retry");
    let mut retried = String::new();
    stream.read_to_string(&mut retried).expect("retried");
    assert!(retried.starts_with("HTTP/1.1 202 Accepted"));
    assert!(retried.contains("\"run_state\":\"accepted\""));
    assert!(retried.contains("loopback-retry-child"));
    assert!(!retried.contains("rmse"));
    assert!(!retried.contains("scientific_acceptance"));
    assert!(child.wait().expect("wait").success());
}
