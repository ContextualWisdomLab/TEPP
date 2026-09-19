//! Contract tests for the `LineageWeave` project-history stored-request loopback CLI.

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    NaruonHttpExchange, NaruonLiveResponse, NaruonLiveService, PROJECT_HISTORY_CONTRACT_VERSION,
    PROJECT_HISTORY_PATH, PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN, ProjectHistoryEvent,
    ProjectHistoryRequest, ProjectHistoryStoredRequestCliInvocation,
    ProjectHistoryStoredRequestCliVerb, compose_project_history_stored_request_cli_http,
    dispatch_project_history_stored_request_cli, execute_project_history_stored_request_cli,
    lineageweave_project_history_stored_request_exchange,
    loopback_http1_from_project_history_stored_request_exchange,
    read_project_history_stored_request_cli_stdin,
    render_project_history_stored_request_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn sample_request(idempotency_key: &str, project_key: &str) -> ProjectHistoryRequest {
    ProjectHistoryRequest {
        contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "history-tenant".into(),
        project_key: project_key.into(),
        project_name: "Project".into(),
        knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
        focus_event_id: "focus".into(),
        events: vec![ProjectHistoryEvent {
            event_id: "focus".into(),
            event_type_code: "voc_received".into(),
            event_title: "VOC".into(),
            occurred_at: "2026-08-19T09:00:00Z".into(),
            available_at: "2026-08-19T10:00:00Z".into(),
            source_post_id: "post".into(),
            evidence_text: "explicit evidence".into(),
            actor_ids: Vec::new(),
        }],
    }
}

fn project_history_post(request: &ProjectHistoryRequest) -> String {
    let body = request.to_json().expect("history json");
    format!(
        "POST {PROJECT_HISTORY_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        request.idempotency_key,
        body.len()
    )
}

fn get_args<'a>(host: &'a str, idempotency_key: &'a str, consumer: &'a str) -> [&'a str; 11] {
    [
        "get",
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--idempotency-key",
        idempotency_key,
        "--tenant-workspace-id",
        "history-tenant",
    ]
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        ProjectHistoryStoredRequestCliVerb::parse("get").expect("get"),
        ProjectHistoryStoredRequestCliVerb::Get
    );
    assert_eq!(ProjectHistoryStoredRequestCliVerb::Get.as_str(), "get");
    assert_eq!(
        ProjectHistoryStoredRequestCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args("8.8.8.8:80", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-a",
                "--tenant-workspace-id",
                "history-tenant"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--idempotency-key",
                "idem-a",
                "--tenant-workspace-id",
                "history-tenant"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--idempotency-key",
                "idem-a",
                "--tenant-workspace-id",
                "history-tenant"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn from_args_refuses_naruon_slash_body_size_and_pagination() {
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", "unpublished"),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            "{}"
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem/slash", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            get_args(
                "127.0.0.1:18081",
                &"a".repeat(PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN + 1),
                LINEAGEWEAVE_CONSUMER_CODE
            ),
            ""
        )
        .unwrap_err(),
        ApiError::LimitExceeded
    );
    assert_eq!(
        ProjectHistoryStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-a",
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
    let invocation = ProjectHistoryStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_project_history_stored_request_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/project-histories/idem-a/request HTTP/1.1"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn lineageweave_cli_retrieves_stored_request_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let request = sample_request("idem-a", "project-a");
    assert_eq!(
        service
            .handle_http_request(&project_history_post(&request))
            .status_code,
        200
    );
    let invocation = ProjectHistoryStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let got = dispatch_project_history_stored_request_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_project_history_stored_request_cli_stdout(&invocation, &got).expect("out");
    assert_eq!(
        ProjectHistoryRequest::from_json(&stdout).expect("stored"),
        request
    );
    assert!(stdout.contains("evidence_text"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    let mut mismatched = ProjectHistoryRequest::from_json(&got.body).expect("body");
    mismatched.tenant_workspace_id = "other-tenant".into();
    assert_eq!(
        render_project_history_stored_request_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: mismatched.to_json().expect("m")
            }
        ),
        Err(ApiError::InvalidWirePayload)
    );
    let missing = ProjectHistoryStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "missing", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied =
        dispatch_project_history_stored_request_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400);
    assert!(
        render_project_history_stored_request_cli_stdout(&missing, &denied)
            .expect("err")
            .contains("invalid_wire_payload")
    );
    let mut naruon = NaruonLiveService::new();
    assert_eq!(
        naruon
            .handle_http_request(
                &compose_project_history_stored_request_cli_http(&invocation).expect("composed")
            )
            .status_code,
        400
    );
}

