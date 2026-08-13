//! contextual-orchestrator interchange refuses table access and repo-write tokens.

use tepp_api::{
    ApiError, ORCHESTRATOR_INTERPRETATION_PATH, orchestrator_interpretation_exchange,
    refuse_orchestrator_as_scientific_acceptance, refuse_repository_write_secret,
};

#[test]
fn https_post_has_no_credentials_and_does_not_own_science() {
    let exchange = orchestrator_interpretation_exchange(
        "orchestrator.example",
        "idem-unitize-1",
        r#"{"task":"semantic_unitization","snapshot_id":"snap-1"}"#,
    )
    .expect("https");
    assert_eq!(exchange.method(), "POST");
    assert_eq!(
        exchange.target_url(),
        format!("https://orchestrator.example{ORCHESTRATOR_INTERPRETATION_PATH}")
    );
    assert!(ORCHESTRATOR_INTERPRETATION_PATH.contains("interpretation"));
    let header_names: Vec<_> = exchange
        .headers()
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(header_names.contains(&"content-type"));
    assert!(header_names.contains(&"tepp-consumer"));
    assert!(
        !header_names
            .iter()
            .any(|name| name.contains("authorization")
                || name.contains("cookie")
                || name.contains("token")
                || name.contains("copilot")
                || name.contains("github"))
    );
    assert!(!exchange.body().contains("COPILOT_GITHUB_TOKEN"));
    assert_eq!(
        refuse_orchestrator_as_scientific_acceptance(),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn table_access_and_non_https_origins_fail_closed() {
    for host in [
        "",
        "bad host",
        "h@x",
        "h/sql",
        "h?x",
        "h#x",
        "postgres.db",
        "jdbc.db",
    ] {
        assert_eq!(
            orchestrator_interpretation_exchange(host, "idem-1", "{}"),
            Err(ApiError::AuthorizationDenied)
        );
    }
    assert_eq!(
        orchestrator_interpretation_exchange("tables.example", "idem-1", "{}"),
        Err(ApiError::AuthorizationDenied)
    );
}

#[test]
fn repository_write_and_review_agent_secrets_are_refused() {
    assert_eq!(
        refuse_repository_write_secret("COPILOT_GITHUB_TOKEN"),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        refuse_repository_write_secret("review-agent-github-token"),
        Err(ApiError::AuthorizationDenied)
    );
    refuse_repository_write_secret("NVIDIA_NIM_API_KEY").expect("nim allowed as name");
    assert_eq!(
        orchestrator_interpretation_exchange("", "idem-1", "{}"),
        Err(ApiError::AuthorizationDenied)
    );
    assert_eq!(
        orchestrator_interpretation_exchange("ok.example", "", "{}"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        orchestrator_interpretation_exchange("ok.example", "idem-1", ""),
        Err(ApiError::InvalidWirePayload)
    );
}
