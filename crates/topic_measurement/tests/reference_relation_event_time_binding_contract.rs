//! Observed transition intervals must agree with the modeled event-time coordinates.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{ReferenceTopicInput, SparseMatrix, TopicMeasurementError};
use uuid::Uuid;

fn event_time(day: u8, hour: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-09-{day:02}T{hour:02}:00:00Z"))
        .expect("event time")
}

fn interval(day: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_time(day, 0)),
        TemporalBoundary::Included(event_time(day, 12)),
        TemporalPrecision::Second,
    )
    .expect("bounded interval")
}

fn observed_transition(
    source: Uuid,
    target: Uuid,
    source_day: u8,
    target_day: u8,
) -> RelationEdge {
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(source_day),
        interval(target_day),
    )
    .expect("valid forward transition")
}

fn admitted_context(document_ids: &[Uuid]) -> (CorpusSnapshot, MembershipNetwork) {
    let available = AvailableTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-10-01T00:00:00Z").expect("cutoff");
    let group_id = GroupId::from_uuid(Uuid::from_u128(100));
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for document_id in document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("eligible document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*document_id),
                    group_id,
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1, 0),
                    event_time(10, 0),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }
    (snapshot, memberships)
}

fn counts() -> SparseMatrix {
    SparseMatrix::from_csr(
        2,
        2,
        vec![0, 1, 2],
        vec![0, 1],
        vec![2.0, 2.0],
    )
    .expect("counts")
}

#[test]
fn observed_transition_intervals_must_contain_modeled_event_times() {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let modeled_event_times = vec![event_time(1, 6), event_time(2, 6)];
    let (snapshot, memberships) = admitted_context(&document_ids);

    let mut matching_relations = RelationGraph::new();
    matching_relations
        .insert(observed_transition(document_ids[0], document_ids[1], 1, 2))
        .expect("insert matching relation");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids.clone(),
        &counts(),
        &modeled_event_times,
        None,
        &memberships,
        &matching_relations,
    )
    .expect("matching relation intervals must remain admissible");

    let mut mismatched_relations = RelationGraph::new();
    mismatched_relations
        .insert(observed_transition(document_ids[0], document_ids[1], 3, 4))
        .expect("insert independently valid forward relation");
    assert_eq!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids,
            &counts(),
            &modeled_event_times,
            None,
            &memberships,
            &mismatched_relations,
        )
        .err(),
        Some(TopicMeasurementError::InvalidModelInput)
    );
}
