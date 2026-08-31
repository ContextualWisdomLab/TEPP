//! Contract tests for loopback `GET /v1/project-histories/{idempotency_key}`.

use tepp_api::{
    ApiError, PROJECT_HISTORY_PATH, lineageweave_project_history_retrieval_exchange,
    project_history_retrieval_path_id, refuse_metrics_on_project_history_retrieval_payload,
};

#[test]
fn project_history_retrieval_is_metric_free_get_without_credentials() {
    let exchange =
        lineageweave_project_history_retrieval_exchange("https://tepp.example.test", "idem-a")
            .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(
        exchange
            .target_url
            .ends_with("/v1/project-histories/idem-a")
    );
    assert!(exchange.body.is_empty());
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
    );
    assert_eq!(
        project_history_retrieval_path_id("/v1/project-histories/idem-a").expect("id"),
        "idem-a"
    );
    assert_eq!(
        project_history_retrieval_path_id(PROJECT_HISTORY_PATH),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn project_history_retrieval_refuses_metrics_naruon_origins_and_collection_path() {
    assert_eq!(
        refuse_metrics_on_project_history_retrieval_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_project_history_retrieval_payload(r#"{"causal_score":1}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        lineageweave_project_history_retrieval_exchange("http://insecure.example", "idem-a"),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        project_history_retrieval_path_id("/v1/analysis-runs/idem-a"),
        Err(ApiError::InvalidWirePayload)
    );
}
