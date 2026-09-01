//! Loopback interpretation POSTs stay hypothetical and fail closed (ADR 0010/0011).

use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use orchestrator_live::{
    InterpretationRunAccepted, InterpretationRunCollection, InterpretationRunRequest,
    OrchestrationMode, OrchestratorLiveError, OrchestratorLiveService,
    DEFAULT_INTERPRETATION_BYTE_LIMIT, INTERPRETATION_RUN_CONTRACT_VERSION,
    INTERPRETATION_RUN_PATH, LIVE_HEADER_BYTE_LIMIT, LIVE_HEADER_COUNT_LIMIT,
};

fn sample_request() -> InterpretationRunRequest {
    InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "orch-live-idem-001",
        "orch-tenant-workspace-demo",
        "tepp-snapshot-demo-001",
        "2026-08-01T00:00:00Z",
        OrchestrationMode::Direct,
        2048,
        vec!["span-001".into()],
        false,
    )
    .expect("sample")
}

fn orchestrator_headers(idempotency_key: &str) -> Vec<(String, String)> {
    vec![
        ("Host".into(), "127.0.0.1".into()),
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "contextual-orchestrator".into()),
        ("tepp-contract-version".into(), "1".into()),
        ("idempotency-key".into(), idempotency_key.to_owned()),
    ]
}

fn collection_headers() -> Vec<(String, String)> {
    vec![
        ("Host".into(), "127.0.0.1".into()),
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "contextual-orchestrator".into()),
        ("tepp-contract-version".into(), "1".into()),
    ]
}

fn http_request(method: &str, path: &str, headers: &[(String, String)], body: &str) -> String {
    let mut request = format!("{method} {path} HTTP/1.1\r\n");
    for (name, value) in headers {
        write!(request, "{name}: {value}\r\n").expect("header");
    }
    write!(request, "content-length: {}\r\n\r\n{body}", body.len()).expect("len");
    request
}

fn interpretation_http(request: &InterpretationRunRequest) -> String {
    http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &orchestrator_headers(request.idempotency_key()),
        &request.to_json().expect("json"),
    )
}

fn serve_once(payload: &[u8]) -> orchestrator_live::OrchestratorLiveResponse {
    let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
    let address = service.local_addr().expect("address");
    let worker = thread::spawn(move || service.serve_one());
    let mut stream = TcpStream::connect(address).expect("connect");
    stream.write_all(payload).expect("write");
    let mut received = String::new();
    match stream.read_to_string(&mut received) {
        Ok(_) => assert!(!received.is_empty()),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
        Err(error) => panic!("read: {error}"),
    }
    worker.join().expect("join").expect("serve")
}

fn error_code(body: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(body).expect("envelope");
    value["error_code"].as_str().expect("error_code").to_owned()
}

