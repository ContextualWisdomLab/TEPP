//! Contract tests for contextual-orchestrator interpretation-run cancel.

use orchestrator_live::{
    contextual_orchestrator_interpretation_run_cancel_exchange, interpretation_run_cancel_path_id,
    OrchestratorLiveError, CONTEXTUAL_ORCHESTRATOR_CONSUMER_CODE,
};

#[test]
fn interpretation_run_cancel_is_metric_free_post_without_credentials() {
    let exchange = contextual_orchestrator_interpretation_run_cancel_exchange(
        "https://tepp.example.test",
        "orch-live-idem-001",
    )
    .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert!(exchange
        .target_url
        .ends_with("/v1/interpretation-runs/orch-live-idem-001/cancel"));
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
        interpretation_run_cancel_path_id("/v1/interpretation-runs/orch-live-idem-001/cancel")
            .expect("id"),
        "orch-live-idem-001"
    );
}

#[test]
fn interpretation_run_cancel_refuses_collection_get_by_id_and_insecure_origins() {
    assert_eq!(
        interpretation_run_cancel_path_id("/v1/interpretation-runs"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        interpretation_run_cancel_path_id("/v1/interpretation-runs/idem-a"),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
    assert_eq!(
        contextual_orchestrator_interpretation_run_cancel_exchange(
            "http://tepp.example.test",
            "idem-a"
        ),
        Err(OrchestratorLiveError::InvalidWirePayload)
    );
}
