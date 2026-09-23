//! Scientific recovery candidates must share one exact numerical training state.

use corpus_split::{CorpusDocument, CorpusSnapshot, admit_rolling_origin_partition};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{ModelSelectionError, RollingOriginRecoveryEvaluation};
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

fn fit(
    training_input: &ReferenceTopicTrainingInput,
    topic_count: usize,
) -> ReferenceTopicTrainingFit {
    let config = ReferenceTopicModelConfig::new(topic_count, vec![7, 11, 19], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.25, 0.01, 0.05, 0.2))
        .expect("reference config");
    ReferenceTopicTrainingFit::fit(training_input, &config).expect("training fit")
}

#[test]
fn recovery_rejects_candidates_fit_on_different_training_counts_with_same_identities() {
    let cutoffs = [cutoff(10), cutoff(20)];
    let training_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let evaluation_ids = [Uuid::from_u128(50), Uuid::from_u128(51)];

    let mut training_snapshot = CorpusSnapshot::new();
    for document_id in &training_ids {
        training_snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[0])
            .expect("training document");
    }
    let mut evaluation_snapshot = CorpusSnapshot::new();
    for document_id in &training_ids {
        evaluation_snapshot
            .insert_if_eligible(CorpusDocument::new(*document_id, available(1)), &cutoffs[1])
            .expect("historical document");
    }
    for document_id in evaluation_ids {
        evaluation_snapshot
            .insert_if_eligible(CorpusDocument::new(document_id, available(15)), &cutoffs[1])
            .expect("newly available document");
    }

    let partition = admit_rolling_origin_partition(
        &cutoffs,
        0,
        &training_snapshot,
        &evaluation_snapshot,
        &training_ids,
        &evaluation_ids,
        &[],
    )
    .expect("rolling-origin partition");

    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in training_ids.iter().copied().chain(evaluation_ids) {
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

    let counts_a = SparseMatrix::from_csr(
        6,
        6,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5],
        vec![
            90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
        ],
    )
    .expect("training counts a");
    let counts_b = SparseMatrix::from_csr(
        6,
        6,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 2, 3, 2, 3, 4, 5, 4, 5],
        vec![
            70.0, 30.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0, 90.0, 10.0, 85.0, 15.0,
        ],
    )
    .expect("training counts b");
    let training_times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let training_a = ReferenceTopicTrainingInput::new(
        &training_snapshot,
        training_ids.clone(),
        &counts_a,
        &training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input a");
    let training_b = ReferenceTopicTrainingInput::new(
        &training_snapshot,
        training_ids,
        &counts_b,
        &training_times,
        None,
        &memberships,
        &relations,
    )
    .expect("training input b");

    let candidates = [fit(&training_a, 2), fit(&training_b, 3)];
    let evaluation_counts = SparseMatrix::from_csr(
        2,
        6,
        vec![0, 2, 4],
        vec![0, 1, 4, 5],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("evaluation counts");
    let evaluation_times = [event_time(15), event_time(16)];

    assert!(matches!(
        RollingOriginRecoveryEvaluation::new(
            &partition,
            &candidates,
            &evaluation_ids,
            &evaluation_counts,
            &evaluation_times,
            None,
            &memberships,
        ),
        Err(ModelSelectionError::PredictiveCandidateTrainingStateMismatch)
    ));
}