#[test]
fn loopback_bind_refuses_non_loopback_and_unbound_serve() {
    assert_eq!(
        OrchestratorLiveService::bind("0.0.0.0:0".parse::<SocketAddr>().expect("unspec"))
            .expect_err("denied"),
        OrchestratorLiveError::AuthorizationDenied
    );
    assert_eq!(
        OrchestratorLiveService::bind("8.8.8.8:0".parse::<SocketAddr>().expect("public"))
            .expect_err("denied"),
        OrchestratorLiveError::AuthorizationDenied
    );
    assert_eq!(
        OrchestratorLiveService::bind("[::]:0".parse::<SocketAddr>().expect("v6-unspec"))
            .expect_err("denied"),
        OrchestratorLiveError::AuthorizationDenied
    );
    let first = OrchestratorLiveService::bind_loopback().expect("first bind");
    let addr = first.local_addr().expect("addr");
    assert!(addr.ip().is_loopback());
    assert_eq!(
        OrchestratorLiveService::bind(addr).expect_err("in use"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        OrchestratorLiveService::new()
            .local_addr()
            .expect_err("no sock"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        OrchestratorLiveService::new()
            .serve_one()
            .expect_err("no sock"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        OrchestratorLiveService::default()
            .serve_one()
            .expect_err("default"),
        OrchestratorLiveError::InvalidWirePayload
    );
}

#[test]
fn handle_http_accepts_interpretation_run_and_replays_idempotent_retries() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let first = service.handle_http_request(&interpretation_http(&request));
    assert_eq!(first.status_code, 202);
    assert_eq!(first.reason_phrase, "Accepted");
    let accepted = InterpretationRunAccepted::from_json(&first.body).expect("accepted");
    assert_eq!(accepted.idempotency_key(), request.idempotency_key());
    assert_eq!(accepted.orchestration_mode(), OrchestrationMode::Direct);
    assert_eq!(accepted.claim_status(), "hypothetical");
    assert!(!accepted.scientific_authority());
    assert!(!accepted.interpretation_run_id().is_empty());

    let replay = service.handle_http_request(&interpretation_http(&request));
    assert_eq!(replay.status_code, 202);
    assert_eq!(replay.body, first.body);

    let conflicting = InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        request.idempotency_key(),
        request.tenant_workspace_id(),
        request.snapshot_id(),
        request.knowledge_cutoff(),
        OrchestrationMode::Verify,
        2048,
        vec!["span-001".into()],
        false,
    )
    .expect("conflict");
    let conflict = service.handle_http_request(&interpretation_http(&conflicting));
    assert_eq!(conflict.status_code, 400);
    assert_eq!(error_code(&conflict.body), "invalid_wire_payload");
}

#[test]
fn handle_http_refuses_scientific_authority_and_unknown_source_text() {
    let mut service = OrchestratorLiveService::new();
    let promoted = InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "orch-live-idem-sci",
        "orch-tenant-workspace-demo",
        "tepp-snapshot-demo-001",
        "2026-08-01T00:00:00Z",
        OrchestrationMode::Committee,
        4096,
        vec!["span-002".into()],
        true,
    );
    assert_eq!(
        promoted.expect_err("sci"),
        OrchestratorLiveError::ScientificAuthorityRefused
    );

    let mut json = sample_request().to_json().expect("json");
    json = json.replace(
        "\"scientific_authority\":false",
        "\"scientific_authority\":true",
    );
    let headers = orchestrator_headers("orch-live-idem-001");
    let response = service.handle_http_request(&http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &headers,
        &json,
    ));
    assert_eq!(response.status_code, 422);
    assert_eq!(error_code(&response.body), "scientific_authority_refused");
    assert!(!response.body.contains("token"));

    let with_source = sample_request()
        .to_json()
        .expect("json")
        .replace('}', ",\"source_text\":\"secret-body\"}");
    let leaked = service.handle_http_request(&http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &headers,
        &with_source,
    ));
    assert_eq!(leaked.status_code, 400);
    assert!(!leaked.body.contains("secret-body"));
}

#[test]
fn handle_http_refuses_methods_paths_and_table_hosts() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let body = request.to_json().expect("json");
    let headers = orchestrator_headers(request.idempotency_key());

    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                INTERPRETATION_RUN_PATH,
                &headers,
                &body
            ))
            .status_code,
        400
    );
    let over_budget = body.replace(
        "\"compute_budget_tokens\":2048",
        "\"compute_budget_tokens\":1000001",
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                INTERPRETATION_RUN_PATH,
                &headers,
                &over_budget,
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                "/v1/tables/document_record",
                &headers,
                &body
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request("POST", "/sql", &headers, &body))
            .status_code,
        400
    );
    let mut postgres_host = headers.clone();
    postgres_host[0] = ("Host".into(), "postgres.example.test".into());
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                INTERPRETATION_RUN_PATH,
                &postgres_host,
                &body
            ))
            .status_code,
        400
    );
    let mut jdbc_host = headers.clone();
    jdbc_host[0] = ("Host".into(), "jdbc.example.test".into());
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                INTERPRETATION_RUN_PATH,
                &jdbc_host,
                &body
            ))
            .status_code,
        400
    );
    let query = service.handle_http_request(&http_request(
        "POST",
        "/v1/interpretation-runs?drop=1",
        &headers,
        &body,
    ));
    assert_eq!(query.status_code, 400);
    let http10 = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.0\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key(),
        body.len()
    );
    assert_eq!(service.handle_http_request(&http10).status_code, 400);
}

