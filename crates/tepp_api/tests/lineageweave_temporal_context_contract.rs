//! RED contract for TEPP temporal evidence used by LineageWeave Ask surfaces.

use std::fmt::Write as _;

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
    TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY, TEMPORAL_CONTEXT_CONTRACT_VERSION,
    TEMPORAL_CONTEXT_PATH, TemporalContextEvent, TemporalContextRequest,
    TemporalContextResponse, build_temporal_context, lineageweave_temporal_context_exchange,
};

fn event(
    event_id: &str,
    post_id: &str,
    event_type: &str,
    label: &str,
    event_time: &str,
    available_time: &str,
    actors: &[&str],
) -> TemporalContextEvent {
    TemporalContextEvent {
        event_id: event_id.into(),
        source_post_id: post_id.into(),
        event_type_code: event_type.into(),
        event_label: label.into(),
        event_time: event_time.into(),
        available_time: available_time.into(),
        project_reference: Some("project-alpha".into()),
        actor_references: actors.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn request() -> TemporalContextRequest {
    TemporalContextRequest {
        contract_version: TEMPORAL_CONTEXT_CONTRACT_VERSION,
        consumer_code: LINEAGEWEAVE_CONSUMER_CODE.into(),
        knowledge_cutoff: "2026-08-20T00:00:00Z".into(),
        subject_post_id: Some("post-voc".into()),
        events: vec![
            event(
                "event-voc",
                "post-voc",
                "voc_received",
                "VOC 접수",
                "2026-08-01T09:00:00Z",
                "2026-08-01T10:00:00Z",
                &["actor-support"],
            ),
            event(
                "event-order",
                "post-order",
                "order_awarded",
                "수주",
                "2022-03-01T09:00:00Z",
                "2022-03-01T10:00:00Z",
                &["actor-sales"],
            ),
            event(
                "event-delivery",
                "post-delivery",
                "delivered",
                "납품",
                "2024-02-01T09:00:00Z",
                "2024-02-01T10:00:00Z",
                &["actor-operations"],
            ),
            event(
                "event-spec",
                "post-spec",
                "specification_changed",
                "사양 변경",
                "2023-06-01T09:00:00Z",
                "2023-06-01T10:00:00Z",
                &["actor-engineering"],
            ),
        ],
    }
}

fn http_request(body: &str) -> String {
    let mut value = format!("POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\n");
    for (name, header_value) in [
        ("Host", "127.0.0.1"),
        ("content-type", "application/json"),
        ("tepp-consumer", LINEAGEWEAVE_CONSUMER_CODE),
        ("tepp-contract-version", "1"),
        ("idempotency-key", "timeline-001"),
    ] {
        write!(value, "{name}: {header_value}\r\n").expect("header");
    }
    write!(value, "content-length: {}\r\n\r\n{body}", body.len()).expect("body");
    value
}

#[test]
fn temporal_context_orders_events_and_marks_only_candidate_gaps() {
    let response = build_temporal_context(&request()).expect("context");
    assert_eq!(response.contract_version, TEMPORAL_CONTEXT_CONTRACT_VERSION);
    assert_eq!(response.claim_boundary, TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY);
    assert_eq!(
        response
            .timeline_events
            .iter()
            .map(|item| item.event_id.as_str())
            .collect::<Vec<_>>(),
        vec!["event-order", "event-spec", "event-delivery", "event-voc"]
    );
    assert_eq!(
        response
            .timeline_events
            .iter()
            .map(|item| item.sequence_ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(response.timeline_events[3].is_subject);
    assert_eq!(response.temporal_relations.len(), 3);
    assert!(response.temporal_relations.iter().all(|item| item.relation_code == "before"));
    assert!(
        response
            .transition_gap_candidates
            .iter()
            .all(|item| item.evidence_status_code == "candidate_not_causal")
    );
    assert!(
        response
            .transition_gap_candidates
            .iter()
            .any(|item| item.from_event_id == "event-spec" && item.to_event_id == "event-delivery")
    );
    assert_eq!(
        response.source_post_ids,
        vec!["post-order", "post-spec", "post-delivery", "post-voc"]
    );
}

#[test]
fn temporal_context_rejects_leakage_duplicates_and_unpublished_consumers() {
    let mut future = request();
    future.events[0].available_time = "2026-09-01T00:00:00Z".into();
    assert_eq!(build_temporal_context(&future), Err(ApiError::InvalidWirePayload));

    let mut duplicate = request();
    duplicate.events[1].event_id = duplicate.events[0].event_id.clone();
    assert_eq!(build_temporal_context(&duplicate), Err(ApiError::InvalidWirePayload));

    let mut hostile = request();
    hostile.consumer_code = "unpublished-consumer".into();
    assert_eq!(build_temporal_context(&hostile), Err(ApiError::InvalidWirePayload));
}

#[test]
fn lineageweave_exchange_and_live_listener_return_the_same_context() {
    let request = request();
    let exchange = lineageweave_temporal_context_exchange("https://tepp.example.test", &request)
        .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert_eq!(exchange.target_url, "https://tepp.example.test/v1/temporal-context");
    assert!(exchange.headers.iter().all(|(name, _)| {
        !name.eq_ignore_ascii_case("authorization") && !name.to_ascii_lowercase().contains("token")
    }));

    let mut service = AnalysisRunLiveService::new();
    let response = service.handle_http_request(&http_request(&request.to_json().expect("json")));
    assert_eq!(response.status_code, 200);
    let live = TemporalContextResponse::from_json(&response.body).expect("response");
    assert_eq!(live, build_temporal_context(&request).expect("direct"));
}
