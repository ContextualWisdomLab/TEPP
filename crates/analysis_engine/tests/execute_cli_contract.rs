//! GAP-003A published execute CLI against spawned tepp-loopback TCP.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use analysis_engine::{
    ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION, ScientificAcceptanceExecuteCliInvocation,
    ScientificAcceptanceExecuteCliVerb, VALIDATION_CPU_F64_MODEL,
    loopback_http1_from_naruon_exchange,
};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunRequest, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
    NARUON_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    lineageweave_analysis_run_exchange, naruon_analysis_run_exchange,
};

const HTTPS_ORIGIN: &str = "https://tepp.example.com";

fn spawn_loopback(request_limit: &str) -> (std::process::Child, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", request_limit])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    (child, address)
}

fn request(idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "tenant-workspace-execute-cli".into(),
        snapshot_id: "snapshot-execute-cli".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: VALIDATION_CPU_F64_MODEL.into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into(),
    }
}

fn execute_json(run_id: &str, idempotency_key: &str) -> String {
    serde_json::json!({
        "contract_version": ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
        "run_id": run_id,
        "idempotency_key": idempotency_key,
        "seed": 42,
        "se_gate_k": 3.0,
        "completed_at": "2026-08-31T13:00:00Z",
        "study_label": "loopback-cli-recovery",
        "authored_by_llm": false,
        "corpus": {
            "snapshot_id": "snapshot-execute-cli",
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

fn accept_run(address: &str, idempotency_key: &str, consumer: &str) -> String {
    let request = request(idempotency_key);
    let create = if consumer == LINEAGEWEAVE_CONSUMER_CODE {
        lineageweave_analysis_run_exchange(HTTPS_ORIGIN, &request).expect("lineage create")
    } else {
        naruon_analysis_run_exchange(HTTPS_ORIGIN, &request).expect("naruon create")
    };
    let http = loopback_http1_from_naruon_exchange(&create, address.trim()).expect("create http");
    let mut stream = std::net::TcpStream::connect(address.trim()).expect("connect");
    stream.write_all(http.as_bytes()).expect("write create");
    let mut response = String::new();
    std::io::Read::read_to_string(&mut stream, &mut response).expect("read create");
    assert!(response.starts_with("HTTP/1.1 202 Accepted"), "{response}");
    let body = response.split("\r\n\r\n").nth(1).expect("body");
    serde_json::from_str::<serde_json::Value>(body).expect("json")["run_id"]
        .as_str()
        .expect("run_id")
        .to_owned()
}

fn run_execute_cli(host: &str, consumer: &str, body: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-execute"))
        .args([
            "execute",
            "--host",
            host,
            "--origin",
            HTTPS_ORIGIN,
            "--consumer",
            consumer,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn execute cli");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(body.as_bytes())
        .expect("write execute");
    child.wait_with_output().expect("wait execute")
}

#[test]
fn execute_cli_verb_is_execute_only() {
    assert_eq!(
        ScientificAcceptanceExecuteCliVerb::parse("execute").expect("execute"),
        ScientificAcceptanceExecuteCliVerb::Execute
    );
    assert_eq!(
        ScientificAcceptanceExecuteCliVerb::parse("create"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ScientificAcceptanceExecuteCliInvocation::from_args(
            ["execute", "--host", "8.8.8.8:80", "--origin", HTTPS_ORIGIN],
            execute_json("tepp-run-1", "idem-1")
        ),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn naruon_and_lineageweave_execute_cli_over_spawned_loopback_tcp() {
    let (mut child, address) = spawn_loopback("4");
    let host = address.trim();

    let naruon_run_id = accept_run(&address, "idem-naruon-execute-cli", NARUON_CONSUMER_CODE);
    let naruon = run_execute_cli(
        host,
        NARUON_CONSUMER_CODE,
        &execute_json(&naruon_run_id, "idem-naruon-execute-cli"),
    );
    assert!(naruon.status.success(), "{naruon:?}");
    let naruon_body = String::from_utf8(naruon.stdout).expect("utf8");
    assert!(naruon_body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    assert!(naruon_body.contains("scientific_acceptance"));

    let lineage_run_id = accept_run(
        &address,
        "idem-lineageweave-execute-cli",
        LINEAGEWEAVE_CONSUMER_CODE,
    );
    let lineage = run_execute_cli(
        host,
        LINEAGEWEAVE_CONSUMER_CODE,
        &execute_json(&lineage_run_id, "idem-lineageweave-execute-cli"),
    );
    assert!(lineage.status.success(), "{lineage:?}");
    let lineage_body = String::from_utf8(lineage.stdout).expect("utf8");
    assert!(lineage_body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));

    let refused = Command::new(env!("CARGO_BIN_EXE_tepp-execute"))
        .args([
            "execute",
            "--host",
            "8.8.8.8:80",
            "--origin",
            HTTPS_ORIGIN,
            "--consumer",
            NARUON_CONSUMER_CODE,
        ])
        .stdin(Stdio::piped())
        .output()
        .expect("non-loopback");
    assert!(!refused.status.success());
    assert!(child.wait().expect("wait").success());
}