#[test]
fn handle_http_enumerates_interpretation_runs_on_collection_get() {
    let mut service = OrchestratorLiveService::new();
    let empty = service.handle_http_request(&http_request(
        "GET",
        INTERPRETATION_RUN_PATH,
        &collection_headers(),
        "",
    ));
    assert_eq!(empty.status_code, 200);
    let empty_page = InterpretationRunCollection::from_json(&empty.body).expect("empty");
    assert!(empty_page.items.is_empty());
    assert_eq!(empty_page.next_cursor, None);

    let first = sample_request();
    assert_eq!(
        service
            .handle_http_request(&interpretation_http(&first))
            .status_code,
        202
    );
    let second = InterpretationRunRequest::new(
        INTERPRETATION_RUN_CONTRACT_VERSION,
        "orch-live-idem-002",
        "orch-tenant-workspace-demo",
        "tepp-snapshot-demo-001",
        "2026-08-01T00:00:00Z",
        OrchestrationMode::Verify,
        2048,
        vec!["span-001".into()],
        false,
    )
    .expect("second");
    assert_eq!(
        service
            .handle_http_request(&interpretation_http(&second))
            .status_code,
        202
    );

    let listed = service.handle_http_request(&http_request(
        "GET",
        INTERPRETATION_RUN_PATH,
        &collection_headers(),
        "",
    ));
    assert_eq!(listed.status_code, 200);
    let page = InterpretationRunCollection::from_json(&listed.body).expect("page");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].idempotency_key, "orch-live-idem-001");
    assert_eq!(page.items[1].idempotency_key, "orch-live-idem-002");
    assert!(page
        .items
        .iter()
        .all(|item| item.claim_status == "hypothetical"));
    assert!(page.items.iter().all(|item| !item.scientific_authority));
    assert!(!listed.body.contains("rmse"));
    assert!(!listed.body.contains("evidence_span_ids"));
    assert!(!listed.body.contains("tepp.scientific_acceptance.v1"));

    let mut limited_headers = collection_headers();
    limited_headers.push(("tepp-page-limit".into(), "1".into()));
    let limited = service.handle_http_request(&http_request(
        "GET",
        INTERPRETATION_RUN_PATH,
        &limited_headers,
        "",
    ));
    assert_eq!(limited.status_code, 200);
    let limited_page = InterpretationRunCollection::from_json(&limited.body).expect("limited");
    assert_eq!(limited_page.items.len(), 1);
    assert_eq!(
        limited_page.next_cursor.as_deref(),
        Some("orch-live-idem-001")
    );

    let mut cursor_headers = collection_headers();
    cursor_headers.push(("tepp-page-cursor".into(), "orch-live-idem-001".into()));
    cursor_headers.push(("tepp-page-limit".into(), "1".into()));
    let rest = service.handle_http_request(&http_request(
        "GET",
        INTERPRETATION_RUN_PATH,
        &cursor_headers,
        "",
    ));
    assert_eq!(rest.status_code, 200);
    let rest_page = InterpretationRunCollection::from_json(&rest.body).expect("rest");
    assert_eq!(rest_page.items.len(), 1);
    assert_eq!(rest_page.items[0].idempotency_key, "orch-live-idem-002");
    assert_eq!(rest_page.next_cursor, None);
}

#[test]
fn handle_http_retrieves_one_interpretation_run_on_get_by_id() {
    let mut service = OrchestratorLiveService::new();
    let first = sample_request();
    assert_eq!(
        service
            .handle_http_request(&interpretation_http(&first))
            .status_code,
        202
    );
    let got = service.handle_http_request(&http_request(
        "GET",
        "/v1/interpretation-runs/orch-live-idem-001",
        &collection_headers(),
        "",
    ));
    assert_eq!(got.status_code, 200, "{}", got.body);
    assert!(got
        .body
        .contains("\"idempotency_key\":\"orch-live-idem-001\""));
    assert!(got.body.contains("\"claim_status\":\"hypothetical\""));
    assert!(got.body.contains("\"scientific_authority\":false"));
    assert!(!got.body.contains("rmse"));
    assert!(!got.body.contains("evidence_span_ids"));
    assert!(!got.body.contains("tenant_workspace_id"));
    assert!(!got.body.contains("tepp.scientific_acceptance.v1"));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                "/v1/interpretation-runs/missing-key",
                &collection_headers(),
                "",
            ))
            .status_code,
        400
    );
    let mut paged = collection_headers();
    paged.push(("tepp-page-limit".into(), "1".into()));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                "/v1/interpretation-runs/orch-live-idem-001",
                &paged,
                "",
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                "/v1/interpretation-runs/orch-live-idem-001/extra",
                &collection_headers(),
                "",
            ))
            .status_code,
        400
    );
}

