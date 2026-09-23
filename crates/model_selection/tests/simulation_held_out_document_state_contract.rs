//! Held-out document-state recovery must remain frozen-fit and out-of-temporal-sample.
//!
//! This contract is deliberately narrower than the full #680 repeated recovery
//! experiment. It proves that the #719 local-state owner can consume realistic
//! simulator rows after a temporal cutoff, preserve the training fit, align the
//! recovered simplex into the simulator truth topic basis, and produce finite
//! recovery diagnostics over the exact evaluation document identities.

use std::collections::{BTreeMap, BTreeSet};

use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
};
use tepp_simulation::{SimulatedDocument, SimulationConfig, TruthManifest, generate};
use topic_measurement::{
    ReferenceTopicModelConfig, ReferenceTopicTrainingFit, ReferenceTopicTrainingInput, SparseMatrix,
};
use uuid::Uuid;
use validation_core::{
    align_topic_probability_rows, mean_absolute_parameter_bias, realign_topic_probability_rows,
    root_mean_square_error,
};

const SEED: u64 = 719_001;

fn simulation_config() -> SimulationConfig {
    SimulationConfig::new(
        SEED, 9, 2, 3, 12, 6, 0, 0, 500, 3_000, 3_000, 3_000,
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

#[test]
fn simulated_future_rows_recover_local_states_without_refitting_the_training_model() {
    let manifest = generate(simulation_config()).expect("known-truth simulation");
    manifest
        .verify_invariants()
        .expect("truth manifest invariants");

    let training_cutoff = cutoff_after_event(&manifest, manifest.events()[3].event_id());
    let evaluation_cutoff = cutoff_after_event(&manifest, manifest.events()[4].event_id());
    let training_document_ids: Vec<_> = manifest
        .documents()
        .iter()
        .filter(|document| document.available_time().instant() <= training_cutoff.instant())
        .map(SimulatedDocument::document_id)
        .collect();
    let evaluation_document_ids: Vec<_> = manifest
        .documents()
        .iter()
        .filter(|document| {
            document.available_time().instant() > training_cutoff.instant()
                && document.available_time().instant() <= evaluation_cutoff.instant()
        })
        .map(SimulatedDocument::document_id)
        .collect();
    assert!(!training_document_ids.is_empty());
    assert!(!evaluation_document_ids.is_empty());
    let training_identity_set: BTreeSet<_> = training_document_ids.iter().copied().collect();
    assert!(
        evaluation_document_ids
            .iter()
            .all(|document_id| !training_identity_set.contains(document_id)),
        "held-out identities must not enter the training state"
    );

    let mut training_snapshot = CorpusSnapshot::new();
    for document in manifest
        .documents()
        .iter()
        .filter(|document| training_identity_set.contains(&document.document_id()))
    {
        training_snapshot
            .insert_if_eligible(
                CorpusDocument::new(document.document_id(), document.available_time()),
                &training_cutoff,
            )
            .expect("training document is available at the frozen cutoff");
    }

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
    let config = ReferenceTopicModelConfig::new(
        manifest.topic_truth().true_k(),
        vec![7, 11, 19],
        2_000,
        0.001,
    )
    .and_then(|config| config.with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2))
    .expect("truth-K deterministic fit config");
    let fit = ReferenceTopicTrainingFit::fit(&training_input, &config).expect("truth-K training fit");
    let frozen_model = fit.reference_fit().model().clone();

    let evaluation_document_term = count_matrix(
        &evaluation_document_ids,
        vocabulary_size,
        &counts_by_document,
    );
    let evaluation_event_times: Vec<_> = evaluation_document_ids
        .iter()
        .map(|document_id| event_times[document_id])
        .collect();
    let recovered = fit
        .infer_held_out_document_topic_proportions(
            &evaluation_document_ids,
            &evaluation_document_term,
            &evaluation_event_times,
            None,
            &memberships,
        )
        .expect("frozen-fit held-out local-state recovery");
    assert_eq!(fit.reference_fit().model(), &frozen_model);
    assert_eq!(recovered.len(), evaluation_document_ids.len());

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
        let truth_state = truth_state_by_document
            .get(document_id)
            .expect("evaluation identity retains simulator truth");
        assert_eq!(truth_state.len(), recovered_state.len());
        truth_flat.extend_from_slice(truth_state);
        recovered_flat.extend_from_slice(recovered_state);
    }

    let rmse = root_mean_square_error(&truth_flat, &recovered_flat)
        .expect("held-out document-state RMSE");
    let mean_absolute_residual = mean_absolute_parameter_bias(
        &truth_flat,
        std::slice::from_ref(&recovered_flat),
    )
    .expect("held-out non-cancelling state residual");
    assert!(rmse.is_finite() && rmse >= 0.0);
    assert!(mean_absolute_residual.is_finite() && mean_absolute_residual >= 0.0);
    assert!(
        aligned.iter().all(|row| {
            row.iter().all(|value| value.is_finite() && *value > 0.0)
                && (row.iter().sum::<f64>() - 1.0).abs() < 1.0e-12
        }),
        "recovered evaluation states must remain valid simplices"
    );
}
