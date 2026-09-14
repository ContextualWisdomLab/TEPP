//! The packaged `tepp-export-get` binary serves one loopback export-retrieval GET.

use std::process::{Command, Stdio};

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ExportAuthorizationRequest, ExportRetrieval,
    NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH,
};

const ORIGIN: &str = "https://tepp.example.test";

fn authorize_body() -> String {
    serde_json::to_string(&ExportAuthorizationRequest {
        tenant_workspace_id: "cli-export-tenant".into(),
        principal_id: "principal-analyst-cli".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-bin-1".into(),
        includes_source_text: false,
    })
    .expect("json")
}

fn export_post_http(body: &str, idempotency_key: &str) -> String {
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn mint_export(service: &mut AnalysisRunLiveService, idempotency_key: &str) -> ExportRetrieval {
    let posted = service.handle_http_request(&export_post_http(&authorize_body(), idempotency_key));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    ExportRetrieval::from_json(&posted.body).expect("minted")
}

#[test]
fn binary_success_run_prints_metric_free_export_identity_and_exits_zero() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let minted = mint_export(&mut service, "cli-export-bin-1");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });

    let output = Command::new(env!("CARGO_BIN_EXE_tepp-export-get"))
        .args([
            "get",
            "--host",
            addr.as_str(),
            "--origin",
            ORIGIN,
            "--export-id",
            minted.export_id.as_str(),
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run binary");

    handle.join().expect("join");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let retrieved = ExportRetrieval::from_json(stdout.trim()).expect("parsed identity");
    assert_eq!(retrieved, minted);
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains("tepp.scientific_acceptance.v1"));
}

#[test]
fn binary_failure_run_on_invalid_args_exits_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_tepp-export-get"))
        .args(["get"])
        .stdin(Stdio::null())
        .output()
        .expect("run binary");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn binary_failure_run_on_missing_export_exits_failure() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });

    let output = Command::new(env!("CARGO_BIN_EXE_tepp-export-get"))
        .args([
            "get",
            "--host",
            addr.as_str(),
            "--origin",
            ORIGIN,
            "--export-id",
            "missing-export",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run binary");

    handle.join().expect("join");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("invalid_wire_payload"));
}
