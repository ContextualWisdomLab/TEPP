//! Contract tests for the `LineageWeave` project-history loopback CLI.

use std::io::Write;
use std::process::{Command, Stdio};

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, PROJECT_HISTORY_CONTRACT_VERSION,
    ProjectHistoryCliInvocation, ProjectHistoryCliVerb, ProjectHistoryEvent, ProjectHistoryRequest,
    compose_project_history_cli_http,
};

fn query_body() -> String {
    ProjectHistoryRequest {
        contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
        idempotency_key: "lineageweave-project-cli-contract-1".into(),
        tenant_workspace_id: "tenant-demo".into(),
        project_key: "project-acme".into(),
        project_name: "Acme renewal".into(),
        knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
        focus_event_id: "event-voc".into(),
        events: vec![ProjectHistoryEvent {
            event_id: "event-voc".into(),
            event_type_code: "voc_received".into(),
            event_title: "VOC received".into(),
            occurred_at: "2026-07-30T09:00:00Z".into(),
            available_at: "2026-07-30T09:00:00Z".into(),
            source_post_id: "post-voc".into(),
            evidence_text: "evidence for VOC received".into(),
            actor_ids: vec!["person-3".into()],
        }],
    }
    .to_json()
    .expect("json")
}

#[test]
fn project_history_cli_is_metric_free_post_without_credentials() {
    assert_eq!(
        ProjectHistoryCliVerb::parse("query").expect("verb"),
        ProjectHistoryCliVerb::Query
    );
    let invocation = ProjectHistoryCliInvocation::from_args(
        [
            "query",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ],
        query_body(),
    )
    .expect("invocation");
    let http = compose_project_history_cli_http(&invocation).expect("http");
    assert!(http.starts_with("POST /v1/project-histories HTTP/1.1"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("tepp.scientific_acceptance.v1"));
    assert!(!http.contains("/analysis-runs"));
    assert!(!http.contains("/v1/exports"));
}

#[test]
fn project_history_cli_refuses_non_loopback_unknown_verbs_and_metrics() {
    assert_eq!(
        ProjectHistoryCliInvocation::from_args(
            [
                "query",
                "--host",
                "8.8.8.8:80",
                "--origin",
                "https://tepp.example.test"
            ],
            query_body()
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        ProjectHistoryCliVerb::parse("cancel"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCliInvocation::from_args(
            [
                "query",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "https://tepp.example.test"
            ],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn project_history_binary_queries_loopback_and_rejects_unknown_verbs() {
    let binary = env!("CARGO_BIN_EXE_tepp-project-history");
    assert!(
        !Command::new(binary)
            .arg("unknown")
            .output()
            .expect("run")
            .status
            .success()
    );

    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let host = service.local_addr().expect("address").to_string();
    let server = std::thread::spawn(move || service.serve_one().expect("serve"));
    let mut child = Command::new(binary)
        .args([
            "query",
            "--host",
            &host,
            "--origin",
            "https://tepp.example.test",
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(query_body().as_bytes())
        .expect("write");
    let output = child.wait_with_output().expect("wait");
    server.join().expect("join");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .expect("utf8")
            .contains("temporal_association_only")
    );
}
