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
