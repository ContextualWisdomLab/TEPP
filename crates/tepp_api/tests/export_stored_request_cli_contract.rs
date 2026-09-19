//! Contract tests for the naruon export stored-request loopback CLI.

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ApiError, EXPORT_RETRIEVAL_ID_MAX_LEN,
    ExportAuthorizationRequest, ExportRetrieval, ExportStoredRequestCliInvocation,
    ExportStoredRequestCliVerb, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    NARUON_EXPORT_PATH, NaruonHttpExchange, NaruonLiveResponse, NaruonLiveService,
    compose_export_stored_request_cli_http, dispatch_export_stored_request_cli,
    execute_export_stored_request_cli, loopback_http1_from_export_stored_request_exchange,
    naruon_export_stored_request_exchange, read_export_stored_request_cli_stdin,
    render_export_stored_request_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn sample_request() -> ExportAuthorizationRequest {
    ExportAuthorizationRequest {
        tenant_workspace_id: "export-cli-tenant".into(),
        principal_id: "principal-analyst-1".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-cli-1".into(),
        includes_source_text: false,
    }
}

fn export_post(request: &ExportAuthorizationRequest, idempotency_key: &str) -> String {
    let body = tepp_api_wire_json(request);
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn tepp_api_wire_json(request: &ExportAuthorizationRequest) -> String {
    serde_json::to_string(request).expect("request json")
}

fn get_args<'a>(host: &'a str, export_id: &'a str, consumer: &'a str) -> [&'a str; 9] {
    [
        "get",
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--export-id",
        export_id,
    ]
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        ExportStoredRequestCliVerb::parse("get").expect("get"),
        ExportStoredRequestCliVerb::Get
    );
    assert_eq!(ExportStoredRequestCliVerb::Get.as_str(), "get");
    assert_eq!(
        ExportStoredRequestCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            get_args("8.8.8.8:80", "export-1", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--export-id",
                "export-1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--export-id",
                "export-1"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--export-id",
                "export-1"
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
        ExportStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", "unpublished"),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
            "{}"
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export/slash", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            get_args(
                "127.0.0.1:18081",
                &"a".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
                NARUON_CONSUMER_CODE
            ),
            ""
        )
        .unwrap_err(),
        ApiError::LimitExceeded
    );
    assert_eq!(
        ExportStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--export-id",
                "export-1",
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
    let invocation = ExportStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_export_stored_request_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/exports/export-1/request HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn naruon_cli_retrieves_stored_request_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-idem-1"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval = ExportRetrieval::from_json(&posted.body).expect("posted retrieval");
    let invocation = ExportStoredRequestCliInvocation::from_args(
        get_args(
            "127.0.0.1:18081",
            &retrieval.export_id,
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    let got = dispatch_export_stored_request_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_export_stored_request_cli_stdout(&invocation, &got).expect("out");
    let stored: ExportAuthorizationRequest = serde_json::from_str(&stdout).expect("stored");
    assert_eq!(stored, request);
    assert!(stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    let missing = ExportStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "missing-export", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied = dispatch_export_stored_request_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400);
    assert!(
        render_export_stored_request_cli_stdout(&missing, &denied)
            .expect("err")
            .contains("invalid_wire_payload")
    );
    let mut naruon = NaruonLiveService::new();
    assert_eq!(
        naruon
            .handle_http_request(
                &compose_export_stored_request_cli_http(&invocation).expect("composed")
            )
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_success() {
    let invocation = ExportStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_export_stored_request_cli_stdout(
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
        render_export_stored_request_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"tenant_workspace_id":"t","rmse":1.0}"#.into()
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_export_stored_request_cli_stdout(
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
fn loopback_http1_refuses_non_get_collection_get_by_id_cancel_and_credentials() {
    let host = "127.0.0.1:18081";
    let exchange = naruon_export_stored_request_exchange(ORIGIN, "export-1").expect("ex");
    let ok = loopback_http1_from_export_stored_request_exchange(&exchange, host).expect("ok");
    assert!(ok.starts_with("GET /v1/exports/export-1/request HTTP/1.1"));
    let mut posted = exchange.clone();
    posted.method = "POST";
    assert_eq!(
        loopback_http1_from_export_stored_request_exchange(&posted, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut by_id = exchange.clone();
    by_id.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1");
    assert_eq!(
        loopback_http1_from_export_stored_request_exchange(&by_id, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut cancel = exchange.clone();
    cancel.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1/cancel");
    assert_eq!(
        loopback_http1_from_export_stored_request_exchange(&cancel, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let credentialed = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1/request"),
        headers: vec![("authorization".into(), "secret".into())],
        body: String::new(),
    };
    assert_eq!(
        loopback_http1_from_export_stored_request_exchange(&credentialed, host).unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn execute_over_tcp_and_stdin_reader() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-idem-tcp"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval = ExportRetrieval::from_json(&posted.body).expect("posted retrieval");
    let export_id = retrieval.export_id.clone();
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = ExportStoredRequestCliInvocation::from_args(
        get_args(addr.as_str(), &export_id, NARUON_CONSUMER_CODE),
        "",
    )
    .expect("tcp");
    let response = execute_export_stored_request_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stored: ExportAuthorizationRequest = serde_json::from_str(
        &render_export_stored_request_cli_stdout(&invocation, &response).expect("stdout"),
    )
    .expect("parsed");
    assert_eq!(stored.artifact_id, "artifact-cli-1");
    handle.join().expect("join");
    assert!(
        read_export_stored_request_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty()
    );
    assert!(
        read_export_stored_request_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("pipe")
            .is_empty()
    );
}

#[test]
fn binary_reports_redacted_success_and_failure_statuses() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request();
    let posted = service.handle_http_request(&export_post(&request, "export-idem-bin"));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let retrieval = ExportRetrieval::from_json(&posted.body).expect("posted retrieval");
    let export_id = retrieval.export_id.clone();
    let handle = std::thread::spawn(move || {
        service.serve_one().expect("success request");
        service.serve_one().expect("missing request");
    });
    let binary = env!("CARGO_BIN_EXE_tepp-export-request");
    let run = |export_id: &str| {
        std::process::Command::new(binary)
            .args(get_args(&addr, export_id, NARUON_CONSUMER_CODE))
            .output()
            .expect("binary")
    };
    let success = run(&export_id);
    assert!(
        success.status.success(),
        "{}",
        String::from_utf8_lossy(&success.stderr)
    );
    assert!(String::from_utf8_lossy(&success.stdout).contains("artifact-cli-1"));
    let failure = run("missing-export");
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("invalid API wire payload"));
    handle.join().expect("server");
}
