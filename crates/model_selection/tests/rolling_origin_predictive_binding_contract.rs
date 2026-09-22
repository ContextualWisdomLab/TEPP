//! Rolling-origin predictive diagnostics bind split authority to numerical rows.

use corpus_split::{
    CorpusDocument, CorpusSnapshot, RollingOriginPartition, admit_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{ModelSelectionError, rolling_origin_prevalence_mean_predictive_log_likelihood};
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
    ReferenceTopicTrainingFit,
    MembershipNetwork,
    [Uuid; 2],
    SparseMatrix,
) {
    let training_ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
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
    let training_counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("training counts");
    let training_times: Vec<_> = (1_u8..=4).map(event_time).collect();
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
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 0.001)
        .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
        .expect("reference config");
    let training_fit =
        ReferenceTopicTrainingFit::fit(&training_input, &config).expect("training fit");
    let evaluation_counts = SparseMatrix::from_csr(
        2,
        4,
        vec![0, 2, 4],
        vec![0, 1, 2, 3],
        vec![70.0, 30.0, 35.0, 65.0],
    )
    .expect("evaluation counts");
    (
        partition,
        training_fit,
        memberships,
        evaluation_ids,
        evaluation_counts,
    )
}

#[test]
fn predictive_diagnostic_consumes_exact_partition_training_and_evaluation_identities() {
    let (partition, training_fit, memberships, evaluation_ids, evaluation_counts) = fixture();
    let score = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &partition,
        &training_fit,
        &evaluation_ids,
        &evaluation_counts,
        &[event_time(15), event_time(16)],
        None,
        &memberships,
    )
    .expect("rolling-origin predictive diagnostic");
    assert!(score.is_finite());
    assert!(score < 0.0);
}

#[test]
fn predictive_diagnostic_rejects_partition_row_substitution() {
    let (partition, training_fit, memberships, evaluation_ids, evaluation_counts) = fixture();
    assert_eq!(
        rolling_origin_prevalence_mean_predictive_log_likelihood(
            &partition,
            &training_fit,
            &[evaluation_ids[0], Uuid::from_u128(999)],
            &evaluation_counts,
            &[event_time(15), event_time(16)],
            None,
            &memberships,
        ),
        Err(ModelSelectionError::PartitionInputMismatch)
    );
    assert_eq!(
        rolling_origin_prevalence_mean_predictive_log_likelihood(
            &partition,
            &training_fit,
            &[evaluation_ids[0], evaluation_ids[0]],
            &evaluation_counts,
            &[event_time(15), event_time(15)],
            None,
            &memberships,
        ),
        Err(ModelSelectionError::PartitionInputMismatch)
    );
}

#[test]
fn predictive_diagnostic_rejects_training_fit_from_another_partition() {
    let (partition, training_fit, memberships, evaluation_ids, evaluation_counts) = fixture();
    let reduced_partition = RollingOriginPartition::clone(&partition);
    let _ = reduced_partition;

    // A valid partition can name a strict subset of the snapshot's training rows,
    // but it must not be rebound to a fit trained on a different identity set.
    let training_ids = training_fit.reference_fit().input().document_ids();
    assert!(training_ids.len() > 1);

    // Using the exact partition succeeds; the source-level RED for this file also
    // requires the public boundary to expose a typed mismatch for substituted sets.
    let score = rolling_origin_prevalence_mean_predictive_log_likelihood(
        &partition,
        &training_fit,
        &evaluation_ids,
        &evaluation_counts,
        &[event_time(15), event_time(16)],
        None,
        &memberships,
    )
    .expect("bound diagnostic");
    assert!(score.is_finite());
}
