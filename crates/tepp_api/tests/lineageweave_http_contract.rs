//! `LineageWeave` uses the published asynchronous TEPP analysis-run boundary.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunCancelRequest,
    AnalysisRunLiveService, AnalysisRunRequest, AnalysisRunRetryParent, AnalysisRunRetryRequest,
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
    NARUON_LIVE_HEADER_BYTE_LIMIT, lineageweave_analysis_run_exchange,
    lineageweave_analysis_run_retry_parent_exchange,
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
    assert!(
        exchange
            .headers
            .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
    );
    assert!(
        exchange
            .headers
            .contains(&("idempotency-key".into(), run.idempotency_key.clone()))
    );
    assert!(exchange.headers.iter().all(|(name, _)| {
        !matches!(
            name.to_ascii_lowercase().as_str(),
            "authorization" | "proxy-authorization" | "cookie" | "x-api-key"
        )
    }));
}

#[test]
fn live_listener_accepts_lineageweave_and_isolates_consumer_idempotency() {
    let loopback = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
    assert!(
        loopback
            .local_addr()
            .expect("loopback address")
            .ip()
            .is_loopback()
    );
    assert_eq!(
        AnalysisRunLiveService::bind("0.0.0.0:0".parse().expect("non-loopback address"))
            .expect_err("non-loopback bind must fail"),
        ApiError::AuthorizationDenied
    );

    let run = sample_run();
    let mut service = AnalysisRunLiveService::new();

    let naruon = service.handle_http_request(&http_request(NARUON_CONSUMER_CODE, &run));
    let lineageweave = service.handle_http_request(&http_request(LINEAGEWEAVE_CONSUMER_CODE, &run));

    assert_eq!(naruon.status_code, 202);
    assert_eq!(lineageweave.status_code, 202);
    let naruon_accepted = AnalysisRunAccepted::from_json(&naruon.body).expect("naruon ack");
    let lineageweave_accepted =
        AnalysisRunAccepted::from_json(&lineageweave.body).expect("lineageweave ack");
    assert_ne!(naruon_accepted.run_id, lineageweave_accepted.run_id);
    assert_eq!(lineageweave_accepted.run_state, "accepted");
    assert_eq!(lineageweave_accepted.idempotency_key, run.idempotency_key);

    let replay = service.handle_http_request(&http_request(LINEAGEWEAVE_CONSUMER_CODE, &run));
    assert_eq!(replay.status_code, 202);
    assert_eq!(replay.body, lineageweave.body);

    let mut conflict = run.clone();
    conflict.snapshot_id = "lineageweave-snapshot-conflict".into();
    let conflict_response =
        service.handle_http_request(&http_request(LINEAGEWEAVE_CONSUMER_CODE, &conflict));
    assert_eq!(conflict_response.status_code, 400);
}

#[test]
fn live_listener_refuses_an_unpublished_consumer() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(service.handle_http_request("").status_code, 400);
    assert_eq!(
        service
            .handle_http_request(&"x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT))
            .status_code,
        413
    );
    let duplicate_headers = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nhost: 127.0.0.1\r\ncontent-length: 0\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&duplicate_headers).status_code,
        400
    );
    let response =
        service.handle_http_request(&http_request("unpublished-consumer", &sample_run()));
    assert_eq!(response.status_code, 400);
}

#[test]
fn live_listener_serves_lineageweave_over_loopback() {
    let run = sample_run();
    let mut service = AnalysisRunLiveService::bind_loopback().expect("loopback bind");
    let address = service.local_addr().expect("loopback address");
    let worker = thread::spawn(move || service.serve_one());
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    stream
        .write_all(http_request(LINEAGEWEAVE_CONSUMER_CODE, &run).as_bytes())
        .expect("request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response");
    assert!(response.starts_with("HTTP/1.1 202 Accepted"));
    assert_eq!(
        worker.join().expect("join").expect("served").status_code,
        202
    );
}

