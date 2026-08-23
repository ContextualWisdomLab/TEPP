//! Live loopback HTTP/1.1 naruon POSTs stay versioned and fail closed (ADR 0011).

use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunRequest, AnalyticalPurpose,
    ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT, ErrorEnvelope, ExportAuthorizationRequest,
    NARUON_ANALYSIS_RUN_PATH, NARUON_EXPORT_PATH, NARUON_LIVE_HEADER_BYTE_LIMIT,
    NARUON_LIVE_HEADER_COUNT_LIMIT, NARUON_LIVE_IO_TIMEOUT, NaruonLiveService,
    naruon_analysis_run_exchange, naruon_export_exchange,
};

fn sample_run() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "naruon-live-idem-001".into(),
        tenant_workspace_id: "naruon-tenant-workspace-demo".into(),
        snapshot_id: "tepp-snapshot-demo-001".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "topic-measurement-v1".into(),
        output_profile: "naruon-consumer-validation-report".into(),
    }
}

fn sample_export() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "naruon-tenant-workspace-demo".into(),
        principal_id: "naruon-service".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "tepp-export-demo-001".into(),
        includes_source_text: false,
    }
}

fn naruon_headers(idempotency_key: &str) -> Vec<(String, String)> {
    vec![
        ("Host".into(), "127.0.0.1".into()),
        ("content-type".into(), "application/json".into()),
        ("tepp-consumer".into(), "naruon".into()),
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

fn analysis_http(run: &AnalysisRunRequest) -> String {
    http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &naruon_headers(&run.idempotency_key),
        &run.to_json().expect("run json"),
    )
}

fn export_http(request: &ExportAuthorizationRequest, idempotency_key: &str) -> String {
    http_request(
        "POST",
        NARUON_EXPORT_PATH,
        &naruon_headers(idempotency_key),
        &serde_json::to_string(request).expect("export json"),
    )
}

fn envelope(body: &str) -> ErrorEnvelope {
    serde_json::from_str(body).expect("error envelope")
}

#[test]
fn loopback_bind_refuses_non_loopback_and_in_use_ports() {
    assert_eq!(
        NaruonLiveService::bind("0.0.0.0:0".parse::<SocketAddr>().expect("unspec"))
            .expect_err("denied"),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        NaruonLiveService::bind("8.8.8.8:0".parse::<SocketAddr>().expect("public"))
            .expect_err("denied"),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        NaruonLiveService::bind("[::]:0".parse::<SocketAddr>().expect("v6-unspec"))
            .expect_err("denied"),
        ApiError::AuthorizationDenied
    );
    let first = NaruonLiveService::bind_loopback().expect("first bind");
    let addr = first.local_addr().expect("addr");
    assert!(addr.ip().is_loopback());
    assert_eq!(
        NaruonLiveService::bind(addr).expect_err("in use"),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        NaruonLiveService::new().local_addr().expect_err("no sock"),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        NaruonLiveService::new().serve_one().expect_err("no sock"),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        NaruonLiveService::default()
            .serve_one()
            .expect_err("default"),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn handle_http_accepts_analysis_run_and_replays_idempotent_retries() {
    let mut service = NaruonLiveService::new();
    let run = sample_run();
    let first = service.handle_http_request(&analysis_http(&run));
    assert_eq!(first.status_code, 202);
    assert_eq!(first.reason_phrase, "Accepted");
    let accepted = AnalysisRunAccepted::from_json(&first.body).expect("accepted");
    assert_eq!(accepted.idempotency_key, run.idempotency_key);
    assert_eq!(accepted.run_state, "accepted");
    assert!(!accepted.run_id.is_empty());

    let replay = service.handle_http_request(&analysis_http(&run));
    assert_eq!(replay.status_code, 202);
    assert_eq!(replay.body, first.body);

    let mut conflicting = run.clone();
    conflicting.snapshot_id = "other-snapshot".into();
    let conflict = service.handle_http_request(&analysis_http(&conflicting));
    assert_eq!(conflict.status_code, 400);
    assert_eq!(
        envelope(&conflict.body).error_code(),
        "invalid_wire_payload"
    );
}

#[test]
fn handle_http_keys_idempotency_replay_by_tenant_and_key() {
    let mut service = NaruonLiveService::new();
    let first = sample_run();
    let mut second = first.clone();
    second.tenant_workspace_id = "naruon-tenant-workspace-other".into();
    let a = service.handle_http_request(&analysis_http(&first));
    let b = service.handle_http_request(&analysis_http(&second));
    assert_eq!(a.status_code, 202);
    assert_eq!(b.status_code, 202);
    let accepted_a = AnalysisRunAccepted::from_json(&a.body).expect("a");
    let accepted_b = AnalysisRunAccepted::from_json(&b.body).expect("b");
    assert_ne!(accepted_a.run_id, accepted_b.run_id);
}

#[test]
fn handle_http_authorizes_modular_export_and_refuses_other_purposes() {
    let mut service = NaruonLiveService::new();
    let allowed = sample_export();
    let ok = service.handle_http_request(&export_http(&allowed, "export-op-a"));
    assert_eq!(ok.status_code, 200);
    assert_eq!(ok.reason_phrase, "OK");
    assert!(ok.body.contains("purpose_bound_export_allowed"));
    assert!(!ok.body.contains("token"));

    let denied = ExportAuthorizationRequest {
        purpose: AnalyticalPurpose::OperationalMonitoring,
        ..allowed.clone()
    };
    let forbidden = service.handle_http_request(&export_http(&denied, "export-op-b"));
    assert_eq!(forbidden.status_code, 403);
    assert_eq!(
        envelope(&forbidden.body).error_code(),
        "authorization_denied"
    );

    let same_as_principal =
        service.handle_http_request(&export_http(&allowed, allowed.principal_id.as_str()));
    assert_eq!(same_as_principal.status_code, 400);
    assert_eq!(
        envelope(&same_as_principal.body).error_code(),
        "invalid_wire_payload"
    );
}

#[test]
fn handle_http_refuses_methods_paths_versions_and_table_hosts() {
    let mut service = NaruonLiveService::new();
    let run = sample_run();
    let body = run.to_json().expect("json");
    let headers = naruon_headers(&run.idempotency_key);

    let get = service.handle_http_request(&http_request(
        "GET",
        NARUON_ANALYSIS_RUN_PATH,
        &headers,
        &body,
    ));
    assert_eq!(get.status_code, 400);

    let unknown = service.handle_http_request(&http_request(
        "POST",
        "/v1/tables/document_record",
        &headers,
        &body,
    ));
    assert_eq!(unknown.status_code, 400);

    let sql = service.handle_http_request(&http_request("POST", "/sql", &headers, &body));
    assert_eq!(sql.status_code, 400);

    let query = service.handle_http_request(&http_request(
        "POST",
        "/v1/analysis-runs?drop=1",
        &headers,
        &body,
    ));
    assert_eq!(query.status_code, 400);

    assert_eq!(
        service
            .handle_http_request("POST /v1/analysis-runs HTTP/1.1")
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(&"x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT))
            .status_code,
        413
    );
    assert_eq!(
        service
            .handle_http_request(
                "POST /v1/analysis-runs HTTP/1.1 extra\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "POST /v1/analysis-runs#drop HTTP/1.1\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request("POST /proxy://target HTTP/1.1\r\ncontent-length: 0\r\n\r\n")
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request(
                "POST /v1/analysis-runs HTTP/1.1\r\ncontent-length: 0\r\ncontent-length: 0\r\n\r\n"
            )
            .status_code,
        400
    );

    let http10 = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.0\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    );
    assert_eq!(service.handle_http_request(&http10).status_code, 400);

    let mut postgres_host = headers.clone();
    postgres_host[0] = ("Host".into(), "postgres.example.test".into());
    let table_host = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &postgres_host,
        &body,
    ));
    assert_eq!(table_host.status_code, 400);

    let mut jdbc_host = headers;
    jdbc_host[0] = ("Host".into(), "jdbc.example.test".into());
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                NARUON_ANALYSIS_RUN_PATH,
                &jdbc_host,
                &body
            ))
            .status_code,
        400
    );
}

