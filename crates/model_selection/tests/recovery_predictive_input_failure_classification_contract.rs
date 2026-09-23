//! Structural held-out payload defects must not become numerical recovery failures.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, admit_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    ModelSelectionError, admit_recovery_replication_result,
    rolling_origin_prevalence_mean_predictive_log_likelihood,
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

fn transition(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
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
    .expect("forward transition")
}

#[test]
fn malformed_evaluation_geometry_invalidates_recovery_instead_of_entering_failure_denominator() {
    let training_ids = [
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(3),
    ];
    let evaluation_id = Uuid::from_u128(10);
    let train_cutoff = cutoff(10);
    let test_cutoff = cutoff(20);

    let mut training_snapshot = CorpusSnapshot::new();
    let mut evaluation_snapshot = CorpusSnapshot::new();
    for document_id in training_ids {
        training_snapshot
            .insert_if_eligible(CorpusDocument::new(document_id, available(1)), &train_cutoff)
            .expect("training snapshot document");
        evaluation_snapshot
            .insert_if_eligible(CorpusDocument::new(document_id, available(1)), &test_cutoff)
            .expect("cumulative evaluation snapshot training document");
    }
    evaluation_snapshot
        .insert_if_eligible(CorpusDocument::new(evaluation_id, available(15)), &test_cutoff)
        .expect("held-out document");

    let partition = admit_rolling_origin_partition(
        &[train_cutoff, test_cutoff],
        0,
        &training_snapshot,
        &evaluation_snapshot,
        &training_ids,
        &[evaluation_id],
        &[],
    )
    .expect("admitted rolling-origin partition");

    let group = GroupId::from_uuid(Uuid::from_u128(100));
    let mut memberships = MembershipNetwork::new();
    for document_id in training_ids.into_iter().chain([evaluation_id]) {
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
    relations
        .insert(transition(training_ids[0], training_ids[1], 1, 2))
        .expect("first transition");
    relations
        .insert(transition(training_ids[1], training_ids[2], 2, 3))
        .expect("second transition");

    let training_counts = SparseMatrix::from_csr(
        3,
        2,
        vec![0, 2, 4, 6],
        vec![0, 1, 0, 1, 0, 1],
        vec![90.0, 10.0, 60.0, 40.0, 10.0, 90.0],
    )
    .expect("training counts");
    let training_input = ReferenceTopicTrainingInput::new(
        &training_snapshot,
        training_ids.to_vec(),
        &training_counts,
        &[event_time(1), event_time(2), event_time(3)],
        None,
        &memberships,
        &relations,
    )
    .expect("training input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    let training_fit =
        ReferenceTopicTrainingFit::fit(&training_input, &config).expect("training fit");

    // The partition identities are valid, but this held-out matrix declares a
    // three-term vocabulary against the fit's exact two-term training vocabulary.
    let malformed_evaluation_counts = SparseMatrix::from_csr(
        1,
        3,
        vec![0, 2],
        vec![0, 2],
        vec![70.0, 30.0],
    )
    .expect("malformed evaluation counts remain a valid sparse matrix");
    let score = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &partition,
        &training_fit,
        &[evaluation_id],
        &malformed_evaluation_counts,
        &[event_time(15)],
        None,
        &memberships,
    );

    assert_eq!(
        score,
        Err(ModelSelectionError::PredictiveEvaluationInputInvalid)
    );
    assert_eq!(
        admit_recovery_replication_result(score),
        Err(ModelSelectionError::PredictiveEvaluationInputInvalid),
        "deterministic held-out geometry invalidity must stay outside the failed-replication denominator"
    );
}
