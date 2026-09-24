//! Indexed execution boundary for prospective coverage-calibration replications.
//!
//! This module composes existing temporal, Membership, relation, topic-measurement,
//! model-selection, and validation owners. It does not duplicate their numerical
//! arithmetic or mint simulation truth as production evidence authority.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use corpus_split::{
    CorpusDocument, CorpusSnapshot, LeakageLink, LeakageLinkKind,
    admit_expanding_rolling_origin_partition,
};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{FittedCandidateKConfig, ModelSelectionError, fit_declared_recovery_candidates};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval, TemporalPrecision,
};
use tepp_simulation::{
    CoverageCalibrationSimulationDesign, DocumentMethodEffect, SimulatedDocument, TruthManifest,
    generate,
};
use topic_measurement::{
    FitBoundDocumentMarginalCovariance, FittedDocumentCoordinateSummary, ReferenceTopicTrainingFit,
    ReferenceTopicTrainingInput, SparseMatrix, additive_log_ratio,
};
use uuid::Uuid;
use validation_core::{
    CoverageCalibrationDesign, CoverageCalibrationReplicationOutcome, align_topic_probability_rows,
    interval_coverage, normal_marginal_interval_bounds, realign_additive_log_ratio,
    realign_additive_log_ratio_covariance,
};

const RECOVERY_FIT_SEEDS: [u64; 3] = [7, 11, 19];
const RECOVERY_MAXIMUM_ITERATIONS: usize = 2_000;
const RECOVERY_TOLERANCE: f64 = 0.001;

/// Fail-closed error from one declared coverage-calibration replication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageCalibrationExecutionError {
    /// The declared replication ordinal or simulation scenario could not be reconstructed.
    InvalidSimulationScenario,
    /// The generated truth manifest violated its owner invariants.
    InvalidTruthManifest,
    /// A generated event had no availability evidence from which to derive a cutoff.
    MissingEventAvailability,
    /// A generated document lacked the observed EventTime required by the declared study.
    MissingObservedEventTime,
    /// Simulation Membership could not be projected through the canonical Membership owner.
    InvalidMembershipProjection,
    /// Observed simulation relations could not be projected through the relation owner.
    InvalidRelationProjection,
    /// The generated corpus could not be admitted at a declared knowledge cutoff.
    InvalidCorpusSnapshot,
    /// The canonical rolling-origin partition rejected the declared window geometry.
    InvalidRollingOriginPartition,
    /// Generated term counts could not be represented by the canonical sparse matrix.
    InvalidCountMatrix,
    /// The canonical topic-training input rejected the composed scientific state.
    InvalidTrainingInput,
    /// Model-selection rejected a structural or configuration condition.
    ModelSelection(ModelSelectionError),
    /// Fitted topic identity could not be aligned to known-truth topic identity.
    InvalidTopicAlignment,
    /// Fit-bound coordinate or covariance geometry could not support interval construction.
    InvalidPosteriorGeometry,
    /// A fitted document could not be joined to its simulator truth state.
    MissingTruthState,
    /// A window-level coverage result was structurally invalid.
    InvalidCoverage,
    /// A bounded platform index could not be represented in the scientific identity geometry.
    ArithmeticOverflow,
}

impl fmt::Display for CoverageCalibrationExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidSimulationScenario => "invalid coverage calibration simulation scenario",
            Self::InvalidTruthManifest => "invalid coverage calibration truth manifest",
            Self::MissingEventAvailability => "coverage calibration event has no availability evidence",
            Self::MissingObservedEventTime => "coverage calibration document has no observed event time",
            Self::InvalidMembershipProjection => "invalid coverage calibration membership projection",
            Self::InvalidRelationProjection => "invalid coverage calibration relation projection",
            Self::InvalidCorpusSnapshot => "invalid coverage calibration corpus snapshot",
            Self::InvalidRollingOriginPartition => "invalid coverage calibration rolling-origin partition",
            Self::InvalidCountMatrix => "invalid coverage calibration count matrix",
            Self::InvalidTrainingInput => "invalid coverage calibration topic-training input",
            Self::ModelSelection(error) => return error.fmt(formatter),
            Self::InvalidTopicAlignment => "invalid coverage calibration topic alignment",
            Self::InvalidPosteriorGeometry => "invalid coverage calibration posterior geometry",
            Self::MissingTruthState => "coverage calibration fitted document has no truth state",
            Self::InvalidCoverage => "invalid coverage calibration interval coverage",
            Self::ArithmeticOverflow => "coverage calibration identity arithmetic overflow",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CoverageCalibrationExecutionError {}

