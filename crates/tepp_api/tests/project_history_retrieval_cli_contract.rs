//! Contract tests for the `LineageWeave` project-history retrieval loopback CLI.

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    NaruonHttpExchange, NaruonLiveResponse, NaruonLiveService, PROJECT_HISTORY_CONTRACT_VERSION,
    PROJECT_HISTORY_PATH, PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN, ProjectHistoryEvent,
    ProjectHistoryProjection, ProjectHistoryRequest, ProjectHistoryRetrievalCliInvocation,
    ProjectHistoryRetrievalCliVerb, ProjectHistoryRetrievalReceipt,
    compose_project_history_retrieval_cli_http, dispatch_project_history_retrieval_cli,
    execute_project_history_retrieval_cli, lineageweave_project_history_retrieval_exchange,
    loopback_http1_from_project_history_retrieval_exchange,
    read_project_history_retrieval_cli_stdin, render_project_history_retrieval_cli_stdout,
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
        ProjectHistoryRetrievalCliVerb::parse("get").expect("get"),
        ProjectHistoryRetrievalCliVerb::Get
    );
    assert_eq!(ProjectHistoryRetrievalCliVerb::Get.as_str(), "get");
    assert_eq!(
        ProjectHistoryRetrievalCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryRetrievalCliVerb::parse("GET"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args("8.8.8.8:80", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--idempotency-key",
                "idem-a",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-a",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
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
            ],
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn from_args_accepts_slash_and_refuses_naruon_body_size_tenant_and_pagination() {
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", NARUON_CONSUMER_CODE),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", "unpublished"),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            "{}",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem/slash", LINEAGEWEAVE_CONSUMER_CODE),
            "",
        )
        .is_ok()
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            get_args(
                "127.0.0.1:18081",
                &"a".repeat(PROJECT_HISTORY_RETRIEVAL_ID_MAX_LEN + 1),
                LINEAGEWEAVE_CONSUMER_CODE,
            ),
            "",
        )
        .unwrap_err(),
        ApiError::LimitExceeded
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-a",
                "--page-limit",
                "1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "idem-a",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut hostile_tenant = get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE);
    hostile_tenant[10] = "tenant\nother";
    assert_eq!(
        ProjectHistoryRetrievalCliInvocation::from_args(hostile_tenant, "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_get_without_credentials() {
    let invocation = ProjectHistoryRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_project_history_retrieval_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/project-histories/idem-a HTTP/1.1"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    assert!(http.contains("tepp-tenant-workspace-id: history-tenant"));
    assert!(http.contains("content-length: 0"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("idempotency-key:"));
    assert!(!http.contains("tepp-page-limit"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
    assert!(!http.contains("/analysis-runs"));
}

#[test]
fn lineageweave_cli_retrieves_stored_projection_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let request = sample_request("idem-a", "project-a");
    let posted = service.handle_http_request(&project_history_post(&request));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    let stored = ProjectHistoryProjection::from_json(&posted.body).expect("stored");

    let invocation = ProjectHistoryRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let got = dispatch_project_history_retrieval_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_project_history_retrieval_cli_stdout(&invocation, &got).expect("out");
    let retrieved = ProjectHistoryProjection::from_json(&stdout).expect("retrieved");
    assert_eq!(retrieved, stored);
    assert_eq!(retrieved.inference_status, "temporal_association_only");
    assert!(stdout.contains("evidence_text"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    assert!(!stdout.contains("causal_score"));

    let mut mismatched = ProjectHistoryRetrievalReceipt::from_json(&got.body).expect("receipt");
    mismatched.tenant_workspace_id = "other-tenant".into();
    assert_eq!(
        render_project_history_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: mismatched.to_json().expect("mismatched receipt"),
            },
        ),
        Err(ApiError::InvalidWirePayload)
    );
    let mut mismatched = ProjectHistoryRetrievalReceipt::from_json(&got.body).expect("receipt");
    mismatched.idempotency_key = "other-id".into();
    for status_code in [200, 202] {
        assert_eq!(
            render_project_history_retrieval_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code,
                    reason_phrase: "OK",
                    body: mismatched.to_json().expect("mismatched receipt"),
                },
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }

    let missing = ProjectHistoryRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "missing", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied = dispatch_project_history_retrieval_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400, "{}", denied.body);
    let denied_stdout =
        render_project_history_retrieval_cli_stdout(&missing, &denied).expect("err");
    assert!(denied_stdout.contains("invalid_wire_payload"));
    assert!(!denied_stdout.contains(SCHEMA));

    let mut naruon = NaruonLiveService::new();
    let composed = compose_project_history_retrieval_cli_http(&invocation).expect("composed");
    assert_eq!(naruon.handle_http_request(&composed).status_code, 400);
}

#[test]
fn render_refuses_metrics_schema_and_empty_success() {
    let invocation = ProjectHistoryRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_project_history_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: String::new(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_project_history_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_project_history_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: format!(r#"{{"schema_version":"{SCHEMA}"}}"#),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    for (status_code, error_code) in [(500, "invalid_wire_payload"), (400, "limit_exceeded")] {
        assert_eq!(
            render_project_history_retrieval_cli_stdout(
                &invocation,
                &NaruonLiveResponse {
                    status_code,
                    reason_phrase: "Error",
                    body: format!(
                        r#"{{"error_code":"{error_code}","message":"redacted","request_id":"req-1","retryable":false}}"#
                    ),
                },
            ),
            Err(ApiError::InvalidWirePayload)
        );
    }
    for (status_code, error_code) in [
        (403, "authorization_denied"),
        (413, "limit_exceeded"),
        (422, "unsupported_contract_version"),
    ] {
        let rendered = render_project_history_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code,
                reason_phrase: "Error",
                body: format!(
                    r#"{{"error_code":"{error_code}","message":"redacted","request_id":"req-1","retryable":false}}"#
                ),
            },
        )
        .expect("status-matched envelope");
        assert!(rendered.contains(error_code));
    }
}

#[test]
fn loopback_http1_refuses_non_get_collection_extra_and_credentials() {
    let host = "127.0.0.1:18081";
    let exchange =
        lineageweave_project_history_retrieval_exchange(ORIGIN, "history-tenant", "idem-a")
            .expect("ex");
    let ok = loopback_http1_from_project_history_retrieval_exchange(&exchange, host).expect("ok");
    assert!(ok.starts_with("GET /v1/project-histories/idem-a HTTP/1.1"));
    let mut posted = exchange.clone();
    posted.method = "POST";
    assert_eq!(
        loopback_http1_from_project_history_retrieval_exchange(&posted, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut collection = exchange.clone();
    collection.target_url = format!("{ORIGIN}{PROJECT_HISTORY_PATH}");
    assert_eq!(
        loopback_http1_from_project_history_retrieval_exchange(&collection, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut extra = exchange.clone();
    extra.target_url = format!("{ORIGIN}{PROJECT_HISTORY_PATH}/idem-a/extra");
    assert_eq!(
        loopback_http1_from_project_history_retrieval_exchange(&extra, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut paged = exchange.clone();
    paged.headers.push(("tepp-page-limit".into(), "1".into()));
    assert_eq!(
        loopback_http1_from_project_history_retrieval_exchange(&paged, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let credentialed = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{PROJECT_HISTORY_PATH}/idem-a"),
        headers: vec![("authorization".into(), "secret".into())],
        body: String::new(),
    };
    assert_eq!(
        loopback_http1_from_project_history_retrieval_exchange(&credentialed, host).unwrap_err(),
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
    let invocation = ProjectHistoryRetrievalCliInvocation::from_args(
        get_args(addr.as_str(), "idem-tcp", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("tcp");
    let response = execute_project_history_retrieval_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout =
        render_project_history_retrieval_cli_stdout(&invocation, &response).expect("stdout");
    let retrieved = ProjectHistoryProjection::from_json(&stdout).expect("parsed");
    assert_eq!(retrieved.inference_status, "temporal_association_only");
    assert_eq!(retrieved.project_key, "project-tcp");
    handle.join().expect("join");
    let empty = read_project_history_retrieval_cli_stdin(true, std::io::empty()).expect("tty");
    assert!(empty.is_empty());
    let piped =
        read_project_history_retrieval_cli_stdin(false, std::io::Cursor::new(b"")).expect("pipe");
    assert!(piped.is_empty());
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
    let binary = env!("CARGO_BIN_EXE_tepp-project-history-get");
    let run = |idempotency_key: &str| {
        std::process::Command::new(binary)
            .args(get_args(&addr, idempotency_key, LINEAGEWEAVE_CONSUMER_CODE))
            .output()
            .expect("binary")
    };
    let success = run("idem-bin");
    assert!(success.status.success());
    assert!(String::from_utf8_lossy(&success.stdout).contains("project-bin"));
    assert!(success.stderr.is_empty());
    let failure = run("missing");
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("invalid API wire payload"));
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("history-tenant"));
    handle.join().expect("server");
}
