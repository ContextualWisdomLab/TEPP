//! Fit-origin binding, precision, and draw contracts for the CPU `f64` reference.

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
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_reference_topic_model,
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

/// An injected positive simplex characterizes builder range, not actual-fit reachability.
#[test]
fn precision_preserves_representable_tiny_off_diagonal_with_large_count() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let ordinary =
        SparseMatrix::from_csr(6, 4, vec![0, 1, 2, 3, 4, 5, 6], vec![0; 6], vec![100.; 6])
            .expect("small counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids.clone(),
        &ordinary,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("admitted input");
    let config = ReferenceTopicModelConfig::new(3, vec![7], 5_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1., 0., 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let mut model = fit_reference_topic_model(&input, &config)
        .expect("actual small CPU fit supplies DTO shape");
    model.document_topic_proportions = vec![vec![1e-170, 1e-170, 1.]; 6];
    let counts =
        SparseMatrix::from_csr(6, 4, vec![0, 1, 2, 3, 4, 5, 6], vec![0; 6], vec![1e100; 6])
            .expect("large finite counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("admitted larger counts");
    let expected = (-1e100_f64 * 1e-170_f64) * 1e-170_f64;
    assert!(expected.is_normal() && expected < 0.);
    let topics = (201_u128..=203).map(Uuid::from_u128).collect();
    let precision = input
        .build_joint_coordinate_precision(&model, &config, topics)
        .expect("injected positive simplex");
    println!(
        "RANGE expected={expected} actual={}",
        precision.values()[0][1]
    );
    assert_eq!(precision.values()[0][1].to_bits(), expected.to_bits());
    assert_eq!(precision.values()[1][0].to_bits(), expected.to_bits());
}
