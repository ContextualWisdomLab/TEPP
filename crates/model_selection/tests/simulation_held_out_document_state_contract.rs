//! Held-out document-state recovery must remain frozen-fit and out-of-temporal-sample.
//!
//! This contract proves that the #719 local-state owner consumes only later
//! rolling-origin evaluation rows, that #720 binds those rows to the admitted
//! partition, and that held-out state recovery is collapsed within each DGP
//! replication before denominator-preserving Monte Carlo aggregation.

use std::collections::{BTreeMap, BTreeSet};

use corpus_split::{
    CorpusDocument, CorpusSnapshot, LeakageLink, LeakageLinkKind,
    RollingOriginPartition, admit_expanding_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    FittedCandidateKConfig, ModelSelectionError, admit_recovery_replication_result,
    fit_declared_recovery_candidates, rolling_origin_held_out_document_topic_proportions,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
};
use tepp_simulation::{
    DocumentMethodEffect, SimulatedDocument, SimulationConfig, TruthManifest, generate,
};
use topic_measurement::{ReferenceTopicTrainingFit, ReferenceTopicTrainingInput, SparseMatrix};
use uuid::Uuid;
use validation_core::{
    align_topic_probability_rows, mean_absolute_parameter_bias, realign_topic_probability_rows,
    root_mean_square_error, summarize_recovery_metric_replications,
};

const SEED: u64 = 719_001;
const REPLICATION_SEEDS: [u64; 4] = [101, 211, 307, 401];
const FIRST_TRAINING_EVENT_INDEX: usize = 3;

fn simulation_config(seed: u64) -> SimulationConfig {
    SimulationConfig::new(
        seed, 9, 2, 3, 12, 6, 0, 0, 500, 3_000, 3_000, 3_000,
    )
    .expect("held-out recovery simulation config")
}

fn cutoff_after_event(manifest: &TruthManifest, event_id: Uuid) -> KnowledgeCutoff {
    let latest = manifest
        .documents()
        .iter()
        .filter(|document| document.event_id() == event_id)
        .map(SimulatedDocument::available_time)
        .max_by_key(|available_time| available_time.instant())
        .expect("generated event owns documents");
    KnowledgeCutoff::parse_rfc3339(&latest.to_rfc3339()).expect("availability cutoff")
}

fn recovery_cutoffs(manifest: &TruthManifest) -> Vec<KnowledgeCutoff> {
    manifest.events()[FIRST_TRAINING_EVENT_INDEX..]
        .iter()
        .map(|event| cutoff_after_event(manifest, event.event_id()))
        .collect()
}

fn snapshot_at(manifest: &TruthManifest, cutoff: &KnowledgeCutoff) -> CorpusSnapshot {
    let mut snapshot = CorpusSnapshot::new();
    for document in manifest.documents() {
        if document.available_time().instant() <= cutoff.instant() {
            snapshot
                .insert_if_eligible(
                    CorpusDocument::new(document.document_id(), document.available_time()),
                    cutoff,
                )
                .expect("cutoff-eligible simulated document");
        }
    }
    snapshot
}

fn event_time_by_document(manifest: &TruthManifest) -> BTreeMap<Uuid, EventTime> {
    manifest
        .documents()
        .iter()
        .map(|document| {
            (
                document.document_id(),
                document
                    .observed_event_time()
                    .expect("scientific fixture declares zero EventTime missingness"),
            )
        })
        .collect()
}

fn membership_network(manifest: &TruthManifest) -> MembershipNetwork {
    let event_times = event_time_by_document(manifest);
    let mut network = MembershipNetwork::new();
    for document in manifest.documents() {
        let event_time = event_times[&document.document_id()];
        for membership in document.memberships() {
            network
                .insert(
                    MembershipAssignment::new(
                        MemberId::from_uuid(document.document_id()),
                        GroupId::from_uuid(membership.group_id()),
                        MembershipRole::from_wire_name(membership.role_label())
                            .expect("simulation role belongs to canonical vocabulary"),
                        MembershipWeight::new(f64::from(membership.weight_bps()) / 10_000.0)
                            .expect("simulation membership weight"),
                        event_time,
                        event_time,
                    )
                    .expect("simulation membership validity"),
                )
                .expect("unique simulation membership assignment");
        }
    }
    network
}

fn leakage_links(manifest: &TruthManifest) -> Vec<LeakageLink> {
    let mut links = Vec::new();
    let mut documents_by_event: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for document in manifest.documents() {
        documents_by_event
            .entry(document.event_id())
            .or_default()
            .push(document.document_id());
        if let Some(parent) = document.parent_document_id() {
            let kind = match document.method_effect() {
                DocumentMethodEffect::Revision => LeakageLinkKind::Revision,
                DocumentMethodEffect::Translation => LeakageLinkKind::Translation,
                _ => LeakageLinkKind::CopiedVariant,
            };
            links.push(LeakageLink {
                left: parent,
                right: document.document_id(),
                kind,
            });
        }
    }
    for document_ids in documents_by_event.values_mut() {
        document_ids.sort_unstable();
        for pair in document_ids.windows(2) {
            links.push(LeakageLink {
                left: pair[0],
                right: pair[1],
                kind: LeakageLinkKind::SameEpisode,
            });
        }
    }
    links
}

