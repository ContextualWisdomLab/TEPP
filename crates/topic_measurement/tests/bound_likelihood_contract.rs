//! New scratch-only bound likelihood acceptance; not GPU or adoption evidence.

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

/// Fit only the synthetic six-document corpus through the public CPU f64 entry.
fn precision_origin_fit(
    input: &ReferenceTopicInput,
) -> (
    ReferenceTopicModelConfig,
    topic_measurement::FitOriginBoundModel,
) {
    let config = ReferenceTopicModelConfig::new(2, vec![7], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let model = fit_origin_bound_topic_model(input.clone(), config.clone())
        .expect("real converged CPU fit");
    assert!(model.model().objective.is_finite());
    (config, model)
}

/// Real fitted coordinates use the exact owned CPU scoring semantics.
#[test]
fn bound_likelihood_matches_actual_public_scorer() {
    let input = precision_origin_input();
    let (_, bound) = precision_origin_fit(&input);
    let expected = input
        .in_sample_log_likelihood(bound.model())
        .expect("legacy public scorer");
    let actual = bound
        .in_sample_log_likelihood()
        .expect("owned actual fit scorer");
    assert!(actual.is_finite() && actual < 0.0);
    assert_eq!(actual.to_bits(), expected.to_bits());
    println!(
        "BOUND_LIKELIHOOD actual={actual:.17e} iterations={}",
        bound.model().iterations
    );
}

/// Mutating a public model clone cannot alter the privately owned fit score.
#[test]
fn bound_likelihood_cannot_be_replaced_by_foreign_clone() {
    let input = precision_origin_input();
    let (_, bound) = precision_origin_fit(&input);
    let before = bound.in_sample_log_likelihood().expect("owned score");
    let mut foreign = bound.model().clone();
    foreign.topic_term_probabilities = vec![vec![0.25; 4]; 2];
    let other = input
        .in_sample_log_likelihood(&foreign)
        .expect("valid independent scoring DTO");
    assert_ne!(other.to_bits(), before.to_bits());
    assert_eq!(
        bound
            .in_sample_log_likelihood()
            .expect("still owned")
            .to_bits(),
        before.to_bits()
    );
}