#[test]
fn render_refuses_metrics_schema_and_empty_success() {
    let invocation = ProjectHistoryStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_project_history_stored_request_cli_stdout(
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
        render_project_history_stored_request_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"rmse":1.0}"#.into()
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_project_history_stored_request_cli_stdout(
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
    let exchange =
        lineageweave_project_history_stored_request_exchange(ORIGIN, "history-tenant", "idem-a")
            .expect("ex");
    let ok =
        loopback_http1_from_project_history_stored_request_exchange(&exchange, host).expect("ok");
    assert!(ok.starts_with("GET /v1/project-histories/idem-a/request HTTP/1.1"));
    let mut posted = exchange.clone();
    posted.method = "POST";
    assert_eq!(
        loopback_http1_from_project_history_stored_request_exchange(&posted, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut by_id = exchange.clone();
    by_id.target_url = format!("{ORIGIN}{PROJECT_HISTORY_PATH}/idem-a");
    assert_eq!(
        loopback_http1_from_project_history_stored_request_exchange(&by_id, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut cancel = exchange.clone();
    cancel.target_url = format!("{ORIGIN}{PROJECT_HISTORY_PATH}/idem-a/cancel");
    assert_eq!(
        loopback_http1_from_project_history_stored_request_exchange(&cancel, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let credentialed = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{PROJECT_HISTORY_PATH}/idem-a/request"),
        headers: vec![("authorization".into(), "secret".into())],
        body: String::new(),
    };
    assert_eq!(
        loopback_http1_from_project_history_stored_request_exchange(&credentialed, host)
            .unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn execute_over_tcp_and_stdin_reader() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request("idem-tcp", "project-tcp");
    assert_eq!(
        service
            .handle_http_request(&project_history_post(&request))
            .status_code,
        200
    );
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = ProjectHistoryStoredRequestCliInvocation::from_args(
        get_args(addr.as_str(), "idem-tcp", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("tcp");
    let response = execute_project_history_stored_request_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stored = ProjectHistoryRequest::from_json(
        &render_project_history_stored_request_cli_stdout(&invocation, &response).expect("stdout"),
    )
    .expect("parsed");
    assert_eq!(stored.project_key, "project-tcp");
    handle.join().expect("join");
    assert!(
        read_project_history_stored_request_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty()
    );
    assert!(
        read_project_history_stored_request_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("pipe")
            .is_empty()
    );
}

#[test]
fn binary_reports_redacted_success_and_failure_statuses() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let request = sample_request("idem-bin", "project-bin");
    assert_eq!(
        service
            .handle_http_request(&project_history_post(&request))
            .status_code,
        200
    );
    let handle = std::thread::spawn(move || {
        service.serve_one().expect("success request");
        service.serve_one().expect("missing request");
    });
    let binary = env!("CARGO_BIN_EXE_tepp-project-history-request");
    let run = |idempotency_key: &str| {
        std::process::Command::new(binary)
            .args(get_args(&addr, idempotency_key, LINEAGEWEAVE_CONSUMER_CODE))
            .output()
            .expect("binary")
    };
    let success = run("idem-bin");
    assert!(
        success.status.success(),
        "{}",
        String::from_utf8_lossy(&success.stderr)
    );
    assert!(String::from_utf8_lossy(&success.stdout).contains("project-bin"));
    let failure = run("missing");
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("invalid API wire payload"));
    handle.join().expect("server");
}
