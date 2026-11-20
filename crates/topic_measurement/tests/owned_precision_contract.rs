//! Owned-input precision usability: actual CPU fits, not GPU or recovery evidence.

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
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_origin_bound_topic_model,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-01-{day:02}T12:00:00Z"))
                    .expect("interval end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("bounded interval")
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

fn fixture() -> (
    CorpusSnapshot,
    Vec<Uuid>,
    Vec<EventTime>,
    MembershipNetwork,
    RelationGraph,
) {
    let document_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }

    let organization = GroupId::from_uuid(Uuid::from_u128(100));
    let projects = [
        GroupId::from_uuid(Uuid::from_u128(101)),
        GroupId::from_uuid(Uuid::from_u128(102)),
    ];
    let validity_start = event_time(1);
    let validity_end = event_time(9);
    let mut memberships = MembershipNetwork::new();
    for (index, id) in document_ids.iter().enumerate() {
        let member = MemberId::from_uuid(*id);
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    organization,
                    MembershipRole::Organization,
                    MembershipWeight::full().expect("full"),
                    validity_start,
                    validity_end,
                )
                .expect("organization membership"),
            )
            .expect("insert organization");
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    projects[usize::from(index >= 3)],
                    MembershipRole::Project,
                    MembershipWeight::new(0.75).expect("partial"),
                    validity_start,
                    validity_end,
                )
                .expect("project membership"),
            )
            .expect("insert project");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [
        (0, 1, 1, 2),
        (1, 2, 2, 3),
        (2, 3, 3, 4),
        (3, 4, 4, 5),
        (4, 5, 5, 6),
    ] {
        relations
            .insert(relation(
                document_ids[source],
                document_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }
    (snapshot, document_ids, times, memberships, relations)
}

fn separated_counts() -> SparseMatrix {
    SparseMatrix::from_csr(
        6,
        4,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3],
        vec![
            90.0, 10.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
        ],
    )
    .expect("counts")
}

/// Build the admitted synthetic corpus used by these origin-binding diagnostics.
fn precision_origin_input() -> ReferenceTopicInput {
    let (snapshot, ids, times, memberships, relations) = fixture();
    ReferenceTopicInput::new(
        &snapshot,
        ids,
        &separated_counts(),
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("admitted input")
}

/// Move the admitted input into the fit and build precision without cloning it.
#[test]
fn moved_input_owned_precision_needs_no_external_input_or_config() {
    let input = precision_origin_input();
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let bound = fit_origin_bound_topic_model(input, config).expect("actual CPU fit");
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let precision = bound
        .build_owned_joint_coordinate_precision(topics.clone())
        .expect("owned precision after input and configuration move");
    assert_eq!(
        precision.document_ids(),
        (1_u128..=6).map(Uuid::from_u128).collect::<Vec<_>>()
    );
    assert_eq!(precision.topic_ids(), topics);
    assert_eq!(precision.coordinate_means().len(), 6);
    assert_eq!(precision.values().len(), 6);
    assert!(
        precision
            .values()
            .iter()
            .all(|r| r.len() == 6 && r.iter().all(|x| x.is_finite()))
    );
    let draws = precision.draw_joint_gaussian(11, 2).expect("finite draws");
    assert_eq!(draws.topic_ids(), topics);
    assert_eq!(draws.plausible_values().len(), 12);
}

/// The new entry delegates to the unchanged builder with exactly retained data.
#[test]
fn owned_precision_matches_checked_origin_precision_and_draws() {
    let input = precision_origin_input();
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let bound = fit_origin_bound_topic_model(input.clone(), config.clone()).expect("CPU fit");
    let topics = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let checked = bound
        .build_joint_coordinate_precision(&input, &config, topics.clone())
        .expect("unchanged matching checked entry");
    let owned = bound
        .build_owned_joint_coordinate_precision(topics.clone())
        .expect("owned precision");
    assert_eq!(owned, checked);
    let a = owned.draw_joint_gaussian(11, 2).expect("owned draws");
    let b = checked.draw_joint_gaussian(11, 2).expect("checked draws");
    assert_eq!(a.draws(), b.draws());
    assert_eq!(a.draw_set_id(), b.draw_set_id());
    assert_eq!(a.document_ids(), b.document_ids());
    assert_eq!(a.event_times(), b.event_times());
    assert_eq!(a.topic_ids(), topics);
    let foreign = ReferenceTopicModelConfig::new(2, vec![8], 2_000, 1e-5).expect("foreign config");
    assert!(matches!(
        bound.build_joint_coordinate_precision(&input, &foreign, topics),
        Err(topic_measurement::TopicMeasurementError::InvalidModelInput)
    ));
}

/// Invalid topic labels still receive the original typed builder refusal.
#[test]
fn owned_precision_preserves_topic_identity_admission() {
    let input = precision_origin_input();
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let bound = fit_origin_bound_topic_model(input, config).expect("CPU fit");
    let id = Uuid::from_u128(201);
    for ids in [vec![], vec![id], vec![id, id]] {
        assert!(matches!(
            bound.build_owned_joint_coordinate_precision(ids),
            Err(topic_measurement::TopicMeasurementError::InvalidModelInput)
        ));
    }
    assert!(
        bound
            .build_owned_joint_coordinate_precision(vec![id, Uuid::from_u128(202)])
            .is_ok()
    );
}
