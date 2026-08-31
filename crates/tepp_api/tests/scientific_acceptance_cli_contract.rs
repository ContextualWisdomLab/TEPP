//! Operator-visible loopback CLI contract for GAP-003A.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, NARUON_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
    receipt_json_carries_scientific_metrics,
};

fn request_json() -> String {
    format!(
        r#"{{"contract_version":{ANALYSIS_RUN_CONTRACT_VERSION},"idempotency_key":"cli-bin-idem-1","tenant_workspace_id":"cli-bin-tenant","snapshot_id":"cli-bin-snapshot","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"validation_cpu_f64_v1","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}"}}"#
    )
}

#[test]
fn binary_create_stays_metric_free_and_refuses_non_loopback() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tepp-loopback"))
        .args(["127.0.0.1:0", "1"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn loopback");
    let mut address = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut address)
        .expect("bound address");
    let host = address.trim();

    let mut create = Command::new(env!("CARGO_BIN_EXE_tepp-analysis-run"))
        .args(["create", "--host", host, "--consumer", NARUON_CONSUMER_CODE])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn create");
    create
        .stdin
        .take()
        .expect("stdin")
        .write_all(request_json().as_bytes())
        .expect("write request");
    let output = create.wait_with_output().expect("create wait");
    assert!(output.status.success());
    let body = String::from_utf8(output.stdout).expect("utf8");
    assert!(!receipt_json_carries_scientific_metrics(&body));
    assert!(body.contains("\"accepted\""));
    assert!(!body.contains("tepp.scientific_acceptance.v1"));

    let refused = Command::new(env!("CARGO_BIN_EXE_tepp-analysis-run"))
        .args([
            "create",
            "--host",
            "8.8.8.8:80",
            "--consumer",
            NARUON_CONSUMER_CODE,
        ])
        .stdin(Stdio::piped())
        .output()
        .expect("non-loopback");
    assert!(!refused.status.success());
    assert!(child.wait().expect("wait loopback").success());
}
