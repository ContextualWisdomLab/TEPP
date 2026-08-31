//! GAP-003A typed execute exchanges over the spawned tepp-loopback TCP listener.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use analysis_engine::{
    lineageweave_analysis_run_execute_exchange, loopback_http1_from_execute_exchange,
    loopback_http1_from_naruon_exchange, naruon_analysis_run_execute_exchange,
    ScientificAcceptanceExecuteRequest, ANALYSIS_RUN_EXECUTE_CONTRACT_VERSION,
    SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
    VALIDATION_CPU_F64_MODEL,
};
use tepp_api::{
    lineageweave_analysis_run_exchange, naruon_analysis_run_exchange,
    naruon_analysis_run_status_exchange, AnalysisRunAccepted, AnalysisRunRequest, ApiError,
    NaruonHttpExchange, ANALYSIS_RUN_CONTRACT_VERSION, LINEAGEWEAVE_CONSUMER_CODE,
    NARUON_CONSUMER_CODE, SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE, SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA,
};

const HTTPS_ORIGIN: &str = "https://tepp.example.com";

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

fn request(profile: &str, model: &str, idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "tenant-workspace-execute-tcp".into(),
        snapshot_id: "snapshot-execute-tcp".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: model.into(),
        output_profile: profile.into(),
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
        "study_label": "loopback-tcp-recovery",
        "authored_by_llm": false,
        "corpus": {
            "snapshot_id": "snapshot-execute-tcp",
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

fn status_exchange_for_consumer(
    run_id: &str,
    idempotency_key: &str,
    consumer: &str,
) -> NaruonHttpExchange {
    let mut exchange = naruon_analysis_run_status_exchange(HTTPS_ORIGIN, run_id, idempotency_key)
        .expect("status exchange");
    let header = exchange
        .headers
        .iter_mut()
        .find(|(name, _)| name.eq_ignore_ascii_case("tepp-consumer"))
        .expect("consumer header");
    consumer.clone_into(&mut header.1);
    exchange
}

fn accept_on_tcp(address: &str, create: &NaruonHttpExchange) -> AnalysisRunAccepted {
    let request =
        loopback_http1_from_naruon_exchange(create, address.trim()).expect("create http1");
    let accepted = exchange(address, &request);
    assert!(accepted.starts_with("HTTP/1.1 202 Accepted"), "{accepted}");
    assert!(!accepted.contains("rmse"));
    assert!(!accepted.contains("scientific_acceptance"));
    AnalysisRunAccepted::from_json(response_body(&accepted)).expect("accepted")
}

#[test]
fn execute_loopback_http1_refuses_public_bind_and_non_execute_exchanges() {
    let execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        "tepp-run-1",
        "idem-naruon-execute-tcp",
    ))
    .expect("execute");
    let naruon = naruon_analysis_run_execute_exchange(HTTPS_ORIGIN, &execute).expect("naruon");
    assert_eq!(
        loopback_http1_from_execute_exchange(&naruon, "8.8.8.8:80"),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        loopback_http1_from_execute_exchange(&naruon, "localhost:18081"),
        Err(ApiError::InvalidWirePayload)
    );
    let status =
        naruon_analysis_run_status_exchange(HTTPS_ORIGIN, "tepp-run-1", "idem-naruon-execute-tcp")
            .expect("status");
    assert_eq!(
        loopback_http1_from_execute_exchange(&status, "127.0.0.1:18081"),
        Err(ApiError::InvalidWirePayload)
    );
    let http1 = loopback_http1_from_execute_exchange(&naruon, "127.0.0.1:18081").expect("http1");
    assert!(http1.starts_with("POST /v1/analysis-runs/tepp-run-1/execute HTTP/1.1"));
    assert!(http1.contains("Host: 127.0.0.1:18081"));
    assert!(http1.contains("tepp-consumer: naruon"));
    assert!(!http1.to_ascii_lowercase().contains("authorization"));
    assert!(!http1.contains("scientific_acceptance_json"));
}

#[test]
fn naruon_and_lineageweave_execute_exchanges_over_spawned_loopback_tcp() {
    let (mut child, address) = spawn_loopback("6");
    let host = address.trim();

    let naruon_run = request(
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
        "idem-naruon-execute-tcp",
    );
    let naruon_create = naruon_analysis_run_exchange(HTTPS_ORIGIN, &naruon_run).expect("create");
    let naruon_accepted = accept_on_tcp(&address, &naruon_create);
    let naruon_execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        &naruon_accepted.run_id,
        naruon_run.idempotency_key.as_str(),
    ))
    .expect("naruon execute");
    let naruon_exchange = naruon_analysis_run_execute_exchange(HTTPS_ORIGIN, &naruon_execute)
        .expect("naruon exchange");
    let naruon_http =
        loopback_http1_from_execute_exchange(&naruon_exchange, host).expect("naruon http1");
    let naruon_response = exchange(&address, &naruon_http);
    assert!(
        naruon_response.starts_with("HTTP/1.1 200 OK"),
        "{naruon_response}"
    );
    assert!(naruon_response.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA));
    let naruon_get = loopback_http1_from_naruon_exchange(
        &status_exchange_for_consumer(
            &naruon_accepted.run_id,
            naruon_run.idempotency_key.as_str(),
            NARUON_CONSUMER_CODE,
        ),
        host,
    )
    .expect("naruon get");
    let naruon_status = exchange(&address, &naruon_get);
    assert!(
        naruon_status.starts_with("HTTP/1.1 200 OK"),
        "{naruon_status}"
    );
    assert!(naruon_status.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert!(naruon_status.contains(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE));
    assert!(naruon_status.contains("scientific_acceptance"));

    let lineage_run = request(
        SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
        VALIDATION_CPU_F64_MODEL,
        "idem-lineageweave-execute-tcp",
    );
    let lineage_create =
        lineageweave_analysis_run_exchange(HTTPS_ORIGIN, &lineage_run).expect("lineage create");
    let lineage_accepted = accept_on_tcp(&address, &lineage_create);
    let lineage_execute = ScientificAcceptanceExecuteRequest::from_json(&execute_json(
        &lineage_accepted.run_id,
        lineage_run.idempotency_key.as_str(),
    ))
    .expect("lineage execute");
    let lineage_exchange =
        lineageweave_analysis_run_execute_exchange(HTTPS_ORIGIN, &lineage_execute)
            .expect("lineage exchange");
    let lineage_http =
        loopback_http1_from_execute_exchange(&lineage_exchange, host).expect("lineage http1");
    assert!(lineage_http.contains("tepp-consumer: lineageweave"));
    assert!(!lineage_http.contains("tepp-consumer: naruon"));
    let lineage_response = exchange(&address, &lineage_http);
    assert!(
        lineage_response.starts_with("HTTP/1.1 200 OK"),
        "{lineage_response}"
    );
    let lineage_get = loopback_http1_from_naruon_exchange(
        &status_exchange_for_consumer(
            &lineage_accepted.run_id,
            lineage_run.idempotency_key.as_str(),
            LINEAGEWEAVE_CONSUMER_CODE,
        ),
        host,
    )
    .expect("lineage get");
    let lineage_status = exchange(&address, &lineage_get);
    assert!(
        lineage_status.starts_with("HTTP/1.1 200 OK"),
        "{lineage_status}"
    );
    assert!(lineage_status.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert!(lineage_status.contains(SCIENTIFIC_ACCEPTANCE_HTTP_PROFILE));
    assert!(lineage_status.contains("scientific_acceptance"));
    assert!(lineage_status.contains("rmse"));
    assert!(child.wait().expect("wait").success());
}
