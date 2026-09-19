//! Contract tests for contextual-orchestrator interpretation-run GET-by-id.

use orchestrator_live::{
    contextual_orchestrator_interpretation_run_retrieval_exchange,
    interpretation_run_retrieval_path_id, OrchestratorLiveError,
    CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
};

#[test]
fn interpretation_run_retrieval_is_metric_free_get_without_credentials() {
    let exchange = contextual_orchestrator_interpretation_run_retrieval_exchange(
        "https://tepp.example.test",
        "orch-live-idem-001",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange
        .target_url
        .ends_with("/v1/interpretation-runs/orch-live-idem-001"));
    assert!(exchange.body.is_empty());
    assert!(exchange
        .headers
        .iter()
        .any(|(name, value)| name == "tepp-consumer"
            && value == CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE));
    assert!(!exchange
        .headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("authorization")
            || name.eq_ignore_ascii_case("idempotency-key")));
    assert_eq!(
        interpretation_run_retrieval_path_id("/v1/interpretation-runs/orch-live-idem-001")
            .expect("id"),
        "orch-live-idem-001"
    );
}

#[test]
fn interpretation_run_retrieval_refuses_collection_path_extra_segments_and_insecure_origins() {
    assert_eq!(
        interpretation_run_retrieval_path_id("/v1/interpretation-runs"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        interpretation_run_retrieval_path_id("/v1/interpretation-runs/idem-a/extra"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        contextual_orchestrator_interpretation_run_retrieval_exchange(
            "http://tepp.example.test",
            "idem-a"
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}
