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
fn binary_inspects_an_accepted_analysis_run_stored_request_over_tcp() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", "2"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback service");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let host = address.trim();
    let body = r#"{"contract_version":1,"idempotency_key":"loopback-stored-idem","tenant_workspace_id":"loopback-stored-tenant","snapshot_id":"loopback-stored-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#;
    let create = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-stored-idem\r\ncontent-length: {}\r\n\r\n{body}",
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
    let inspect = format!(
        "GET /v1/analysis-runs/{run_id}/request HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n"
    );
    let mut stream = TcpStream::connect(host).expect("connect stored-request");
    stream.write_all(inspect.as_bytes()).expect("inspect");
    let mut inspected = String::new();
    stream.read_to_string(&mut inspected).expect("inspected");
    assert!(inspected.starts_with("HTTP/1.1 200 OK"));
    assert!(inspected.contains("\"snapshot_id\":\"loopback-stored-snapshot\""));
    assert!(inspected.contains("\"output_profile\":\"calibrated_event_measurement\""));
    assert!(!inspected.contains("rmse"));
    assert!(!inspected.contains("scientific_acceptance"));
    assert!(!inspected.contains("tenant_workspace_id"));
    assert!(child.wait().expect("wait").success());
}
