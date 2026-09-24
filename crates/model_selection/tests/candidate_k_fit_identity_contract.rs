//! Statistical candidate identity comes from the owner-issued reference fit.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::statistical_candidate_from_fit;
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{
    ReferenceTopicFit, ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn interval(day: u8) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(event_time(day)),
        TemporalBoundary::Included(
            EventTime::parse_rfc3339(&format!("2026-01-{day:02}T12:00:00Z"))
                .expect("interval end"),
        ),
        TemporalPrecision::Second,
    )
    .expect("bounded interval")
}

fn admitted_input() -> ReferenceTopicInput {
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

    let group_id = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in &document_ids {
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*document_id),
                    group_id,
                    MembershipRole::Organization,
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
                RelationEvidenceStatus::Observed,
                interval(1),
                interval(2),
            )
            .expect("observed transition"),
        )
        .expect("insert transition");

    let counts = SparseMatrix::from_csr(
        2,
        2,
        vec![0, 2, 4],
        vec![0, 1, 0, 1],
        vec![30.0, 10.0, 10.0, 30.0],
    )
    .expect("term counts");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        None,
        &memberships,
        &relations,
    )
    .expect("admitted input")
}

#[test]
fn statistical_candidate_uses_the_owner_issued_fit_identity() {
    let input = admitted_input();
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 1e-5)
        .expect("reference configuration");
    let fit = ReferenceTopicFit::fit(&input, &config).expect("owner-issued reference fit");

    let candidate = statistical_candidate_from_fit(&fit).expect("owner-bound candidate");
    assert_eq!(candidate.candidate_k(), 2);
}
