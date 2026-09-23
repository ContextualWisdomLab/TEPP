//! Simulator-backed interval coverage keeps rolling-origin windows clustered by DGP replication.

use std::collections::{BTreeMap, BTreeSet};

use corpus_split::{
    CorpusDocument, CorpusSnapshot, LeakageLink, LeakageLinkKind,
    admit_expanding_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{FittedCandidateKConfig, fit_declared_recovery_candidates};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
};
use tepp_simulation::{
    DocumentMethodEffect, SimulatedDocument, SimulationConfig, TruthManifest, generate,
};
use topic_measurement::{
    FitBoundDocumentMarginalCovariance, FittedDocumentCoordinateSummary, ReferenceTopicTrainingInput,
    SparseMatrix, additive_log_ratio,
};
use uuid::Uuid;
use validation_core::{
    align_topic_probability_rows, interval_coverage, normal_marginal_interval_bounds,
    realign_additive_log_ratio, realign_additive_log_ratio_covariance,
    summarize_windowed_coverage_replications,
};

const REPLICATION_SEEDS: [u64; 4] = [101, 211, 307, 401];
const FIRST_TRAINING_EVENT_INDEX: usize = 3;
const NORMAL_95_PERCENT_CRITICAL_VALUE: f64 = 1.959_963_984_540_054;

fn simulation_config(seed: u64) -> SimulationConfig {
    SimulationConfig::new(
        seed, 9, 2, 3, 12, 6, 0, 0, 500, 3_000, 3_000, 3_000,
    )
    .expect("realistic recovery simulation config")
}

