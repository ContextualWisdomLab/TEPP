//! Loopback interpretation POSTs stay hypothetical and fail closed (ADR 0010/0011).

use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use orchestrator_live::{
    DEFAULT_INTERPRETATION_BYTE_LIMIT, INTERPRETATION_RUN_CONTRACT_VERSION,
    INTERPRETATION_RUN_PATH, InterpretationRunAccepted, InterpretationRunRequest,
    LIVE_HEADER_BYTE_LIMIT, LIVE_HEADER_COUNT_LIMIT, OrchestrationMode, OrchestratorLiveError,
    OrchestratorLiveService,
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

    let truncated = b"POST /v1/interpretation-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: contextual-orchestrator\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 4\r\n\r\nab";
    assert_eq!(
        OrchestratorLiveService::read_http_request(&mut Cursor::new(truncated.as_slice()))
            .expect_err("short body"),
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