#[test]
fn handle_http_cancels_one_interpretation_run_and_drops_the_identity() {
    let mut service = OrchestratorLiveService::new();
    let first = sample_request();
    assert_eq!(
        service
            .handle_http_request(&interpretation_http(&first))
            .status_code,
        202
    );
    let mut naruon = collection_headers();
    naruon[2] = ("tepp-consumer".into(), "naruon".into());
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                "/v1/interpretation-runs/orch-live-idem-001/cancel",
                &naruon,
                "",
            ))
            .status_code,
        400
    );
    let cancelled = service.handle_http_request(&http_request(
        "POST",
        "/v1/interpretation-runs/orch-live-idem-001/cancel",
        &collection_headers(),
        "",
    ));
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    assert!(cancelled.body.contains("\"cancelled\":true"));
    assert!(cancelled.body.contains("\"claim_status\":\"hypothetical\""));
    assert!(cancelled.body.contains("\"scientific_authority\":false"));
    assert!(!cancelled.body.contains("rmse"));
    assert!(!cancelled.body.contains("evidence_span_ids"));
    assert!(!cancelled.body.contains("tepp.scientific_acceptance.v1"));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                "/v1/interpretation-runs/orch-live-idem-001",
                &collection_headers(),
                "",
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                "/v1/interpretation-runs/orch-live-idem-001/cancel",
                &collection_headers(),
                "",
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                "/v1/interpretation-runs/orch-live-idem-001/cancel",
                &collection_headers(),
                "{}",
            ))
            .status_code,
        400
    );
}

#[test]
fn handle_http_collection_get_refuses_foreign_consumers_and_hostile_headers() {
    let mut service = OrchestratorLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                "/v1/interpretation-runs/extra",
                &collection_headers(),
                "",
            ))
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                INTERPRETATION_RUN_PATH,
                &collection_headers(),
                "{}",
            ))
            .status_code,
        400
    );
    let mut with_idem = collection_headers();
    with_idem.push(("idempotency-key".into(), "orch-live-idem-001".into()));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                INTERPRETATION_RUN_PATH,
                &with_idem,
                "",
            ))
            .status_code,
        400
    );
    for consumer in ["naruon", "lineageweave"] {
        let mut foreign = collection_headers();
        foreign.retain(|(name, _)| !name.eq_ignore_ascii_case("tepp-consumer"));
        foreign.push(("tepp-consumer".into(), consumer.into()));
        assert_eq!(
            service
                .handle_http_request(&http_request("GET", INTERPRETATION_RUN_PATH, &foreign, "",))
                .status_code,
            400,
            "consumer={consumer}"
        );
    }
    let mut slash_cursor = collection_headers();
    slash_cursor.push(("tepp-page-cursor".into(), "idem/slash".into()));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "GET",
                INTERPRETATION_RUN_PATH,
                &slash_cursor,
                "",
            ))
            .status_code,
        400
    );
    let mut credential = collection_headers();
    credential.push(("Authorization".into(), "Bearer review-agent".into()));
    let denied = service.handle_http_request(&http_request(
        "GET",
        INTERPRETATION_RUN_PATH,
        &credential,
        "",
    ));
    assert_eq!(denied.status_code, 403);
    assert_eq!(error_code(&denied.body), "authorization_denied");
}

#[test]
fn handle_http_refuses_credential_headers_and_reserved_overrides() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let body = request.to_json().expect("json");
    for (name, value, status, code) in [
        (
            "Authorization",
            "Bearer review-agent",
            403,
            "authorization_denied",
        ),
        ("cookie", "a=b", 403, "authorization_denied"),
        ("x-api-key", "k", 403, "authorization_denied"),
        ("x-github-token", "t", 403, "authorization_denied"),
        ("x-copilot-session", "s", 403, "authorization_denied"),
        (
            "x-nim-key",
            "NVIDIA_NIM_API_KEY",
            403,
            "authorization_denied",
        ),
        ("content-type", "text/plain", 400, "invalid_wire_payload"),
        ("tepp-consumer", "naruon", 400, "invalid_wire_payload"),
        ("tepp-contract-version", "0", 400, "invalid_wire_payload"),
        ("idempotency-key", "", 400, "invalid_wire_payload"),
    ] {
        let mut headers = orchestrator_headers(request.idempotency_key());
        if name.eq_ignore_ascii_case("content-type")
            || name.eq_ignore_ascii_case("tepp-consumer")
            || name.eq_ignore_ascii_case("tepp-contract-version")
            || name.eq_ignore_ascii_case("idempotency-key")
        {
            headers.retain(|(existing, _)| !existing.eq_ignore_ascii_case(name));
        }
        headers.push((name.into(), value.into()));
        let response = service.handle_http_request(&http_request(
            "POST",
            INTERPRETATION_RUN_PATH,
            &headers,
            &body,
        ));
        assert_eq!(response.status_code, status, "header={name}");
        assert_eq!(error_code(&response.body), code, "header={name}");
        assert!(!response.body.contains("Bearer"));
        assert!(!response.body.contains("ghs_"));
        assert!(!response.body.contains("NVIDIA_NIM_API_KEY"));
    }
}

