//! LineageWeave uses the published asynchronous TEPP analysis-run boundary.

use std::fmt::Write as _;

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLiveService,
    AnalysisRunRequest, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH,
    lineageweave_analysis_run_exchange,
};

fn sample_run() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "shared-idempotency-key".into(),
        tenant_workspace_id: "shared-tenant-workspace".into(),
        snapshot_id: "lineageweave-snapshot-001".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "tepp-analysis-run-v1".into(),
        output_profile: "calibrated_event_measurement".into(),
    }
}

fn http_request(consumer: &str, run: &AnalysisRunRequest) -> String {
    let body = run.to_json().expect("run json");
    let mut request = format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\n");
    for (name, value) in [
        ("Host", "127.0.0.1"),
        ("content-type", "application/json"),
        ("tepp-consumer", consumer),
        ("tepp-contract-version", "1"),
        ("idempotency-key", run.idempotency_key.as_str()),
    ] {
        write!(request, "{name}: {value}\r\n").expect("header");
    }
    write!(request, "content-length: {}\r\n\r\n{body}", body.len()).expect("body");
    request
}

#[test]
fn lineageweave_exchange_uses_the_published_consumer_header_without_credentials() {
    let run = sample_run();
    let exchange = lineageweave_analysis_run_exchange("https://tepp.example.test", &run)
        .expect("lineageweave exchange");
    assert_eq!(exchange.method, "POST");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs"
    );
    assert!(exchange.headers.contains(&(
        "tepp-consumer".into(),
        LINEAGEWEAVE_CONSUMER_CODE.into()
    )));
    assert!(exchange.headers.contains(&(
        "idempotency-key".into(),
        run.idempotency_key.clone()
    )));
    assert!(exchange.headers.iter().all(|(name, _)| {
        !matches!(
            name.to_ascii_lowercase().as_str(),
            "authorization" | "proxy-authorization" | "cookie" | "x-api-key"
        )
    }));
}

#[test]
fn live_listener_accepts_lineageweave_and_isolates_consumer_idempotency() {
    let run = sample_run();
    let mut service = AnalysisRunLiveService::new();

    let naruon = service.handle_http_request(&http_request("naruon", &run));
    let lineageweave = service.handle_http_request(&http_request(
        LINEAGEWEAVE_CONSUMER_CODE,
        &run,
    ));

    assert_eq!(naruon.status_code, 202);
    assert_eq!(lineageweave.status_code, 202);
    let naruon_accepted = AnalysisRunAccepted::from_json(&naruon.body).expect("naruon ack");
    let lineageweave_accepted =
        AnalysisRunAccepted::from_json(&lineageweave.body).expect("lineageweave ack");
    assert_ne!(naruon_accepted.run_id, lineageweave_accepted.run_id);
    assert_eq!(lineageweave_accepted.run_state, "accepted");
    assert_eq!(lineageweave_accepted.idempotency_key, run.idempotency_key);

    let replay = service.handle_http_request(&http_request(
        LINEAGEWEAVE_CONSUMER_CODE,
        &run,
    ));
    assert_eq!(replay.status_code, 202);
    assert_eq!(replay.body, lineageweave.body);
}

#[test]
fn live_listener_refuses_an_unpublished_consumer() {
    let mut service = AnalysisRunLiveService::new();
    let response = service.handle_http_request(&http_request("unpublished-consumer", &sample_run()));
    assert_eq!(response.status_code, 400);
}
