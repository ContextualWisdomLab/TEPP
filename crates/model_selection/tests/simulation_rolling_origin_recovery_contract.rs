//! Simulator-backed rolling-origin recovery composes the scientific owner path end to end.

use std::collections::{BTreeMap, BTreeSet};

use corpus_split::{
    CorpusDocument, CorpusSnapshot, CorpusSplitError, LeakageLink, LeakageLinkKind,
    RollingOriginPartition, admit_expanding_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{
    FittedCandidateKConfig, ModelSelectionError, RollingOriginRecoveryEvaluation,
    fit_declared_recovery_candidates, rolling_origin_prevalence_mean_predictive_log_likelihood,
    select_declared_rolling_origin_recovery_candidate_k_for_cutoffs,
    selected_k_recovery_summary_from_results,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
};
use tepp_simulation::{
    DocumentMethodEffect, SimulatedDocument, SimulationConfig, TopicDgpConfig, TruthManifest,
    generate,
};
use topic_measurement::{
    PrevalenceFeature, ReferenceTopicTrainingFit, ReferenceTopicTrainingInput, SparseMatrix,
};
use uuid::Uuid;
use validation_core::{
    LinearTimeBasis, align_topic_probability_rows, mean_absolute_parameter_bias,
    realign_additive_log_ratio, realign_topic_probability_rows,
    reexpress_linear_prevalence_time_basis, root_mean_square_error,
    summarize_recovery_metric_replications,
};

const REPLICATION_SEEDS: [u64; 4] = [101, 211, 307, 401];
const FIRST_TRAINING_EVENT_INDEX: usize = 3;
const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

struct WindowFixture {
    partition: RollingOriginPartition,
    fits: Vec<ReferenceTopicTrainingFit>,
    evaluation_document_ids: Vec<Uuid>,
    evaluation_document_term: SparseMatrix,
    evaluation_event_times: Vec<EventTime>,
    topic_term_rmse: f64,
    aligned_topic_term_probabilities: Vec<f64>,
    document_state_rmse: f64,
    mean_absolute_document_state_residual: f64,
    prevalence_intercept_rmse: f64,
    prevalence_time_slope_rmse: f64,
    mean_absolute_prevalence_intercept_residual: f64,
    mean_absolute_prevalence_time_slope_residual: f64,
}

struct RecoveryReplication {
    selected_k: u32,
    mean_topic_term_rmse: f64,
    mean_absolute_topic_term_bias: f64,
    mean_document_state_rmse: f64,
    mean_absolute_document_state_residual: f64,
    mean_prevalence_intercept_rmse: f64,
    mean_prevalence_time_slope_rmse: f64,
    mean_absolute_prevalence_intercept_residual: f64,
    mean_absolute_prevalence_time_slope_residual: f64,
}

struct WindowRecoveryMetrics {
    topic_term_rmse: f64,
    aligned_topic_term_probabilities: Vec<f64>,
    document_state_rmse: f64,
    mean_absolute_document_state_residual: f64,
    prevalence_intercept_rmse: f64,
    prevalence_time_slope_rmse: f64,
    mean_absolute_prevalence_intercept_residual: f64,
    mean_absolute_prevalence_time_slope_residual: f64,
}

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
                        MembershipRole::from_wire_name(membership.role_label()).expect(
                            "simulation membership role belongs to the canonical vocabulary",
                        ),
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
            let method = document.method_effect();
            let kind = if method == DocumentMethodEffect::Revision {
                LeakageLinkKind::Revision
            } else if method == DocumentMethodEffect::Translation {
                LeakageLinkKind::Translation
            } else {
                LeakageLinkKind::CopiedVariant
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

#[allow(clippy::cast_precision_loss)]
fn seconds_from_origin(origin: EventTime, time: EventTime) -> f64 {
    let delta = time
        .instant()
        .as_nanosecond()
        .checked_sub(origin.instant().as_nanosecond())
        .expect("generated training EventTime cannot precede simulator origin");
    delta as f64 / NANOS_PER_SECOND
}

fn window_recovery_metrics(
    manifest: &TruthManifest,
    fits: &[ReferenceTopicTrainingFit],
) -> WindowRecoveryMetrics {
    let truth_topic_terms = manifest.topic_truth().topic_term_probabilities();
    let truth_k = usize::try_from(manifest.topic_truth().true_k()).expect("true topic count fits usize");
    let truth_fit = fits
        .iter()
        .find(|fit| fit.reference_fit().model().topic_term_probabilities.len() == truth_k)
        .expect("complete declared grid retains truth-K fit");
    let reference_fit = truth_fit.reference_fit();
    let fitted = reference_fit.model();
    let alignment = align_topic_probability_rows(
        truth_topic_terms,
        &fitted.topic_term_probabilities,
    )
    .expect("truth-K topic alignment");

    let truth_topic_term_flat: Vec<_> = truth_topic_terms.iter().flatten().copied().collect();
    let aligned_topic_term_probabilities: Vec<_> = alignment
        .truth_to_fitted()
        .iter()
        .flat_map(|fitted_index| fitted.topic_term_probabilities[*fitted_index].iter().copied())
        .collect();
    let topic_term_rmse = root_mean_square_error(
        &truth_topic_term_flat,
        &aligned_topic_term_probabilities,
    )
    .expect("topic-term RMSE");

    let truth_state_by_document: BTreeMap<_, _> = manifest
        .topic_truth()
        .document_states()
        .iter()
        .map(|state| (state.document_id(), state.topic_mixture()))
        .collect();
    assert_eq!(
        truth_state_by_document.len(),
        manifest.topic_truth().document_states().len(),
        "simulator truth document identities must be unique"
    );
    let fit_document_ids = reference_fit.input().document_ids();
    assert_eq!(
        fit_document_ids.len(),
        fitted.document_topic_proportions.len(),
        "fit-retained document identities must bind every fitted topic-state row"
    );
    let aligned_document_states =
        realign_topic_probability_rows(&alignment, &fitted.document_topic_proportions)
            .expect("aligned fitted document states");
    let mut truth_document_state_flat = Vec::new();
    let mut recovered_document_state_flat = Vec::new();
    for (document_id, recovered_state) in fit_document_ids.iter().zip(&aligned_document_states) {
        let truth_state = *truth_state_by_document
            .get(document_id)
            .expect("fit-retained training document must have simulator truth");
        assert_eq!(
            truth_state.len(),
            recovered_state.len(),
            "truth and aligned fitted state widths must match"
        );
        truth_document_state_flat.extend_from_slice(truth_state);
        recovered_document_state_flat.extend_from_slice(recovered_state);
    }
    let document_state_rmse = root_mean_square_error(
        &truth_document_state_flat,
        &recovered_document_state_flat,
    )
    .expect("fit-bound document-state RMSE");
    let mean_absolute_document_state_residual = mean_absolute_parameter_bias(
        &truth_document_state_flat,
        std::slice::from_ref(&recovered_document_state_flat),
    )
    .expect("fit-bound non-cancelling document-state residual");

    let intercept_feature = fitted
        .prevalence_features
        .iter()
        .position(|feature| *feature == PrevalenceFeature::Intercept)
        .expect("reference prevalence basis retains an intercept");
    let event_time_feature = fitted
        .prevalence_features
        .iter()
        .position(|feature| *feature == PrevalenceFeature::EventTime)
        .expect("reference prevalence basis retains EventTime");
    let aligned_fitted_intercepts = realign_additive_log_ratio(
        &alignment,
        &fitted.prevalence_coefficients[intercept_feature],
    )
    .expect("aligned prevalence intercepts");
    let aligned_fitted_time_slopes = realign_additive_log_ratio(
        &alignment,
        &fitted.prevalence_coefficients[event_time_feature],
    )
    .expect("aligned prevalence EventTime slopes");

    let (truth_time_origin, truth_time_center_seconds, truth_time_scale_seconds) = manifest
        .prevalence_time_basis()
        .expect("simulator-owned prevalence EventTime basis");
    let truth_time_basis = LinearTimeBasis::new(
        truth_time_center_seconds,
        truth_time_scale_seconds,
    )
    .expect("finite simulator prevalence EventTime basis");
    let fitted_design_basis = truth_fit.prevalence_design_basis();
    let fitted_origin_seconds = seconds_from_origin(
        truth_time_origin,
        *fitted_design_basis.event_time_origin(),
    );
    let fitted_time_basis = LinearTimeBasis::new(
        fitted_origin_seconds + fitted_design_basis.event_time_location_seconds(),
        fitted_design_basis.event_time_scale_seconds(),
    )
    .expect("finite frozen training EventTime basis");
    let fitted_in_truth_time_basis = reexpress_linear_prevalence_time_basis(
        &aligned_fitted_intercepts,
        &aligned_fitted_time_slopes,
        fitted_time_basis,
        truth_time_basis,
    )
    .expect("prevalence coefficients in simulator EventTime basis");

    let truth_intercepts = manifest.topic_truth().prevalence_intercepts();
    let truth_time_slopes = manifest.topic_truth().prevalence_time_slopes();
    let prevalence_intercept_rmse = root_mean_square_error(
        truth_intercepts,
        fitted_in_truth_time_basis.intercepts(),
    )
    .expect("prevalence intercept RMSE");
    let prevalence_time_slope_rmse = root_mean_square_error(
        truth_time_slopes,
        fitted_in_truth_time_basis.event_time_slopes(),
    )
    .expect("prevalence EventTime slope RMSE");
    let fitted_intercepts_in_truth_basis = fitted_in_truth_time_basis.intercepts().to_vec();
    let fitted_time_slopes_in_truth_basis = fitted_in_truth_time_basis.event_time_slopes().to_vec();
    let mean_absolute_prevalence_intercept_residual = mean_absolute_parameter_bias(
        truth_intercepts,
        std::slice::from_ref(&fitted_intercepts_in_truth_basis),
    )
    .expect("prevalence intercept absolute residual");
    let mean_absolute_prevalence_time_slope_residual = mean_absolute_parameter_bias(
        truth_time_slopes,
        std::slice::from_ref(&fitted_time_slopes_in_truth_basis),
    )
    .expect("prevalence EventTime slope absolute residual");

    WindowRecoveryMetrics {
        topic_term_rmse,
        aligned_topic_term_probabilities,
        document_state_rmse,
        mean_absolute_document_state_residual,
        prevalence_intercept_rmse,
        prevalence_time_slope_rmse,
        mean_absolute_prevalence_intercept_residual,
        mean_absolute_prevalence_time_slope_residual,
    }
}

fn mean(values: &[f64]) -> f64 {
    let count = u32::try_from(values.len()).expect("CI-scale metric count fits u32");
    values.iter().sum::<f64>() / f64::from(count)
}

fn assert_realistic_structure(manifest: &TruthManifest) {
    assert!(
        manifest
            .documents()
            .iter()
            .all(|document| document.memberships().len() == 3),
        "every modeled document must retain the declared multiple-membership design"
    );

    let mut groups_by_role: BTreeMap<&str, BTreeSet<Uuid>> = BTreeMap::new();
    let mut group_occurrences: BTreeMap<Uuid, usize> = BTreeMap::new();
    for document in manifest.documents() {
        for membership in document.memberships() {
            groups_by_role
                .entry(membership.role_label())
                .or_default()
                .insert(membership.group_id());
            *group_occurrences.entry(membership.group_id()).or_default() += 1;
        }
    }
    assert!(
        groups_by_role.values().any(|groups| groups.len() > 1),
        "at least one contextual role must cross-classify documents into multiple groups"
    );
    assert!(
        group_occurrences.values().any(|count| *count > 1),
        "at least one higher-level group must contain multiple documents"
    );
    assert!(
        !manifest
            .observed_document_transition_pairs()
            .expect("observed transition projection")
            .is_empty(),
        "recovery must retain relation-connected structure"
    );
}

fn assert_future_availability_rejected(manifest: &TruthManifest, cutoff: &KnowledgeCutoff) {
    let future = manifest
        .documents()
        .iter()
        .find(|document| document.available_time().instant() > cutoff.instant())
        .expect("fixture retains future evidence after the first recovery cutoff");
    let mut snapshot = CorpusSnapshot::new();
    assert_eq!(
        snapshot.insert_if_eligible(
            CorpusDocument::new(future.document_id(), future.available_time()),
            cutoff,
        ),
        Err(CorpusSplitError::UnavailableAtCutoff)
    );
}

fn assert_generated_rebinding_rejected(
    fixtures: &[WindowFixture],
    memberships: &MembershipNetwork,
) {
    let first = &fixtures[0];
    let later = &fixtures[1];
    let mut substituted_ids = first.evaluation_document_ids.clone();
    substituted_ids[0] = later.evaluation_document_ids[0];
    assert_eq!(
        rolling_origin_prevalence_mean_predictive_log_likelihood(
            &first.partition,
            &first.fits[0],
            &substituted_ids,
            &first.evaluation_document_term,
            &first.evaluation_event_times,
            None,
            memberships,
        ),
        Err(ModelSelectionError::PartitionInputMismatch)
    );
}

#[allow(clippy::too_many_lines)]
fn run_recovery_replication(seed: u64) -> Result<RecoveryReplication, ModelSelectionError> {
    let manifest = generate(simulation_config(seed)).expect("known-truth simulation");
    manifest
        .verify_invariants()
        .expect("truth manifest invariants");
    assert_realistic_structure(&manifest);

    let cutoffs = recovery_cutoffs(&manifest);
    assert_eq!(
        cutoffs.len(),
        6,
        "five rolling-origin windows are predeclared"
    );
    assert_future_availability_rejected(&manifest, &cutoffs[0]);

    let memberships = membership_network(&manifest);
    let relations = relation_graph(&manifest);
    let leakage = leakage_links(&manifest);
    let counts_by_document = topic_counts_by_document(&manifest);
    let event_times = event_time_by_document(&manifest);
    let vocabulary_size = usize::try_from(manifest.topic_truth().vocabulary_size())
        .expect("vocabulary size fits usize");
    let config = FittedCandidateKConfig::new(vec![2, 3, 4], vec![7, 11, 19], 2_000, 0.001)
        .expect("predeclared candidate-K design");

    let mut fixtures = Vec::new();
    for (window_index, cutoff_pair) in cutoffs.windows(2).enumerate() {
        let training_snapshot = snapshot_at(&manifest, &cutoff_pair[0]);
        let evaluation_snapshot = snapshot_at(&manifest, &cutoff_pair[1]);
        let training_snapshot_ids: BTreeSet<_> = training_snapshot.document_ids().collect();
        let evaluation_document_ids: Vec<_> = evaluation_snapshot
            .document_ids()
            .filter(|document_id| !training_snapshot_ids.contains(document_id))
            .collect();
        assert!(
            !evaluation_document_ids.is_empty(),
            "each declared origin must expose newly available evidence"
        );

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
        let fits = fit_declared_recovery_candidates(&training_input, &config)?;
        let recovery = window_recovery_metrics(&manifest, &fits);

        let evaluation_document_term =
            count_matrix(&evaluation_document_ids, vocabulary_size, &counts_by_document);
        let evaluation_event_times = evaluation_document_ids
            .iter()
            .map(|document_id| event_times[document_id])
            .collect();
        fixtures.push(WindowFixture {
            partition,
            fits,
            evaluation_document_ids,
            evaluation_document_term,
            evaluation_event_times,
            topic_term_rmse: recovery.topic_term_rmse,
            aligned_topic_term_probabilities: recovery.aligned_topic_term_probabilities,
            document_state_rmse: recovery.document_state_rmse,
            mean_absolute_document_state_residual: recovery.mean_absolute_document_state_residual,
            prevalence_intercept_rmse: recovery.prevalence_intercept_rmse,
            prevalence_time_slope_rmse: recovery.prevalence_time_slope_rmse,
            mean_absolute_prevalence_intercept_residual: recovery
                .mean_absolute_prevalence_intercept_residual,
            mean_absolute_prevalence_time_slope_residual: recovery
                .mean_absolute_prevalence_time_slope_residual,
        });
    }

    assert_generated_rebinding_rejected(&fixtures, &memberships);
    let evaluations: Result<Vec<_>, _> = fixtures
        .iter()
        .map(|fixture| {
            RollingOriginRecoveryEvaluation::new(
                &fixture.partition,
                &fixture.fits,
                &fixture.evaluation_document_ids,
                &fixture.evaluation_document_term,
                &fixture.evaluation_event_times,
                None,
                &memberships,
            )
        })
        .collect();
    let selected_k = select_declared_rolling_origin_recovery_candidate_k_for_cutoffs(
        &config,
        &cutoffs,
        &evaluations?,
    )?;
    let topic_term_rmse: Vec<_> = fixtures.iter().map(|fixture| fixture.topic_term_rmse).collect();
    let document_state_rmse: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.document_state_rmse)
        .collect();
    let document_state_residual: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.mean_absolute_document_state_residual)
        .collect();
    let prevalence_intercept_rmse: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.prevalence_intercept_rmse)
        .collect();
    let prevalence_time_slope_rmse: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.prevalence_time_slope_rmse)
        .collect();
    let prevalence_intercept_residual: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.mean_absolute_prevalence_intercept_residual)
        .collect();
    let prevalence_time_slope_residual: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.mean_absolute_prevalence_time_slope_residual)
        .collect();
    let truth_flat: Vec<_> = manifest
        .topic_truth()
        .topic_term_probabilities()
        .iter()
        .flatten()
        .copied()
        .collect();
    let recovered_topic_rows: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.aligned_topic_term_probabilities.clone())
        .collect();
    let mean_absolute_topic_term_bias =
        mean_absolute_parameter_bias(&truth_flat, &recovered_topic_rows)
            .expect("aligned parameter-wise topic-term bias");
    Ok(RecoveryReplication {
        selected_k,
        mean_topic_term_rmse: mean(&topic_term_rmse),
        mean_absolute_topic_term_bias,
        mean_document_state_rmse: mean(&document_state_rmse),
        mean_absolute_document_state_residual: mean(&document_state_residual),
        mean_prevalence_intercept_rmse: mean(&prevalence_intercept_rmse),
        mean_prevalence_time_slope_rmse: mean(&prevalence_time_slope_rmse),
        mean_absolute_prevalence_intercept_residual: mean(&prevalence_intercept_residual),
        mean_absolute_prevalence_time_slope_residual: mean(&prevalence_time_slope_residual),
    })
}