#[test]
fn lineageweave_retry_parent_exchange_uses_the_published_consumer_header_without_credentials() {
    let exchange = lineageweave_analysis_run_retry_parent_exchange(
        "https://tepp.example.test",
        "tepp-run-lineage",
    )
    .expect("lineageweave retry-parent exchange");
    assert_eq!(exchange.method, "GET");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/analysis-runs/tepp-run-lineage/parent"
    );
    assert!(exchange.body.is_empty());
    assert!(
        exchange
            .headers
            .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
    );
    assert!(
        !exchange
            .headers
            .contains(&("tepp-consumer".into(), NARUON_CONSUMER_CODE.into()))
    );
    assert!(exchange.headers.iter().all(|(name, _)| {
        !matches!(
            name.to_ascii_lowercase().as_str(),
            "authorization" | "proxy-authorization" | "cookie" | "x-api-key" | "idempotency-key"
        )
    }));
}

#[test]
fn live_listener_inspects_lineageweave_retry_parent_and_isolates_consumers() {
    let run = sample_run();
    let mut service = AnalysisRunLiveService::new();
    let lineageweave = service.handle_http_request(&http_request(LINEAGEWEAVE_CONSUMER_CODE, &run));
    assert_eq!(lineageweave.status_code, 202);
    let accepted = AnalysisRunAccepted::from_json(&lineageweave.body).expect("accepted");
    let inspect = format!(
        "GET {NARUON_ANALYSIS_RUN_PATH}/{}/parent HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
        accepted.run_id
    );
    let inspected = service.handle_http_request(&inspect);
    assert_eq!(inspected.status_code, 200);
    let parent = AnalysisRunRetryParent::from_json(&inspected.body).expect("parent");
    assert_eq!(parent.run_id, accepted.run_id);
    assert_eq!(parent.parent, None);
    assert!(inspected.body.contains("\"parent\":null"));
    assert!(!inspected.body.contains("rmse"));
    assert!(!inspected.body.contains("scientific_acceptance"));
    let naruon_inspect = format!(
        "GET {NARUON_ANALYSIS_RUN_PATH}/{}/parent HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
        accepted.run_id
    );
    assert_eq!(
        service.handle_http_request(&naruon_inspect).status_code,
        400
    );

    let cancel_body = AnalysisRunCancelRequest::new(&accepted.run_id, run.idempotency_key.as_str())
        .expect("cancel dto")
        .to_json()
        .expect("cancel json");
    let cancel = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH}/{}/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{cancel_body}",
        accepted.run_id,
        run.idempotency_key,
        cancel_body.len()
    );
    assert_eq!(service.handle_http_request(&cancel).status_code, 200);
    let retry_key = "lineageweave-retry-parent-idem";
    let retry_body = AnalysisRunRetryRequest::new(&accepted.run_id, retry_key)
        .expect("retry dto")
        .to_json()
        .expect("retry json");
    let retry = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH}/{}/retry HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {retry_key}\r\ncontent-length: {}\r\n\r\n{retry_body}",
        accepted.run_id,
        retry_body.len()
    );
    let retried = service.handle_http_request(&retry);
    assert_eq!(retried.status_code, 202);
    let child = AnalysisRunAccepted::from_json(&retried.body).expect("child");
    let child_inspect = format!(
        "GET {NARUON_ANALYSIS_RUN_PATH}/{}/parent HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\ncontent-length: 0\r\n\r\n",
        child.run_id
    );
    let child_parent = service.handle_http_request(&child_inspect);
    assert_eq!(child_parent.status_code, 200);
    let payload = AnalysisRunRetryParent::from_json(&child_parent.body).expect("child parent");
    assert_eq!(payload.run_id, child.run_id);
    let parent_item = payload.parent.expect("non-null parent");
    assert_eq!(parent_item.run_id, accepted.run_id);
    assert!(!child_parent.body.contains("rmse"));
    assert!(!child_parent.body.contains("scientific_acceptance"));
    assert!(!child_parent.body.contains("tenant_workspace_id"));
    assert!(!child_parent.body.contains("snapshot_id"));
    assert!(!child_parent.body.contains("retried_from"));
}