#[test]
fn handle_http_maps_wire_version_and_limit_errors() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let unsupported = r#"{"contract_version":9,"idempotency_key":"orch-live-idem-001","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","orchestration_mode":"direct","compute_budget_tokens":8,"evidence_span_ids":["span-001"],"scientific_authority":false}"#;
    let version = service.handle_http_request(&http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &orchestrator_headers(request.idempotency_key()),
        unsupported,
    ));
    assert_eq!(version.status_code, 422);
    assert_eq!(error_code(&version.body), "unsupported_contract_version");

    let oversized = "x".repeat(DEFAULT_INTERPRETATION_BYTE_LIMIT + 1);
    let limited = service.handle_http_request(&http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &orchestrator_headers(request.idempotency_key()),
        &oversized,
    ));
    assert_eq!(limited.status_code, 413);
    assert_eq!(error_code(&limited.body), "limit_exceeded");
}

#[test]
fn handle_http_refuses_malformed_framing_and_header_limits() {
    let mut service = OrchestratorLiveService::new();
    assert_eq!(service.handle_http_request("").status_code, 400);
    assert_eq!(
        service
            .handle_http_request("POST /v1/interpretation-runs HTTP/1.1\n\n")
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request("POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .status_code,
        400
    );

    let mut too_many = orchestrator_headers("idem-many");
    for index in 0..=LIVE_HEADER_COUNT_LIMIT {
        too_many.push((format!("x-extra-{index}"), "1".into()));
    }
    let crowded = http_request("POST", INTERPRETATION_RUN_PATH, &too_many, "{}");
    assert_eq!(service.handle_http_request(&crowded).status_code, 413);

    let huge_name = "x".repeat(LIVE_HEADER_BYTE_LIMIT + 8);
    let huge = format!("POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\n{huge_name}: 1\r\n\r\n");
    assert_eq!(service.handle_http_request(&huge).status_code, 413);

    let mismatch = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 4\r\n\r\nab"
    );
    assert_eq!(service.handle_http_request(&mismatch).status_code, 400);

    let header_idem_mismatch = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &orchestrator_headers("other-idem"),
        &sample_request().to_json().expect("json"),
    );
    assert_eq!(
        service
            .handle_http_request(&header_idem_mismatch)
            .status_code,
        400
    );

    let missing_host = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &[
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "contextual-orchestrator".into()),
            ("tepp-contract-version".into(), "1".into()),
            ("idempotency-key".into(), "k".into()),
        ],
        "{}",
    );
    assert_eq!(service.handle_http_request(&missing_host).status_code, 400);

    let mut duplicate_host = orchestrator_headers("dup");
    duplicate_host.push(("Host".into(), "127.0.0.1".into()));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                INTERPRETATION_RUN_PATH,
                &duplicate_host,
                "{}"
            ))
            .status_code,
        400
    );
}