/// Execute one prospectively declared rolling-origin coverage-calibration replication.
///
/// The caller supplies only the versioned simulation design and zero-based
/// replication ordinal. The design owner mints the exact seed and DGP
/// configuration. This boundary then composes canonical cutoff admission,
/// multiple Membership, observed relation projection, truth-`K` recovery fitting,
/// topic alignment, fit-bound covariance, marginal interval construction, and
/// window-level coverage. It has no hidden mutable state, so distinct ordinals
/// may be sharded or resumed independently.
///
/// Only [`ModelSelectionError::RecoveryCandidateFitFailed`] is admitted as a
/// numerical failure outcome. Structural split, identity, provenance, training,
/// alignment, covariance, and coverage invalidity abort the replication instead
/// of entering the scientific failure denominator.
///
/// # Errors
///
/// Returns a typed structural error when the declared scenario or any canonical
/// owner rejects the composed experiment. Owner-classified numerical fitting
/// inability is returned as [`CoverageCalibrationReplicationOutcome::numerical_failure`]
/// inside `Ok` so its declared replication identity remains in the denominator.
pub fn execute_coverage_calibration_replication(
    design: CoverageCalibrationSimulationDesign,
    replication_index: usize,
) -> Result<CoverageCalibrationReplicationOutcome, CoverageCalibrationExecutionError> {
    let config = design
        .config_for_replication(replication_index)
        .map_err(|_| CoverageCalibrationExecutionError::InvalidSimulationScenario)?;
    let manifest = generate(config)
        .map_err(|_| CoverageCalibrationExecutionError::InvalidSimulationScenario)?;
    manifest
        .verify_invariants()
        .map_err(|_| CoverageCalibrationExecutionError::InvalidTruthManifest)?;

    match replication_window_coverages(&manifest)? {
        Some(window_coverages) => Ok(CoverageCalibrationReplicationOutcome::successful(
            replication_index,
            window_coverages,
        )),
        None => Ok(CoverageCalibrationReplicationOutcome::numerical_failure(
            replication_index,
        )),
    }
}

fn cutoff_after_event(
    manifest: &TruthManifest,
    event_id: Uuid,
) -> Result<KnowledgeCutoff, CoverageCalibrationExecutionError> {
    let latest = manifest
        .documents()
        .iter()
        .filter(|document| document.event_id() == event_id)
        .map(SimulatedDocument::available_time)
        .max_by_key(|available_time| available_time.instant())
        .ok_or(CoverageCalibrationExecutionError::MissingEventAvailability)?;
    KnowledgeCutoff::parse_rfc3339(&latest.to_rfc3339())
        .map_err(|_| CoverageCalibrationExecutionError::InvalidSimulationScenario)
}

fn recovery_cutoffs(
    manifest: &TruthManifest,
    validation_design: &CoverageCalibrationDesign,
) -> Result<Vec<KnowledgeCutoff>, CoverageCalibrationExecutionError> {
    let events = manifest.events();
    let source_events = events
        .get(validation_design.first_training_event_index()..)
        .ok_or(CoverageCalibrationExecutionError::InvalidSimulationScenario)?;
    let cutoffs: Result<Vec<_>, _> = source_events
        .iter()
        .map(|event| cutoff_after_event(manifest, event.event_id()))
        .collect();
    let cutoffs = cutoffs?;
    let expected_cutoff_count = validation_design
        .declared_rolling_origin_window_count()
        .checked_add(1)
        .ok_or(CoverageCalibrationExecutionError::ArithmeticOverflow)?;
    if cutoffs.len() != expected_cutoff_count {
        return Err(CoverageCalibrationExecutionError::InvalidSimulationScenario);
    }
    Ok(cutoffs)
}

fn snapshot_at(
    manifest: &TruthManifest,
    cutoff: &KnowledgeCutoff,
) -> Result<CorpusSnapshot, CoverageCalibrationExecutionError> {
    let mut snapshot = CorpusSnapshot::new();
    for document in manifest.documents() {
        if document.available_time().instant() <= cutoff.instant() {
            snapshot
                .insert_if_eligible(
                    CorpusDocument::new(document.document_id(), document.available_time()),
                    cutoff,
                )
                .map_err(|_| CoverageCalibrationExecutionError::InvalidCorpusSnapshot)?;
        }
    }
    Ok(snapshot)
}

