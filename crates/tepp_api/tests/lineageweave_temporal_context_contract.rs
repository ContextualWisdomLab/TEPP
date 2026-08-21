//! Contract tests for TEPP temporal evidence used by `LineageWeave` Ask surfaces.

use std::fmt::Write as _;

use tepp_api::{
    AnalysisRunLiveService, ApiError, LINEAGEWEAVE_CONSUMER_CODE, NARUON_CONSUMER_CODE,
    TEMPORAL_ASSOCIATION_CLAIM_BOUNDARY, TEMPORAL_CONTEXT_CONTRACT_VERSION, TEMPORAL_CONTEXT_PATH,
    TemporalContextEvent, TemporalContextRequest, TemporalContextResponse, build_temporal_context,
    lineageweave_temporal_context_exchange,
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
    http_request_for_consumer(body, LINEAGEWEAVE_CONSUMER_CODE)
}

fn http_request_for_consumer(body: &str, consumer: &str) -> String {
    let mut value = format!("POST {TEMPORAL_CONTEXT_PATH} HTTP/1.1\r\n");
    for (name, header_value) in [
        ("Host", "127.0.0.1"),
        ("content-type", "application/json"),
        ("tepp-consumer", consumer),
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
    assert!(
        response
            .temporal_relations
            .iter()
            .all(|item| item.relation_code == "before")
    );
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
    assert_eq!(
        build_temporal_context(&future),
        Err(ApiError::InvalidWirePayload)
    );

    let mut duplicate = request();
    duplicate.events[1].event_id = duplicate.events[0].event_id.clone();
    assert_eq!(
        build_temporal_context(&duplicate),
        Err(ApiError::InvalidWirePayload)
    );

    let mut hostile = request();
    hostile.consumer_code = "unpublished-consumer".into();
    assert_eq!(
        build_temporal_context(&hostile),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn lineageweave_exchange_and_live_listener_return_the_same_context() {
    let request = request();
    let exchange = lineageweave_temporal_context_exchange("https://tepp.example.test", &request)
        .expect("exchange");
    assert_eq!(exchange.method, "POST");
    assert_eq!(
        exchange.target_url,
        "https://tepp.example.test/v1/temporal-context"
    );
    assert!(exchange.headers.iter().all(|(name, _)| {
        !name.eq_ignore_ascii_case("authorization") && !name.to_ascii_lowercase().contains("token")
    }));
    assert!(
        exchange
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("idempotency-key"))
    );

    let mut service = AnalysisRunLiveService::new();
    let response = service.handle_http_request(&http_request(&request.to_json().expect("json")));
    assert_eq!(response.status_code, 200);
    let live = TemporalContextResponse::from_json(&response.body).expect("response");
    assert_eq!(live, build_temporal_context(&request).expect("direct"));
}

fn single_event_response() -> TemporalContextResponse {
    let mut value = request();
    value.events.truncate(1);
    value.subject_post_id = None;
    build_temporal_context(&value).expect("single event")
}

fn two_event_response() -> TemporalContextResponse {
    let mut value = request();
    value.events.truncate(2);
    value.subject_post_id = None;
    build_temporal_context(&value).expect("two events")
}

#[test]
fn temporal_context_rejects_invalid_requests() {
    let mut empty_events = request();
    empty_events.events.clear();
    assert_eq!(
        build_temporal_context(&empty_events),
        Err(ApiError::InvalidWirePayload)
    );

    let mut too_many_events = request();
    too_many_events.events = (0..1025)
        .map(|index| {
            let mut value = too_many_events.events[0].clone();
            value.event_id = format!("event-{index}");
            value
        })
        .collect();
    assert_eq!(
        build_temporal_context(&too_many_events),
        Err(ApiError::LimitExceeded)
    );

    let mut empty_subject = request();
    empty_subject.subject_post_id = Some(String::new());
    assert_eq!(
        build_temporal_context(&empty_subject),
        Err(ApiError::InvalidWirePayload)
    );

    let mut unknown_subject = request();
    unknown_subject.subject_post_id = Some("missing-post".into());
    assert_eq!(
        build_temporal_context(&unknown_subject),
        Err(ApiError::InvalidWirePayload)
    );

    let mut empty_project = request();
    empty_project.events[0].project_reference = Some(" ".into());
    assert_eq!(
        build_temporal_context(&empty_project),
        Err(ApiError::InvalidWirePayload)
    );

    let mut empty_actors = request();
    empty_actors.events[0].actor_references.clear();
    assert_eq!(
        build_temporal_context(&empty_actors),
        Err(ApiError::InvalidWirePayload)
    );

    let mut no_project = request();
    no_project.events[0].project_reference = None;
    assert!(build_temporal_context(&no_project).is_ok());
}

#[test]
fn temporal_context_rejects_invalid_response_shapes() {
    let single_response = single_event_response();

    let mut invalid_claim = single_response.clone();
    invalid_claim.claim_boundary = "causal".into();
    assert_eq!(invalid_claim.to_json(), Err(ApiError::InvalidWirePayload));

    let mut empty_timeline = single_response.clone();
    empty_timeline.timeline_events.clear();
    assert_eq!(empty_timeline.to_json(), Err(ApiError::InvalidWirePayload));

    let mut missing_source = single_response.clone();
    missing_source.source_post_ids.clear();
    assert_eq!(missing_source.to_json(), Err(ApiError::InvalidWirePayload));

    let two_response = two_event_response();

    let mut missing_relation = two_response.clone();
    missing_relation.temporal_relations.clear();
    assert_eq!(
        missing_relation.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut missing_gap = two_response.clone();
    missing_gap.transition_gap_candidates.clear();
    assert_eq!(missing_gap.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn temporal_context_rejects_invalid_response_fields_and_edges() {
    let single_response = single_event_response();
    for invalid in [
        {
            let mut value = single_response.clone();
            value.timeline_events[0].sequence_ordinal = 1;
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].event_id.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].source_post_id.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].event_type_code.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].event_label.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].event_time.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.timeline_events[0].actor_references.clear();
            value
        },
        {
            let mut value = single_response.clone();
            value.source_post_ids[0] = "different-post".into();
            value
        },
    ] {
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));
    }

    let two_response = two_event_response();
    for invalid in [
        {
            let mut value = two_response.clone();
            value.timeline_events[0].event_time = "2026-08-02T09:00:00Z".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.timeline_events[1].event_id = value.timeline_events[0].event_id.clone();
            value
        },
        {
            let mut value = two_response.clone();
            value.timeline_events[0].project_reference = Some(" ".into());
            value
        },
        {
            let mut value = two_response.clone();
            value.timeline_events[0].actor_references = vec![" ".into()];
            value
        },
        {
            let mut value = two_response.clone();
            value.temporal_relations[0].from_event_id = "wrong".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.temporal_relations[0].to_event_id = "wrong".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.temporal_relations[0].relation_code = "after".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.transition_gap_candidates[0].from_event_id = "wrong".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.transition_gap_candidates[0].to_event_id = "wrong".into();
            value
        },
        {
            let mut value = two_response.clone();
            value.transition_gap_candidates[0].evidence_status_code = "causal".into();
            value
        },
    ] {
        assert_eq!(invalid.to_json(), Err(ApiError::InvalidWirePayload));
    }
}

#[test]
fn temporal_context_rejects_equal_timestamp_id_regressions() {
    let two_response = two_event_response();
    let mut equal_time = two_response.clone();
    equal_time.timeline_events[1].event_time = equal_time.timeline_events[0].event_time.clone();
    assert!(equal_time.to_json().is_ok());

    let mut equal_time_out_of_order = equal_time;
    equal_time_out_of_order.timeline_events[1].event_id = "event-aaa".into();
    assert_eq!(
        equal_time_out_of_order.to_json(),
        Err(ApiError::InvalidWirePayload)
    );
}

#[test]
fn temporal_context_requires_matching_lineageweave_header() {
    let body = request().to_json().expect("request json");
    let response = AnalysisRunLiveService::new()
        .handle_http_request(&http_request_for_consumer(&body, NARUON_CONSUMER_CODE));
    assert_eq!(response.status_code, 400);
}