#[test]
fn handle_http_requires_loopback_host_and_refuses_transfer_encoding() {
    let mut service = NaruonLiveService::new();
    let run = sample_run();
    let body = run.to_json().expect("json");

    let mut localhost_headers = naruon_headers(&run.idempotency_key);
    localhost_headers[0] = ("Host".into(), "localhost".into());
    let localhost = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &localhost_headers,
        &body,
    ));
    assert_eq!(localhost.status_code, 202);

    for host in ["attacker.example.com", "mysql.internal", "8.8.8.8"] {
        let mut headers = naruon_headers(&run.idempotency_key);
        headers[0] = ("Host".into(), host.into());
        let response = service.handle_http_request(&http_request(
            "POST",
            NARUON_ANALYSIS_RUN_PATH,
            &headers,
            &body,
        ));
        assert_eq!(response.status_code, 403, "host={host}");
        assert_eq!(
            envelope(&response.body).error_code(),
            "authorization_denied"
        );
    }

    let mut chunked = naruon_headers(&run.idempotency_key);
    chunked.push(("Transfer-Encoding".into(), "chunked".into()));
    let transfer = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &chunked,
        &body,
    ));
    assert_eq!(transfer.status_code, 400);
    assert_eq!(
        envelope(&transfer.body).error_code(),
        "invalid_wire_payload"
    );
}

