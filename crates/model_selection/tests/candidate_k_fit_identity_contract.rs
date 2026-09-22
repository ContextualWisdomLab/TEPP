//! Candidate labels must identify the fitted topic dimension they score.

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{ModelSelectionError, statistical_candidate_from_fit};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use topic_measurement::{PrevalenceFeature, ReferenceTopicInput, ReferenceTopicModel, SparseMatrix};
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
        vec![3.0, 1.0, 1.0, 3.0],
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

fn two_topic_model() -> ReferenceTopicModel {
    ReferenceTopicModel {
        seed: 7,
        iterations: 4,
        objective: -1.0,
        topic_term_probabilities: vec![vec![0.75, 0.25], vec![0.25, 0.75]],
        document_topic_proportions: vec![vec![0.8, 0.2], vec![0.2, 0.8]],
        document_coordinate_variances: vec![vec![0.1], vec![0.1]],
        prevalence_coefficients: vec![vec![0.0]; 3],
        prevalence_features: vec![
            PrevalenceFeature::Intercept,
            PrevalenceFeature::EventTime,
            PrevalenceFeature::Membership {
                role: MembershipRole::Organization,
                group_id: GroupId::from_uuid(Uuid::from_u128(100)),
            },
        ],
        sequence_edges: Vec::new(),
        connected_post_count: 0,
        lineage_count: 0,
    }
}

#[test]
fn statistical_candidate_rejects_a_k_label_from_another_topic_dimension() {
    let input = admitted_input();
    let model = two_topic_model();

    let matching = statistical_candidate_from_fit(&input, 2, &model).expect("matching K");
    assert_eq!(matching.candidate_k(), 2);
    assert_eq!(
        statistical_candidate_from_fit(&input, 3, &model),
        Err(ModelSelectionError::InvalidDiagnostic)
    );
}
