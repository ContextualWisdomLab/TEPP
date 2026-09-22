//! Owner-issued fit/input/config binding for numerical release projections.

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
    FittedDocumentCoordinateSummary, ReferenceTopicFit, ReferenceTopicInput,
    ReferenceTopicModelConfig, SparseMatrix,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-07-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-07-{day:02}T12:00:00Z")).expect("end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("interval")
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

fn admitted_input(id_offset: u128) -> (ReferenceTopicInput, ReferenceTopicModelConfig) {
    let ids: Vec<_> = (1_u128..=4)
        .map(|value| Uuid::from_u128(id_offset + value))
        .collect();
    let times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for id in &ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*id),
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
            .insert(relation(ids[source], ids[target], source_day, target_day))
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
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("reference input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    (input, config)
}

#[test]
fn owner_issued_fit_keeps_model_input_and_config_in_one_nominal_aggregate() {
    let (input_a, config_a) = admitted_input(0);
    let (input_b, config_b) = admitted_input(1_000);

    let fit_a = ReferenceTopicFit::fit(&input_a, &config_a).expect("fit A");
    let fit_b = ReferenceTopicFit::fit(&input_b, &config_b).expect("fit B");

    assert_eq!(fit_a.input().document_ids(), input_a.document_ids());
    assert_eq!(fit_b.input().document_ids(), input_b.document_ids());
    assert_ne!(fit_a.input().document_ids(), fit_b.input().document_ids());
    assert_eq!(fit_a.config(), &config_a);
    assert_eq!(fit_b.config(), &config_b);

    let summary_a = FittedDocumentCoordinateSummary::from_bound_fit(&fit_a).expect("summary A");
    let summary_b = FittedDocumentCoordinateSummary::from_bound_fit(&fit_b).expect("summary B");
    let summary_a_ids: Vec<_> = summary_a.rows().iter().map(|row| row.document_id()).collect();
    let summary_b_ids: Vec<_> = summary_b.rows().iter().map(|row| row.document_id()).collect();
    assert_eq!(summary_a_ids.as_slice(), input_a.document_ids());
    assert_eq!(summary_b_ids.as_slice(), input_b.document_ids());
    assert_ne!(summary_a_ids, summary_b_ids);

    assert_eq!(
        fit_a.model().posterior_approximation(),
        fit_b.model().posterior_approximation()
    );
}
