//! Operator-visible loopback GET contract for GAP-003A scientific acceptance.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLiveService, AnalysisRunRequest,
    ApiError, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE,
    receipt_json_carries_scientific_metrics, refuse_metrics_on_receipt,
};

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "http-contract-idem-1".into(),
        tenant_workspace_id: "http-contract-tenant".into(),
        snapshot_id: "http-contract-snapshot".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "validation_cpu_f64_v1".into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE.into(),
    }
}

fn post_request(run: &AnalysisRunRequest) -> String {
    let body = run.to_json().expect("body");
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    )
}

fn get_request(run_id: &str, idempotency_key: &str) -> String {
    format!(
        "GET {NARUON_ANALYSIS_RUN_PATH}/{run_id} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
    )
}

#[test]
fn post_receipt_and_get_accepted_status_stay_metric_free() {
    let run = request();
    let mut service = AnalysisRunLiveService::new();
    let accepted = service.handle_http_request(&post_request(&run));
    assert_eq!(accepted.status_code, 202);
    assert!(!receipt_json_carries_scientific_metrics(&accepted.body));
    assert_eq!(refuse_metrics_on_receipt(&accepted.body), Ok(()));
    let accepted_dto = AnalysisRunAccepted::from_json(&accepted.body).expect("accepted");
    let status = service.handle_http_request(&get_request(
        &accepted_dto.run_id,
        run.idempotency_key.as_str(),
    ));
    assert_eq!(status.status_code, 200);
    assert!(status.body.contains("\"accepted\""));
    assert!(!status.body.contains("scientific_acceptance"));
    assert!(!status.body.contains("rmse"));
    assert_eq!(refuse_metrics_on_receipt(&status.body), Ok(()));
    let replay = service.handle_http_request(&post_request(&run));
    assert_eq!(replay.body, accepted.body);
}

#[test]
fn metric_keys_on_the_create_receipt_fail_closed() {
    let run = request();
    let body = run
        .to_json()
        .expect("json")
        .replacen('{', r#"{"rmse":0.02,"#, 1);
    let request = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    );
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(service.handle_http_request(&request).status_code, 400);
    assert_eq!(
        refuse_metrics_on_receipt(&body),
        Err(ApiError::InvalidWirePayload)
    );
}
