//! Owner-issued binding between one frozen training prevalence basis and its fitted state.

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
    ReferenceTopicModelConfig, ReferenceTopicTrainingFit, ReferenceTopicTrainingInput, SparseMatrix,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-08-{day:02}T00:00:00Z")).expect("event time")
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

fn admitted_training(
    id_offset: u128,
) -> (ReferenceTopicTrainingInput, ReferenceTopicModelConfig) {
    let document_ids: Vec<_> = (1_u128..=4)
        .map(|value| Uuid::from_u128(id_offset + value))
        .collect();
    let event_times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-08-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-09-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for document_id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("eligible document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*document_id),
                    GroupId::from_uuid(Uuid::from_u128(id_offset + 100)),
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1),
                    event_time(9),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in
        [(0, 1, 1, 2), (1, 2, 2, 3), (2, 3, 3, 4)]
    {
        relations
            .insert(relation(
                document_ids[source],
                document_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }

    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");
    let training = ReferenceTopicTrainingInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    (training, config)
}

#[test]
fn training_fit_keeps_frozen_prevalence_basis_with_the_exact_fitted_input() {
    let (training_a, config_a) = admitted_training(0);
    let (training_b, config_b) = admitted_training(1_000);

    let fit_a = ReferenceTopicTrainingFit::fit(&training_a, &config_a).expect("training fit A");
    let fit_b = ReferenceTopicTrainingFit::fit(&training_b, &config_b).expect("training fit B");

    assert_eq!(
        fit_a.reference_fit().input().document_ids(),
        training_a.input().document_ids()
    );
    assert_eq!(
        fit_b.reference_fit().input().document_ids(),
        training_b.input().document_ids()
    );
    assert_ne!(
        fit_a.reference_fit().input().document_ids(),
        fit_b.reference_fit().input().document_ids()
    );
    assert_eq!(
        fit_a.prevalence_design_basis().features(),
        fit_a.reference_fit().input().features()
    );
    assert_eq!(
        fit_b.prevalence_design_basis().features(),
        fit_b.reference_fit().input().features()
    );
}
