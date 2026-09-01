//! Contract tests for the naruon export idempotency-lookup loopback CLI.

use tepp_api::{
    compose_export_idempotency_lookup_cli_http, dispatch_export_idempotency_lookup_cli,
    execute_export_idempotency_lookup_cli, loopback_http1_from_export_idempotency_lookup_exchange,
    naruon_export_idempotency_lookup_exchange, read_export_idempotency_lookup_cli_stdin,
    render_export_idempotency_lookup_cli_stdout, AnalysisRunLiveService, AnalyticalPurpose,
    ApiError, ExportAuthorizationRequest, ExportIdempotencyLookup,
    ExportIdempotencyLookupCliInvocation, ExportIdempotencyLookupCliVerb, NaruonHttpExchange,
    NaruonLiveResponse, NaruonLiveService, EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn sample_request() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "export-lookup-cli-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-lookup-cli-1".into(),
        includes_source_text: false,
    }
}

fn export_post(request: &ExportAuthorizationRequest, idempotency_key: &str) -> String {
    let body = serde_json::to_string(request).expect("request json");
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn lookup_args<'a>(host: &'a str, key: &'a str, consumer: &'a str) -> [&'a str; 9] {
    [
        "lookup",
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--idempotency-key",
        key,
    ]
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        ExportIdempotencyLookupCliVerb::parse("lookup").expect("lookup"),
        ExportIdempotencyLookupCliVerb::Lookup
    );
    assert_eq!(ExportIdempotencyLookupCliVerb::Lookup.as_str(), "lookup");
    assert_eq!(
        ExportIdempotencyLookupCliVerb::parse("get"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("8.8.8.8:80", "idem-1", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--idempotency-key",
                "idem-1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--idempotency-key",
                "idem-1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn from_args_refuses_lineageweave_slash_body_size_and_pagination() {
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("127.0.0.1:18081", "idem-1", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("127.0.0.1:18081", "idem-1", "unpublished"),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("127.0.0.1:18081", "idem-1", NARUON_CONSUMER_CODE),
            "{}"
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("127.0.0.1:18081", "idem/slash", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args("127.0.0.1:18081", "by-idempotency", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            lookup_args(
                "127.0.0.1:18081",
                &"a".repeat(EXPORT_IDEMPOTENCY_LOOKUP_KEY_MAX_LEN + 1),
                NARUON_CONSUMER_CODE
            ),
            ""
        )
        .unwrap_err(),
        ApiError::LimitExceeded
    );
    assert_eq!(
        ExportIdempotencyLookupCliInvocation::from_args(
            [
                "lookup",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-1",
                "--page-limit",
                "1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_get_without_credentials() {
    let invocation = ExportIdempotencyLookupCliInvocation::from_args(
        lookup_args("127.0.0.1:18081", "idem-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_export_idempotency_lookup_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/exports/by-idempotency/idem-1 HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.to_ascii_lowercase().contains("idempotency-key:"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn naruon_cli_resolves_export_identity_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-lookup-cli-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let invocation = ExportIdempotencyLookupCliInvocation::from_args(
        lookup_args(
            "127.0.0.1:18081",
            "export-lookup-cli-1",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    let got = dispatch_export_idempotency_lookup_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_export_idempotency_lookup_cli_stdout(&invocation, &got).expect("out");
    let lookup = ExportIdempotencyLookup::from_json(&stdout).expect("lookup");
    assert_eq!(lookup.idempotency_key, "export-lookup-cli-1");
    assert_eq!(lookup.decision_code, "purpose_bound_export_allowed");
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    assert!(!stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("principal_id"));
    let missing = ExportIdempotencyLookupCliInvocation::from_args(
        lookup_args("127.0.0.1:18081", "missing-key", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied = dispatch_export_idempotency_lookup_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400);
    assert!(
        render_export_idempotency_lookup_cli_stdout(&missing, &denied)
            .expect("err")
            .contains("invalid_wire_payload")
    );
    let mut naruon = NaruonLiveService::new();
    assert_eq!(
        naruon
            .handle_http_request(
                &compose_export_idempotency_lookup_cli_http(&invocation).expect("composed")
            )
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_success() {
    let invocation = ExportIdempotencyLookupCliInvocation::from_args(
        lookup_args("127.0.0.1:18081", "idem-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_export_idempotency_lookup_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: String::new()
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_export_idempotency_lookup_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"export_id":"e","rmse":1.0}"#.into()
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_export_idempotency_lookup_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: format!(r#"{{"schema_version":"{SCHEMA}"}}"#)
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn loopback_http1_refuses_non_get_collection_get_by_id_request_and_credentials() {
    let host = "127.0.0.1:18081";
    let exchange = naruon_export_idempotency_lookup_exchange(ORIGIN, "idem-1").expect("ex");
    let ok = loopback_http1_from_export_idempotency_lookup_exchange(&exchange, host).expect("ok");
    assert!(ok.starts_with("GET /v1/exports/by-idempotency/idem-1 HTTP/1.1"));
    let mut posted = exchange.clone();
    posted.method = "POST";
    assert_eq!(
        loopback_http1_from_export_idempotency_lookup_exchange(&posted, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut by_id = exchange.clone();
    by_id.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1");
    assert_eq!(
        loopback_http1_from_export_idempotency_lookup_exchange(&by_id, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut request_path = exchange.clone();
    request_path.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1/request");
    assert_eq!(
        loopback_http1_from_export_idempotency_lookup_exchange(&request_path, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let credentialed = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{NARUON_EXPORT_PATH}/by-idempotency/idem-1"),
        headers: vec![("authorization".into(), "secret".into())],
        body: String::new(),
    };
    assert_eq!(
        loopback_http1_from_export_idempotency_lookup_exchange(&credentialed, host).unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn execute_over_tcp_and_stdin_reader() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-lookup-tcp"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = ExportIdempotencyLookupCliInvocation::from_args(
        lookup_args(addr.as_str(), "export-lookup-tcp", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("tcp");
    let response = execute_export_idempotency_lookup_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let lookup = ExportIdempotencyLookup::from_json(
        &render_export_idempotency_lookup_cli_stdout(&invocation, &response).expect("stdout"),
    )
    .expect("parsed");
    assert_eq!(lookup.idempotency_key, "export-lookup-tcp");
    handle.join().expect("join");
    assert!(
        read_export_idempotency_lookup_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty()
    );
    assert!(
        read_export_idempotency_lookup_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("pipe")
            .is_empty()
    );
}

#[test]
fn binary_reports_redacted_success_and_failure_statuses() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-lookup-bin"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let handle = std::thread::spawn(move || {
        service.serve_one().expect("success request");
        service.serve_one().expect("missing request");
    });
    let binary = env!("CARGO_BIN_EXE_tepp-export-lookup");
    let run = |key: &str| {
        std::process::Command::new(binary)
            .args(lookup_args(&addr, key, NARUON_CONSUMER_CODE))
            .output()
            .expect("binary")
    };
    let success = run("export-lookup-bin");
    assert!(
        success.status.success(),
        "{}",
        String::from_utf8_lossy(&success.stderr)
    );
    assert!(String::from_utf8_lossy(&success.stdout).contains("export-lookup-bin"));
    assert!(!String::from_utf8_lossy(&success.stdout).contains(SCHEMA));
    let failure = run("missing-key");
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("invalid API wire payload"));
    handle.join().expect("server");
}
