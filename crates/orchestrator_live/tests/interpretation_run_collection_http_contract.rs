//! Contract tests for loopback `GET /v1/interpretation-runs`.

use orchestrator_live::{
    INTERPRETATION_RUN_PATH, InterpretationRunCollection, InterpretationRunCollectionItem,
    OrchestrationMode, OrchestratorLiveError,
    contextual_orchestrator_interpretation_run_collection_exchange,
    is_interpretation_run_collection_path, refuse_metrics_on_interpretation_run_collection_payload,
};

#[test]
fn interpretation_run_collection_is_metric_free_get_without_credentials() {
    assert!(is_interpretation_run_collection_path(
        INTERPRETATION_RUN_PATH
    ));
    assert!(!is_interpretation_run_collection_path(
        "/v1/interpretation-runs/extra"
    ));
    let item = InterpretationRunCollectionItem::new(
        "orch-run-1",
        "idem-1",
        OrchestrationMode::Direct,
        "hypothetical",
        false,
    )
    .expect("item");
    let page = InterpretationRunCollection::new(vec![item], None).expect("page");
    let json = page.to_json().expect("json");
    assert!(!json.contains("rmse"));
    assert!(!json.contains("tepp.scientific_acceptance.v1"));
    assert!(!json.contains("evidence_span_ids"));
    assert!(!json.contains("tenant_workspace_id"));
    assert!(!json.contains("compute_budget_tokens"));
    assert!(!json.contains("causal_score"));
    assert!(json.contains("\"claim_status\":\"hypothetical\""));
    assert!(json.contains("\"scientific_authority\":false"));
    let exchange = contextual_orchestrator_interpretation_run_collection_exchange(
        "https://tepp.example.test",
        None,
        None,
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange.target_url.ends_with("/v1/interpretation-runs"));
    assert!(exchange.body.is_empty());
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
                || name.eq_ignore_ascii_case("idempotency-key"))
    );
}

#[test]
fn interpretation_run_collection_refuses_metrics_evidence_and_insecure_origins() {
    assert_eq!(
        refuse_metrics_on_interpretation_run_collection_payload(r#"{"rmse":1.0}"#),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_interpretation_run_collection_payload(r#"{"evidence_text":"x"}"#),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_interpretation_run_collection_payload(r#"{"findings":[]}"#),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        contextual_orchestrator_interpretation_run_collection_exchange(
            "http://insecure.example",
            None,
            None
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert!(!is_interpretation_run_collection_path("/v1/analysis-runs"));
    assert!(!is_interpretation_run_collection_path(
        "/v1/project-histories"
    ));
}