fn cutoff_after_event(manifest: &TruthManifest, event_id: Uuid) -> KnowledgeCutoff {
    let latest = manifest
        .documents()
        .iter()
        .filter(|document| document.event_id() == event_id)
        .map(SimulatedDocument::available_time)
        .max_by_key(|available_time| available_time.instant())
        .expect("every generated event owns documents");
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
                            .expect("canonical simulation membership role"),
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

fn truth_topic_state_by_document(manifest: &TruthManifest) -> BTreeMap<Uuid, Vec<f64>> {
    manifest
        .topic_truth()
        .document_states()
        .iter()
        .map(|state| (state.document_id(), state.topic_mixture().to_vec()))
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

fn truth_k_window_coverage(
    manifest: &TruthManifest,
    fit: &topic_measurement::ReferenceTopicTrainingFit,
) -> f64 {
    let reference_fit = fit.reference_fit();
    let fitted_model = reference_fit.model();
    let truth_topics = manifest.topic_truth().topic_term_probabilities();
    let alignment = align_topic_probability_rows(truth_topics, &fitted_model.topic_term_probabilities)
        .expect("truth-K topic alignment");
    let coordinate_summary = FittedDocumentCoordinateSummary::from_bound_fit(reference_fit)
        .expect("fit-bound document coordinates");
    let truth_by_document = truth_topic_state_by_document(manifest);
    let topic_ids: Vec<_> = (0..coordinate_summary.topic_count())
        .map(|index| {
            Uuid::from_u128(
                10_000 + u128::try_from(index).expect("bounded topic index fits u128"),
            )
        })
        .collect();
    let document_ids: Vec<_> = coordinate_summary
        .rows()
        .iter()
        .map(|row| row.document_id())
        .collect();
    let marginals = FitBoundDocumentMarginalCovariance::from_bound_fit_many(
        reference_fit,
        topic_ids,
        &document_ids,
    )
    .expect("one-factorization full-joint document covariance batch");

    let mut truth = Vec::new();
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (row, marginal) in coordinate_summary.rows().iter().zip(&marginals) {
        assert_eq!(
            marginal.document_id(),
            row.document_id(),
            "batch covariance order must remain bound to fitted document order"
        );
        let fitted_location: Vec<_> = row
            .coordinates()
            .iter()
            .map(|coordinate| coordinate.location())
            .collect();
        let aligned_location = realign_additive_log_ratio(&alignment, &fitted_location)
            .expect("truth-basis fitted ALR location");
        assert_eq!(
            marginal.topic_basis_identity(),
            coordinate_summary.topic_basis_identity(),
            "location and covariance must belong to the same fitted topic basis"
        );
        let aligned_covariance =
            realign_additive_log_ratio_covariance(&alignment, marginal.values())
                .expect("truth-basis covariance");
        let bounds = normal_marginal_interval_bounds(
            &aligned_location,
            &aligned_covariance,
            NORMAL_95_PERCENT_CRITICAL_VALUE,
        )
        .expect("covariance-bound marginal intervals");
        let truth_state = truth_by_document
            .get(&row.document_id())
            .expect("fit-retained document has simulator truth");
        let truth_alr = additive_log_ratio(truth_state).expect("simulator truth ALR");
        assert_eq!(truth_alr.len(), aligned_location.len());
        truth.extend_from_slice(&truth_alr);
        lower.extend_from_slice(bounds.lower());
        upper.extend_from_slice(bounds.upper());
    }

    interval_coverage(&truth, &lower, &upper).expect("window-level interval coverage")
}

#[allow(clippy::too_many_lines)]
fn replication_window_coverages(seed: u64) -> Vec<f64> {
    let manifest = generate(simulation_config(seed)).expect("known-truth simulation");
    manifest
        .verify_invariants()
        .expect("truth manifest invariants");
    let cutoffs = recovery_cutoffs(&manifest);
    assert_eq!(cutoffs.len(), 6, "five rolling-origin windows are declared");

    let memberships = membership_network(&manifest);
    let relations = relation_graph(&manifest);
    let leakage = leakage_links(&manifest);
    let counts_by_document = topic_counts_by_document(&manifest);
    let event_times = event_time_by_document(&manifest);
    let vocabulary_size = usize::try_from(manifest.topic_truth().vocabulary_size())
        .expect("vocabulary size fits usize");
    let truth_k = manifest.topic_truth().true_k();
    assert_eq!(truth_k, 3, "coverage fixture uses the declared K=3 DGP");
    let config = FittedCandidateKConfig::new(vec![truth_k], vec![7, 11, 19], 2_000, 0.001)
        .expect("truth-K recovery fit configuration");

    let mut coverage_by_window = Vec::new();
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
        let training_document_ids: Vec<_> = expanding
            .partition()
            .training_document_ids()
            .iter()
            .copied()
            .collect();
        let training_document_term =
            count_matrix(&training_document_ids, vocabulary_size, &counts_by_document);
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
        let fits = fit_declared_recovery_candidates(&training_input, &config)
            .expect("truth-K fit must be structurally admissible");
        assert_eq!(fits.len(), 1);
        coverage_by_window.push(truth_k_window_coverage(&manifest, &fits[0]));
    }
    coverage_by_window
}

#[test]
fn repeated_truth_k_interval_coverage_uses_dgp_not_interval_count_for_monte_carlo() {
    let coverage_by_replication: Vec<_> = REPLICATION_SEEDS
        .iter()
        .copied()
        .map(replication_window_coverages)
        .collect();
    assert!(coverage_by_replication.iter().all(|windows| windows.len() == 5));
    assert!(coverage_by_replication.iter().flatten().all(|coverage| {
        coverage.is_finite() && (0.0..=1.0).contains(coverage)
    }));

    let summary = summarize_windowed_coverage_replications(
        &coverage_by_replication,
        0.025,
        0.975,
    )
    .expect("between-DGP Monte Carlo coverage summary");

    assert_eq!(summary.replication_count, REPLICATION_SEEDS.len());
    assert!((0.0..=1.0).contains(&summary.mean));
    assert!(summary.standard_deviation.is_finite());
    assert!(summary.standard_error.is_finite());
    assert!((0.0..=1.0).contains(&summary.percentile_lower));
    assert!((0.0..=1.0).contains(&summary.percentile_upper));
}
