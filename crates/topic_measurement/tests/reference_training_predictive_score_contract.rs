//! Fixed-training prevalence-mean predictive scoring must reuse owner-issued coordinates.

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

fn training_fit() -> (ReferenceTopicTrainingFit, MembershipNetwork, Uuid, Uuid, Uuid) {
    let training_ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let focal = Uuid::from_u128(50);
    let companion_near = Uuid::from_u128(51);
    let companion_far = Uuid::from_u128(52);
    let training_times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-08-01T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-09-01T00:00:00Z").expect("cutoff");
    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();

    for document_id in training_ids
        .iter()
        .copied()
        .chain([focal, companion_near, companion_far])
    {
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
    (fit, memberships, focal, companion_near, companion_far)
}

#[test]
fn predictive_score_reuses_training_coordinates_and_never_depends_on_future_batch_context() {
    let (fit, memberships, focal, companion_near, companion_far) = training_fit();
    let focal_counts =
        SparseMatrix::from_csr(1, 4, vec![0, 2], vec![0, 1], vec![70.0, 30.0]).expect("focal");
    let near_counts = SparseMatrix::from_csr(
        2,
        4,
        vec![0, 2, 4],
        vec![0, 1, 2, 3],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("near batch");
    let far_counts = SparseMatrix::from_csr(
        2,
        4,
        vec![0, 2, 4],
        vec![0, 1, 2, 3],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("far batch");

    let focal_score = fit
        .prevalence_mean_predictive_log_likelihoods(
            &[focal],
            &focal_counts,
            &[event_time(5)],
            None,
            &memberships,
        )
        .expect("focal predictive score");
    let near_scores = fit
        .prevalence_mean_predictive_log_likelihoods(
            &[focal, companion_near],
            &near_counts,
            &[event_time(5), event_time(6)],
            None,
            &memberships,
        )
        .expect("near predictive score");
    let far_scores = fit
        .prevalence_mean_predictive_log_likelihoods(
            &[focal, companion_far],
            &far_counts,
            &[event_time(5), event_time(20)],
            None,
            &memberships,
        )
        .expect("far predictive score");

    assert_eq!(focal_score.len(), 1);
    assert_eq!(near_scores.len(), 2);
    assert_eq!(far_scores.len(), 2);
    assert!((focal_score[0] - near_scores[0]).abs() < 1e-12);
    assert!((focal_score[0] - far_scores[0]).abs() < 1e-12);
}

#[test]
fn predictive_score_fails_closed_on_evaluation_count_geometry() {
    let (fit, memberships, focal, _, _) = training_fit();
    let wrong_vocabulary =
        SparseMatrix::from_csr(1, 3, vec![0, 1], vec![0], vec![1.0]).expect("wrong vocab");
    let negative =
        SparseMatrix::from_csr(1, 4, vec![0, 1], vec![0], vec![-1.0]).expect("negative count");

    assert_eq!(
        fit.prevalence_mean_predictive_log_likelihoods(
            &[focal],
            &wrong_vocabulary,
            &[event_time(5)],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
    assert_eq!(
        fit.prevalence_mean_predictive_log_likelihoods(
            &[focal],
            &negative,
            &[event_time(5)],
            None,
            &memberships,
        ),
        Err(TopicMeasurementError::InvalidModelInput)
    );
}
