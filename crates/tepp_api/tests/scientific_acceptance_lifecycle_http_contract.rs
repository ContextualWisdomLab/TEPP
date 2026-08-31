//! Operator-visible loopback running/terminal POST contract for GAP-003A.

use sha2::{Digest, Sha256};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted,
    AnalysisRunLifecycleTransition, AnalysisRunLiveService, AnalysisRunRequest,
    AnalysisRunTerminalResult, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
    SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    naruon_analysis_run_running_exchange, naruon_analysis_run_terminal_exchange,
    receipt_json_carries_scientific_metrics, refuse_metrics_on_receipt,
};

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "lifecycle-contract-idem-1".into(),
        tenant_workspace_id: "lifecycle-contract-tenant".into(),
        snapshot_id: "lifecycle-contract-snapshot".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "validation_cpu_f64_v1".into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into(),
    }
}

fn post_create(run: &AnalysisRunRequest) -> String {
    let body = run.to_json().expect("body");
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    )
}

fn post_lifecycle(path: &str, body: &str, idempotency_key: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn get_status(run_id: &str, idempotency_key: &str) -> String {
    format!(
        "GET {NARUON_ANALYSIS_RUN_PATH}/{run_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[test]
fn production_running_and_terminal_posts_then_get_return_scientific_acceptance() {
    let run = request();
    let mut service = AnalysisRunLiveService::new();
    let accepted = service.handle_http_request(&post_create(&run));
    assert_eq!(accepted.status_code, 202);
    assert!(!receipt_json_carries_scientific_metrics(&accepted.body));
    let accepted_dto = AnalysisRunAccepted::from_json(&accepted.body).expect("accepted");

    let running = AnalysisRunLifecycleTransition::running(
        accepted_dto.run_id.clone(),
        run.idempotency_key.clone(),
    )
    .expect("running");
    let running_exchange =
        naruon_analysis_run_running_exchange("https://tepp.example.com", &running)
            .expect("running exchange");
    assert_eq!(running_exchange.method, "POST");
    assert!(running_exchange.target_url.ends_with("/running"));
    let running_response = service.handle_http_request(&post_lifecycle(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}/running", accepted_dto.run_id),
        &running.to_json().expect("running json"),
        run.idempotency_key.as_str(),
    ));
    assert_eq!(running_response.status_code, 200);
    assert!(running_response.body.contains("\"running\""));
    assert_eq!(refuse_metrics_on_receipt(&running_response.body), Ok(()));
    let get_running = service.handle_http_request(&get_status(
        &accepted_dto.run_id,
        run.idempotency_key.as_str(),
    ));
    assert_eq!(get_running.body, running_response.body);

    let artifact = format!(
        r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA}","output_profile":"{SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE}","binding_sha256":"{}","run_id":"{}"}}"#,
        "ab".repeat(32),
        accepted_dto.run_id
    );
    let digest = sha256_hex(artifact.as_bytes());
    let terminal = AnalysisRunTerminalResult::succeeded(
        &run,
        &accepted_dto,
        "artifact-contract-1",
        digest,
        SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
        "2026-08-02T03:04:05Z",
        AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated").expect("summary"),
    )
    .expect("terminal");
    let transition = AnalysisRunLifecycleTransition::terminal(
        accepted_dto.run_id.clone(),
        run.idempotency_key.clone(),
        terminal,
        Some(artifact),
    )
    .expect("transition");
    let terminal_exchange =
        naruon_analysis_run_terminal_exchange("https://tepp.example.com", &transition)
            .expect("terminal exchange");
    assert!(terminal_exchange.target_url.ends_with("/terminal"));
    let terminal_response = service.handle_http_request(&post_lifecycle(
        &format!(
            "{NARUON_ANALYSIS_RUN_PATH}/{}/terminal",
            accepted_dto.run_id
        ),
        &transition.to_json().expect("terminal json"),
        run.idempotency_key.as_str(),
    ));
    assert_eq!(terminal_response.status_code, 200);
    assert!(
        terminal_response
            .body
            .contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA)
    );
    let get_succeeded = service.handle_http_request(&get_status(
        &accepted_dto.run_id,
        run.idempotency_key.as_str(),
    ));
    assert_eq!(get_succeeded.status_code, 200);
    assert_eq!(get_succeeded.body, terminal_response.body);
}

#[test]
fn failed_terminal_post_cannot_carry_scientific_acceptance() {
    let run = request();
    let mut service = AnalysisRunLiveService::new();
    let accepted = service.handle_http_request(&post_create(&run));
    let accepted_dto = AnalysisRunAccepted::from_json(&accepted.body).expect("accepted");
    let failed = AnalysisRunTerminalResult::failed(
        &run,
        &accepted_dto,
        "2026-08-02T03:04:05Z",
        "estimation_failed",
    )
    .expect("failed");
    let transition = AnalysisRunLifecycleTransition::terminal(
        accepted_dto.run_id.clone(),
        run.idempotency_key.clone(),
        failed,
        None,
    )
    .expect("transition");
    let response = service.handle_http_request(&post_lifecycle(
        &format!(
            "{NARUON_ANALYSIS_RUN_PATH}/{}/terminal",
            accepted_dto.run_id
        ),
        &transition.to_json().expect("json"),
        run.idempotency_key.as_str(),
    ));
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("\"failed\""));
    assert!(!response.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    assert!(!response.body.contains("\"scientific_acceptance\":"));
}
