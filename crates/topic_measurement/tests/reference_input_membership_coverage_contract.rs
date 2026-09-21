//! Structural membership coverage contract for the CPU `f64` topic input.

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
use topic_measurement::{ReferenceTopicInput, SparseMatrix};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(event_time(day)),
            TemporalPrecision::Second,
        )
        .expect("event interval")
    };
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(1),
        interval(2),
    )
    .expect("forward relation")
}

#[test]
fn every_modeled_document_requires_active_membership_at_its_event_time() {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let event_times = vec![event_time(1), event_time(2)];
    let available = AvailableTime::parse_rfc3339("2026-01-03T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");

    let mut snapshot = CorpusSnapshot::new();
    for document_id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("eligible document");
    }

    let counts = SparseMatrix::from_csr(
        2,
        2,
        vec![0, 1, 2],
        vec![0, 1],
        vec![1.0, 1.0],
    )
    .expect("counts");

    let mut memberships = MembershipNetwork::new();
    memberships
        .insert(
            MembershipAssignment::new(
                MemberId::from_uuid(document_ids[0]),
                GroupId::from_uuid(Uuid::from_u128(100)),
                MembershipRole::Organization,
                MembershipWeight::full().expect("full membership"),
                event_time(1),
                event_time(3),
            )
            .expect("membership"),
        )
        .expect("insert membership");

    let mut relations = RelationGraph::new();
    relations
        .insert(relation(document_ids[0], document_ids[1]))
        .expect("insert relation");

    assert!(
        ReferenceTopicInput::new(
            &snapshot,
            document_ids,
            &counts,
            &event_times,
            None,
            &memberships,
            &relations,
        )
        .is_err(),
        "a document without active membership would create an atomistic all-zero membership row",
    );
}
