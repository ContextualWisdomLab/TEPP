//! Numerical admission must not promote inferred relation evidence.

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

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-09-{day:02}T00:00:00Z")).expect("event time")
}

fn interval(day: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_time(day)),
        TemporalBoundary::Included(
            EventTime::parse_rfc3339(&format!("2026-09-{day:02}T12:00:00Z"))
                .expect("interval end"),
        ),
        TemporalPrecision::Second,
    )
    .expect("bounded interval")
}

#[test]
fn inferred_only_transition_graph_is_not_numerical_authority() {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let event_times = vec![event_time(1), event_time(2)];
    let available = AvailableTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-10-01T00:00:00Z").expect("cutoff");

    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    let group_id = GroupId::from_uuid(Uuid::from_u128(100));
    for document_id in &document_ids {
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
                    event_time(1),
                    event_time(3),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }

    let mut relations = RelationGraph::new();
    relations
        .insert(
            RelationEdge::new(
                RelationKind::TransitionsTo,
                RelationEndpointId::from_uuid(document_ids[0]),
                RelationEndpointId::from_uuid(document_ids[1]),
                RelationEvidenceStatus::Inferred,
                interval(1),
                interval(2),
            )
            .expect("valid inferred forward transition"),
        )
        .expect("insert inferred relation");

    let counts = SparseMatrix::from_csr(
        2,
        2,
        vec![0, 1, 2],
        vec![0, 1],
        vec![2.0, 2.0],
    )
    .expect("counts");

    assert_eq!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids,
            &counts,
            &event_times,
            None,
            &memberships,
            &relations,
        )
        .err(),
        Some(TopicMeasurementError::InvalidModelInput)
    );
}
