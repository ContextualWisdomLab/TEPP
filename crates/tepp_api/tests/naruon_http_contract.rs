//! naruon may call TEPP only through versioned HTTP interchange (ADR 0011).

use std::path::PathBuf;
use tepp_api::{
    AnalysisRunRequest, AnalyticalPurpose, ApiError, ExportAuthorizationRequest,
    NARUON_ANALYSIS_RUN_PATH, naruon_analysis_run_exchange,
    naruon_analysis_run_exchange_with_headers, naruon_export_exchange,
    naruon_may_claim_tepp_inference,
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
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "content-type" && value == "application/json")
    );
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "tepp-consumer" && value == "naruon")
    );
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name == "authorization"
                || name.contains("token")
                || name.contains("copilot"))
    );
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
    for name in ["Proxy-Authorization", "x-nim-key", "x-nvidia-session"] {
        assert_eq!(
            naruon_analysis_run_exchange_with_headers(
                "https://tepp.example.test",
                &run,
                &[(name, "secret")]
            ),
            Err(ApiError::AuthorizationDenied),
            "header={name}"
        );
    }
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
    let exchange = naruon_export_exchange("https://tepp.example.test", &allowed, "export-idem-001")
        .expect("export");
    assert_eq!(exchange.method, "POST");
    assert!(exchange.target_url.ends_with("/v1/exports"));
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "idempotency-key" && value == "export-idem-001")
    );

    let denied = ExportAuthorizationRequest {
        purpose: AnalyticalPurpose::OperationalMonitoring,
        ..allowed
    };
    assert_eq!(
        naruon_export_exchange("https://tepp.example.test", &denied, "export-idem-denied"),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn export_exchange_uses_per_export_idempotency_keys() {
    let principal = "naruon-service";
    let first = ExportAuthorizationRequest {
        tenant_workspace_id: "naruon-tenant-workspace-demo".into(),
        principal_id: principal.into(),
        purpose: AnalyticalPurpose::ModularServiceConsumer,
        artifact_id: "tepp-export-demo-001".into(),
        includes_source_text: false,
    };
    let second = ExportAuthorizationRequest {
        artifact_id: "tepp-export-demo-002".into(),
        ..first.clone()
    };
    let a = naruon_export_exchange("https://tepp.example.test", &first, "export-op-a")
        .expect("first export");
    let b = naruon_export_exchange("https://tepp.example.test", &second, "export-op-b")
        .expect("second export");
    let key_a = a
        .headers
        .iter()
        .find(|(name, _)| name == "idempotency-key")
        .map(|(_, value)| value.as_str())
        .expect("key a");
    let key_b = b
        .headers
        .iter()
        .find(|(name, _)| name == "idempotency-key")
        .map(|(_, value)| value.as_str())
        .expect("key b");
    assert_ne!(key_a, key_b);
    assert_ne!(key_a, principal);
    assert_ne!(key_b, principal);
    assert_eq!(
        naruon_export_exchange("https://tepp.example.test", &first, ""),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn reserved_standard_headers_cannot_be_redefined() {
    let run = sample_run();
    for (name, value) in [
        ("Content-Type", "text/plain"),
        ("tepp-consumer", "hostile"),
        ("tepp-contract-version", "0"),
        ("Idempotency-Key", "override"),
    ] {
        assert_eq!(
            naruon_analysis_run_exchange_with_headers(
                "https://tepp.example.test",
                &run,
                &[(name, value)]
            ),
            Err(ApiError::InvalidWirePayload),
            "header={name}"
        );
    }
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
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "x-request-id" && value == "naruon-trace-001")
    );
    assert!(
        exchange
            .headers
            .iter()
            .any(|(name, value)| name == "x-correlation-id" && value == "corr-42")
    );
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
