//! Public contract for fit-bound document marginal covariance from joint precision.

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
    FitBoundDocumentMarginalCovariance, FittedTopicBasisIdentity, ReferenceTopicFit,
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, TopicMeasurementError,
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
fn relation_coupled_marginal_uses_full_inverse_precision() {
    let fit = fit();
    let topic_ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let precision = fit
        .build_joint_coordinate_precision(topic_ids.clone())
        .expect("fit-owned joint precision");
    let document_id = fit.input().document_ids()[0];
    let marginal = precision
        .document_marginal_covariance(document_id)
        .expect("fit-bound marginal covariance");

    assert_eq!(marginal.document_id(), document_id);
    assert_eq!(marginal.topic_ids(), topic_ids);
    assert_eq!(marginal.values().len(), 1);
    assert_eq!(marginal.values()[0].len(), 1);

    let full_inverse_marginal = marginal.values()[0][0];
    let reciprocal_precision_diagonal = 1.0 / precision.values()[0][0];
    let isolated_document_block_inverse = reciprocal_precision_diagonal;
    assert!(full_inverse_marginal.is_finite() && full_inverse_marginal > 0.0);
    assert!((full_inverse_marginal - reciprocal_precision_diagonal).abs() > 1.0e-12);
    assert!((full_inverse_marginal - isolated_document_block_inverse).abs() > 1.0e-12);
}

#[test]
fn fit_bound_marginal_retains_owner_topic_basis_identity() {
    let fit = fit();
    let expected_basis =
        FittedTopicBasisIdentity::from_bound_fit(&fit).expect("fit-owned topic basis");
    let topic_ids = vec![Uuid::from_u128(201), Uuid::from_u128(202)];
    let document_id = fit.input().document_ids()[0];

    let marginal = FitBoundDocumentMarginalCovariance::from_bound_fit(
        &fit,
        topic_ids.clone(),
        document_id,
    )
    .expect("fit-bound marginal covariance with basis identity");

    assert_eq!(marginal.document_id(), document_id);
    assert_eq!(marginal.topic_ids(), topic_ids);
    assert_eq!(marginal.topic_basis_identity(), &expected_basis);
    assert_eq!(marginal.values().len(), 1);
    assert_eq!(marginal.values()[0].len(), 1);
}

#[test]
fn missing_document_identity_fails_closed() {
    let fit = fit();
    let precision = fit
        .build_joint_coordinate_precision(vec![Uuid::from_u128(201), Uuid::from_u128(202)])
        .expect("fit-owned joint precision");

    assert_eq!(
        precision.document_marginal_covariance(Uuid::from_u128(999)),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}
