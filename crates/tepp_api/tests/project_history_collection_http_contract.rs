//! Contract tests for loopback `GET /v1/project-histories`.

use tepp_api::{
    ApiError, PROJECT_HISTORY_PATH, ProjectHistoryCollection, ProjectHistoryCollectionItem,
    is_project_history_collection_path, lineageweave_project_history_collection_exchange,
    refuse_metrics_on_project_history_collection_payload,
};

#[test]
fn project_history_collection_is_metric_free_get_without_credentials() {
    assert!(is_project_history_collection_path(PROJECT_HISTORY_PATH));
    let item = ProjectHistoryCollectionItem::new(
        "project",
        "idem-1",
        "2026-08-19T23:59:59Z",
        "temporal_association_only",
    )
    .expect("item");
    let page = ProjectHistoryCollection::new(vec![item], None).expect("page");
    let json = page.to_json().expect("json");
    assert!(!json.contains("rmse"));
    assert!(!json.contains("tepp.scientific_acceptance.v1"));
    assert!(!json.contains("evidence_text"));
    assert!(!json.contains("findings"));
    let exchange = lineageweave_project_history_collection_exchange(
        "https://tepp.example.test",
        "tenant-a",
        None,
        None,
    )
    .expect("exchange");
    assert_eq!(exchange.method, "GET");
    assert!(exchange.target_url.ends_with("/v1/project-histories"));
    assert!(
        !exchange
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
    );
}

#[test]
fn project_history_collection_refuses_metrics_evidence_and_insecure_origins() {
    assert_eq!(
        refuse_metrics_on_project_history_collection_payload(r#"{"rmse":1.0}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        refuse_metrics_on_project_history_collection_payload(r#"{"evidence_text":"x"}"#),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        lineageweave_project_history_collection_exchange(
            "http://insecure.example",
            "tenant-a",
            None,
            None,
        ),
        Err(ApiError::InvalidWirePayload)
    );
    assert!(!is_project_history_collection_path("/v1/analysis-runs"));
}