fn event_time_by_document(
    manifest: &TruthManifest,
) -> Result<BTreeMap<Uuid, EventTime>, CoverageCalibrationExecutionError> {
    manifest
        .documents()
        .iter()
        .map(|document| {
            document
                .observed_event_time()
                .map(|event_time| (document.document_id(), event_time))
                .ok_or(CoverageCalibrationExecutionError::MissingObservedEventTime)
        })
        .collect()
}

fn membership_network(
    manifest: &TruthManifest,
    event_times: &BTreeMap<Uuid, EventTime>,
) -> Result<MembershipNetwork, CoverageCalibrationExecutionError> {
    let mut network = MembershipNetwork::new();
    for document in manifest.documents() {
        let event_time = event_times
            .get(&document.document_id())
            .copied()
            .ok_or(CoverageCalibrationExecutionError::MissingObservedEventTime)?;
        for membership in document.memberships() {
            let role = MembershipRole::from_wire_name(membership.role_label())
                .map_err(|_| CoverageCalibrationExecutionError::InvalidMembershipProjection)?;
            let weight = MembershipWeight::new(f64::from(membership.weight_bps()) / 10_000.0)
                .map_err(|_| CoverageCalibrationExecutionError::InvalidMembershipProjection)?;
            let assignment = MembershipAssignment::new(
                MemberId::from_uuid(document.document_id()),
                GroupId::from_uuid(membership.group_id()),
                role,
                weight,
                event_time,
                event_time,
            )
            .map_err(|_| CoverageCalibrationExecutionError::InvalidMembershipProjection)?;
            network
                .insert(assignment)
                .map_err(|_| CoverageCalibrationExecutionError::InvalidMembershipProjection)?;
        }
    }
    Ok(network)
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

fn point_interval(
    time: EventTime,
) -> Result<TemporalInterval<EventTime>, CoverageCalibrationExecutionError> {
    TemporalInterval::bounded(
        TemporalBoundary::Included(time),
        TemporalBoundary::Included(time),
        TemporalPrecision::Second,
    )
    .map_err(|_| CoverageCalibrationExecutionError::InvalidRelationProjection)
}

fn relation_graph(
    manifest: &TruthManifest,
    event_times: &BTreeMap<Uuid, EventTime>,
) -> Result<RelationGraph, CoverageCalibrationExecutionError> {
    let transition_pairs = manifest
        .observed_document_transition_pairs()
        .map_err(|_| CoverageCalibrationExecutionError::InvalidRelationProjection)?;
    let mut relations = RelationGraph::new();
    for (source, target) in transition_pairs {
        let source_time = event_times
            .get(&source)
            .copied()
            .ok_or(CoverageCalibrationExecutionError::MissingObservedEventTime)?;
        let target_time = event_times
            .get(&target)
            .copied()
            .ok_or(CoverageCalibrationExecutionError::MissingObservedEventTime)?;
        let edge = RelationEdge::new(
            RelationKind::TransitionsTo,
            RelationEndpointId::from_uuid(source),
            RelationEndpointId::from_uuid(target),
            RelationEvidenceStatus::Observed,
            point_interval(source_time)?,
            point_interval(target_time)?,
        )
        .map_err(|_| CoverageCalibrationExecutionError::InvalidRelationProjection)?;
        relations
            .insert(edge)
            .map_err(|_| CoverageCalibrationExecutionError::InvalidRelationProjection)?;
    }
    Ok(relations)
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
) -> Result<SparseMatrix, CoverageCalibrationExecutionError> {
    let mut offsets = Vec::with_capacity(document_ids.len() + 1);
    let mut indices = Vec::new();
    let mut values = Vec::new();
    offsets.push(0);
    for document_id in document_ids {
        let counts = counts_by_document
            .get(document_id)
            .ok_or(CoverageCalibrationExecutionError::MissingTruthState)?;
        for (term, count) in counts.iter().copied().enumerate() {
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
    .map_err(|_| CoverageCalibrationExecutionError::InvalidCountMatrix)
}

fn truth_k_window_coverage(
    manifest: &TruthManifest,
    fit: &ReferenceTopicTrainingFit,
) -> Result<f64, CoverageCalibrationExecutionError> {
    let reference_fit = fit.reference_fit();
    let fitted_model = reference_fit.model();
    let truth_topics = manifest.topic_truth().topic_term_probabilities();
    let alignment = align_topic_probability_rows(truth_topics, &fitted_model.topic_term_probabilities)
        .map_err(|_| CoverageCalibrationExecutionError::InvalidTopicAlignment)?;
    let coordinate_summary = FittedDocumentCoordinateSummary::from_bound_fit(reference_fit)
        .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
    let truth_by_document = truth_topic_state_by_document(manifest);
    let mut topic_ids = Vec::with_capacity(coordinate_summary.topic_count());
    for index in 0..coordinate_summary.topic_count() {
        let offset = u128::try_from(index)
            .map_err(|_| CoverageCalibrationExecutionError::ArithmeticOverflow)?;
        topic_ids.push(Uuid::from_u128(10_000_u128 + offset));
    }
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
    .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
    if marginals.len() != coordinate_summary.rows().len() {
        return Err(CoverageCalibrationExecutionError::InvalidPosteriorGeometry);
    }

    let normal_critical_value =
        CoverageCalibrationDesign::tepp_nominal_95_v1().normal_critical_value();
    let mut truth = Vec::new();
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (row, marginal) in coordinate_summary.rows().iter().zip(&marginals) {
        if marginal.document_id() != row.document_id()
            || marginal.topic_basis_identity() != coordinate_summary.topic_basis_identity()
        {
            return Err(CoverageCalibrationExecutionError::InvalidPosteriorGeometry);
        }
        let fitted_location: Vec<_> = row
            .coordinates()
            .iter()
            .map(|coordinate| coordinate.location())
            .collect();
        let aligned_location = realign_additive_log_ratio(&alignment, &fitted_location)
            .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
        let aligned_covariance = realign_additive_log_ratio_covariance(&alignment, marginal.values())
            .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
        let bounds = normal_marginal_interval_bounds(
            &aligned_location,
            &aligned_covariance,
            normal_critical_value,
        )
        .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
        let truth_state = truth_by_document
            .get(&row.document_id())
            .ok_or(CoverageCalibrationExecutionError::MissingTruthState)?;
        let truth_alr = additive_log_ratio(truth_state)
            .map_err(|_| CoverageCalibrationExecutionError::InvalidPosteriorGeometry)?;
        if truth_alr.len() != aligned_location.len() {
            return Err(CoverageCalibrationExecutionError::InvalidPosteriorGeometry);
        }
        truth.extend_from_slice(&truth_alr);
        lower.extend_from_slice(bounds.lower());
        upper.extend_from_slice(bounds.upper());
    }

    interval_coverage(&truth, &lower, &upper)
        .map_err(|_| CoverageCalibrationExecutionError::InvalidCoverage)
}

#[allow(clippy::too_many_lines)]
fn replication_window_coverages(
    manifest: &TruthManifest,
) -> Result<Option<Vec<f64>>, CoverageCalibrationExecutionError> {
    let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
    let declared_window_count = validation_design.declared_rolling_origin_window_count();
    let cutoffs = recovery_cutoffs(manifest, &validation_design)?;
    let event_times = event_time_by_document(manifest)?;
    let memberships = membership_network(manifest, &event_times)?;
    let relations = relation_graph(manifest, &event_times)?;
    let leakage = leakage_links(manifest);
    let counts_by_document = topic_counts_by_document(manifest);
    let vocabulary_size = usize::try_from(manifest.topic_truth().vocabulary_size())
        .map_err(|_| CoverageCalibrationExecutionError::ArithmeticOverflow)?;
    let truth_k = manifest.topic_truth().true_k();
    let config = FittedCandidateKConfig::new(
        vec![truth_k],
        RECOVERY_FIT_SEEDS.to_vec(),
        RECOVERY_MAXIMUM_ITERATIONS,
        RECOVERY_TOLERANCE,
    )
    .map_err(CoverageCalibrationExecutionError::ModelSelection)?;

    let mut coverage_by_window = Vec::with_capacity(declared_window_count);
    for (window_index, cutoff_pair) in cutoffs.windows(2).enumerate() {
        let training_snapshot = snapshot_at(manifest, &cutoff_pair[0])?;
        let evaluation_snapshot = snapshot_at(manifest, &cutoff_pair[1])?;
        let training_snapshot_ids: BTreeSet<_> = training_snapshot.document_ids().collect();
        let evaluation_document_ids: Vec<_> = evaluation_snapshot
            .document_ids()
            .filter(|document_id| !training_snapshot_ids.contains(document_id))
            .collect();
        if evaluation_document_ids.is_empty() {
            return Err(CoverageCalibrationExecutionError::InvalidRollingOriginPartition);
        }
        let expanding = admit_expanding_rolling_origin_partition(
            &cutoffs,
            window_index,
            &training_snapshot,
            &evaluation_snapshot,
            &evaluation_document_ids,
            &leakage,
        )
        .map_err(|_| CoverageCalibrationExecutionError::InvalidRollingOriginPartition)?;
        let training_document_ids: Vec<_> = expanding
            .partition()
            .training_document_ids()
            .iter()
            .copied()
            .collect();
        let training_document_term = count_matrix(
            &training_document_ids,
            vocabulary_size,
            &counts_by_document,
        )?;
        let training_event_times: Result<Vec<_>, _> = training_document_ids
            .iter()
            .map(|document_id| {
                event_times
                    .get(document_id)
                    .copied()
                    .ok_or(CoverageCalibrationExecutionError::MissingObservedEventTime)
            })
            .collect();
        let training_event_times = training_event_times?;
        let training_input = ReferenceTopicTrainingInput::new(
            &training_snapshot,
            training_document_ids,
            &training_document_term,
            &training_event_times,
            None,
            &memberships,
            &relations,
        )
        .map_err(|_| CoverageCalibrationExecutionError::InvalidTrainingInput)?;
        let fits = match fit_declared_recovery_candidates(&training_input, &config) {
            Ok(fits) => fits,
            Err(ModelSelectionError::RecoveryCandidateFitFailed) => return Ok(None),
            Err(error) => return Err(CoverageCalibrationExecutionError::ModelSelection(error)),
        };
        if fits.len() != 1 {
            return Err(CoverageCalibrationExecutionError::InvalidTrainingInput);
        }
        coverage_by_window.push(truth_k_window_coverage(manifest, &fits[0])?);
    }
    if coverage_by_window.len() != declared_window_count {
        return Err(CoverageCalibrationExecutionError::InvalidRollingOriginPartition);
    }
    Ok(Some(coverage_by_window))
}

#[cfg(test)]
mod tests {
    use super::{
        CoverageCalibrationExecutionError, cutoff_after_event,
        execute_coverage_calibration_replication, recovery_cutoffs, replication_window_coverages,
    };
    use tepp_simulation::{CoverageCalibrationSimulationDesign, generate};
    use validation_core::CoverageCalibrationDesign;

    #[test]
    fn regression_seed_uses_the_validation_owner_rolling_origin_geometry() {
        let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
        let validation_design = CoverageCalibrationDesign::tepp_nominal_95_v1();
        let manifest = generate(
            design
                .regression_config_for_seed(101)
                .expect("declared regression DGP shape"),
        )
        .expect("known-truth simulation");
        manifest.verify_invariants().expect("truth invariants");
        let cutoffs = recovery_cutoffs(&manifest, &validation_design)
            .expect("owner-issued rolling-origin cutoff geometry");
        assert_eq!(
            cutoffs.len(),
            validation_design.declared_rolling_origin_window_count() + 1
        );
        let first_declared_event = manifest.events()[validation_design.first_training_event_index()]
            .event_id();
        let expected_first_cutoff = cutoff_after_event(&manifest, first_declared_event)
            .expect("latest availability cutoff for first declared event");
        assert_eq!(cutoffs[0], expected_first_cutoff);

        let windows = replication_window_coverages(&manifest)
            .expect("structurally valid regression composition")
            .expect("regression fit must succeed for the fixed seed");
        assert_eq!(
            windows.len(),
            validation_design.declared_rolling_origin_window_count()
        );
        assert!(
            windows
                .iter()
                .all(|coverage| coverage.is_finite() && (0.0..=1.0).contains(coverage))
        );
    }

    #[test]
    fn declared_ordinal_is_fail_closed_outside_the_scenario() {
        let design = CoverageCalibrationSimulationDesign::rolling_origin_coverage_v1();
        assert_eq!(
            execute_coverage_calibration_replication(
                design,
                design.attempted_replication_count(),
            ),
            Err(CoverageCalibrationExecutionError::InvalidSimulationScenario)
        );
    }
}
