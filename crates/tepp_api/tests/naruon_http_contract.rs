//! naruon may call TEPP only through versioned HTTP interchange (ADR 0011).

use std::path::PathBuf;
use tepp_api::{
    naruon_analysis_run_exchange, naruon_analysis_run_exchange_with_headers,
    naruon_export_exchange, naruon_may_claim_tepp_inference, AnalysisRunRequest, AnalyticalPurpose,
    ApiError, ExportAuthorizationRequest, NARUON_ANALYSIS_RUN_PATH,
};

fn sample_run() -> AnalysisRunRequest {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("examples");
    path.push("naruon_modular_analysis_run_request_v1.json");
    let payload = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing example {}: {error}", path.display()));
    AnalysisRunRequest::from_json(&payload).expect("committed naruon example")
}

#[test]
fn analysis_run_exchange_posts_versioned_json_without_credentials() {
    let exchange =
        naruon_analysis_run_exchange("https://tepp.example.test", &sample_run()).expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert_eq!(
        exchange.target_url,
        format!("https://tepp.example.test{NARUON_ANALYSIS_RUN_PATH}")
    );
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "content-type" && value == "application/json"));
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "tepp-consumer" && value == "naruon"));
    assert!(!exchange
        .headers
        .iter()
        .any(|(name, _)| name == "authorization"
            || name.contains("token")
            || name.contains("copilot")));
    let decoded = AnalysisRunRequest::from_json(&exchange.body).expect("body");
    assert_eq!(decoded.tenant_workspace_id, "naruon-tenant-workspace-demo");
}

#[test]
fn table_access_and_non_https_origins_fail_closed() {
    let run = sample_run();
    for origin in [
        "",
        "postgres://tepp.example.test/tepp",
        "postgresql://tepp.example.test/tepp",
        "jdbc:postgresql://tepp.example.test/tepp",
        "https://tepp.example.test/sql",
        "https://tepp.example.test/tables/document_record",
        "http://tepp.example.test",
        "https://tepp.example.test/v1/analysis-runs'; DROP",
    ] {
        assert_eq!(
            naruon_analysis_run_exchange(origin, &run),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
}

#[test]
fn review_and_copilot_headers_are_authorization_denied() {
    let run = sample_run();
    assert_eq!(
        naruon_analysis_run_exchange_with_headers(
            "https://tepp.example.test",
            &run,
            &[("authorization", "Bearer review-agent")]
        ),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        naruon_analysis_run_exchange_with_headers(
            "https://tepp.example.test",
            &run,
            &[("x-copilot-github-token", "ghs_example")]
        ),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn export_exchange_requires_modular_service_consumer() {
    let allowed = ExportAuthorizationRequest {
        tenant_workspace_id: "naruon-tenant-workspace-demo".into(),
        principal_id: "naruon-service".into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "tepp-export-demo-001".into(),
        includes_source_text: false,
    };
    let exchange = naruon_export_exchange("https://tepp.example.test", &allowed).expect("export");
    assert_eq!(exchange.method, "POST");
    assert!(exchange.target_url.ends_with("/v1/exports"));

    let denied = ExportAuthorizationRequest {
        purpose: AnalyticalPurpose::OperationalMonitoring,
        ..allowed
    };
    assert_eq!(
        naruon_export_exchange("https://tepp.example.test", &denied),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn lexical_heuristics_cannot_claim_tepp_inference() {
    assert!(naruon_may_claim_tepp_inference("tepp_topic_measurement").is_ok());
    assert_eq!(
        naruon_may_claim_tepp_inference("tfidf"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        naruon_may_claim_tepp_inference("bm25"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        naruon_may_claim_tepp_inference("keyword"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        naruon_may_claim_tepp_inference(""),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn allowed_extra_headers_are_forwarded_on_analysis_run_exchange() {
    let run = sample_run();
    let exchange = naruon_analysis_run_exchange_with_headers(
        "https://tepp.example.test",
        &run,
        &[
            ("x-request-id", "naruon-trace-001"),
            ("x-correlation-id", "corr-42"),
        ],
    )
    .expect("non-credential headers are allowed");
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "x-request-id" && value == "naruon-trace-001"));
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "x-correlation-id" && value == "corr-42"));
}

#[test]
fn host_names_that_imply_table_access_fail_closed() {
    let run = sample_run();
    // Path-bearing origins are refused by host-shape checks; these hosts pass
    // shape validation and exercise the table-access token refusal arm.
    for origin in ["https://postgres.example.test", "https://jdbc.example.test"] {
        assert_eq!(
            naruon_analysis_run_exchange(origin, &run),
            Err(ApiError::InvalidWirePayload),
            "origin={origin}"
        );
    }
}