#[test]
fn public_contract_covers_library_parser_short_circuits() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let body = request.to_json().expect("json");
    let headers = orchestrator_headers(request.idempotency_key());

    assert_eq!(
        service
            .handle_http_request(&http_request("POST", "relative", &headers, &body))
            .status_code,
        400
    );
    let zero_budget = body.replace(
        "\"compute_budget_tokens\":2048",
        "\"compute_budget_tokens\":0",
    );
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                INTERPRETATION_RUN_PATH,
                &headers,
                &zero_budget,
            ))
            .status_code,
        400
    );

    let no_length = format!("POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert_eq!(service.handle_http_request(&no_length).status_code, 400);
    let empty_length = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length:\r\n\r\n"
    );
    assert_eq!(service.handle_http_request(&empty_length).status_code, 400);
    let duplicate_length = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: 0\r\ncontent-length: 0\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&duplicate_length).status_code,
        400
    );
    let invalid_length = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: +1\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&invalid_length).status_code,
        400
    );

    let duplicate_header = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &[
            ("Host".into(), "127.0.0.1".into()),
            ("Host".into(), "127.0.0.1".into()),
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "contextual-orchestrator".into()),
            ("tepp-contract-version".into(), "1".into()),
            ("idempotency-key".into(), request.idempotency_key().into()),
        ],
        &body,
    );
    assert_eq!(
        service.handle_http_request(&duplicate_header).status_code,
        400
    );
    let control_header = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &[
            ("Host".into(), "127.0.0.1".into()),
            ("Bad\u{0001}Name".into(), "value".into()),
        ],
        &body,
    );
    assert_eq!(
        service.handle_http_request(&control_header).status_code,
        400
    );

    for credential_header in ["x-github", "x-copilot"] {
        let mut credential_headers = headers.clone();
        credential_headers.push((credential_header.into(), "value".into()));
        assert_eq!(
            service
                .handle_http_request(&http_request(
                    "POST",
                    INTERPRETATION_RUN_PATH,
                    &credential_headers,
                    &body,
                ))
                .status_code,
            403
        );
    }
}

#[test]
fn public_contract_covers_accepted_validation() {
    assert_eq!(
        InterpretationRunAccepted::from_json(
            r#"{"contract_version":1,"interpretation_run_id":"r","orchestration_mode":"direct","claim_status":"hypothetical","scientific_authority":true,"idempotency_key":"i"}"#
        )
        .expect_err("authority"),
        OrchestratorLiveError::ScientificAuthorityRefused
    );
    assert_eq!(
        InterpretationRunAccepted::from_json(
            r#"{"contract_version":1,"interpretation_run_id":"r","orchestration_mode":"direct","claim_status":"accepted","scientific_authority":false,"idempotency_key":"i"}"#
        )
        .expect_err("claim"),
        OrchestratorLiveError::ScientificAuthorityRefused
    );
    assert_eq!(
        InterpretationRunAccepted::new(" ", OrchestrationMode::Direct, "i").expect_err("id"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        InterpretationRunAccepted::from_json(&"x".repeat(DEFAULT_INTERPRETATION_BYTE_LIMIT + 1))
            .expect_err("limit"),
        OrchestratorLiveError::LimitExceeded
    );
}

#[test]
fn public_contract_covers_request_validation_edges() {
    let request = sample_request();
    let body = request.to_json().expect("json");
    let mut value: serde_json::Value = serde_json::from_str(&body).expect("value");

    value["evidence_span_ids"] = serde_json::json!([]);
    assert_eq!(
        InterpretationRunRequest::from_json(&value.to_string()).expect_err("empty spans"),
        OrchestratorLiveError::InvalidWirePayload
    );
    value["evidence_span_ids"] = serde_json::json!(vec!["span"; 33]);
    assert_eq!(
        InterpretationRunRequest::from_json(&value.to_string()).expect_err("many spans"),
        OrchestratorLiveError::InvalidWirePayload
    );

    for snapshot_id in [
        "postgres://db",
        "jdbc://db",
        "127.0.0.1/sql",
        "127.0.0.1/tables/document_record",
        "bad'host",
        "bad;host",
        "bad\\host",
        "bad host",
        "bad\\u0001host",
    ] {
        value["evidence_span_ids"] = serde_json::json!(["span"]);
        value["snapshot_id"] = serde_json::Value::String(snapshot_id.into());
        assert_eq!(
            InterpretationRunRequest::from_json(&value.to_string()).expect_err("table label"),
            OrchestratorLiveError::InvalidWirePayload,
            "snapshot_id={snapshot_id}"
        );
    }
}

#[test]
fn public_contract_covers_http_path_and_header_edges() {
    let mut service = OrchestratorLiveService::new();
    let request = sample_request();
    let body = request.to_json().expect("json");
    let headers = orchestrator_headers(request.idempotency_key());

    for path in [
        "/v1/interpretation-runs#fragment",
        "https://tepp.example/v1/interpretation-runs",
        "/v1/interpretation-runs://reserved",
    ] {
        assert_eq!(
            service
                .handle_http_request(&http_request("POST", path, &headers, &body))
                .status_code,
            400,
            "path={path}"
        );
    }
    let extra_request_line = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1 extra\r\nHost: 127.0.0.1\r\ncontent-length: 0\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&extra_request_line).status_code,
        400
    );
    let missing_path = "POST\r\nHost: 127.0.0.1\r\ncontent-length: 0\r\n\r\n";
    assert_eq!(service.handle_http_request(missing_path).status_code, 400);
    let no_delimiter_limit = "x".repeat(LIVE_HEADER_BYTE_LIMIT);
    assert_eq!(
        service.handle_http_request(&no_delimiter_limit).status_code,
        413
    );
    for header_line in ["malformed", ": empty-name"] {
        let malformed = format!(
            "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\n{header_line}\r\ncontent-length: 0\r\n\r\n"
        );
        assert_eq!(service.handle_http_request(&malformed).status_code, 400);
    }
    let whitespace_name = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nBad Name: value\r\ncontent-length: 0\r\n\r\n"
    );
    assert_eq!(
        service.handle_http_request(&whitespace_name).status_code,
        400
    );
    for credential_header in ["cookie", "x-api-key", "x-nvidia_nim_api_key"] {
        let mut credential_headers = headers.clone();
        credential_headers.push((credential_header.into(), "value".into()));
        assert_eq!(
            service
                .handle_http_request(&http_request(
                    "POST",
                    INTERPRETATION_RUN_PATH,
                    &credential_headers,
                    &body,
                ))
                .status_code,
            403,
            "header={credential_header}"
        );
    }
}

