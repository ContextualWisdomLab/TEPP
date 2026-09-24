//! Batch marginal covariance reuses one owner factorization without changing scalar results.

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
    FitBoundDocumentMarginalCovariance, ReferenceTopicFit, ReferenceTopicInput,
    ReferenceTopicModelConfig, SparseMatrix, TopicMeasurementError,
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
    for document_id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("eligible document");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*document_id),
                    GroupId::from_uuid(Uuid::from_u128(100)),
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
        6,
        vec![0, 3, 6, 9, 12],
        vec![0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5],
        vec![80.0, 15.0, 5.0, 75.0, 20.0, 5.0, 5.0, 20.0, 75.0, 5.0, 15.0, 80.0],
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
    let config = ReferenceTopicModelConfig::new(3, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    ReferenceTopicFit::fit(&input, &config).expect("converged reference fit")
}

#[test]
fn batch_marginals_match_scalar_owner_and_preserve_requested_order() {
    let fit = fit();
    let topic_ids = vec![
        Uuid::from_u128(201),
        Uuid::from_u128(202),
        Uuid::from_u128(203),
    ];
    let requested = vec![fit.input().document_ids()[2], fit.input().document_ids()[0]];
    let precision = fit
        .build_joint_coordinate_precision(topic_ids.clone())
        .expect("fit-owned joint precision");

    let batch = precision
        .document_marginal_covariances(&requested)
        .expect("one-factorization batch marginals");
    assert_eq!(batch.len(), requested.len());
    for (index, document_id) in requested.iter().copied().enumerate() {
        let scalar = precision
            .document_marginal_covariance(document_id)
            .expect("scalar marginal");
        assert_eq!(batch[index].document_id(), document_id);
        assert_eq!(batch[index].event_time(), scalar.event_time());
        assert_eq!(batch[index].topic_ids(), scalar.topic_ids());
        assert_eq!(batch[index].values(), scalar.values());
    }

    let fit_bound = FitBoundDocumentMarginalCovariance::from_bound_fit_many(
        &fit,
        topic_ids,
        &requested,
    )
    .expect("fit-bound batch marginals");
    assert_eq!(fit_bound.len(), requested.len());
    assert_eq!(fit_bound[0].document_id(), requested[0]);
    assert_eq!(fit_bound[1].document_id(), requested[1]);
}

#[test]
fn batch_marginals_reject_empty_duplicate_and_missing_document_requests() {
    let fit = fit();
    let topic_ids = vec![
        Uuid::from_u128(201),
        Uuid::from_u128(202),
        Uuid::from_u128(203),
    ];
    let precision = fit
        .build_joint_coordinate_precision(topic_ids)
        .expect("fit-owned joint precision");
    let first = fit.input().document_ids()[0];

    assert_eq!(
        precision.document_marginal_covariances(&[]),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        precision.document_marginal_covariances(&[first, first]),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        precision.document_marginal_covariances(&[Uuid::from_u128(999)]),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}
