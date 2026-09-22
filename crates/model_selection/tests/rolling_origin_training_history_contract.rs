//! Rolling-origin candidate-K evaluation must retain previously evaluated evidence in later training states.

use corpus_split::{CorpusDocument, CorpusSnapshot, admit_rolling_origin_partition};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    RollingOriginPredictiveEvaluation, select_rolling_origin_predictive_candidate_k_across_windows,
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
fn multi_window_selection_rejects_training_history_that_forgets_prior_evaluation_rows() {
    let cutoffs = [cutoff(10), cutoff(20), cutoff(30)];
    let training_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let first_evaluation_ids = [Uuid::from_u128(50)];
    let second_evaluation_ids = [Uuid::from_u128(60)];

    let mut snapshot10 = CorpusSnapshot::new();
    for document_id in &training_ids {
        snapshot10
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[0])
            .expect("cutoff-10 training document");
    }
    let mut snapshot20 = CorpusSnapshot::new();
    for document_id in &training_ids {
        snapshot20
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[1])
            .expect("cutoff-20 historical document");
    }
    snapshot20
        .insert_if_eligible(
            CorpusDocument::new(first_evaluation_ids[0], available(15)),
            &cutoffs[1],
        )
        .expect("cutoff-20 newly available document");
    let mut snapshot30 = CorpusSnapshot::new();
    snapshot30
        .insert_if_eligible(
            CorpusDocument::new(second_evaluation_ids[0], available(25)),
            &cutoffs[2],
        )
        .expect("cutoff-30 newly available document");

    let first_partition = admit_rolling_origin_partition(
        &cutoffs,
        0,
        &snapshot10,
        &snapshot20,
        &training_ids,
        &first_evaluation_ids,
        &[],
    )
    .expect("first partition");
    let stale_second_partition = admit_rolling_origin_partition(
        &cutoffs,
        1,
        &snapshot20,
        &snapshot30,
        &training_ids,
        &second_evaluation_ids,
        &[],
    )
    .expect("locally valid but non-cumulative second partition");

    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in training_ids
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
    for index in 0..training_ids.len() - 1 {
        relations
            .insert(relation(
                training_ids[index],
                training_ids[index + 1],
                index as u8 + 1,
                index as u8 + 2,
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
        &snapshot10,
        training_ids,
        &training_counts,
        &training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input");
    let k2 = fit(&training_input, 2);
    let k3 = fit(&training_input, 3);
    let candidates = [k2, k3];

    let first_evaluation_counts =
        SparseMatrix::from_csr(1, 6, vec![0, 2], vec![0, 1], vec![70.0, 30.0])
            .expect("first evaluation counts");
    let second_evaluation_counts =
        SparseMatrix::from_csr(1, 6, vec![0, 2], vec![4, 5], vec![35.0, 65.0])
            .expect("second evaluation counts");
    let first_evaluation_times = [event_time(15)];
    let second_evaluation_times = [event_time(25)];

    let windows = [
        RollingOriginPredictiveEvaluation::new(
            &first_partition,
            &candidates,
            &first_evaluation_ids,
            &first_evaluation_counts,
            &first_evaluation_times,
            None,
            &memberships,
        ),
        RollingOriginPredictiveEvaluation::new(
            &stale_second_partition,
            &candidates,
            &second_evaluation_ids,
            &second_evaluation_counts,
            &second_evaluation_times,
            None,
            &memberships,
        ),
    ];

    assert!(
        select_rolling_origin_predictive_candidate_k_across_windows(&windows).is_err(),
        "a later rolling-origin training state must not forget evidence evaluated in the preceding window"
    );
}
