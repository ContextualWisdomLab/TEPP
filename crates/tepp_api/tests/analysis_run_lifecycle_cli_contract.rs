//! GAP-003A naruon/LineageWeave analysis-run lifecycle CLI.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted, AnalysisRunLifecycleCliInvocation,
    AnalysisRunLifecycleCliVerb, AnalysisRunLifecycleTransition, AnalysisRunLiveService,
    AnalysisRunRequest, AnalysisRunStatus, AnalysisRunStatusState, AnalysisRunTerminalResult,
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
    NaruonLiveResponse, compose_analysis_run_lifecycle_cli_http,
    dispatch_analysis_run_lifecycle_cli, execute_analysis_run_lifecycle_cli,
    read_analysis_run_lifecycle_cli_stdin, render_analysis_run_lifecycle_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn request(idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "cli-lifecycle-tenant".into(),
        snapshot_id: "cli-lifecycle-snapshot".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: "tepp-analysis-run-v1".into(),
        output_profile: "calibrated_event_measurement".into(),
    }
}

fn create_http(run: &AnalysisRunRequest, consumer: &str, host: &str) -> String {
    let body = run.to_json().expect("json");
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
        run.idempotency_key,
        body.len()
    )
}

fn lifecycle_args<'a>(
    verb: &'a str,
    host: &'a str,
    run_id: &'a str,
    idempotency_key: &'a str,
    consumer: &'a str,
) -> [&'a str; 11] {
    [
        verb,
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--run-id",
        run_id,
        "--idempotency-key",
        idempotency_key,
    ]
}

