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
fn binary_lists_retry_children_over_tcp_after_cancel_and_retry() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", "4"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback service");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let host = address.trim();
    let body = r#"{"contract_version":1,"idempotency_key":"loopback-retries-idem","tenant_workspace_id":"loopback-retries-tenant","snapshot_id":"loopback-retries-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#;
    let create = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retries-idem\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect create");
    stream.write_all(create.as_bytes()).expect("create");
    let mut created = String::new();
    stream.read_to_string(&mut created).expect("created");
    assert!(created.starts_with("HTTP/1.1 202 Accepted"));
    let json_start = created.find("{\"contract_version\"").expect("json");
    let accepted: serde_json::Value =
        serde_json::from_str(&created[json_start..]).expect("accepted json");
    let run_id = accepted["run_id"].as_str().expect("run_id");
    assert!(!created[json_start..].contains("rmse"));

    let cancel_body = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","idempotency_key":"loopback-retries-idem"}}"#
    );
    let cancel = format!(
        "POST /v1/analysis-runs/{run_id}/cancel HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retries-idem\r\ncontent-length: {}\r\n\r\n{cancel_body}",
        cancel_body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect cancel");
    stream.write_all(cancel.as_bytes()).expect("cancel");
    let mut cancelled = String::new();
    stream.read_to_string(&mut cancelled).expect("cancelled");
    assert!(cancelled.starts_with("HTTP/1.1 200 OK"));

    let retry_body = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","idempotency_key":"loopback-retries-retry-idem"}}"#
    );
    let retry = format!(
        "POST /v1/analysis-runs/{run_id}/retry HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-retries-retry-idem\r\ncontent-length: {}\r\n\r\n{retry_body}",
        retry_body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect retry");
    stream.write_all(retry.as_bytes()).expect("retry");
    let mut retried = String::new();
    stream.read_to_string(&mut retried).expect("retried");
    assert!(retried.starts_with("HTTP/1.1 202 Accepted"));
    let retry_json = retried.find("{\"contract_version\"").expect("retry json");
    let child_accepted: serde_json::Value =
        serde_json::from_str(&retried[retry_json..]).expect("child json");
    let child_id = child_accepted["run_id"].as_str().expect("child run_id");
    assert_ne!(child_id, run_id);

    let inspect = format!(
        "GET /v1/analysis-runs/{run_id}/retries HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    );
    let mut stream = TcpStream::connect(host).expect("connect retries");
    stream.write_all(inspect.as_bytes()).expect("inspect");
    let mut inspected = String::new();
    stream.read_to_string(&mut inspected).expect("inspected");
    assert!(inspected.starts_with("HTTP/1.1 200 OK"));
    assert!(inspected.contains(&format!("\"run_id\":\"{run_id}\"")));
    assert!(inspected.contains(&format!("\"run_id\":\"{child_id}\"")));
    assert!(inspected.contains("\"retries\":[") || inspected.contains("\"retries\": ["));
    assert!(!inspected.contains("rmse"));
    assert!(!inspected.contains("scientific_acceptance"));
    assert!(!inspected.contains("tenant_workspace_id"));
    assert!(!inspected.contains("snapshot_id"));
    assert!(child.wait().expect("wait").success());
}