#[test]
fn serve_one_covers_stream_framing_limits() {
    let request = sample_request();
    let zero_body = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &orchestrator_headers(request.idempotency_key()),
        "",
    );
    assert_eq!(serve_once(zero_body.as_bytes()).status_code, 400);

    let oversized_header = vec![b'x'; LIVE_HEADER_BYTE_LIMIT + 1];
    assert_eq!(serve_once(&oversized_header).status_code, 413);

    let huge_length = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: {}\r\n\r\n",
        DEFAULT_INTERPRETATION_BYTE_LIMIT + 1
    );
    assert_eq!(serve_once(huge_length.as_bytes()).status_code, 413);
}

#[test]
fn read_http_request_covers_transport_and_limit_errors() {
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(Vec::<u8>::new()))
            .expect_err("eof"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut TimeoutRead).expect_err("timeout"),
        OrchestratorLiveError::LimitExceeded
    );
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut OtherRead).expect_err("other"),
        OrchestratorLiveError::InvalidWirePayload
    );
    let oversized = vec![b'x'; LIVE_HEADER_BYTE_LIMIT + 1];
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(oversized)).expect_err("limit"),
        OrchestratorLiveError::LimitExceeded
    );
    let oversized_slice = vec![b'x'; LIVE_HEADER_BYTE_LIMIT + 1];
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(oversized_slice.as_slice()))
            .expect_err("slice limit"),
        OrchestratorLiveError::LimitExceeded
    );

    let request = interpretation_http(&sample_request());
    let parsed = OrchestratorLiveService::read_http_request(&mut Cursor::new(request.as_bytes()))
        .expect("read");
    assert_eq!(parsed, request);

    let response = OrchestratorLiveService::new().handle_http_request(&request);
    assert_eq!(
        OrchestratorLiveService::write_response(&mut FailingWriter, &response).expect_err("write"),
        OrchestratorLiveError::InvalidWirePayload
    );
    assert_eq!(
        OrchestratorLiveService::write_response(&mut FlushFailWriter, &response)
            .expect_err("flush"),
        OrchestratorLiveError::InvalidWirePayload
    );
    let mut ok = Cursor::new(Vec::<u8>::new());
    OrchestratorLiveService::write_response(&mut ok, &response).expect("ok write");
    assert!(ok.into_inner().starts_with(b"HTTP/1.1 202"));

    let mut invalid_utf8 = b"POST /v1/interpretation-runs HTTP/1.1\r\n".to_vec();
    invalid_utf8.push(0xff);
    invalid_utf8.extend_from_slice(b"\r\n\r\n");
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(invalid_utf8))
            .expect_err("utf8"),
        OrchestratorLiveError::InvalidWirePayload
    );

    let huge_len = format!(
        "POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: {}\r\n\r\n",
        DEFAULT_INTERPRETATION_BYTE_LIMIT + 1
    );
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(huge_len.into_bytes()))
            .expect_err("declared limit"),
        OrchestratorLiveError::LimitExceeded
    );

    let mut invalid_body = b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 1\r\n\r\n".to_vec();
    invalid_body.push(0xff);
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(invalid_body))
            .expect_err("body utf8"),
        OrchestratorLiveError::InvalidWirePayload
    );

    let zero = b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 0\r\n\r\n";
    assert!(OrchestratorLiveService::read_http_request(&mut Cursor::new(zero.as_slice())).is_ok());
    assert!(OrchestratorLiveService::read_http_request(&mut Cursor::new(zero.to_vec())).is_ok());

    let truncated = b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 4\r\n\r\nab";
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(truncated.as_slice()))
            .expect_err("short body"),
        OrchestratorLiveError::InvalidWirePayload
    );
}

