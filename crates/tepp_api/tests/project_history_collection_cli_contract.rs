//! Contract tests for the `LineageWeave` project-history collection loopback CLI.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use tepp_api::{
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, PROJECT_HISTORY_PATH,
    ProjectHistoryCollection, ProjectHistoryCollectionCliInvocation,
    ProjectHistoryCollectionCliVerb, compose_project_history_collection_cli_http,
    lineageweave_project_history_collection_exchange,
    loopback_http1_from_project_history_collection_exchange,
};

const ORIGIN: &str = "https://tepp.example.test";

#[test]
fn collection_cli_list_is_metric_free_get_without_credentials() {
    assert_eq!(
        ProjectHistoryCollectionCliVerb::parse("list").expect("list"),
        ProjectHistoryCollectionCliVerb::List
    );
    let invocation = ProjectHistoryCollectionCliInvocation::from_args(
        [
            "list",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
        ],
        "",
    )
    .expect("invocation");
    assert_eq!(invocation.consumer, LINEAGEWEAVE_CONSUMER_CODE);
    let http = compose_project_history_collection_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/project-histories HTTP/1.1"));
    assert!(!http.contains("authorization"));
    assert!(!http.contains("idempotency-key"));
    assert!(!http.contains("copilot"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    let exchange =
        lineageweave_project_history_collection_exchange(ORIGIN, None, None).expect("exchange");
    let rendered =
        loopback_http1_from_project_history_collection_exchange(&exchange, "127.0.0.1:18081")
            .expect("loopback");
    assert!(rendered.contains(PROJECT_HISTORY_PATH));
    assert_eq!(
        ProjectHistoryCollection::new(Vec::new(), None)
            .expect("empty")
            .contract_version,
        1
    );
}

#[test]
fn collection_cli_refuses_naruon_non_loopback_unknown_verbs_and_metric_bodies() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let host = listener.local_addr().expect("loopback address").to_string();
    let body = ProjectHistoryCollection::new(Vec::new(), None)
        .expect("empty collection")
        .to_json()
        .expect("collection JSON");
    let expected = serde_json::from_str::<serde_json::Value>(&body).expect("expected JSON");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept CLI");
        let mut request = [0_u8; 4096];
        let read = stream.read(&mut request).expect("read request");
        assert!(request[..read].ends_with(b"\r\n\r\n"));
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
        .expect("write response");
    });
    let accepted = Command::new(env!("CARGO_BIN_EXE_tepp-project-histories"))
        .args([
            "list",
            "--host",
            &host,
            "--origin",
            ORIGIN,
            "--page-limit",
            "1",
            "--page-cursor",
            "before",
        ])
        .output()
        .expect("run successful binary");
    server.join().expect("join loopback server");
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(accepted.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&accepted.stdout).expect("stdout JSON"),
        expected
    );

    let rejected = Command::new(env!("CARGO_BIN_EXE_tepp-project-histories"))
        .arg("unknown")
        .output()
        .expect("run binary");
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert!(!rejected.stderr.is_empty());
    assert_eq!(
        ProjectHistoryCollectionCliInvocation::from_args(
            ["list", "--host", "8.8.8.8:80", "--origin", ORIGIN],
            ""
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        ProjectHistoryCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--consumer",
                NARUON_CONSUMER_CODE
            ],
            ""
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCollectionCliVerb::parse("query"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ProjectHistoryCollectionCliInvocation::from_args(
            ["list", "--host", "127.0.0.1:18081", "--origin", ORIGIN],
            r#"{"rmse":1.0}"#
        ),
        Err(ApiError::InvalidWirePayload)
    );
}
