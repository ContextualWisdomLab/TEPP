//! The packaged loopback binary serves temporal-context and execute.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use analysis_engine::{
    ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION, SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
    SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION, VALIDATION_CPU_F64_MODEL,
};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
    SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
};

fn spawn_loopback(request_limit: &str) -> (Child, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", request_limit])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback service");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    (child, address)
}

fn exchange(address: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(address.trim()).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout");
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("write timeout");
    stream.write_all(request.as_bytes()).expect("request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    response
}

fn http_post(
    address: &str,
    path: &str,
    body: &str,
    consumer: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        address.trim(),
        body.len()
    )
}

fn http_get(address: &str, path: &str, consumer: &str, idempotency_key: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n",
        address.trim()
    )
}

fn execute_body(run_id: &str) -> String {
    serde_json::json!({
        "contract_version": ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
        "run_id": run_id,
        "idempotency_key": "idem-loopback-binary",
        "seed": 42,
        "se_gate_k": 3.0,
        "completed_at": "2026-08-31T11:00:00Z",
        "study_label": "loopback-binary-recovery",
        "authored_by_llm": false,
        "corpus": {
            "snapshot_id": "snapshot-binary",
            "evidence_units": [
                {
                    "evidence_id": "evidence-1",
                    "event_time": "2026-07-01T00:00:00Z",
                    "available_time": "2026-07-10T00:00:00Z",
                    "membership_count": 1
                },
                {
                    "evidence_id": "evidence-2",
                    "event_time": "2026-07-01T00:00:00Z",
                    "available_time": "2026-07-20T00:00:00Z",
                    "membership_count": 1
                },
                {
                    "evidence_id": "future",
                    "event_time": "2026-07-01T00:00:00Z",
                    "available_time": "2026-08-02T00:00:00Z",
                    "membership_count": 1
                }
            ]
        },
        "truth": [0.70, 0.55, 0.40, -0.20, 0.85],
        "recovered": [0.70, 0.55, 0.40, -0.20, 0.85],
        "interval_lower": [0.50, 0.35, 0.20, -0.40, 0.65],
        "interval_upper": [0.90, 0.75, 0.60, 0.00, 1.00],
        "truth_times": [1.0, 2.0, 3.0, 4.0, 5.0],
        "recovered_times": [1.1, 1.9, 3.2, 3.8, 5.1]
    })
    .to_string()
}

fn response_body(response: &str) -> &str {
    response.split("\r\n\r\n").nth(1).expect("http body")
}

#[test]
fn binary_serves_one_bounded_temporal_context_request() {
    let (mut child, address) = spawn_loopback("1");
    let body = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":"post-1","events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"health_probe","event_label":"Health probe","event_time":"2026-08-20T00:00:00Z","available_time":"2026-08-20T00:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;
    let request = format!(
        "POST /v1/temporal-context HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: lineageweave\r\ntepp-contract-version: 1\r\ncontent-length: {}\r\n\r\n{body}",
        address.trim(),
        body.len()
    );
    let response = exchange(&address, &request);
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains("association_not_causal"));
    assert!(child.wait().expect("wait").success());
}

#[test]
fn binary_executes_scientific_acceptance_without_caller_artifact() {
    let (mut child, address) = spawn_loopback("3");
    let create = serde_json::json!({
        "contract_version": ANALYSIS_RUN_CONTRACT_VERSION,
        "idempotency_key": "idem-loopback-binary",
        "tenant_workspace_id": "tenant-workspace-binary",
        "snapshot_id": "snapshot-binary",
        "knowledge_cutoff": "2026-08-01T00:00:00Z",
        "model_contract_version": VALIDATION_CPU_F64_MODEL,
        "output_profile": SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
    })
    .to_string();
    let accepted = exchange(
        &address,
        &http_post(
            &address,
            NARUON_ANALYSIS_RUN_PATH,
            &create,
            NARUON_CONSUMER_CODE,
            "idem-loopback-binary",
        ),
    );
    assert!(accepted.starts_with("HTTP/1.1 202 Accepted"), "{accepted}");
    assert!(!accepted.contains("rmse"));
    assert!(!accepted.contains("scientific_acceptance"));
    let run_id = serde_json::from_str::<serde_json::Value>(response_body(&accepted))
        .expect("accepted json")["run_id"]
        .as_str()
        .expect("run_id")
        .to_owned();

    let body = execute_body(&run_id);
    assert!(!body.contains("scientific_acceptance_json"));
    assert!(!body.contains("rmse"));
    let execute = exchange(
        &address,
        &http_post(
            &address,
            &format!("{NARUON_ANALYSIS_RUN_PATH}/{run_id}/execute"),
            &body,
            NARUON_CONSUMER_CODE,
            "idem-loopback-binary",
        ),
    );
    assert!(execute.starts_with("HTTP/1.1 200 OK"), "{execute}");
    assert!(execute.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    assert!(execute.contains("scientific_acceptance"));
    assert!(execute.contains("\"succeeded\""));

    let get = exchange(
        &address,
        &http_get(
            &address,
            &format!("{NARUON_ANALYSIS_RUN_PATH}/{run_id}"),
            NARUON_CONSUMER_CODE,
            "idem-loopback-binary",
        ),
    );
    assert!(get.starts_with("HTTP/1.1 200 OK"), "{get}");
    assert!(get.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert!(get.contains(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE));
    assert!(get.contains("scientific_acceptance"));
    assert!(get.contains("rmse"));
    assert!(child.wait().expect("wait").success());
}
