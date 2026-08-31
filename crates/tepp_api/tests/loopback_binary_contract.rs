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
fn binary_records_a_lineageweave_running_transition_over_tcp() {
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
    let body = r#"{"contract_version":1,"idempotency_key":"loopback-lifecycle-idem","tenant_workspace_id":"loopback-lifecycle-tenant","snapshot_id":"loopback-lifecycle-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#;
    let create = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-lifecycle-idem\r\ncontent-length: {}\r\n\r\n{body}",
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
    let running = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","run_state":"running","idempotency_key":"loopback-lifecycle-idem"}}"#
    );
    let post = format!(
        "POST /v1/analysis-runs/{run_id}/running HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-lifecycle-idem\r\ncontent-length: {}\r\n\r\n{running}",
        running.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect running");
    stream.write_all(post.as_bytes()).expect("running");
    let mut status = String::new();
    stream.read_to_string(&mut status).expect("running body");
    assert!(status.starts_with("HTTP/1.1 200 OK"));
    assert!(status.contains("\"run_state\":\"running\""));
    assert!(!status.contains("rmse"));
    assert!(!status.contains("scientific_acceptance"));
    assert!(child.wait().expect("wait").success());
}

#[test]
fn binary_records_running_status_from_tepp_lifecycle_cli() {
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
    let body = r#"{"contract_version":1,"idempotency_key":"loopback-lifecycle-cli-idem","tenant_workspace_id":"loopback-lifecycle-cli-tenant","snapshot_id":"loopback-lifecycle-cli-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"tepp-analysis-run-v1","output_profile":"calibrated_event_measurement"}"#;
    let create = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\nidempotency-key: loopback-lifecycle-cli-idem\r\ncontent-length: {}\r\n\r\n{body}",
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
    let output = Command::new(env!("CARGO_BIN_EXE_tepp-lifecycle"))
        .args([
            "running",
            "--host",
            host,
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            "lineageweave",
            "--run-id",
            run_id,
            "--idempotency-key",
            "loopback-lifecycle-cli-idem",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("tepp-lifecycle");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("\"run_state\":\"running\""));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("scientific_acceptance"));
    assert!(child.wait().expect("wait").success());
}