#[test]
fn handle_http_refuses_credential_headers_and_reserved_overrides() {
    let mut service = NaruonLiveService::new();
    let run = sample_run();
    let body = run.to_json().expect("json");
    for (name, value, status, code) in [
        (
            "Authorization",
            "Bearer review-agent",
            403,
            "authorization_denied",
        ),
        ("cookie", "a=b", 403, "authorization_denied"),
        ("x-api-key", "k", 403, "authorization_denied"),
        ("x-apikey", "k", 403, "authorization_denied"),
        ("x-api_key", "k", 403, "authorization_denied"),
        ("X-ApiKey", "k", 403, "authorization_denied"),
        ("x-github-token", "t", 403, "authorization_denied"),
        ("x-copilot-session", "s", 403, "authorization_denied"),
        (
            "Proxy-Authorization",
            "Basic review-agent",
            403,
            "authorization_denied",
        ),
        (
            "x-nvidia-nim-key",
            "nvapi-example",
            403,
            "authorization_denied",
        ),
        ("content-type", "text/plain", 400, "invalid_wire_payload"),
        ("tepp-consumer", "hostile", 400, "invalid_wire_payload"),
        ("tepp-contract-version", "0", 400, "invalid_wire_payload"),
        ("idempotency-key", "", 400, "invalid_wire_payload"),
    ] {
        let mut headers = naruon_headers(&run.idempotency_key);
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
            NARUON_ANALYSIS_RUN_PATH,
            &headers,
            &body,
        ));
        assert_eq!(response.status_code, status, "header={name}");
        assert_eq!(envelope(&response.body).error_code(), code, "header={name}");
        assert!(!response.body.contains("Bearer"));
        assert!(!response.body.contains("ghs_"));
        assert!(!response.body.contains("nvapi-"));
    }
}

#[test]
fn handle_http_maps_wire_version_and_limit_errors() {
    let mut service = NaruonLiveService::new();
    let run = sample_run();
    let unsupported = r#"{"contract_version":9,"idempotency_key":"naruon-live-idem-001","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"2026-08-01T00:00:00Z","model_contract_version":"m","output_profile":"o"}"#;
    let version = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &naruon_headers(&run.idempotency_key),
        unsupported,
    ));
    assert_eq!(version.status_code, 422);
    assert_eq!(
        envelope(&version.body).error_code(),
        "unsupported_contract_version"
    );

    let oversized = "x".repeat(DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1);
    let limited = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &naruon_headers(&run.idempotency_key),
        &oversized,
    ));
    assert_eq!(limited.status_code, 413);
    assert_eq!(envelope(&limited.body).error_code(), "limit_exceeded");

    let not_rfc3339 = r#"{"contract_version":1,"idempotency_key":"naruon-live-idem-001","tenant_workspace_id":"t","snapshot_id":"s","knowledge_cutoff":"k","model_contract_version":"m","output_profile":"o"}"#;
    let cutoff = service.handle_http_request(&http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &naruon_headers(&run.idempotency_key),
        not_rfc3339,
    ));
    assert_eq!(cutoff.status_code, 400);
}

