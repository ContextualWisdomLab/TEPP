//! Rolling-origin candidate-K selection uses admitted predictive evidence.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, RollingOriginPartition, admit_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    ModelSelectionError, rolling_origin_prevalence_mean_predictive_log_likelihood,
    select_rolling_origin_predictive_candidate_k,
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
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn cutoff(day: u8) -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("cutoff")
}

fn available(day: u8) -> AvailableTime {
    AvailableTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("available")
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

fn fixture() -> (
    RollingOriginPartition,
    ReferenceTopicTrainingInput,
    MembershipNetwork,
    [Uuid; 2],
    SparseMatrix,
) {
    let training_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let evaluation_ids = [Uuid::from_u128(50), Uuid::from_u128(51)];
    let train_cutoff = cutoff(10);
    let test_cutoff = cutoff(20);
    let mut train_snapshot = CorpusSnapshot::new();
    let mut evaluation_snapshot = CorpusSnapshot::new();
    for document_id in &training_ids {
        train_snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &train_cutoff)
            .expect("training snapshot document");
        evaluation_snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &test_cutoff)
            .expect("cumulative evaluation snapshot training document");
    }
    for document_id in evaluation_ids {
        evaluation_snapshot
            .insert_if_eligible(CorpusDocument::new(document_id, available(15)), &test_cutoff)
            .expect("new evaluation document");
    }
    let partition = admit_rolling_origin_partition(
        &[train_cutoff, test_cutoff],
        0,
        &train_snapshot,
        &evaluation_snapshot,
        &training_ids,
        &evaluation_ids,
        &[],
    )
    .expect("rolling-origin partition");

    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in training_ids
        .iter()
        .copied()
        .chain(evaluation_ids)
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
                training_ids[source],
                training_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }
    let training_counts = SparseMatrix::from_csr(
        6,
        6,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5],
        vec![
            90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
        ],
    )
    .expect("training counts");
    let training_times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let training_input = ReferenceTopicTrainingInput::new(
        &train_snapshot,
        training_ids,
        &training_counts,
        &training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input");
    let evaluation_counts = SparseMatrix::from_csr(
        2,
        6,
        vec![0, 2, 4],
        vec![0, 1, 4, 5],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("evaluation counts");
    (
        partition,
        training_input,
        memberships,
        evaluation_ids,
        evaluation_counts,
    )
}

fn fit(training_input: &ReferenceTopicTrainingInput, topic_count: usize) -> ReferenceTopicTrainingFit {
    let config = ReferenceTopicModelConfig::new(topic_count, vec![7, 11, 19], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    ReferenceTopicTrainingFit::fit(training_input, &config).expect("training fit")
}

#[test]
fn selector_uses_predictive_scores_from_actual_owner_issued_candidate_fits() {
    let (partition, training_input, memberships, evaluation_ids, evaluation_counts) = fixture();
    let k2 = fit(&training_input, 2);
    let k3 = fit(&training_input, 3);
    let evaluation_times = [event_time(15), event_time(16)];
    let score2 = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &partition,
        &k2,
        &evaluation_ids,
        &evaluation_counts,
        &evaluation_times,
        None,
        &memberships,
    )
    .expect("K=2 predictive score");
    let score3 = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &partition,
        &k3,
        &evaluation_ids,
        &evaluation_counts,
        &evaluation_times,
        None,
        &memberships,
    )
    .expect("K=3 predictive score");
    let expected = if score3 > score2 { 3 } else { 2 };

    assert_eq!(
        select_rolling_origin_predictive_candidate_k(
            &partition,
            &[k2.clone(), k3.clone()],
            &evaluation_ids,
            &evaluation_counts,
            &evaluation_times,
            None,
            &memberships,
        )
        .expect("predictive candidate selection"),
        expected
    );
    assert_eq!(
        select_rolling_origin_predictive_candidate_k(
            &partition,
            &[k3, k2],
            &evaluation_ids,
            &evaluation_counts,
            &evaluation_times,
            None,
            &memberships,
        )
        .expect("order-invariant predictive candidate selection"),
        expected
    );
}

#[test]
fn selector_fails_closed_for_empty_or_duplicate_candidate_dimensions() {
    let (partition, training_input, memberships, evaluation_ids, evaluation_counts) = fixture();
    let evaluation_times = [event_time(15), event_time(16)];
    assert_eq!(
        select_rolling_origin_predictive_candidate_k(
            &partition,
            &[],
            &evaluation_ids,
            &evaluation_counts,
            &evaluation_times,
            None,
            &memberships,
        ),
        Err(ModelSelectionError::EmptyCandidateSet)
    );

    let k2 = fit(&training_input, 2);
    assert_eq!(
        select_rolling_origin_predictive_candidate_k(
            &partition,
            &[k2.clone(), k2],
            &evaluation_ids,
            &evaluation_counts,
            &evaluation_times,
            None,
            &memberships,
        ),
        Err(ModelSelectionError::DuplicateCandidateK)
    );
}
