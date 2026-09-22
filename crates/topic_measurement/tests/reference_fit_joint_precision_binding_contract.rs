//! Public contract for fit-owned joint-precision construction.

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
    FittedDocumentCoordinateSummary, PosteriorApproximation, ReferenceTopicFit,
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-09-{day:02}T00:00:00Z")).expect("event time")
}

fn transition(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-09-{day:02}T12:00:00Z"))
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
    .expect("forward transition")
}

fn fit() -> ReferenceTopicFit {
    let document_ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let event_times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-09-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-10-01T00:00:00Z").expect("cutoff");

    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    let group_id = GroupId::from_uuid(Uuid::from_u128(100));
    for document_id in &document_ids {
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
            .insert(transition(
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
    let input = ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &event_times,
        None,
        &memberships,
        &relations,
    )
    .expect("reference input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    ReferenceTopicFit::fit(&input, &config).expect("converged reference fit")
}

#[test]
fn owner_fit_builds_joint_precision_without_detached_input_model_or_config() {
    let fit = fit();
    let topic_ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let precision = fit
        .build_joint_coordinate_precision(topic_ids.clone())
        .expect("fit-owned joint precision");
    let summary = FittedDocumentCoordinateSummary::from_bound_fit(&fit)
        .expect("fit-owned document coordinates");

    assert_eq!(
        precision.approximation(),
        PosteriorApproximation::JointGaussNewtonLaplace
    );
    assert_eq!(precision.document_ids(), fit.input().document_ids());
    assert_eq!(precision.topic_ids(), topic_ids);

    let summary_locations: Vec<f64> = summary
        .rows()
        .iter()
        .flat_map(|row| row.coordinates().iter().map(|coordinate| coordinate.location()))
        .collect();
    assert_eq!(precision.coordinate_means(), summary_locations);
    assert_eq!(precision.values().len(), summary_locations.len());
    assert!(precision.values().iter().enumerate().all(|(row, values)| {
        values.iter().enumerate().all(|(column, value)| {
            value.is_finite()
                && (value - precision.values()[column][row]).abs() <= f64::EPSILON
        })
    }));
}