#[test]
fn handle_http_refuses_malformed_framing_and_header_limits() {
    let mut service = NaruonLiveService::new();
    assert_eq!(service.handle_http_request("").status_code, 400);
    assert_eq!(
        service
            .handle_http_request("POST /v1/analysis-runs HTTP/1.1\n\n")
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request("POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .status_code,
        400
    );
    assert_eq!(
        service
            .handle_http_request("NOT-A-REQUEST-LINE\r\n\r\n")
            .status_code,
        400
    );

    let mut too_many = naruon_headers("idem-many");
    for index in 0..=NARUON_LIVE_HEADER_COUNT_LIMIT {
        too_many.push((format!("x-extra-{index}"), "1".into()));
    }
    let crowded = http_request("POST", NARUON_ANALYSIS_RUN_PATH, &too_many, "{}");
    assert_eq!(service.handle_http_request(&crowded).status_code, 413);

    let huge_name = "x".repeat(NARUON_LIVE_HEADER_BYTE_LIMIT + 8);
    let huge = format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\n{huge_name}: 1\r\n\r\n");
    assert_eq!(service.handle_http_request(&huge).status_code, 413);

    let mismatch = format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 4\r\n\r\nab"
    );
    assert_eq!(service.handle_http_request(&mismatch).status_code, 400);

    let lf_header = format!("POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost 127.0.0.1\r\n\r\n");
    assert_eq!(service.handle_http_request(&lf_header).status_code, 400);

    let missing_host = http_request(
        "POST",
        NARUON_ANALYSIS_RUN_PATH,
        &[
            ("content-type".into(), "application/json".into()),
            ("tepp-consumer".into(), "naruon".into()),
            ("tepp-contract-version".into(), "1".into()),
            ("idempotency-key".into(), "k".into()),
        ],
        "{}",
    );
    assert_eq!(service.handle_http_request(&missing_host).status_code, 400);

    let header_idem_mismatch = {
        let run = sample_run();
        http_request(
            "POST",
            NARUON_ANALYSIS_RUN_PATH,
            &naruon_headers("other-idem"),
            &run.to_json().expect("json"),
        )
    };
    assert_eq!(
        service
            .handle_http_request(&header_idem_mismatch)
            .status_code,
        400
    );

    let mut duplicate_host = naruon_headers("dup");
    duplicate_host.push(("Host".into(), "127.0.0.1".into()));
    assert_eq!(
        service
            .handle_http_request(&http_request(
                "POST",
                NARUON_ANALYSIS_RUN_PATH,
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
        NaruonLiveService::read_http_request(&mut Cursor::new(Vec::<u8>::new())).expect_err("eof"),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        NaruonLiveService::read_http_request(&mut TimeoutRead).expect_err("timeout"),
        ApiError::LimitExceeded
    );
    assert_eq!(
        NaruonLiveService::read_http_request(&mut OtherRead).expect_err("other"),
        ApiError::InvalidWirePayload
    );
    let oversized = vec![b'x'; NARUON_LIVE_HEADER_BYTE_LIMIT + 1];
    assert_eq!(
        NaruonLiveService::read_http_request(&mut Cursor::new(oversized)).expect_err("limit"),
        ApiError::LimitExceeded
    );

    let run = sample_run();
    let request = analysis_http(&run);
    let parsed =
        NaruonLiveService::read_http_request(&mut Cursor::new(request.as_bytes())).expect("read");
    assert_eq!(parsed, request);

    let mut failing = Cursor::new(Vec::<u8>::new());
    let response = NaruonLiveService::new().handle_http_request(&request);
    assert_eq!(
        NaruonLiveService::write_response(&mut FailingWriter, &response).expect_err("write"),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        NaruonLiveService::write_response(&mut FlushFailWriter, &response).expect_err("flush"),
        ApiError::InvalidWirePayload
    );
    NaruonLiveService::write_response(&mut failing, &response).expect("ok write");
    assert!(failing.into_inner().starts_with(b"HTTP/1.1 202"));

    let mut invalid_utf8 = b"POST /v1/analysis-runs HTTP/1.1\r\n".to_vec();
    invalid_utf8.push(0xff);
    invalid_utf8.extend_from_slice(b"\r\n\r\n");
    assert_eq!(
        NaruonLiveService::read_http_request(&mut Cursor::new(invalid_utf8)).expect_err("utf8"),
        ApiError::InvalidWirePayload
    );

    let mut invalid_body = b"POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 1\r\n\r\n".to_vec();
    invalid_body.push(0xff);
    assert_eq!(
        NaruonLiveService::read_http_request(&mut Cursor::new(invalid_body))
            .expect_err("body utf8"),
        ApiError::InvalidWirePayload
    );

    let zero = b"POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 0\r\n\r\n";
    assert!(NaruonLiveService::read_http_request(&mut Cursor::new(zero.as_slice())).is_ok());
    assert!(NaruonLiveService::read_http_request(&mut Cursor::new(zero.to_vec())).is_ok());

    let truncated = b"POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: naruon\r\ntepp-contract-version: 1\r\nidempotency-key: k\r\ncontent-length: 4\r\n\r\nab";
    assert_eq!(
        NaruonLiveService::read_http_request(&mut Cursor::new(truncated.as_slice()))
            .expect_err("short body"),
        ApiError::InvalidWirePayload
    );

    let huge_len = format!(
        "POST /v1/analysis-runs HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-length: {}\r\n\r\n",
        DEFAULT_ANALYSIS_RUN_BYTE_LIMIT + 1
    );
    assert_eq!(
        NaruonLiveService::read_http_request(&mut Cursor::new(huge_len.into_bytes()))
            .expect_err("declared limit"),
        ApiError::LimitExceeded
    );
}

#[test]
fn serve_one_accepts_committed_naruon_exchange_over_loopback_tcp() {
    let run = sample_run();
    let exchange = naruon_analysis_run_exchange("https://tepp.example.test", &run).expect("ex");
    let mut service = NaruonLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let worker = thread::spawn(move || service.serve_one());

    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("rt");
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("wt");
    let mut headers = naruon_headers(&run.idempotency_key);
    headers[0] = ("Host".into(), format!("{addr}"));
    for extra in &exchange.headers {
        if extra.0 == "content-type"
            || extra.0 == "tepp-consumer"
            || extra.0 == "tepp-contract-version"
            || extra.0 == "idempotency-key"
        {
            continue;
        }
        headers.push(extra.clone());
    }
    let payload = http_request("POST", NARUON_ANALYSIS_RUN_PATH, &headers, &exchange.body);
    stream.write_all(payload.as_bytes()).expect("write");
    let mut received = String::new();
    stream.read_to_string(&mut received).expect("read");
    assert!(received.starts_with("HTTP/1.1 202 Accepted"));
    assert!(received.contains("\"run_state\":\"accepted\""));
    let served = worker.join().expect("join").expect("serve");
    assert_eq!(served.status_code, 202);

    let mut idle_listener = NaruonLiveService::bind_loopback().expect("bind2");
    let idle_addr = idle_listener.local_addr().expect("addr2");
    let idle_worker = thread::spawn(move || idle_listener.serve_one());
    drop(TcpStream::connect(idle_addr).expect("connect2"));
    let idle_response = idle_worker.join().expect("join2").expect("served closed");
    assert_eq!(idle_response.status_code, 400);

    let mut empty_listener = NaruonLiveService::bind_loopback().expect("bind3");
    let empty_addr = empty_listener.local_addr().expect("addr3");
    let empty_worker = thread::spawn(move || empty_listener.serve_one());
    let mut empty_stream = TcpStream::connect(empty_addr).expect("connect3");
    empty_stream
        .write_all(
            http_request(
                "POST",
                NARUON_ANALYSIS_RUN_PATH,
                &naruon_headers("empty-body"),
                "",
            )
            .as_bytes(),
        )
        .expect("write3");
    let mut empty_received = String::new();
    empty_stream
        .read_to_string(&mut empty_received)
        .expect("read3");
    assert!(empty_received.starts_with("HTTP/1.1 400 Bad Request"));
    assert_eq!(
        empty_worker
            .join()
            .expect("join3")
            .expect("served empty")
            .status_code,
        400
    );
}

#[test]
fn serve_one_authorizes_export_over_loopback_tcp() {
    let request = sample_export();
    let exchange = naruon_export_exchange("https://tepp.example.test", &request, "export-tcp-001")
        .expect("ex");
    let mut service = NaruonLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let worker = thread::spawn(move || service.serve_one());

    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("rt");
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("wt");
    let mut headers = naruon_headers("export-tcp-001");
    headers[0] = ("Host".into(), format!("{addr}"));
    let payload = http_request("POST", NARUON_EXPORT_PATH, &headers, &exchange.body);
    stream.write_all(payload.as_bytes()).expect("write");
    let mut received = String::new();
    stream.read_to_string(&mut received).expect("read");
    assert!(received.starts_with("HTTP/1.1 200 OK"));
    assert!(received.contains("purpose_bound_export_allowed"));
    let served = worker.join().expect("join").expect("serve");
    assert_eq!(served.status_code, 200);
}

#[test]
fn serve_one_maps_partial_request_timeout_to_limit_exceeded() {
    let mut service = NaruonLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr");
    let worker = thread::spawn(move || service.serve_one());
    let stream = TcpStream::connect(addr).expect("connect");
    let started = Instant::now();
    let served = worker.join().expect("join").expect("timeout mapped");
    drop(stream);
    assert!(started.elapsed() >= NARUON_LIVE_IO_TIMEOUT);
    assert_eq!(served.status_code, 413);
    assert_eq!(envelope(&served.body).error_code(), "limit_exceeded");
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
