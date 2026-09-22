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
use topic_measurement::{
    ReferenceTopicFit, ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix,
    TopicMeasurementError,
};
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

fn transition(
    source: Uuid,
    target: Uuid,
    status: RelationEvidenceStatus,
    source_day: u8,
    target_day: u8,
) -> RelationEdge {
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        status,
        interval(source_day),
        interval(target_day),
    )
    .expect("valid forward transition")
}

fn snapshot_and_memberships(
    document_ids: &[Uuid],
    membership_end_day: u8,
) -> (CorpusSnapshot, MembershipNetwork) {
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
                    event_time(1),
                    event_time(membership_end_day),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }
    (snapshot, memberships)
}

#[test]
fn inferred_only_transition_graph_is_not_numerical_authority() {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let event_times = vec![event_time(1), event_time(2)];
    let (snapshot, memberships) = snapshot_and_memberships(&document_ids, 3);

    let mut relations = RelationGraph::new();
    relations
        .insert(transition(
            document_ids[0],
            document_ids[1],
            RelationEvidenceStatus::Inferred,
            1,
            2,
        ))
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

#[test]
fn mixed_graph_fits_identically_to_its_observed_transition_subset() {
    let document_ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let event_times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let (snapshot, memberships) = snapshot_and_memberships(&document_ids, 5);
    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");

    let mut observed = RelationGraph::new();
    for edge in [
        transition(
            document_ids[0],
            document_ids[1],
            RelationEvidenceStatus::Observed,
            1,
            2,
        ),
        transition(
            document_ids[2],
            document_ids[3],
            RelationEvidenceStatus::Observed,
            3,
            4,
        ),
    ] {
        observed.insert(edge).expect("insert observed relation");
    }

    let mut mixed = observed.clone();
    mixed
        .insert(transition(
            document_ids[1],
            document_ids[2],
            RelationEvidenceStatus::Inferred,
            2,
            3,
        ))
        .expect("insert inferred relation");

    let observed_input = ReferenceTopicInput::new(
        &snapshot,
        document_ids.clone(),
        &counts,
        &event_times,
        None,
        &memberships,
        &observed,
    )
    .expect("observed input");
    let mixed_input = ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        None,
        &memberships,
        &mixed,
    )
    .expect("mixed input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("configuration");

    let observed_fit = ReferenceTopicFit::fit(&observed_input, &config).expect("observed fit");
    let mixed_fit = ReferenceTopicFit::fit(&mixed_input, &config).expect("mixed fit");
    assert_eq!(mixed_fit.model(), observed_fit.model());
}
