//! `LineageWeave` project-history requests remain cutoff-safe and non-causal.

use tepp_api::{
    ApiError, LINEAGEWEAVE_CONSUMER_CODE, PROJECT_HISTORY_CONTRACT_VERSION, PROJECT_HISTORY_PATH,
    ProjectHistoryEvent, ProjectHistoryProjection, ProjectHistoryRequest,
    lineageweave_project_history_exchange, project_history_projection,
};

fn event(
    event_id: &str,
    event_type_code: &str,
    event_title: &str,
    occurred_at: &str,
    source_post_id: &str,
    actor_ids: &[&str],
) -> ProjectHistoryEvent {
    ProjectHistoryEvent {
        event_id: event_id.into(),
        event_type_code: event_type_code.into(),
        event_title: event_title.into(),
        occurred_at: occurred_at.into(),
        available_at: occurred_at.into(),
        source_post_id: source_post_id.into(),
        evidence_text: format!("evidence for {event_title}"),
        actor_ids: actor_ids.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn sample_request() -> ProjectHistoryRequest {
    ProjectHistoryRequest {
        contract_version: PROJECT_HISTORY_CONTRACT_VERSION,
        idempotency_key: "lineageweave-project-acme-voc-1".into(),
        tenant_workspace_id: "tenant-demo".into(),
        project_key: "project-acme".into(),
        project_name: "Acme renewal".into(),
        knowledge_cutoff: "2026-08-19T23:59:59Z".into(),
        focus_event_id: "event-voc".into(),
        events: vec![
            event(
                "event-rebid",
                "rebid_started",
                "Rebid",
                "2026-08-10T09:00:00Z",
                "post-rebid",
                &["person-3"],
            ),
            event(
                "event-award",
                "contract_awarded",
                "Contract award",
                "2022-03-11T09:00:00Z",
                "post-award",
                &["person-1"],
            ),
            event(
                "event-spec",
                "specification_changed",
                "Specification change",
                "2023-06-15T09:00:00Z",
                "post-spec",
                &["person-1", "person-2"],
            ),
            event(
                "event-delivery",
                "delivered",
                "Delivery",
                "2024-02-20T09:00:00Z",
                "post-delivery",
                &["person-2"],
            ),
            event(
                "event-handoff",
                "handoff_recorded",
                "Operational handoff",
                "2024-03-01T09:00:00Z",
                "post-handoff",
                &["person-2", "person-3"],
            ),
            event(
                "event-voc",
                "voc_received",
                "VOC received",
                "2026-07-30T09:00:00Z",
                "post-voc",
                &["person-3"],
            ),
        ],
    }
}

#[test]
fn projection_orders_the_cycle_and_explains_only_explicit_temporal_evidence() {
    let projection = project_history_projection(&sample_request()).expect("projection");

    assert_eq!(
        projection.contract_version,
        PROJECT_HISTORY_CONTRACT_VERSION
    );
    assert_eq!(projection.focus_event_id, "event-voc");
    assert_eq!(
        projection.knowledge_cutoff,
        sample_request().knowledge_cutoff
    );
    assert_eq!(projection.inference_status, "temporal_association_only");
    assert_eq!(projection.participant_count, 3);
    assert_eq!(
        projection
            .events
            .iter()
            .map(|item| item.event_type_code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "contract_awarded",
            "specification_changed",
            "delivered",
            "handoff_recorded",
            "voc_received",
            "rebid_started",
        ]
    );
    let finding_codes = projection
        .findings
        .iter()
        .map(|finding| finding.finding_code.as_str())
        .collect::<Vec<_>>();
    assert!(finding_codes.contains(&"specification_change_before_focus"));
    assert!(finding_codes.contains(&"handoff_before_focus"));
    assert!(finding_codes.contains(&"rebid_after_focus"));
    assert!(finding_codes.contains(&"specification_change_and_handoff_before_focus"));
    assert!(
        projection
            .findings
            .iter()
            .all(|finding| !finding.evidence_post_ids.is_empty())
    );
}

#[test]
fn projection_rejects_future_evidence_duplicates_and_unknown_json_fields() {
    let mut future = sample_request();
    future.events[0].available_at = "2026-08-20T00:00:00Z".into();
    assert_eq!(
        project_history_projection(&future),
        Err(ApiError::InvalidWirePayload)
    );

    let mut duplicate = sample_request();
    duplicate.events[1].event_id = duplicate.events[0].event_id.clone();
    assert_eq!(
        project_history_projection(&duplicate),
        Err(ApiError::InvalidWirePayload)
    );

    let json = sample_request().to_json().expect("json");
    let hostile = json.replacen('{', "{\"unpublished_causal_score\":1,", 1);
    assert_eq!(
        ProjectHistoryRequest::from_json(&hostile),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn request_json_with_explicit_limit_round_trips_a_valid_contract() {
    let request = sample_request();
    let payload = request.to_json().expect("request json");
    let parsed = ProjectHistoryRequest::from_json_with_limit(&payload, payload.len() + 1)
        .expect("request with explicit limit");
    assert_eq!(parsed, request);
}

#[test]
fn lineageweave_exchange_uses_the_versioned_credential_free_tepp_path() {
    let exchange =
        lineageweave_project_history_exchange("https://tepp.example.test", &sample_request())
            .expect("exchange");

    assert_eq!(
        exchange.target_url,
        format!("https://tepp.example.test{PROJECT_HISTORY_PATH}")
    );
    assert_eq!(exchange.method, "POST");
    assert!(
        exchange
            .headers
            .contains(&("tepp-consumer".into(), LINEAGEWEAVE_CONSUMER_CODE.into()))
    );
    assert!(
        exchange
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("authorization"))
    );
    assert_eq!(
        lineageweave_project_history_exchange("http://tepp.example.test", &sample_request()),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn projection_response_revalidates_cutoff_order_findings_and_payload_size() {
    let projection = project_history_projection(&sample_request()).expect("projection");
    let payload = projection.to_json().expect("projection json");
    assert_eq!(
        ProjectHistoryProjection::from_json_with_limit(&payload, payload.len() - 1),
        Err(ApiError::LimitExceeded)
    );

    let mut too_many: serde_json::Value = serde_json::from_str(&payload).expect("value");
    let events = too_many["events"].as_array_mut().expect("events");
    let template = events[0].clone();
    while events.len() <= 128 {
        events.push(template.clone());
    }
    let too_many_json = serde_json::to_string(&too_many).expect("too many json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&too_many_json),
        Err(ApiError::LimitExceeded)
    );

    let mut future_cutoff: serde_json::Value = serde_json::from_str(&payload).expect("value");
    future_cutoff["knowledge_cutoff"] = serde_json::Value::String("2999-01-01T00:00:00Z".into());
    let future_cutoff_json = serde_json::to_string(&future_cutoff).expect("future cutoff json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&future_cutoff_json),
        Err(ApiError::InvalidWirePayload)
    );

    let mut duplicate: serde_json::Value = serde_json::from_str(&payload).expect("value");
    duplicate["events"][1]["event_id"] = duplicate["events"][0]["event_id"].clone();
    let duplicate_json = serde_json::to_string(&duplicate).expect("duplicate json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&duplicate_json),
        Err(ApiError::InvalidWirePayload)
    );

    let mut reversed: serde_json::Value = serde_json::from_str(&payload).expect("value");
    reversed["events"][1]["occurred_at"] = serde_json::Value::String("2020-01-01T00:00:00Z".into());
    let reversed_json = serde_json::to_string(&reversed).expect("reversed json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&reversed_json),
        Err(ApiError::InvalidWirePayload)
    );

    let mut equal_time: serde_json::Value = serde_json::from_str(&payload).expect("value");
    equal_time["events"][1]["occurred_at"] = equal_time["events"][0]["occurred_at"].clone();
    equal_time["events"][1]["available_at"] = equal_time["events"][0]["available_at"].clone();
    let equal_time_json = serde_json::to_string(&equal_time).expect("equal time json");
    assert!(ProjectHistoryProjection::from_json(&equal_time_json).is_ok());

    equal_time["events"][1]["event_id"] = serde_json::Value::String("event-aaa".into());
    let equal_id_regression = serde_json::to_string(&equal_time).expect("equal id json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&equal_id_regression),
        Err(ApiError::InvalidWirePayload)
    );

    let mut value: serde_json::Value = serde_json::from_str(&payload).expect("value");
    value["events"][0]["available_at"] = serde_json::Value::String("2026-08-20T00:00:00Z".into());
    let future = serde_json::to_string(&value).expect("future json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&future),
        Err(ApiError::InvalidWirePayload)
    );

    let mut value: serde_json::Value = serde_json::from_str(&payload).expect("value");
    value["findings"] = serde_json::json!([{
        "finding_code": "causal_score",
        "summary": "causal",
        "related_event_ids": ["event-award"],
        "evidence_post_ids": ["post-award"]
    }]);
    let fabricated = serde_json::to_string(&value).expect("fabricated json");
    assert_eq!(
        ProjectHistoryProjection::from_json(&fabricated),
        Err(ApiError::InvalidWirePayload)
    );
}
