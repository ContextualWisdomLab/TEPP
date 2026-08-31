//! GAP-003A naruon/LineageWeave scientific-acceptance execute consumer exchange.

use std::fmt::Write as _;

use analysis_engine::{
    ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION, SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
    SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION, ScientificAcceptanceExecuteRequest,
    ScientificAcceptanceLoopbackService, VALIDATION_CPU_F64_MODEL,
    lineageweave_analysis_run_execute_exchange, naruon_analysis_run_execute_exchange,
};
use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunRequest, ApiError,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, NaruonHttpExchange,
    SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
    lineageweave_analysis_run_exchange, naruon_analysis_run_exchange,
};

fn request(profile: &str, model: &str, idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "tenant-workspace-execute".into(),
        snapshot_id: "snapshot-execute".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: model.into(),
        output_profile: profile.into(),
    }
}

fn execute_json(run_id: &str, idempotency_key: &str, authored_by_llm: bool) -> String {
    serde_json::json!({
        "contract_version": ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
        "run_id": run_id,
        "idempotency_key": idempotency_key,
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

fn http_from_exchange(exchange: &NaruonHttpExchange) -> String {
    let without_scheme = exchange
        .target_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .expect("scheme");
    let path = without_scheme
        .find('/')
        .map(|index| &without_scheme[index..])
        .expect("path");
    let mut request = format!("{} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n", exchange.method);
    for (name, value) in &exchange.headers {
        write!(request, "{name}: {value}\r\n").expect("header");
    }
    write!(
        request,
        "content-length: {}\r\n\r\n{}",
        exchange.body.len(),
        exchange.body
    )
    .expect("len");
    request
}

fn http_get(path: &str, consumer: &str, idempotency_key: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
    )
}

fn accept_from_exchange(
    service: &mut ScientificAcceptanceLoopbackService,
    exchange: &NaruonHttpExchange,
) -> AnalysisRunAccepted {
    let accepted = service.handle_http_request(&http_from_exchange(exchange));
    assert_eq!(accepted.status_code, 202, "{}", accepted.body);
    assert!(!accepted.body.contains("rmse"));
    AnalysisRunAccepted::from_json(&accepted.body).expect("accepted")
}

#[test]
fn naruon_execute_exchange_is_typed_https_post_without_artifact_or_credentials() {
    let execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        "tepp-run-1",
        "idem-naruon-execute",
        false,
    ))
    .expect("execute");
    let exchange = naruon_analysis_run_execute_exchange("https://tepp.example.com", &execute)
        .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/analysis-runs/tepp-run-1/execute")
    );
    assert!(
        exchange
            .headers
            .contains(&("tepp-consumer".into(), NARUON_CONSUMER_CODE.into()))
    );
    assert!(!exchange.body.contains("scientific_acceptance_json"));
    assert!(!exchange.body.contains("rmse"));
    assert!(!exchange.headers.iter().any(|(name, _)| {
        name.to_ascii_lowercase().contains("authorization")
            || name.to_ascii_lowercase().contains("token")
            || name.to_ascii_lowercase().contains("copilot")
    }));
    assert_eq!(
        naruon_analysis_run_execute_exchange("http://tepp.example.com", &execute),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn lineageweave_execute_exchange_replaces_only_the_consumer_identity() {
    let execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        "tepp-run-1",
        "idem-lineageweave-execute",
        false,
    ))
    .expect("execute");
    let exchange = lineageweave_analysis_run_execute_exchange("https://tepp.example.com", &execute)
        .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/analysis-runs/tepp-run-1/execute")
    );
    assert!(
        exchange
            .headers
            .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
    );
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(_, value)| value == NARUON_CONSUMER_CODE)
    );
}

#[test]
fn execute_exchange_refuses_llm_metrics_and_unknown_artifact_fields() {
    assert!(
        ScientificAcceptanceExecuteRequest::from_json(&execute_json(
            "tepp-run-1",
            "idem-naruon-execute",
            true,
        ))
        .is_err()
    );
    let with_rmse = execute_json("tepp-run-1", "idem-naruon-execute", false).replacen(
        '{',
        r#"{"rmse":0.1,"#,
        1,
    );
    assert!(ScientificAcceptanceExecuteRequest::from_json(&with_rmse).is_err());
    let with_artifact = execute_json("tepp-run-1", "idem-naruon-execute", false).replacen(
        '{',
        r#"{"scientific_acceptance_json":"{}","#,
        1,
    );
    assert!(ScientificAcceptanceExecuteRequest::from_json(&with_artifact).is_err());
}

#[test]
fn naruon_and_lineageweave_execute_exchanges_produce_scientific_acceptance() {
    let mut service = ScientificAcceptanceLoopbackService::new();
    let naruon_run = request(
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
        "idem-naruon-execute",
    );
    let naruon_create =
        naruon_analysis_run_exchange("https://tepp.example.com", &naruon_run).expect("create");
    let naruon_accepted = accept_from_exchange(&mut service, &naruon_create);
    let naruon_execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        &naruon_accepted.run_id,
        naruon_run.idempotency_key.as_str(),
        false,
    ))
    .expect("execute");
    let naruon_exchange =
        naruon_analysis_run_execute_exchange("https://tepp.example.com", &naruon_execute)
            .expect("naruon execute");
    let naruon_response = service.handle_http_request(&http_from_exchange(&naruon_exchange));
    assert_eq!(naruon_response.status_code, 200, "{}", naruon_response.body);
    assert!(
        naruon_response
            .body
            .contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA)
    );

    let lineage_run = request(
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
        "idem-lineageweave-execute",
    );
    let lineage_create =
        lineageweave_analysis_run_exchange("https://tepp.example.com", &lineage_run)
            .expect("lineage create");
    let lineage_accepted = accept_from_exchange(&mut service, &lineage_create);
    let lineage_execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        &lineage_accepted.run_id,
        lineage_run.idempotency_key.as_str(),
        false,
    ))
    .expect("lineage execute");
    let lineage_exchange =
        lineageweave_analysis_run_execute_exchange("https://tepp.example.com", &lineage_execute)
            .expect("lineage execute exchange");
    let lineage_response = service.handle_http_request(&http_from_exchange(&lineage_exchange));
    assert_eq!(
        lineage_response.status_code, 200,
        "{}",
        lineage_response.body
    );
    let get = service.handle_http_request(&http_get(
        &format!("{NARUON_ANALYSIS_RUN_PATH}/{}", lineage_accepted.run_id),
        LINEAGEWEAVE_CONSUMER_CODE,
        lineage_run.idempotency_key.as_str(),
    ));
    assert_eq!(get.status_code, 200, "{}", get.body);
    assert!(get.body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert!(get.body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE));
    assert!(get.body.contains("scientific_acceptance"));
}
