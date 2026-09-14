//! GAP-003A naruon export-retrieval CLI.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;

use tepp_api::{
    AnalysisRunLiveService, AnalyticalPurpose, ApiError, EXPORT_RETRIEVAL_ID_MAX_LEN,
    ExportAuthorizationRequest, ExportRetrieval, ExportRetrievalCliInvocation,
    ExportRetrievalCliVerb, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE, NARUON_EXPORT_PATH,
    NaruonHttpExchange, NaruonLiveResponse, NaruonLiveService, compose_export_retrieval_cli_http,
    dispatch_export_retrieval_cli, execute_export_retrieval_cli,
    loopback_http1_from_export_retrieval_exchange, naruon_export_retrieval_exchange,
    read_export_retrieval_cli_stdin, render_export_retrieval_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";

fn authorize_body() -> String {
    serde_json::to_string(&ExportAuthorizationRequest {
        tenant_workspace_id: "cli-export-tenant".into(),
        principal_id: "principal-analyst-cli".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "artifact-cli-1".into(),
        includes_source_text: false,
    })
    .expect("json")
}

fn export_post_http(body: &str, idempotency_key: &str) -> String {
    format!(
        "POST {NARUON_EXPORT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn mint_export(service: &mut AnalysisRunLiveService, idempotency_key: &str) -> ExportRetrieval {
    let posted = service.handle_http_request(&export_post_http(&authorize_body(), idempotency_key));
    assert_eq!(posted.status_code, 200, "{}", posted.body);
    ExportRetrieval::from_json(&posted.body).expect("minted")
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
        ExportRetrievalCliVerb::parse("get").expect("get"),
        ExportRetrievalCliVerb::Get
    );
    assert_eq!(ExportRetrievalCliVerb::Get.as_str(), "get");
    assert_eq!(
        ExportRetrievalCliVerb::parse("authorize"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportRetrievalCliVerb::parse("GET"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            get_args("8.8.8.8:80", "export-1", NARUON_CONSUMER_CODE),
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--export-id",
                "export-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "localhost:18081",
                "--origin",
                ORIGIN,
                "--export-id",
                "export-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--export-id",
                "export-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn from_args_refuses_body_consumer_size_and_unknown_flags() {
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
            "{}",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", LINEAGEWEAVE_CONSUMER_CODE),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            get_args("127.0.0.1:18081", "export-1", "unpublished"),
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            get_args(
                "127.0.0.1:18081",
                &"e".repeat(EXPORT_RETRIEVAL_ID_MAX_LEN + 1),
                NARUON_CONSUMER_CODE,
            ),
            "",
        )
        .unwrap_err(),
        ApiError::LimitExceeded
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--idempotency-key",
                "export-idem-1",
                "--export-id",
                "export-1",
            ],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn compose_is_typed_https_get_without_credentials() {
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_export_retrieval_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/exports/export-1 HTTP/1.1"));
    assert!(http.contains("tepp-consumer: naruon"));
    assert!(http.contains("content-length: 0"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("idempotency-key"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
    assert!(!http.contains("/analysis-runs"));
}

#[test]
fn naruon_cli_retrieves_minted_export_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    let minted = mint_export(&mut service, "cli-export-idem-1");
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(
            "127.0.0.1:18081",
            minted.export_id.as_str(),
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("naruon");
    let got = dispatch_export_retrieval_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_export_retrieval_cli_stdout(&invocation, &got).expect("out");
    let payload = ExportRetrieval::from_json(&stdout).expect("retrieval");
    assert_eq!(payload, minted);
    assert_eq!(payload.artifact_id, "artifact-cli-1");
    assert_eq!(payload.decision_code, "purpose_bound_export_allowed");
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    assert!(!stdout.contains("tenant_workspace_id"));
    assert!(!stdout.contains("principal_id"));
    assert!(!stdout.contains("includes_source_text"));
    assert!(!stdout.contains("source_text"));

    let missing = ExportRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "missing-export", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied = dispatch_export_retrieval_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400, "{}", denied.body);
    let denied_stdout = render_export_retrieval_cli_stdout(&missing, &denied).expect("err");
    assert!(denied_stdout.contains("invalid_wire_payload"));
    assert!(!denied_stdout.contains(SCHEMA));

    let mut naruon = NaruonLiveService::new();
    let composed = compose_export_retrieval_cli_http(&invocation).expect("composed");
    assert_eq!(naruon.handle_http_request(&composed).status_code, 400);
}

#[test]
fn render_refuses_metrics_schema_and_identity_mismatch() {
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "export-2", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_export_retrieval_cli_stdout(
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
        render_export_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"export_id":"export-2","artifact_id":"a","decision_code":"purpose_bound_export_allowed","purpose":"modular_service_consumer","idempotency_key":"k","rmse":1.0}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_export_retrieval_cli_stdout(
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
    assert_eq!(
        render_export_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"export_id":"export-1","artifact_id":"a","decision_code":"purpose_bound_export_allowed","purpose":"modular_service_consumer","idempotency_key":"k"}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn loopback_http1_refuses_non_get_and_collection_paths() {
    let host = "127.0.0.1:18081";
    let exchange = naruon_export_retrieval_exchange(ORIGIN, "export-1").expect("ex");
    let ok = loopback_http1_from_export_retrieval_exchange(&exchange, host).expect("ok");
    assert!(ok.starts_with("GET /v1/exports/export-1 HTTP/1.1"));
    let mut posted = exchange.clone();
    posted.method = "POST";
    assert_eq!(
        loopback_http1_from_export_retrieval_exchange(&posted, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut collection = exchange.clone();
    collection.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}");
    assert_eq!(
        loopback_http1_from_export_retrieval_exchange(&collection, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let mut extra = exchange;
    extra.target_url = format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1/extra");
    assert_eq!(
        loopback_http1_from_export_retrieval_exchange(&extra, host).unwrap_err(),
        ApiError::InvalidWirePayload
    );
    let credentialed = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1"),
        headers: vec![("authorization".into(), "secret".into())],
        body: String::new(),
    };
    assert_eq!(
        loopback_http1_from_export_retrieval_exchange(&credentialed, host).unwrap_err(),
        ApiError::AuthorizationDenied
    );
}

#[test]
fn execute_over_tcp_and_stdin_reader() {
    let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
    let addr = service.local_addr().expect("addr").to_string();
    let minted = mint_export(&mut service, "cli-export-tcp");
    let handle = std::thread::spawn(move || {
        drop(service.serve_one());
    });
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(
            addr.as_str(),
            minted.export_id.as_str(),
            NARUON_CONSUMER_CODE,
        ),
        "",
    )
    .expect("tcp");
    let response = execute_export_retrieval_cli(&invocation).expect("execute");
    assert_eq!(response.status_code, 200, "{}", response.body);
    let stdout = render_export_retrieval_cli_stdout(&invocation, &response).expect("stdout");
    assert_eq!(ExportRetrieval::from_json(&stdout).expect("parsed"), minted);
    handle.join().expect("join");
    let empty = read_export_retrieval_cli_stdin(true, std::io::empty()).expect("tty");
    assert!(empty.is_empty());
    let piped = read_export_retrieval_cli_stdin(false, std::io::Cursor::new(b"")).expect("pipe");
    assert!(piped.is_empty());
}

/// Bind an ephemeral loopback listener, hand back one raw HTTP/1.1 response on
/// the first accepted connection, then close it so the client's read-to-end
/// observes EOF. Returns the bound `host:port`.
fn respond_once(response: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr").to_string();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            // Drain the client's request before writing so the OS sends a
            // clean FIN (not RST) when this thread drops the stream.
            let mut sink = [0_u8; 4096];
            let _ = stream.read(&mut sink);
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    addr
}

#[test]
fn parse_flags_refuses_bare_tokens_and_duplicate_or_missing_values() {
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(["get", "not-a-flag"], "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(["get", "--host"], "").unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        ExportRetrievalCliInvocation::from_args(
            ["get", "--host", "127.0.0.1:1", "--host", "127.0.0.1:2"],
            "",
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn loopback_http1_filters_host_and_content_length_headers() {
    let exchange = NaruonHttpExchange {
        method: "GET",
        target_url: format!("{ORIGIN}{NARUON_EXPORT_PATH}/export-1"),
        headers: vec![
            ("Host".into(), "attacker.example".into()),
            ("Content-Length".into(), "999".into()),
            ("tepp-consumer".into(), NARUON_CONSUMER_CODE.into()),
        ],
        body: String::new(),
    };
    let request =
        loopback_http1_from_export_retrieval_exchange(&exchange, "127.0.0.1:18081").expect("ok");
    assert_eq!(request.matches("Host:").count(), 1);
    assert_eq!(
        request
            .to_ascii_lowercase()
            .matches("content-length:")
            .count(),
        1
    );
    assert!(!request.contains("999"));
    assert!(!request.contains("attacker.example"));
    assert!(request.contains("tepp-consumer"));
}

#[test]
fn render_refuses_non_200_success_status() {
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args("127.0.0.1:18081", "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        render_export_retrieval_cli_stdout(
            &invocation,
            &NaruonLiveResponse {
                status_code: 202,
                reason_phrase: "Accepted",
                body: r#"{"contract_version":1}"#.into(),
            }
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_rejects_non_http11_status_line() {
    let addr = respond_once("HTTP/1.0 200 OK\r\ncontent-length: 2\r\n\r\n{}".to_owned());
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(addr.as_str(), "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        execute_export_retrieval_cli(&invocation).unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_maps_every_published_reason_phrase() {
    for (code, body) in [
        (202u16, "{}"),
        (400, r#"{"error":"invalid_wire_payload"}"#),
        (403, r#"{"error":"authorization_denied"}"#),
        (413, r#"{"error":"limit_exceeded"}"#),
        (422, r#"{"error":"invalid_wire_payload"}"#),
    ] {
        let reason = match code {
            202 => "Accepted",
            400 => "Bad Request",
            403 => "Forbidden",
            413 => "Payload Too Large",
            422 => "Unprocessable Entity",
            _ => unreachable!("test only iterates published codes"),
        };
        let response = format!(
            "HTTP/1.1 {code} {reason}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        let addr = respond_once(response);
        let invocation = ExportRetrievalCliInvocation::from_args(
            get_args(addr.as_str(), "export-1", NARUON_CONSUMER_CODE),
            "",
        )
        .expect("invocation");
        let got = execute_export_retrieval_cli(&invocation).expect("response");
        assert_eq!(got.status_code, code);
        assert_eq!(got.reason_phrase, reason);
        assert_eq!(got.body, body);
    }
}

#[test]
fn execute_over_tcp_rejects_unpublished_status_code() {
    let addr = respond_once(
        "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 2\r\n\r\n{}".to_owned(),
    );
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(addr.as_str(), "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        execute_export_retrieval_cli(&invocation).unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_rejects_duplicate_content_length_header() {
    let addr = respond_once(
        "HTTP/1.1 200 OK\r\ncontent-length: 2\r\ncontent-length: 2\r\n\r\n{}".to_owned(),
    );
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(addr.as_str(), "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        execute_export_retrieval_cli(&invocation).unwrap_err(),
        ApiError::InvalidWirePayload
    );
}

#[test]
fn execute_over_tcp_rejects_content_length_body_mismatch() {
    let addr = respond_once("HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\n{}".to_owned());
    let invocation = ExportRetrievalCliInvocation::from_args(
        get_args(addr.as_str(), "export-1", NARUON_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    assert_eq!(
        execute_export_retrieval_cli(&invocation).unwrap_err(),
        ApiError::InvalidWirePayload
    );
}
