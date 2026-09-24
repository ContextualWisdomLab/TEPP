//! Multi-window rolling-origin selection aggregates predictive evidence without caller-side voting.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, admit_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    ModelSelectionError, RollingOriginPredictiveEvaluation,
    rolling_origin_prevalence_mean_predictive_log_likelihood,
    select_rolling_origin_predictive_candidate_k_across_windows,
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

fn fit(training_input: &ReferenceTopicTrainingInput, topic_count: usize) -> ReferenceTopicTrainingFit {
    let config = ReferenceTopicModelConfig::new(topic_count, vec![7, 11, 19], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    ReferenceTopicTrainingFit::fit(training_input, &config).expect("training fit")
}

#[test]
fn selector_aggregates_actual_predictive_scores_over_two_canonical_windows() {
    let cutoffs = [cutoff(10), cutoff(20), cutoff(30)];
    let first_training_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let first_evaluation_ids = [Uuid::from_u128(50), Uuid::from_u128(51)];
    let second_evaluation_ids = [Uuid::from_u128(60), Uuid::from_u128(61)];
    let second_training_ids: Vec<_> = first_training_ids
        .iter()
        .copied()
        .chain(first_evaluation_ids)
        .collect();

    let mut snapshot10 = CorpusSnapshot::new();
    for document_id in &first_training_ids {
        snapshot10
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[0])
            .expect("cutoff-10 training document");
    }
    let mut snapshot20 = CorpusSnapshot::new();
    for document_id in &first_training_ids {
        snapshot20
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[1])
            .expect("cutoff-20 historical document");
    }
    for document_id in first_evaluation_ids {
        snapshot20
            .insert_if_eligible(CorpusDocument::new(document_id, available(15)), &cutoffs[1])
            .expect("cutoff-20 newly available document");
    }
    let mut snapshot30 = CorpusSnapshot::new();
    for document_id in second_evaluation_ids {
        snapshot30
            .insert_if_eligible(CorpusDocument::new(document_id, available(25)), &cutoffs[2])
            .expect("cutoff-30 newly available document");
    }

    let first_partition = admit_rolling_origin_partition(
        &cutoffs,
        0,
        &snapshot10,
        &snapshot20,
        &first_training_ids,
        &first_evaluation_ids,
        &[],
    )
    .expect("first rolling-origin partition");
    let second_partition = admit_rolling_origin_partition(
        &cutoffs,
        1,
        &snapshot20,
        &snapshot30,
        &second_training_ids,
        &second_evaluation_ids,
        &[],
    )
    .expect("second rolling-origin partition");

    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in first_training_ids
        .iter()
        .copied()
        .chain(first_evaluation_ids)
        .chain(second_evaluation_ids)
    {
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(document_id),
                    group,
                    MembershipRole::Project,
                    MembershipWeight::full().expect("full membership"),
                    event_time(1),
                    event_time(31),
                )
                .expect("membership"),
            )
            .expect("insert membership");
    }

    let mut relations = RelationGraph::new();
    let ordered_ids: Vec<_> = first_training_ids
        .iter()
        .copied()
        .chain(first_evaluation_ids)
        .chain(second_evaluation_ids)
        .collect();
    let ordered_days = [1_u8, 2, 3, 4, 5, 6, 15, 16, 25, 26];
    for index in 0..ordered_ids.len() - 1 {
        relations
            .insert(relation(
                ordered_ids[index],
                ordered_ids[index + 1],
                ordered_days[index],
                ordered_days[index + 1],
            ))
            .expect("insert relation");
    }

    let first_training_counts = SparseMatrix::from_csr(
        6,
        6,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5],
        vec![
            90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
        ],
    )
    .expect("first training counts");
    let first_training_times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let first_training_input = ReferenceTopicTrainingInput::new(
        &snapshot10,
        first_training_ids.clone(),
        &first_training_counts,
        &first_training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("first training input");

    let second_training_counts = SparseMatrix::from_csr(
        8,
        6,
        vec![0, 2, 4, 6, 8, 10, 12, 14, 16],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5, 0, 1, 4, 5],
        vec![
            90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
            70.0, 30.0, 35.0, 65.0,
        ],
    )
    .expect("second training counts");
    let second_training_times = [
        event_time(1),
        event_time(2),
        event_time(3),
        event_time(4),
        event_time(5),
        event_time(6),
        event_time(15),
        event_time(16),
    ];
    let second_training_input = ReferenceTopicTrainingInput::new(
        &snapshot20,
        second_training_ids,
        &second_training_counts,
        &second_training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("second training input");

    let first_k2 = fit(&first_training_input, 2);
    let first_k3 = fit(&first_training_input, 3);
    let second_k2 = fit(&second_training_input, 2);
    let second_k3 = fit(&second_training_input, 3);

    let first_evaluation_counts = SparseMatrix::from_csr(
        2,
        6,
        vec![0, 2, 4],
        vec![0, 1, 4, 5],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("first evaluation counts");
    let second_evaluation_counts = SparseMatrix::from_csr(
        2,
        6,
        vec![0, 2, 4],
        vec![2, 3, 4, 5],
        vec![65.0, 35.0, 25.0, 75.0],
    )
    .expect("second evaluation counts");
    let first_evaluation_times = [event_time(15), event_time(16)];
    let second_evaluation_times = [event_time(25), event_time(26)];

    let k2_total = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &first_partition,
        &first_k2,
        &first_evaluation_ids,
        &first_evaluation_counts,
        &first_evaluation_times,
        None,
        &memberships,
    )
    .expect("first K=2 score")
        + rolling_origin_prevalence_mean_predictive_log_likelihood(
            &second_partition,
            &second_k2,
            &second_evaluation_ids,
            &second_evaluation_counts,
            &second_evaluation_times,
            None,
            &memberships,
        )
        .expect("second K=2 score");
    let k3_total = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &first_partition,
        &first_k3,
        &first_evaluation_ids,
        &first_evaluation_counts,
        &first_evaluation_times,
        None,
        &memberships,
    )
    .expect("first K=3 score")
        + rolling_origin_prevalence_mean_predictive_log_likelihood(
            &second_partition,
            &second_k3,
            &second_evaluation_ids,
            &second_evaluation_counts,
            &second_evaluation_times,
            None,
            &memberships,
        )
        .expect("second K=3 score");
    let expected = if k3_total > k2_total { 3 } else { 2 };

    let first_candidates = [first_k2.clone(), first_k3.clone()];
    let second_candidates = [second_k3.clone(), second_k2.clone()];
    let windows = [
        RollingOriginPredictiveEvaluation::new(
            &first_partition,
            &first_candidates,
            &first_evaluation_ids,
            &first_evaluation_counts,
            &first_evaluation_times,
            None,
            &memberships,
        ),
        RollingOriginPredictiveEvaluation::new(
            &second_partition,
            &second_candidates,
            &second_evaluation_ids,
            &second_evaluation_counts,
            &second_evaluation_times,
            None,
            &memberships,
        ),
    ];
    assert_eq!(
        select_rolling_origin_predictive_candidate_k_across_windows(&windows)
            .expect("multi-window predictive selection"),
        expected
    );

    let mismatched_candidates = [second_k2];
    let mismatched_windows = [
        windows[0],
        RollingOriginPredictiveEvaluation::new(
            &second_partition,
            &mismatched_candidates,
            &second_evaluation_ids,
            &second_evaluation_counts,
            &second_evaluation_times,
            None,
            &memberships,
        ),
    ];
    assert_eq!(
        select_rolling_origin_predictive_candidate_k_across_windows(&mismatched_windows),
        Err(ModelSelectionError::PredictiveCandidateSetMismatch)
    );

    let disconnected_windows = [windows[1], windows[0]];
    assert_eq!(
        select_rolling_origin_predictive_candidate_k_across_windows(&disconnected_windows),
        Err(ModelSelectionError::RollingOriginWindowMismatch)
    );
}
