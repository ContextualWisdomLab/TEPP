//! Held-out topic-state recovery must optimize only evaluation-local coordinates.

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
    TopicMeasurementError,
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

fn training_fit() -> (ReferenceTopicTrainingFit, MembershipNetwork, Uuid, Uuid) {
    let training_ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let focal = Uuid::from_u128(50);
    let companion = Uuid::from_u128(51);
    let training_times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-08-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-09-01T00:00:00Z").expect("cutoff");
    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();

    for document_id in training_ids.iter().copied().chain([focal, companion]) {
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(document_id),
                    group,
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1),
                    event_time(25),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }
    for document_id in &training_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available), &cutoff)
            .expect("eligible training document");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in
        [(0, 1, 1, 2), (1, 2, 2, 3), (2, 3, 3, 4)]
    {
        relations
            .insert(relation(
                training_ids[source],
                training_ids[target],
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
    .expect("training counts");
    let training = ReferenceTopicTrainingInput::new(
        &snapshot,
        training_ids,
        &counts,
        &training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    let fit = ReferenceTopicTrainingFit::fit(&training, &config).expect("training fit");
    (fit, memberships, focal, companion)
}

#[test]
fn held_out_state_uses_counts_without_batch_context_or_global_refit() {
    let (fit, memberships, focal, companion) = training_fit();
    let original_model = fit.reference_fit().model().clone();
    let focal_counts =
        SparseMatrix::from_csr(1, 4, vec![0, 2], vec![0, 1], vec![95.0, 5.0]).expect("focal");
    let batch_counts = SparseMatrix::from_csr(
        2,
        4,
        vec![0, 2, 4],
        vec![0, 1, 2, 3],
        vec![95.0, 5.0, 5.0, 95.0],
    )
    .expect("batch");
    let contrasting_counts =
        SparseMatrix::from_csr(1, 4, vec![0, 2], vec![2, 3], vec![5.0, 95.0])
            .expect("contrasting focal");

    let single = fit
        .infer_held_out_document_topic_proportions(
            &[focal],
            &focal_counts,
            &[event_time(5)],
            None,
            &memberships,
        )
        .expect("single held-out state");
    let batched = fit
        .infer_held_out_document_topic_proportions(
            &[focal, companion],
            &batch_counts,
            &[event_time(5), event_time(20)],
            None,
            &memberships,
        )
        .expect("batched held-out states");
    let contrasting = fit
        .infer_held_out_document_topic_proportions(
            &[focal],
            &contrasting_counts,
            &[event_time(5)],
            None,
            &memberships,
        )
        .expect("contrasting held-out state");

    assert_eq!(fit.reference_fit().model(), &original_model);
    assert_eq!(single.len(), 1);
    assert_eq!(batched.len(), 2);
    assert_eq!(single[0].len(), 2);
    for (left, right) in single[0].iter().zip(&batched[0]) {
        assert!((left - right).abs() < 1.0e-12);
    }
    assert!(
        single[0]
            .iter()
            .zip(&contrasting[0])
            .any(|(left, right)| (left - right).abs() > 1.0e-3),
        "held-out local state must respond to the focal document counts"
    );
    for row in [&single[0], &batched[0], &batched[1], &contrasting[0]] {
        assert!(row.iter().all(|value| value.is_finite() && *value > 0.0));
        assert!((row.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
    }
}

#[test]
fn held_out_state_fails_closed_on_evaluation_geometry() {
    let (fit, memberships, focal, _) = training_fit();
    let valid =
        SparseMatrix::from_csr(1, 4, vec![0, 2], vec![0, 1], vec![70.0, 30.0]).expect("valid");
    let wrong_rows = SparseMatrix::from_csr(
        2,
        4,
        vec![0, 1, 2],
        vec![0, 1],
        vec![1.0, 1.0],
    )
    .expect("wrong rows");
    let wrong_vocabulary =
        SparseMatrix::from_csr(1, 3, vec![0, 1], vec![0], vec![1.0]).expect("wrong vocab");
    let empty = SparseMatrix::from_csr(1, 4, vec![0, 0], vec![], vec![]).expect("empty row");
    let negative =
        SparseMatrix::from_csr(1, 4, vec![0, 1], vec![0], vec![-1.0]).expect("negative count");
    let zero = SparseMatrix::from_csr(1, 4, vec![0, 1], vec![0], vec![0.0]).expect("zero count");
    let overflowed_total = SparseMatrix::from_csr(
        1,
        4,
        vec![0, 2],
        vec![0, 1],
        vec![f64::MAX, f64::MAX],
    )
    .expect("overflowed total");

    assert_eq!(
        fit.infer_held_out_document_topic_proportions(
            &[focal],
            &wrong_rows,
            &[event_time(5)],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        fit.infer_held_out_document_topic_proportions(
            &[focal],
            &wrong_vocabulary,
            &[event_time(5)],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        fit.infer_held_out_document_topic_proportions(
            &[focal],
            &valid,
            &[],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    for invalid in [&empty, &negative, &zero, &overflowed_total] {
        assert_eq!(
            fit.infer_held_out_document_topic_proportions(
                &[focal],
                invalid,
                &[event_time(5)],
                None,
                &memberships,
            ),
            Err(TopicMeasurementError::InvalidModelInput)
        );
    }
    assert_eq!(
        fit.infer_held_out_document_topic_proportions(
            &[focal],
            &valid,
            &[event_time(5)],
            None,
            &MembershipNetwork::new(),
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}