fn accept(
    service: &mut AnalysisRunLiveService,
    idempotency_key: &str,
    consumer: &str,
) -> AnalysisRunAccepted {
    let created = service.handle_http_request(&create_http(
        &request(idempotency_key),
        consumer,
        "127.0.0.1:18081",
    ));
    assert_eq!(created.status_code, 202, "{}", created.body);
    AnalysisRunAccepted::from_json(&created.body).expect("accepted")
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        AnalysisRunLifecycleCliVerb::parse("running").expect("running"),
        AnalysisRunLifecycleCliVerb::Running
    );
    assert_eq!(AnalysisRunLifecycleCliVerb::Running.as_str(), "running");
    assert_eq!(AnalysisRunLifecycleCliVerb::Terminal.as_str(), "terminal");
    assert_eq!(
        AnalysisRunLifecycleCliVerb::parse("retry"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            lifecycle_args(
                "running",
                "8.8.8.8:80",
                "tepp-run-1",
                "idem-1",
                NARUON_CONSUMER_CODE
            ),
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            [
                "running",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            [
                "running",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn unpublished_consumer_credentials_and_empty_terminal_fail_closed() {
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            lifecycle_args(
                "running",
                "127.0.0.1:18081",
                "tepp-run-1",
                "idem-1",
                "unpublished"
            ),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            [
                "running",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        AnalysisRunLifecycleCliInvocation::from_args(
            lifecycle_args(
                "terminal",
                "127.0.0.1:18081",
                "tepp-run-1",
                "idem-1",
                NARUON_CONSUMER_CODE
            ),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_post_running_without_credentials() {
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            "127.0.0.1:18081",
            "tepp-run-1",
            "idem-1",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    let http = compose_analysis_run_lifecycle_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/analysis-runs/tepp-run-1/running HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(http.contains("idempotency-key: idem-1"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn naruon_and_lineageweave_cli_record_running_status() {
    let mut service = AnalysisRunLiveService::new();
    let naruon = accept(&mut service, "cli-lifecycle-naruon", NARUON_CONSUMER_CODE);
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            "127.0.0.1:18081",
            naruon.run_id.as_str(),
            "cli-lifecycle-naruon",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("naruon");
    let running = dispatch_analysis_run_lifecycle_cli(&mut service, &invocation).expect("running");
    assert_eq!(running.status_code, 200, "{}", running.body);
    let stdout = render_analysis_run_lifecycle_cli_stdout(&invocation, &running).expect("stdout");
    let status = AnalysisRunStatus::from_json(&stdout).expect("status");
    assert_eq!(status.run_state, AnalysisRunStatusState::Running);
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));

    let lineage = accept(
        &mut service,
        "cli-lifecycle-lineage",
        LINEAGEWEAVE_CONSUMER_CODE,
    );
    let lineage_invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            "127.0.0.1:18081",
            lineage.run_id.as_str(),
            "cli-lifecycle-lineage",
            LINEAGEWEAVE_CONSUMER_CODE,
        ),
        "",
    )
    .expect("lineage");
    let lineage_http = compose_analysis_run_lifecycle_cli_http(&lineage_invocation).expect("http");
    assert!(lineage_http.contains("tepp-consumer: lineageweave"));
    assert!(!lineage_http.contains("tepp-consumer: naruon"));
    let lineage_running =
        dispatch_analysis_run_lifecycle_cli(&mut service, &lineage_invocation).expect("lineage");
    assert_eq!(lineage_running.status_code, 200, "{}", lineage_running.body);
    let lineage_stdout =
        render_analysis_run_lifecycle_cli_stdout(&lineage_invocation, &lineage_running)
            .expect("lineage stdout");
    assert!(lineage_stdout.contains("\"running\""));
    assert!(!lineage_stdout.contains(SCHEMA));

    let mismatched = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            "127.0.0.1:18081",
            naruon.run_id.as_str(),
            "cli-lifecycle-naruon",
            LINEAGEWEAVE_CONSUMER_CODE,
        ),
        "",
    )
    .expect("mismatch");
    let denied = dispatch_analysis_run_lifecycle_cli(&mut service, &mismatched).expect("denied");
    assert_eq!(denied.status_code, 400, "{}", denied.body);
    let denied_stdout =
        render_analysis_run_lifecycle_cli_stdout(&mismatched, &denied).expect("err");
    assert!(denied_stdout.contains("invalid_wire_payload"));
    assert!(!denied_stdout.contains(SCHEMA));
}

#[test]
fn terminal_cli_records_failed_status() {
    let mut service = AnalysisRunLiveService::new();
    let accepted = accept(&mut service, "cli-lifecycle-failed", NARUON_CONSUMER_CODE);
    let run = request("cli-lifecycle-failed");
    let failed = AnalysisRunTerminalResult::failed(
        &run,
        &accepted,
        "2026-08-02T03:04:05Z",
        "estimation_failed",
    )
    .expect("failed");
    let transition = AnalysisRunLifecycleTransition::terminal(
        accepted.run_id.clone(),
        run.idempotency_key.clone(),
        failed,
        None,
    )
    .expect("transition");
    let body = transition.to_json().expect("json");
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "terminal",
            "127.0.0.1:18081",
            accepted.run_id.as_str(),
            "cli-lifecycle-failed",
            NARUON_CONSUMER_CODE,
        ),
        body,
    )
    .expect("terminal");
    let http = compose_analysis_run_lifecycle_cli_http(&invocation).expect("http");
    assert!(http.contains("/terminal HTTP/1.1"));
    let response = dispatch_analysis_run_lifecycle_cli(&mut service, &invocation).expect("post");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_analysis_run_lifecycle_cli_stdout(&invocation, &response).expect("stdout");
    let status = AnalysisRunStatus::from_json(&stdout).expect("status");
    assert_eq!(status.run_state, AnalysisRunStatusState::Failed);
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
}

#[test]
fn render_refuses_empty_and_metrics() {
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            "127.0.0.1:18081",
            "tepp-run-1",
            "idem-1",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_analysis_run_lifecycle_cli_stdout(
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
        render_analysis_run_lifecycle_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"running","idempotency_key":"idem-1","rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_and_stdin_reader() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let accepted = accept(&mut service, "cli-lifecycle-tcp", NARUON_CONSUMER_CODE);
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = AnalysisRunLifecycleCliInvocation::from_args(
        lifecycle_args(
            "running",
            addr.as_str(),
            accepted.run_id.as_str(),
            "cli-lifecycle-tcp",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("tcp");
    let response = execute_analysis_run_lifecycle_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    handle.join().expect("join");
    let empty = read_analysis_run_lifecycle_cli_stdin(true, std::io::empty()).expect("tty");
    assert!(empty.is_empty());
    let piped =
        read_analysis_run_lifecycle_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("pipe");
    assert_eq!(piped, "{}");
}
