//! GAP-003A naruon/LineageWeave analysis-run retry CLI.

use tepp_api::{
    compose_analysis_run_retry_cli_http, dispatch_analysis_run_retry_cli,
    execute_analysis_run_retry_cli, read_analysis_run_retry_cli_stdin,
    render_analysis_run_retry_cli_stdout, AnalysisRunAccepted, AnalysisRunLiveService,
    AnalysisRunRequest, AnalysisRunRetryCliInvocation, AnalysisRunRetryCliVerb, ApiError,
    NaruonLiveResponse, ANALYSIS_RUN_CONTRACT_VERSION, ANALYSIS_RUN_RETRY_CONTRACT_VERSION,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn request(idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "cli-retry-tenant".into(),
        snapshot_id: "cli-retry-snapshot".into(),
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

fn cancel_http(run_id: &str, consumer: &str, host: &str, idempotency_key: &str) -> String {
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/cancel HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: 0\r\n\r\n"
    )
}

fn retry_args<'a>(
    host: &'a str,
    run_id: &'a str,
    idempotency_key: &'a str,
    consumer: &'a str,
) -> [&'a str; 11] {
    [
        "retry",
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

fn accept_and_cancel(
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
    let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
    let cancelled = service.handle_http_request(&cancel_http(
        &accepted.run_id,
        consumer,
        "127.0.0.1:18081",
        idempotency_key,
    ));
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    accepted
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        AnalysisRunRetryCliVerb::parse("retry").expect("retry"),
        AnalysisRunRetryCliVerb::Retry
    );
    assert_eq!(AnalysisRunRetryCliVerb::Retry.as_str(), "retry");
    assert_eq!(
        AnalysisRunRetryCliVerb::parse("cancel"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(
            retry_args(
                "8.8.8.8:80",
                "tepp-run-1",
                "idem-child",
                NARUON_CONSUMER_CODE
            ),
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(
            [
                "retry",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-child",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(
            [
                "retry",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-child",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(
            [
                "retry",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--run-id",
                "tepp-run-1",
                "--idempotency-key",
                "idem-child",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    let mismatched = format!(
        r#"{{"contract_version":{ANALYSIS_RUN_RETRY_CONTRACT_VERSION},"run_id":"other","idempotency_key":"idem-child"}}"#
    );
    assert_eq!(
        AnalysisRunRetryCliInvocation::from_args(
            retry_args(
                "127.0.0.1:18081",
                "tepp-run-1",
                "idem-child",
                NARUON_CONSUMER_CODE
            ),
            mismatched,
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_post_retry_without_credentials() {
    let invocation = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            "127.0.0.1:18081",
            "tepp-run-1",
            "idem-child",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    let http = compose_analysis_run_retry_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/analysis-runs/tepp-run-1/retry HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(http.contains("idempotency-key: idem-child"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn naruon_and_lineageweave_cli_retry_cancelled_runs() {
    let mut service = AnalysisRunLiveService::new();
    let parent = accept_and_cancel(&mut service, "cli-retry-naruon", NARUON_CONSUMER_CODE);
    let invocation = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            "127.0.0.1:18081",
            parent.run_id.as_str(),
            "cli-retry-naruon-child",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("naruon");
    let retried = dispatch_analysis_run_retry_cli(&mut service, &invocation).expect("retry");
    assert_eq!(retried.status_code, 202, "{}", retried.body);
    let stdout = render_analysis_run_retry_cli_stdout(&invocation, &retried).expect("stdout");
    let child = AnalysisRunAccepted::from_json(&stdout).expect("child");
    assert_ne!(child.run_id, parent.run_id);
    assert_eq!(child.idempotency_key, "cli-retry-naruon-child");
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));

    let lineage_parent = accept_and_cancel(
        &mut service,
        "cli-retry-lineage",
        LINEAGEWEAVE_CONSUMER_CODE,
    );
    let lineage = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            "127.0.0.1:18081",
            lineage_parent.run_id.as_str(),
            "cli-retry-lineage-child",
            LINEAGEWEAVE_CONSUMER_CODE,
        ),
        "",
    )
    .expect("lineage");
    let lineage_http = compose_analysis_run_retry_cli_http(&lineage).expect("http");
    assert!(lineage_http.contains("tepp-consumer: lineageweave"));
    assert!(!lineage_http.contains("tepp-consumer: naruon"));
    let lineage_retried =
        dispatch_analysis_run_retry_cli(&mut service, &lineage).expect("lineage retry");
    assert_eq!(lineage_retried.status_code, 202, "{}", lineage_retried.body);
    let lineage_stdout =
        render_analysis_run_retry_cli_stdout(&lineage, &lineage_retried).expect("lineage stdout");
    let lineage_child = AnalysisRunAccepted::from_json(&lineage_stdout).expect("lineage child");
    assert_ne!(lineage_child.run_id, lineage_parent.run_id);

    let accepted_only = service.handle_http_request(&create_http(
        &request("cli-retry-accepted"),
        NARUON_CONSUMER_CODE,
        "127.0.0.1:18081",
    ));
    let accepted = AnalysisRunAccepted::from_json(&accepted_only.body).expect("accepted only");
    let refused = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            "127.0.0.1:18081",
            accepted.run_id.as_str(),
            "cli-retry-accepted-child",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("refused");
    let denied = dispatch_analysis_run_retry_cli(&mut service, &refused).expect("denied");
    assert_eq!(denied.status_code, 400, "{}", denied.body);
    let denied_stdout = render_analysis_run_retry_cli_stdout(&refused, &denied).expect("err");
    assert!(denied_stdout.contains("invalid_wire_payload"));
    assert!(!denied_stdout.contains(SCHEMA));
}

#[test]
fn render_refuses_metrics_and_parent_identity() {
    let invocation = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            "127.0.0.1:18081",
            "tepp-run-1",
            "idem-child",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_analysis_run_retry_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 202,
                reason_phrase: "Accepted",
                body: String::new(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_analysis_run_retry_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 202,
                reason_phrase: "Accepted",
                body: r#"{"contract_version":1,"run_id":"tepp-run-2","run_state":"accepted","idempotency_key":"idem-child","rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_analysis_run_retry_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 202,
                reason_phrase: "Accepted",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-child"}"#.into(),
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
    let parent = accept_and_cancel(&mut service, "cli-retry-tcp", NARUON_CONSUMER_CODE);
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = AnalysisRunRetryCliInvocation::from_args(
        retry_args(
            addr.as_str(),
            parent.run_id.as_str(),
            "cli-retry-tcp-child",
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("tcp");
    let response = execute_analysis_run_retry_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 202, "{}", response.body);
    handle.join().expect("join");
    let empty = read_analysis_run_retry_cli_stdin(true, std::io::empty()).expect("tty");
    assert!(empty.is_empty());
    let piped =
        read_analysis_run_retry_cli_stdin(false, std::io::Cursor::new(b"{}")).expect("pipe");
    assert_eq!(piped, "{}");
}