#[test]
fn read_http_request_covers_slice_reader_edges() {
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new([].as_slice()))
            .expect_err("empty slice"),
        OrchestratorLiveError::InvalidWirePayload
    );
    let oversized = vec![b'x'; LIVE_HEADER_BYTE_LIMIT + 1];
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(oversized.as_slice()))
            .expect_err("oversized slice"),
        OrchestratorLiveError::LimitExceeded
    );
    let huge_length = format!(
        "POST {INTERPRETATION_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: {}\r\n\r\n",
        DEFAULT_INTERPRETATION_BYTE_LIMIT + 1
    );
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(huge_length.as_bytes()))
            .expect_err("huge length slice"),
        OrchestratorLiveError::LimitExceeded
    );
    let mut invalid_header = b"POST /v1/interpretation-runs HTTP/1.1\r\n".to_vec();
    invalid_header.push(0xff);
    invalid_header.extend_from_slice(b"\r\n\r\n");
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(invalid_header.as_slice()))
            .expect_err("header utf8 slice"),
        OrchestratorLiveError::InvalidWirePayload
    );
    let invalid_body = b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: 1\r\n\r\n\xff";
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(invalid_body.as_slice()))
            .expect_err("body utf8 slice"),
        OrchestratorLiveError::InvalidWirePayload
    );
    let truncated =
        b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: 2\r\n\r\na";
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(truncated.as_slice()))
            .expect_err("truncated slice"),
        OrchestratorLiveError::InvalidWirePayload
    );
}

#[test]
fn serve_one_accepts_interpretation_run_over_loopback_tcp() {
    let request = sample_request();
    let mut service = OrchestratorLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let worker = thread::spawn(move || service.serve_one());

    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("rt");
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("wt");
    let mut headers = orchestrator_headers(request.idempotency_key());
    headers[0] = ("Host".into(), format!("{addr}"));
    let payload = http_request(
        "POST",
        INTERPRETATION_RUN_PATH,
        &headers,
        &request.to_json().expect("json"),
    );
    stream.write_all(payload.as_bytes()).expect("write");
    let mut received = String::new();
    stream.read_to_string(&mut received).expect("read");
    assert!(received.starts_with("HTTP/1.1 202 Accepted"));
    assert!(received.contains("\"claim_status\":\"hypothetical\""));
    assert!(received.contains("\"scientific_authority\":false"));
    let served = worker.join().expect("join").expect("serve");
    assert_eq!(served.status_code, 202);

    let mut idle_listener = OrchestratorLiveService::bind_loopback().expect("bind2");
    let idle_addr = idle_listener.local_addr().expect("addr2");
    let idle_worker = thread::spawn(move || idle_listener.serve_one());
    drop(TcpStream::connect(idle_addr).expect("connect2"));
    let idle_response = idle_worker.join().expect("join2").expect("served closed");
    assert_eq!(idle_response.status_code, 400);
}

struct TimeoutRead;

impl Read for TimeoutRead {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))
    }
}

struct OtherRead;

impl Read for OtherRead {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("broken"))
    }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("write failed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct FlushFailWriter;

impl Write for FlushFailWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("flush failed"))
    }
}