fn point_interval(time: EventTime) -> TemporalInterval<EventTime> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(time),
        TemporalBoundary::Included(time),
        TemporalPrecision::Second,
    )
    .expect("point EventTime interval")
}

fn relation_graph(manifest: &TruthManifest) -> RelationGraph {
    let event_times = event_time_by_document(manifest);
    let mut relations = RelationGraph::new();
    for (source, target) in manifest
        .observed_document_transition_pairs()
        .expect("observed transition projection")
    {
        relations
            .insert(
                RelationEdge::new(
                    RelationKind::TransitionsTo,
                    RelationEndpointId::from_uuid(source),
                    RelationEndpointId::from_uuid(target),
                    RelationEvidenceStatus::Observed,
                    point_interval(event_times[&source]),
                    point_interval(event_times[&target]),
                )
                .expect("forward observed transition"),
            )
            .expect("unique observed transition");
    }
    relations
}

fn topic_counts_by_document(manifest: &TruthManifest) -> BTreeMap<Uuid, Vec<u32>> {
    manifest
        .topic_truth()
        .document_states()
        .iter()
        .map(|state| (state.document_id(), state.term_counts().to_vec()))
        .collect()
}

fn count_matrix(
    document_ids: &[Uuid],
    vocabulary_size: usize,
    counts_by_document: &BTreeMap<Uuid, Vec<u32>>,
) -> SparseMatrix {
    let mut offsets = Vec::with_capacity(document_ids.len() + 1);
    let mut indices = Vec::new();
    let mut values = Vec::new();
    offsets.push(0);
    for document_id in document_ids {
        for (term, count) in counts_by_document[document_id]
            .iter()
            .copied()
            .enumerate()
        {
            if count > 0 {
                indices.push(term);
                values.push(f64::from(count));
            }
        }
        offsets.push(indices.len());
    }
    SparseMatrix::from_csr(
        document_ids.len(),
        vocabulary_size,
        offsets,
        indices,
        values,
    )
    .expect("simulator count matrix")
}

fn mean(values: &[f64]) -> f64 {
    let count = u32::try_from(values.len()).expect("CI-scale window count fits u32");
    values.iter().sum::<f64>() / f64::from(count)
}

fn held_out_metrics(
    manifest: &TruthManifest,
    partition: &RollingOriginPartition,
    fit: &ReferenceTopicTrainingFit,
    evaluation_document_ids: &[Uuid],
    evaluation_document_term: &SparseMatrix,
    evaluation_event_times: &[EventTime],
    memberships: &MembershipNetwork,
) -> Result<(f64, f64), ModelSelectionError> {
    let recovered = rolling_origin_held_out_document_topic_proportions(
        partition,
        fit,
        evaluation_document_ids,
        evaluation_document_term,
        evaluation_event_times,
        None,
        memberships,
    )?;
    let alignment = align_topic_probability_rows(
        manifest.topic_truth().topic_term_probabilities(),
        &fit.reference_fit().model().topic_term_probabilities,
    )
    .expect("truth-K global topic alignment");
    let aligned = realign_topic_probability_rows(&alignment, &recovered)
        .expect("held-out states in simulator truth topic order");
    let truth_state_by_document: BTreeMap<_, _> = manifest
        .topic_truth()
        .document_states()
        .iter()
        .map(|state| (state.document_id(), state.topic_mixture()))
        .collect();
    let mut truth_flat = Vec::new();
    let mut recovered_flat = Vec::new();
    for (document_id, recovered_state) in evaluation_document_ids.iter().zip(&aligned) {
        let truth_state = *truth_state_by_document
            .get(document_id)
            .expect("evaluation identity retains simulator truth");
        assert_eq!(truth_state.len(), recovered_state.len());
        truth_flat.extend_from_slice(truth_state);
        recovered_flat.extend_from_slice(recovered_state);
    }
    let rmse = root_mean_square_error(&truth_flat, &recovered_flat)
        .expect("held-out document-state RMSE");
    let absolute_residual = mean_absolute_parameter_bias(
        &truth_flat,
        std::slice::from_ref(&recovered_flat),
    )
    .expect("held-out non-cancelling state residual");
    Ok((rmse, absolute_residual))
}

