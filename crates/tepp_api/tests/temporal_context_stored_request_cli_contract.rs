//! Contract tests for the `LineageWeave` temporal-context stored-request loopback CLI.

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    NaruonHttpExchange, NaruonLiveResponse, NaruonLiveService, TEMPORAL_CONTEXT_PATH,
    TemporalContextRequest, TemporalContextStoredRequestCliInvocation,
    TemporalContextStoredRequestCliVerb, compose_temporal_context_stored_request_cli_http,
    dispatch_temporal_context_stored_request_cli,
    lineageweave_temporal_context_stored_request_exchange,
    loopback_http1_from_temporal_context_stored_request_exchange,
    read_temporal_context_stored_request_cli_stdin,
    render_temporal_context_stored_request_cli_stdout,
};

const ORIGIN: &str = "https://tepp.example.test";
const SCHEMA: &str = "tepp.scientific_acceptance.v1";
const TEMPORAL_BODY: &str = r#"{"contract_version":1,"consumer_code":"lineageweave","knowledge_cutoff":"2026-08-20T00:00:00Z","subject_post_id":null,"events":[{"event_id":"event-1","source_post_id":"post-1","event_type_code":"order_awarded","event_label":"Order awarded","event_time":"2026-08-01T09:00:00Z","available_time":"2026-08-01T10:00:00Z","project_reference":null,"actor_references":["actor-1"]}]}"#;

fn post_http(idempotency_key: &str) -> String {
    format!(
        "POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\ntepp-consumer: {LINEAGEWEAVE_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {idempotency_key}\r\ncontent-length: {}\r\n\r\n{TEMPORAL_BODY}",
        TEMPORAL_BODY.len()
    )
}

fn get_args<'a>(host: &'a str, idempotency_key: &'a str, consumer: &'a str) -> [&'a str; 9] {
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
    ]
}

#[test]
fn verbs_and_from_args_fail_closed() {
    assert_eq!(
        TemporalContextStoredRequestCliVerb::parse("get").expect("get"),
        TemporalContextStoredRequestCliVerb::Get
    );
    assert_eq!(TemporalContextStoredRequestCliVerb::Get.as_str(), "get");
    assert_eq!(
        TemporalContextStoredRequestCliVerb::parse("list"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            get_args("8.8.8.8:80", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            get_args("localhost:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                "http://tepp.example.test",
                "--idempotency-key",
                "idem-a"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            [
                "get",
                "--host",
                "127.0.0.1:18081",
                "--origin",
                ORIGIN,
                "--authorization",
                "secret",
                "--idempotency-key",
                "idem-a"
            ],
            ""
        )
        .unwrap_err(),
        ApiError::AuthorizationDenied
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", NARUON_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
            "{}"
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        TemporalContextStoredRequestCliInvocation::from_args(
            get_args("127.0.0.1:18081", "a/b", LINEAGEWEAVE_CONSUMER_CODE),
            ""
        )
        .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert!(
        read_temporal_context_stored_request_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty()
    );
}

#[test]
fn compose_is_typed_https_get_without_credentials() {
    let invocation = TemporalContextStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let http = compose_temporal_context_stored_request_cli_http(&invocation).expect("http");
    assert!(http.starts_with("GET /v1/temporal-context/idem-a/request HTTP/1.1"));
    assert!(http.contains("tepp-consumer: lineageweave"));
    assert!(http.contains("content-length: 0"));
    assert!(!http.contains("idempotency-key:"));
    assert!(!http.to_ascii_lowercase().contains("authorization"));
    assert!(!http.contains("rmse"));
    assert!(!http.contains(SCHEMA));
}

#[test]
fn lineageweave_cli_retrieves_stored_request_and_naruon_live_stays_post_only() {
    let mut service = AnalysisRunLiveService::new();
    assert_eq!(
        service
            .handle_http_request(&post_http("idem-a"))
            .status_code,
        200
    );
    let invocation = TemporalContextStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "idem-a", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("invocation");
    let got = dispatch_temporal_context_stored_request_cli(&mut service, &invocation).expect("get");
    assert_eq!(got.status_code, 200, "{}", got.body);
    let stdout = render_temporal_context_stored_request_cli_stdout(&invocation, &got).expect("out");
    let stored = TemporalContextRequest::from_json(&stdout).expect("stored");
    let original = TemporalContextRequest::from_json(TEMPORAL_BODY).expect("original");
    assert_eq!(stored, original);
    assert!(stdout.contains("event_label"));
    assert!(!stdout.contains("rmse"));
    assert!(!stdout.contains(SCHEMA));
    let missing = TemporalContextStoredRequestCliInvocation::from_args(
        get_args("127.0.0.1:18081", "missing", LINEAGEWEAVE_CONSUMER_CODE),
        "",
    )
    .expect("missing");
    let denied =
        dispatch_temporal_context_stored_request_cli(&mut service, &missing).expect("denied");
    assert_eq!(denied.status_code, 400);
    assert!(
        render_temporal_context_stored_request_cli_stdout(&missing, &denied)
            .expect("err")
            .contains("invalid_wire_payload")
    );
    let mut naruon = NaruonLiveService::new();
    assert_eq!(
        naruon
            .handle_http_request(
                &compose_temporal_context_stored_request_cli_http(&invocation).expect("composed")
            )
            .status_code,
        400
    );
    let exchange =
        lineageweave_temporal_context_stored_request_exchange(ORIGIN, "idem-a").expect("ex");
    let posted = NaruonHttpExchange {
        method: "POST",
        target_url: exchange.target_url,
        headers: exchange.headers,
        body: exchange.body,
    };
    assert_eq!(
        loopback_http1_from_temporal_context_stored_request_exchange(&posted, "127.0.0.1:18081")
            .unwrap_err(),
        ApiError::InvalidWirePayload
    );
    assert_eq!(
        render_temporal_context_stored_request_cli_stdout(
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
}