#[test]
fn repeated_known_truth_recovery_reports_unconditional_failure_and_monte_carlo_uncertainty() {
    let outcomes: Vec<_> = REPLICATION_SEEDS
        .iter()
        .copied()
        .map(run_recovery_replication)
        .collect();
    let mut selected_results = Vec::with_capacity(outcomes.len());
    let mut topic_term_rmse = Vec::new();
    let mut topic_term_mean_absolute_bias = Vec::new();
    let mut document_state_rmse = Vec::new();
    let mut document_state_mean_absolute_residual = Vec::new();
    let mut prevalence_intercept_rmse = Vec::new();
    let mut prevalence_time_slope_rmse = Vec::new();
    let mut prevalence_intercept_mean_absolute_residual = Vec::new();
    let mut prevalence_time_slope_mean_absolute_residual = Vec::new();
    for outcome in outcomes {
        match outcome {
            Ok(replication) => {
                selected_results.push(Ok(replication.selected_k));
                topic_term_rmse.push(replication.mean_topic_term_rmse);
                topic_term_mean_absolute_bias.push(replication.mean_absolute_topic_term_bias);
                document_state_rmse.push(replication.mean_document_state_rmse);
                document_state_mean_absolute_residual
                    .push(replication.mean_absolute_document_state_residual);
                prevalence_intercept_rmse.push(replication.mean_prevalence_intercept_rmse);
                prevalence_time_slope_rmse.push(replication.mean_prevalence_time_slope_rmse);
                prevalence_intercept_mean_absolute_residual
                    .push(replication.mean_absolute_prevalence_intercept_residual);
                prevalence_time_slope_mean_absolute_residual
                    .push(replication.mean_absolute_prevalence_time_slope_residual);
            }
            Err(error) => selected_results.push(Err(error)),
        }
    }

    let truth_k = TopicDgpConfig::ci_default().true_topic_count();
    let summary = selected_k_recovery_summary_from_results(selected_results, truth_k)
        .expect("structurally valid repeated recovery experiment");

    assert_eq!(summary.truth_k(), truth_k);
    assert_eq!(summary.replication_count(), REPLICATION_SEEDS.len());
    assert_eq!(
        summary.success_count() + summary.failure_count(),
        summary.replication_count()
    );
    assert!(summary.failure_rate().is_finite());
    assert!(
        summary
            .failure_rate_monte_carlo_standard_error()
            .is_finite()
    );
    assert!(
        summary.success_count() >= 2,
        "CI-scale scientific recovery must retain enough successful replications to estimate Monte Carlo uncertainty"
    );
    for successful_metric_count in [
        topic_term_rmse.len(),
        topic_term_mean_absolute_bias.len(),
        document_state_rmse.len(),
        document_state_mean_absolute_residual.len(),
        prevalence_intercept_rmse.len(),
        prevalence_time_slope_rmse.len(),
        prevalence_intercept_mean_absolute_residual.len(),
        prevalence_time_slope_mean_absolute_residual.len(),
    ] {
        assert_eq!(successful_metric_count, summary.success_count());
    }
    for metric_values in [
        &topic_term_rmse,
        &topic_term_mean_absolute_bias,
        &document_state_rmse,
        &document_state_mean_absolute_residual,
        &prevalence_intercept_rmse,
        &prevalence_time_slope_rmse,
        &prevalence_intercept_mean_absolute_residual,
        &prevalence_time_slope_mean_absolute_residual,
    ] {
        assert!(metric_values.iter().all(|value| value.is_finite()));
    }

    let topic_rmse_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &topic_term_rmse,
        0.025,
        0.975,
    )
    .expect("topic-term RMSE recovery summary");
    let topic_bias_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &topic_term_mean_absolute_bias,
        0.025,
        0.975,
    )
    .expect("topic-term bias recovery summary");
    let document_state_rmse_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &document_state_rmse,
        0.025,
        0.975,
    )
    .expect("document-state RMSE recovery summary");
    let document_state_residual_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &document_state_mean_absolute_residual,
        0.025,
        0.975,
    )
    .expect("document-state absolute-residual recovery summary");
    let prevalence_intercept_rmse_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &prevalence_intercept_rmse,
        0.025,
        0.975,
    )
    .expect("prevalence-intercept RMSE recovery summary");
    let prevalence_time_slope_rmse_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &prevalence_time_slope_rmse,
        0.025,
        0.975,
    )
    .expect("prevalence-time-slope RMSE recovery summary");
    let prevalence_intercept_residual_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &prevalence_intercept_mean_absolute_residual,
        0.025,
        0.975,
    )
    .expect("prevalence-intercept absolute-residual recovery summary");
    let prevalence_time_slope_residual_recovery = summarize_recovery_metric_replications(
        summary.replication_count(),
        &prevalence_time_slope_mean_absolute_residual,
        0.025,
        0.975,
    )
    .expect("prevalence-time-slope absolute-residual recovery summary");

    for metric in [
        topic_rmse_recovery,
        topic_bias_recovery,
        document_state_rmse_recovery,
        document_state_residual_recovery,
        prevalence_intercept_rmse_recovery,
        prevalence_time_slope_rmse_recovery,
        prevalence_intercept_residual_recovery,
        prevalence_time_slope_residual_recovery,
    ] {
        assert_eq!(
            metric.attempted_replication_count(),
            summary.replication_count()
        );
        assert_eq!(metric.successful_replication_count(), summary.success_count());
        assert_eq!(metric.failure_count(), summary.failure_count());
        assert!((metric.failure_rate() - summary.failure_rate()).abs() < 1.0e-12);
        assert!(
            (metric.failure_rate_standard_error()
                - summary.failure_rate_monte_carlo_standard_error())
            .abs()
                < 1.0e-12
        );
    }

    let monte_carlo_summaries = [
        topic_rmse_recovery
            .successful_metric_summary()
            .expect("successful topic-term RMSE summary"),
        topic_bias_recovery
            .successful_metric_summary()
            .expect("successful topic-term bias summary"),
        document_state_rmse_recovery
            .successful_metric_summary()
            .expect("successful document-state RMSE summary"),
        document_state_residual_recovery
            .successful_metric_summary()
            .expect("successful document-state absolute-residual summary"),
        prevalence_intercept_rmse_recovery
            .successful_metric_summary()
            .expect("successful prevalence-intercept RMSE summary"),
        prevalence_time_slope_rmse_recovery
            .successful_metric_summary()
            .expect("successful prevalence-time-slope RMSE summary"),
        prevalence_intercept_residual_recovery
            .successful_metric_summary()
            .expect("successful prevalence-intercept absolute-residual summary"),
        prevalence_time_slope_residual_recovery
            .successful_metric_summary()
            .expect("successful prevalence-time-slope absolute-residual summary"),
    ];
    for monte_carlo in monte_carlo_summaries {
        assert_eq!(monte_carlo.replication_count, summary.success_count());
        assert!(monte_carlo.mean.is_finite());
        assert!(monte_carlo.standard_deviation.is_finite());
        assert!(monte_carlo.standard_error.is_finite());
        assert!(monte_carlo.percentile_lower.is_finite());
        assert!(monte_carlo.percentile_upper.is_finite());
    }

    assert!(summary.bias().is_some());
    assert!(summary.root_mean_square_error().is_some());
    assert!(summary.bias_monte_carlo_standard_error().is_some());
    assert!(summary.rmse_monte_carlo_standard_error().is_some());
}