#[test]
fn simulated_future_rows_recover_local_states_without_refitting_or_identity_rebinding() {
    let manifest = generate(simulation_config(SEED)).expect("known-truth simulation");
    manifest
        .verify_invariants()
        .expect("truth manifest invariants");

    let training_cutoff = cutoff_after_event(&manifest, manifest.events()[3].event_id());
    let evaluation_cutoff = cutoff_after_event(&manifest, manifest.events()[4].event_id());
    let cutoffs = vec![training_cutoff.clone(), evaluation_cutoff.clone()];
    let training_snapshot = snapshot_at(&manifest, &training_cutoff);
    let evaluation_snapshot = snapshot_at(&manifest, &evaluation_cutoff);
    let training_snapshot_ids: BTreeSet<_> = training_snapshot.document_ids().collect();
    let evaluation_document_ids: Vec<_> = evaluation_snapshot
        .document_ids()
        .filter(|document_id| !training_snapshot_ids.contains(document_id))
        .collect();
    assert!(!evaluation_document_ids.is_empty());

    let expanding = admit_expanding_rolling_origin_partition(
        &cutoffs,
        0,
        &training_snapshot,
        &evaluation_snapshot,
        &evaluation_document_ids,
        &leakage_links(&manifest),
    )
    .expect("owner-admitted leakage-safe held-out partition");
    let partition = expanding.partition().clone();
    let training_document_ids: Vec<_> =
        partition.training_document_ids().iter().copied().collect();
    let admitted_evaluation_ids: Vec<_> =
        partition.evaluation_document_ids().iter().copied().collect();

    let memberships = membership_network(&manifest);
    let relations = relation_graph(&manifest);
    let event_times = event_time_by_document(&manifest);
    let counts_by_document = topic_counts_by_document(&manifest);
    let vocabulary_size = usize::try_from(manifest.topic_truth().vocabulary_size())
        .expect("vocabulary size fits usize");
    let training_document_term = count_matrix(
        &training_document_ids,
        vocabulary_size,
        &counts_by_document,
    );
    let training_event_times: Vec<_> = training_document_ids
        .iter()
        .map(|document_id| event_times[document_id])
        .collect();
    let training_input = ReferenceTopicTrainingInput::new(
        &training_snapshot,
        training_document_ids.clone(),
        &training_document_term,
        &training_event_times,
        None,
        &memberships,
        &relations,
    )
    .expect("owner-admitted training state");
    let config = FittedCandidateKConfig::new(
        vec![manifest.topic_truth().true_k()],
        vec![7, 11, 19],
        2_000,
        0.001,
    )
    .expect("truth-K recovery fit config");
    let fits = fit_declared_recovery_candidates(&training_input, &config)
        .expect("truth-K training fit");
    let fit = &fits[0];
    let frozen_model = fit.reference_fit().model().clone();

    let evaluation_document_term = count_matrix(
        &admitted_evaluation_ids,
        vocabulary_size,
        &counts_by_document,
    );
    let evaluation_event_times: Vec<_> = admitted_evaluation_ids
        .iter()
        .map(|document_id| event_times[document_id])
        .collect();
    let (rmse, absolute_residual) = held_out_metrics(
        &manifest,
        &partition,
        fit,
        &admitted_evaluation_ids,
        &evaluation_document_term,
        &evaluation_event_times,
        &memberships,
    )
    .expect("partition-bound frozen-fit held-out state recovery");
    assert_eq!(fit.reference_fit().model(), &frozen_model);
    assert!(rmse.is_finite() && rmse >= 0.0);
    assert!(absolute_residual.is_finite() && absolute_residual >= 0.0);

    let mut rebound_ids = admitted_evaluation_ids.clone();
    rebound_ids[0] = training_document_ids[0];
    assert_eq!(
        rolling_origin_held_out_document_topic_proportions(
            &partition,
            fit,
            &rebound_ids,
            &evaluation_document_term,
            &evaluation_event_times,
            None,
            &memberships,
        ),
        Err(ModelSelectionError::PartitionInputMismatch)
    );
}

