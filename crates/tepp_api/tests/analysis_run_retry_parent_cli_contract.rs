//! GAP-003A naruon/LineageWeave analysis-run retry-parent CLI.

use tepp_api::{
    compose_analysis_run_retry_parent_cli_http, dispatch_analysis_run_retry_parent_cli,
    execute_analysis_run_retry_parent_cli, read_analysis_run_retry_parent_cli_stdin,
    render_analysis_run_retry_parent_cli_stdout, AnalysisRunAccepted, AnalysisRunLiveService,
    AnalysisRunRequest, AnalysisRunRetryParent, AnalysisRunRetryParentCliInvocation,
    AnalysisRunRetryParentCliVerb, ApiError, NaruonLiveResponse, ANALYSIS_RUN_CONTRACT_VERSION,
    LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn request(idempotency_key: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: idempotency_key.into(),
        tenant_workspace_id: "cli-parent-tenant".into(),
        snapshot_id: "cli-parent-snapshot".into(),
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
    let body = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","idempotency_key":"{idempotency_key}"}}"#
    );
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/cancel HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn retry_http(run_id: &str, consumer: &str, host: &str, idempotency_key: &str) -> String {
    let body = format!(
        r#"{{"contract_version":1,"run_id":"{run_id}","idempotency_key":"{idempotency_key}"}}"#
    );
    format!(
        "POST {NARUON_ANALYSIS_RUN_PATH}/{run_id}/retry HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn parent_args<'a>(host: &'a str, run_id: &'a str, consumer: &'a str) -> [&'a str; 9] {
    [
        "parent",
        "--host",
        host,
        "--origin",
        ORIGIN,
        "--consumer",
        consumer,
        "--run-id",
        run_id,
    ]
}

fn accept_cancel_retry(
    service: &mut AnalysisRunLiveService,
    idempotency_key: &str,
    child_key: &str,
    consumer: &str,
) -> (AnalysisRunAccepted, AnalysisRunAccepted) {
    let created = service.handle_http_request(&create_http(
        &request(idempotency_key),
        consumer,
        "127.0.0.1:18081",
    ));
    assert_eq!(created.status_code, 202, "{}", created.body);
    let parent = AnalysisRunAccepted::from_json(&created.body).expect("accepted");
    let cancelled = service.handle_http_request(&cancel_http(
        &parent.run_id,
        consumer,
        "127.0.0.1:18081",
        idempotency_key,
    ));
    assert_eq!(cancelled.status_code, 200, "{}", cancelled.body);
    let retried = service.handle_http_request(&retry_http(
        &parent.run_id,
        consumer,
        "127.0.0.1:18081",
        child_key,
    ));
    assert_eq!(retried.status_code, 202, "{}", retried.body);
    let child = AnalysisRunAccepted::from_json(&retried.body).expect("child");
    assert_ne!(child.run_id, parent.run_id);
    (parent, child)
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        AnalysisRunRetryParentCliVerb::parse("parent").expect("parent"),
        AnalysisRunRetryParentCliVerb::Parent
    );
    assert_eq!(AnalysisRunRetryParentCliVerb::Parent.as_str(), "parent");
    assert_eq!(
        AnalysisRunRetryParentCliVerb::parse("retry"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(
            parent_args("8.8.8.8:80", "tepp-run-1", NARUON_CONSUMER_CODE),
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(
            [
                "parent",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--run-id",
                "tepp-run-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(
            [
                "parent",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--run-id",
                "tepp-run-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(
            [
                "parent",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--run-id",
                "tepp-run-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        AnalysisRunRetryParentCliInvocation::from_args(
            parent_args("127.0.0.1:18081", "tepp-run-1", NARUON_CONSUMER_CODE),
            "{}",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_get_parent_without_credentials() {
    let invocation = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args("127.0.0.1:18081", "tepp-run-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_analysis_run_retry_parent_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/analysis-runs/tepp-run-1/parent HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(http.contains("content-length: 0"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn naruon_and_lineageweave_cli_inspect_retry_parent() {
    let mut service = AnalysisRunLiveService::new();
    let (parent, child) = accept_cancel_retry(
        &mut service,
        "cli-parent-naruon",
        "cli-parent-naruon-child",
        NARUON_CONSUMER_CODE,
    );
    let invocation = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args(
            "127.0.0.1:18081",
            child.run_id.as_str(),
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("naruon");
    let inspected = dispatch_analysis_run_retry_parent_cli(&mut service, &invocation).expect("get");
    assert_eq!(inspected.status_code, 200, "{}", inspected.body);
    let stdout = render_analysis_run_retry_parent_cli_stdout(&invocation, &inspected).expect("out");
    let payload = AnalysisRunRetryParent::from_json(&stdout).expect("parent");
    assert_eq!(payload.run_id, child.run_id);
    let parent_row = payload.parent.expect("non-null parent");
    assert_eq!(parent_row.run_id, parent.run_id);
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    assert!(!stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("retried_from"));

    let original = service.handle_http_request(&create_http(
        &request("cli-parent-original"),
        NARUON_CONSUMER_CODE,
        "127.0.0.1:18081",
    ));
    let original_accepted = AnalysisRunAccepted::from_json(&original.body).expect("original");
    let original_inv = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args(
            "127.0.0.1:18081",
            original_accepted.run_id.as_str(),
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("original inv");
    let original_got =
        dispatch_analysis_run_retry_parent_cli(&mut service, &original_inv).expect("original get");
    assert_eq!(original_got.status_code, 200, "{}", original_got.body);
    let original_stdout =
        render_analysis_run_retry_parent_cli_stdout(&original_inv, &original_got).expect("null");
    let original_payload =
        AnalysisRunRetryParent::from_json(&original_stdout).expect("null parent");
    assert!(original_payload.parent.is_none());
    assert!(original_stdout.contains("\"parent\":null"));

    let lineage_pair = accept_cancel_retry(
        &mut service,
        "cli-parent-lineage",
        "cli-parent-lineage-child",
        LINEAGEWEAVE_CONSUMER_CODE,
    );
    let lineage = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args(
            "127.0.0.1:18081",
            lineage_pair.1.run_id.as_str(),
            LINEAGEWEAVE_CONSUMER_CODE,
        ),
        "",
    )
    .expect("lineage");
    let lineage_http = compose_analysis_run_retry_parent_cli_http(&lineage).expect("http");
    assert!(lineage_http.contains("tepp-consumer: lineageweave"));
    assert!(!lineage_http.contains("tepp-consumer: naruon"));
    let lineage_got =
        dispatch_analysis_run_retry_parent_cli(&mut service, &lineage).expect("lineage get");
    assert_eq!(lineage_got.status_code, 200, "{}", lineage_got.body);
    let lineage_stdout =
        render_analysis_run_retry_parent_cli_stdout(&lineage, &lineage_got).expect("lineage out");
    let lineage_payload = AnalysisRunRetryParent::from_json(&lineage_stdout).expect("lineage");
    assert_eq!(
        lineage_payload.parent.expect("lineage parent").run_id,
        lineage_pair.0.run_id
    );

    let unknown = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args("127.0.0.1:18081", "missing-run", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("unknown");
    let denied = dispatch_analysis_run_retry_parent_cli(&mut service, &unknown).expect("denied");
    assert_eq!(denied.status_code, 400, "{}", denied.body);
    let denied_stdout =
        render_analysis_run_retry_parent_cli_stdout(&unknown, &denied).expect("err");
    assert!(denied_stdout.contains("invalid_wire_payload"));
    assert!(!denied_stdout.contains(SCHEMA));
}

#[test]
fn render_refuses_metrics_and_identity_mismatch() {
    let invocation = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args("127.0.0.1:18081", "tepp-run-2", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_analysis_run_retry_parent_cli_stdout(
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
        render_analysis_run_retry_parent_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"run_id":"tepp-run-2","run_state":"accepted","idempotency_key":"idem-child","parent":null,"rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_analysis_run_retry_parent_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"run_id":"tepp-run-1","run_state":"accepted","idempotency_key":"idem-child","parent":null}"#.into(),
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
    let (_parent, child) = accept_cancel_retry(
        &mut service,
        "cli-parent-tcp",
        "cli-parent-tcp-child",
        NARUON_CONSUMER_CODE,
    );
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = AnalysisRunRetryParentCliInvocation::from_args(
        parent_args(addr.as_str(), child.run_id.as_str(), NARUON_CONSUMER_CODE),
        "",
    )
    .expect("tcp");
    let response = execute_analysis_run_retry_parent_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    handle.join().expect("join");
    let empty = read_analysis_run_retry_parent_cli_stdin(true, std::io::empty()).expect("tty");
    assert!(empty.is_empty());
    let piped =
        read_analysis_run_retry_parent_cli_stdin(false, std::io::Cursor::new(b"")).expect("pipe");
    assert!(piped.is_empty());
}
