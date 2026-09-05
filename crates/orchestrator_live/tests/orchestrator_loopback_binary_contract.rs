//! The packaged orchestrator loopback binary serves interpretation-run POST.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

#[test]
fn binary_serves_one_bounded_interpretation_run_request() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-orchestrator-loopback"))
        .args(["127.0.0.1:0", "1"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn orchestrator loopback");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let body = r#"{"contract_version":1,"idempotency_key":"orch-bin-idem-1","tenant_workspace_id":"orch-tenant-demo","snapshot_id":"tepp-snapshot-demo-001","knowledge_cutoff":"2026-08-01T00:00:00Z","orchestration_mode":"direct","compute_budget_tokens":2048,"evidence_span_ids":["span-001"],"scientific_authority":false}"#;
    let request = format!(
        "POST /v1/interpretation-runs HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: orch-bin-idem-1\r\ncontent-length: {}\r\n\r\n{body}",
        address.trim(),
        body.len()
    );
    let mut stream = TcpStream::connect(address.trim()).expect("connect");
    stream.write_all(request.as_bytes()).expect("request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    assert!(response.starts_with("HTTP/1.1 202 Accepted"), "{response}");
    assert!(response.contains("hypothetical"));
    assert!(response.contains("\"scientific_authority\":false"));
    assert!(!response.contains("tepp.scientific_acceptance.v1"));
    assert!(child.wait().expect("wait").success());
}