fn run_recovery_replication(seed: u64) -> Result<Option<(f64, f64)>, ModelSelectionError> {
    let manifest = generate(simulation_config(seed)).expect("known-truth simulation");
    manifest
        .verify_invariants()
        .expect("truth manifest invariants");
    let cutoffs = recovery_cutoffs(&manifest);
    assert_eq!(cutoffs.len(), 6, "five held-out windows are predeclared");

    let memberships = membership_network(&manifest);
    let relations = relation_graph(&manifest);
    let leakage = leakage_links(&manifest);
    let event_times = event_time_by_document(&manifest);
    let counts_by_document = topic_counts_by_document(&manifest);
    let vocabulary_size = usize::try_from(manifest.topic_truth().vocabulary_size())
        .expect("vocabulary size fits usize");
    let config = FittedCandidateKConfig::new(
        vec![manifest.topic_truth().true_k()],
        vec![7, 11, 19],
        2_000,
        0.001,
    )
    .expect("predeclared truth-K recovery design");

    let mut window_rmse = Vec::new();
    let mut window_absolute_residual = Vec::new();
    for (window_index, cutoff_pair) in cutoffs.windows(2).enumerate() {
        let training_snapshot = snapshot_at(&manifest, &cutoff_pair[0]);
        let evaluation_snapshot = snapshot_at(&manifest, &cutoff_pair[1]);
        let training_snapshot_ids: BTreeSet<_> = training_snapshot.document_ids().collect();
        let evaluation_document_ids: Vec<_> = evaluation_snapshot
            .document_ids()
            .filter(|document_id| !training_snapshot_ids.contains(document_id))
            .collect();
        assert!(!evaluation_document_ids.is_empty());
        let expanding = admit_expanding_rolling_origin_partition(
            &cutoffs,
            window_index,
            &training_snapshot,
            &evaluation_snapshot,
            &evaluation_document_ids,
            &leakage,
        )
        .expect("owner-admitted leakage-safe expanding partition");
        let partition = expanding.partition().clone();
        let training_document_ids: Vec<_> =
            partition.training_document_ids().iter().copied().collect();
        let admitted_evaluation_ids: Vec<_> =
            partition.evaluation_document_ids().iter().copied().collect();
        let training_document_term = count_matrix(
            &training_document_ids,
            vocabulary_size,
            &counts_by_document,
        );
        let training_event_times: Vec<_> = training_document_ids
            .iter()
            .map(|document_id| event_times[document_id])
            .collect();
        let training_input = ReferenceTopicTrainingInput::new(
            &training_snapshot,
            training_document_ids,
            &training_document_term,
            &training_event_times,
            None,
            &memberships,
            &relations,
        )
        .expect("owner-admitted numerical training state");
        let fits = match admit_recovery_replication_result(fit_declared_recovery_candidates(
            &training_input,
            &config,
        ))? {
            Some(fits) => fits,
            None => return Ok(None),
        };
        let fit = &fits[0];
        let evaluation_document_term = count_matrix(
            &admitted_evaluation_ids,
            vocabulary_size,
            &counts_by_document,
        );
        let evaluation_event_times: Vec<_> = admitted_evaluation_ids
            .iter()
            .map(|document_id| event_times[document_id])
            .collect();
        let metrics = match admit_recovery_replication_result(held_out_metrics(
            &manifest,
            &partition,
            fit,
            &admitted_evaluation_ids,
            &evaluation_document_term,
            &evaluation_event_times,
            &memberships,
        ))? {
            Some(metrics) => metrics,
            None => return Ok(None),
        };
        window_rmse.push(metrics.0);
        window_absolute_residual.push(metrics.1);
    }

    Ok(Some((mean(&window_rmse), mean(&window_absolute_residual))))
}

#[test]
fn repeated_held_out_state_recovery_preserves_attempted_dgp_denominator() {
    let mut successful_rmse = Vec::new();
    let mut successful_absolute_residual = Vec::new();
    for outcome in REPLICATION_SEEDS
        .iter()
        .copied()
        .map(run_recovery_replication)
    {
        match outcome.expect("structurally valid held-out recovery experiment") {
            Some((rmse, absolute_residual)) => {
                successful_rmse.push(rmse);
                successful_absolute_residual.push(absolute_residual);
            }
            None => {}
        }
    }

    let rmse_summary = summarize_recovery_metric_replications(
        REPLICATION_SEEDS.len(),
        &successful_rmse,
        0.025,
        0.975,
    )
    .expect("held-out state RMSE recovery summary");
    let residual_summary = summarize_recovery_metric_replications(
        REPLICATION_SEEDS.len(),
        &successful_absolute_residual,
        0.025,
        0.975,
    )
    .expect("held-out state absolute-residual recovery summary");

    for summary in [rmse_summary, residual_summary] {
        assert_eq!(summary.attempted_replication_count(), REPLICATION_SEEDS.len());
        assert_eq!(
            summary.successful_replication_count() + summary.failure_count(),
            summary.attempted_replication_count()
        );
        assert!(summary.failure_rate().is_finite());
        assert!(summary.failure_rate_standard_error().is_finite());
        assert!(summary.successful_replication_count() >= 2);
        let monte_carlo = summary
            .successful_metric_summary()
            .expect("two or more successful DGP replications retain MC uncertainty");
        assert_eq!(monte_carlo.replication_count, summary.successful_replication_count());
        assert!(monte_carlo.mean.is_finite());
        assert!(monte_carlo.standard_deviation.is_finite());
        assert!(monte_carlo.standard_error.is_finite());
        assert!(monte_carlo.percentile_lower.is_finite());
        assert!(monte_carlo.percentile_upper.is_finite());
    }
}
