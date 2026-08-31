//! GAP-003A engine-on-loopback scientific-acceptance contract.

use analysis_engine::{
    ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION, SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
    SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION, ScientificAcceptanceLoopbackService,
    VALIDATION_CPU_F64_MODEL,
};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunRequest,
    NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
    SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
};

fn request(profile: &str, model: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-loopback-execute".into(),
        tenant_workspace_id: "tenant-workspace-execute".into(),
        snapshot_id: "snapshot-execute".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: model.into(),
        output_profile: profile.into(),
    }
}

fn http_post(path: &str, body: &str, consumer: &str, idempotency_key: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn http_get(path: &str, consumer: &str, idempotency_key: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
    )
}

fn execute_body(run_id: &str, authored_by_llm: bool) -> String {
    serde_json::json!({
        "contract_version": ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
        "run_id": run_id,
        "idempotency_key": "idem-loopback-execute",
        "seed": 42,
        "se_gate_k": 3.0,
        "completed_at": "2026-08-31T10:00:00Z",
        "study_label": "loopback-recovery",
        "authored_by_llm": authored_by_llm,
        "corpus": {
            "snapshot_id": "snapshot-execute",
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

fn accept_run(
    service: &mut ScientificAcceptanceLoopbackService,
    profile: &str,
    model: &str,
) -> AnalysisRunAccepted {
    let run = request(profile, model);
    let accepted = service.handle_http_request(&http_post(
        NARUON_ANALYSIS_RUN_PATH,
        &run.to_json().expect("create json"),
        NARUON_CONSUMER_CODE,
        run.idempotency_key.as_str(),
    ));
    assert_eq!(accepted.status_code, 202, "{}", accepted.body);
    assert!(!accepted.body.contains("rmse"));
    assert!(!accepted.body.contains("scientific_acceptance"));
    AnalysisRunAccepted::from_json(&accepted.body).expect("accepted")
}

#[test]
fn execute_produces_scientific_acceptance_without_caller_artifact() {
    let mut service = ScientificAcceptanceLoopbackService::new();
    let accepted = accept_run(
        &mut service,
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
    );
    let body = execute_body(&accepted.run_id, false);
    assert!(!body.contains("scientific_acceptance_json"));
    assert!(!body.contains("rmse"));
    let execute = service.handle_http_request(&http_post(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}/execute", accepted.run_id),
        &body,
        NARUON_CONSUMER_CODE,
        "idem-loopback-execute",
    ));
    assert_eq!(execute.status_code, 200, "{}", execute.body);
    assert!(execute.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    assert!(execute.body.contains("scientific_acceptance"));
    assert!(execute.body.contains("\"succeeded\""));

    let get = service.handle_http_request(&http_get(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}", accepted.run_id),
        NARUON_CONSUMER_CODE,
        "idem-loopback-execute",
    ));
    assert_eq!(get.status_code, 200, "{}", get.body);
    assert!(get.body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert!(get.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE));
    assert!(get.body.contains("scientific_acceptance"));
    assert!(get.body.contains("rmse"));
}

#[test]
fn execute_fail_closed_for_hostile_bodies_and_bindings() {
    let mut service = ScientificAcceptanceLoopbackService::new();
    let accepted = accept_run(
        &mut service,
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
    );
    let body = execute_body(&accepted.run_id, false);
    let path = format!("{NARUON_ANALYSIS_RUN_PATH}/{}/execute", accepted.run_id);

    let with_rmse = body.replacen('{', r#"{"rmse":0.1,"#, 1);
    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &with_rmse,
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );

    let with_artifact = body.replacen('{', r#"{"scientific_acceptance_json":"{}","#, 1);
    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &with_artifact,
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );

    let llm = execute_body(&accepted.run_id, true);
    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &llm,
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );

    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &body,
                "lineageweave",
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );

    assert_eq!(
        service
            .handle_http_request(&http_post(
                &format!("{NARUON_ANALYSIS_RUN_PATH}/tepp-run-999/execute"),
                &execute_body("tepp-run-999", false),
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );

    let wrong_version = body.replace(
        &format!("\"contract_version\":{ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION}"),
        "\"contract_version\":9",
    );
    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &wrong_version,
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        422
    );

    let ok = service.handle_http_request(&http_post(
        &path,
        &body,
        NARUON_CONSUMER_CODE,
        "idem-loopback-execute",
    ));
    assert_eq!(ok.status_code, 200, "{}", ok.body);
    assert_eq!(
        service
            .handle_http_request(&http_post(
                &path,
                &body,
                NARUON_CONSUMER_CODE,
                "idem-loopback-execute",
            ))
            .status_code,
        400
    );
}

#[test]
fn execute_refuses_wrong_profile_and_leaves_accepted_metric_free() {
    let mut service = ScientificAcceptanceLoopbackService::new();
    let accepted = accept_run(
        &mut service,
        "calibrated_event_measurement",
        VALIDATION_CPU_F64_MODEL,
    );
    let execute = service.handle_http_request(&http_post(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}/execute", accepted.run_id),
        &execute_body(&accepted.run_id, false),
        NARUON_CONSUMER_CODE,
        "idem-loopback-execute",
    ));
    assert_eq!(execute.status_code, 400, "{}", execute.body);
    let get = service.handle_http_request(&http_get(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}", accepted.run_id),
        NARUON_CONSUMER_CODE,
        "idem-loopback-execute",
    ));
    assert_eq!(get.status_code, 200, "{}", get.body);
    assert!(get.body.contains("\"accepted\""));
    assert!(!get.body.contains("rmse"));
    assert!(!get.body.contains("scientific_acceptance"));
}
