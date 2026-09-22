//! Frozen training-prevalence coordinate contract for held-out projection.

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
    PrevalenceFeature, ReferenceTopicTrainingInput, SparseMatrix, TopicMeasurementError,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
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
        interval(source_day),
        interval(target_day),
    )
    .expect("forward relation")
}

fn membership(
    member: Uuid,
    group: Uuid,
    start_day: u8,
    end_day: u8,
) -> MembershipAssignment {
    MembershipAssignment::new(
        MemberId::from_uuid(member),
        GroupId::from_uuid(group),
        MembershipRole::Organization,
        MembershipWeight::full().expect("full membership"),
        event_time(start_day),
        event_time(end_day),
    )
    .expect("membership")
}

fn training_input(
    covariates: Option<&SparseMatrix>,
) -> (ReferenceTopicTrainingInput, MembershipNetwork) {
    let document_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let event_times = vec![event_time(1), event_time(3)];
    let available = AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-01-04T00:00:00Z").expect("cutoff");
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
        vec![2.0, 3.0],
    )
    .expect("counts");
    let group = Uuid::from_u128(100);
    let mut memberships = MembershipNetwork::new();
    for document_id in &document_ids {
        memberships
            .insert(membership(*document_id, group, 1, 30))
            .expect("training membership");
    }
    for document_id in [
        Uuid::from_u128(3),
        Uuid::from_u128(4),
        Uuid::from_u128(5),
    ] {
        memberships
            .insert(membership(document_id, group, 1, 30))
            .expect("evaluation membership");
    }
    let mut relations = RelationGraph::new();
    relations
        .insert(relation(document_ids[0], document_ids[1], 1, 3))
        .expect("training relation");
    let input = ReferenceTopicTrainingInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        covariates,
        &memberships,
        &relations,
    )
    .expect("training input");
    (input, memberships)
}

#[test]
fn held_out_rows_use_only_the_frozen_training_time_transform() {
    let (training, memberships) = training_input(None);
    let basis = training.prevalence_design_basis();
    assert_eq!(basis.features(), training.input().features());
    assert_eq!(basis.event_time_origin(), &event_time(1));
    assert!((basis.event_time_location_seconds() - 86_400.0).abs() < f64::EPSILON);
    assert!((basis.event_time_scale_seconds() - 86_400.0).abs() < f64::EPSILON);

    let focal = Uuid::from_u128(3);
    let batch_a = basis
        .project(
            &[focal, Uuid::from_u128(4)],
            &[event_time(5), event_time(6)],
            None,
            &memberships,
        )
        .expect("first evaluation batch");
    let batch_b = basis
        .project(
            &[focal, Uuid::from_u128(5)],
            &[event_time(5), event_time(20)],
            None,
            &memberships,
        )
        .expect("second evaluation batch");

    assert_eq!(batch_a[0], batch_b[0]);
    assert!((batch_a[0][1] - 3.0).abs() < f64::EPSILON);
    assert!(batch_b[1][1] > batch_b[0][1]);

    assert_eq!(
        basis.project(&[], &[], None, &memberships),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        basis.project(&[focal], &[], None, &memberships),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        basis.project(&[focal, focal], &[event_time(5), event_time(5)], None, &memberships),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    let unexpected_covariates = SparseMatrix::from_csr(
        1,
        1,
        vec![0, 1],
        vec![0],
        vec![1.0],
    )
    .expect("unexpected covariates");
    assert_eq!(
        basis.project(
            &[focal],
            &[event_time(5)],
            Some(&unexpected_covariates),
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        basis.project(
            &[Uuid::from_u128(77)],
            &[event_time(5)],
            None,
            &MembershipNetwork::new(),
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}

#[test]
fn projection_fails_closed_on_coordinates_absent_from_training_basis() {
    let training_covariates = SparseMatrix::from_csr(
        2,
        1,
        vec![0, 1, 2],
        vec![0, 0],
        vec![1.0, 2.0],
    )
    .expect("training covariates");
    let (training, memberships) = training_input(Some(&training_covariates));
    let basis = training.prevalence_design_basis();
    assert!(basis.features().contains(&PrevalenceFeature::Covariate(0)));

    assert_eq!(
        basis.project(
            &[Uuid::from_u128(3)],
            &[event_time(5)],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );

    let wider_covariates = SparseMatrix::from_csr(
        1,
        2,
        vec![0, 1],
        vec![1],
        vec![1.0],
    )
    .expect("wider covariates");
    assert_eq!(
        basis.project(
            &[Uuid::from_u128(3)],
            &[event_time(5)],
            Some(&wider_covariates),
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );

    let unseen = Uuid::from_u128(9);
    let mut incompatible_memberships = MembershipNetwork::new();
    incompatible_memberships
        .insert(membership(unseen, Uuid::from_u128(999), 1, 30))
        .expect("unseen membership");
    let compatible_covariates = SparseMatrix::from_csr(
        1,
        1,
        vec![0, 1],
        vec![0],
        vec![1.0],
    )
    .expect("compatible covariates");
    assert_eq!(
        basis.project(
            &[unseen],
            &[event_time(5)],
            Some(&compatible_covariates),
            &incompatible_memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}
