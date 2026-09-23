//! Scientific-recovery binding for a predeclared candidate-`K` design.

use std::collections::{BTreeMap, BTreeSet};

use corpus_split::{RollingOriginPartition, RollingOriginWindow, rolling_origin_windows};
use membership_core::MembershipNetwork;
use temporal_core::{EventTime, KnowledgeCutoff};
use topic_measurement::{
    ReferenceTopicModelConfig, ReferenceTopicTrainingFit, SparseMatrix,
};
use uuid::Uuid;

use crate::{
    FittedCandidateKConfig, ModelSelectionError, RollingOriginPredictiveEvaluation,
    select_rolling_origin_predictive_candidate_k_across_windows,
};

fn unique_candidate_topic_counts(
    topic_counts: impl IntoIterator<Item = usize>,
    candidate_fit_count: usize,
) -> Result<BTreeSet<usize>, ModelSelectionError> {
    let topic_counts: BTreeSet<_> = topic_counts.into_iter().collect();
    if topic_counts.len() != candidate_fit_count {
        return Err(ModelSelectionError::DuplicateCandidateK);
    }
    Ok(topic_counts)
}

fn validate_shared_training_state(
    candidate_training_fits: &[ReferenceTopicTrainingFit],
) -> Result<(), ModelSelectionError> {
    let Some(first) = candidate_training_fits.first() else {
        return Ok(());
    };
    if candidate_training_fits.iter().skip(1).any(|fit| {
        !first
            .training_input()
            .shares_numerical_training_state(fit.training_input())
    }) {
        return Err(ModelSelectionError::PredictiveCandidateTrainingStateMismatch);
    }
    Ok(())
}

fn validate_recovery_window_sequence(
    expected: &[RollingOriginWindow],
    observed: &[RollingOriginWindow],
) -> Result<(), ModelSelectionError> {
    if expected == observed {
        Ok(())
    } else {
        Err(ModelSelectionError::RecoveryWindowSetMismatch)
    }
}

/// One rolling-origin evaluation whose fitted candidate dimensions and exact
/// numerical configurations are retained for comparison against the predeclared
/// recovery design.
///
/// This is a scientific-acceptance composition value. It does not mint split,
/// source, Membership, relation, or fitted-model authority; those values remain
/// owned by the wrapped partition, training fits, and numerical inputs.
pub struct RollingOriginRecoveryEvaluation<'a> {
    predictive: RollingOriginPredictiveEvaluation<'a>,
    window: RollingOriginWindow,
    candidate_topic_counts: BTreeSet<usize>,
    candidate_configurations: BTreeMap<usize, ReferenceTopicModelConfig>,
    candidate_fit_count: usize,
}

impl<'a> RollingOriginRecoveryEvaluation<'a> {
    /// Group one predictive window while retaining fitted dimensions and the
    /// exact owner-issued configuration attached to each training fit.
    ///
    /// Every candidate in the window must also originate from one exact retained
    /// numerical training state. Candidate K is the experimental factor; term
    /// counts, event times, frozen prevalence coordinates, and admitted transition
    /// pairs cannot vary by K while still entering one recovery comparison.
    /// The owner-issued rolling-origin window is retained so scientific recovery
    /// can prove that no leading or trailing horizon was dropped after a fit
    /// failure.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::DuplicateCandidateK`] when two supplied
    /// training fits represent the same fitted topic dimension, or
    /// [`ModelSelectionError::PredictiveCandidateTrainingStateMismatch`] when
    /// candidate fits were produced from different numerical training states.
    pub fn new(
        partition: &'a RollingOriginPartition,
        candidate_training_fits: &'a [ReferenceTopicTrainingFit],
        evaluation_document_ids: &'a [Uuid],
        evaluation_document_term: &'a SparseMatrix,
        evaluation_event_times: &'a [EventTime],
        evaluation_covariates: Option<&'a SparseMatrix>,
        memberships: &'a MembershipNetwork,
    ) -> Result<Self, ModelSelectionError> {
        validate_shared_training_state(candidate_training_fits)?;
        let candidate_topic_counts = unique_candidate_topic_counts(
            candidate_training_fits.iter().map(|fit| {
                fit.reference_fit()
                    .model()
                    .topic_term_probabilities
                    .len()
            }),
            candidate_training_fits.len(),
        )?;
        let candidate_configurations = candidate_training_fits
            .iter()
            .map(|fit| {
                (
                    fit.reference_fit()
                        .model()
                        .topic_term_probabilities
                        .len(),
                    fit.reference_fit().config().clone(),
                )
            })
            .collect();

        Ok(Self {
            predictive: RollingOriginPredictiveEvaluation::new(
                partition,
                candidate_training_fits,
                evaluation_document_ids,
                evaluation_document_term,
                evaluation_event_times,
                evaluation_covariates,
                memberships,
            ),
            window: *partition.window(),
            candidate_topic_counts,
            candidate_configurations,
            candidate_fit_count: candidate_training_fits.len(),
        })
    }
}

/// Select candidate `K` only when every supplied rolling-origin window covers the
/// predeclared scientific candidate grid and numerical design exactly.
///
/// The declared grid comes from [`FittedCandidateKConfig`], not from successful
/// fits. Therefore a candidate that fails fitting and disappears from every
/// supplied window cannot silently shrink the recovery design. Each surviving fit
/// must also retain the exact candidate-specific [`ReferenceTopicModelConfig`]
/// derived from the same declared recovery design, including deterministic seeds,
/// convergence controls, and hyperparameters. A dimension-compatible fit from a
/// different optimizer configuration is a failed scientific replication, not an
/// interchangeable candidate. [`RollingOriginRecoveryEvaluation::new`] separately
/// requires every candidate within one window to share one exact numerical
/// training state, preventing K from being confounded with different observations.
///
/// This function still knows only the supplied window slice. Scientific acceptance
/// that must prove the *complete declared temporal horizon* should use
/// [`select_declared_rolling_origin_recovery_candidate_k_for_cutoffs`].
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when there are no windows,
/// [`ModelSelectionError::PredictiveCandidateGridMismatch`] when any window's
/// fitted candidate dimensions differ from the declared grid,
/// [`ModelSelectionError::PredictiveCandidateConfigurationMismatch`] when a
/// fitted candidate was produced under a different numerical configuration, or
/// propagates the generic rolling-origin predictive selector's cutoff, identity,
/// candidate, and numerical failures.
pub fn select_declared_rolling_origin_recovery_candidate_k(
    config: &FittedCandidateKConfig,
    evaluations: &[RollingOriginRecoveryEvaluation<'_>],
) -> Result<u32, ModelSelectionError> {
    if evaluations.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }

    #[allow(clippy::cast_possible_truncation)]
    let declared: BTreeSet<usize> = config
        .candidate_topic_counts()
        .iter()
        .map(|candidate_k| *candidate_k as usize)
        .collect();
    for evaluation in evaluations {
        if evaluation.candidate_fit_count != declared.len()
            || evaluation.candidate_topic_counts != declared
        {
            return Err(ModelSelectionError::PredictiveCandidateGridMismatch);
        }
        for &candidate_k in config.candidate_topic_counts() {
            #[allow(clippy::cast_possible_truncation)]
            let topic_count = candidate_k as usize;
            let expected = config.reference_topic_model_config(candidate_k)?;
            if evaluation.candidate_configurations.get(&topic_count) != Some(&expected) {
                return Err(ModelSelectionError::PredictiveCandidateConfigurationMismatch);
            }
        }
    }

    let predictive: Vec<_> = evaluations
        .iter()
        .map(|evaluation| evaluation.predictive)
        .collect();
    select_rolling_origin_predictive_candidate_k_across_windows(&predictive)
}

/// Select candidate `K` only when scientific recovery covers the complete
/// owner-derived rolling-origin horizon.
///
/// `ordered_cutoffs` is converted through the canonical
/// [`rolling_origin_windows`] owner. The resulting sequence must match the
/// recovery evaluations exactly in count and order before any predictive score is
/// considered. A caller therefore cannot salvage a successful prefix or suffix
/// after a numerical fit failure in a leading or trailing window. The older
/// slice-only selector remains available for lower-level contract use, while this
/// path is the one required for #680-style complete-horizon recovery evidence.
///
/// This check proves recovery-design completeness only. It does not authenticate
/// source evidence, EventTime, Membership provenance, or relation activation.
///
/// # Errors
///
/// Returns [`ModelSelectionError::RecoveryWindowSetMismatch`] when the cutoff
/// sequence cannot form the declared rolling-origin design or when supplied
/// evaluations omit, reorder, or substitute any owner-derived window. Otherwise
/// propagates the candidate-grid, configuration, identity, structural-input, and
/// numerical errors from [`select_declared_rolling_origin_recovery_candidate_k`].
pub fn select_declared_rolling_origin_recovery_candidate_k_for_cutoffs(
    config: &FittedCandidateKConfig,
    ordered_cutoffs: &[KnowledgeCutoff],
    evaluations: &[RollingOriginRecoveryEvaluation<'_>],
) -> Result<u32, ModelSelectionError> {
    let expected = rolling_origin_windows(ordered_cutoffs)
        .map_err(|_| ModelSelectionError::RecoveryWindowSetMismatch)?;
    let observed: Vec<_> = evaluations.iter().map(|evaluation| evaluation.window).collect();
    validate_recovery_window_sequence(&expected, &observed)?;
    select_declared_rolling_origin_recovery_candidate_k(config, evaluations)
}

#[cfg(test)]
mod tests {
    use super::{
        unique_candidate_topic_counts, validate_recovery_window_sequence,
        validate_shared_training_state,
    };
    use crate::ModelSelectionError;
    use corpus_split::rolling_origin_windows;
    use temporal_core::KnowledgeCutoff;

    fn cutoff(day: u8) -> KnowledgeCutoff {
        KnowledgeCutoff::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z"))
            .expect("cutoff")
    }

    #[test]
    fn candidate_dimension_set_is_unique() {
        assert_eq!(
            unique_candidate_topic_counts([2, 3], 2).expect("unique dimensions"),
            [2_usize, 3].into_iter().collect()
        );
        assert_eq!(
            unique_candidate_topic_counts([2, 2], 2),
            Err(ModelSelectionError::DuplicateCandidateK)
        );
        assert_eq!(validate_shared_training_state(&[]), Ok(()));
    }

    #[test]
    fn scientific_recovery_window_sequence_rejects_prefix_and_suffix_shrinkage() {
        let windows = rolling_origin_windows(&[cutoff(10), cutoff(20), cutoff(30), cutoff(31)])
            .expect("windows");
        assert_eq!(validate_recovery_window_sequence(&windows, &windows), Ok(()));
        assert_eq!(
            validate_recovery_window_sequence(&windows, &windows[1..]),
            Err(ModelSelectionError::RecoveryWindowSetMismatch)
        );
        assert_eq!(
            validate_recovery_window_sequence(&windows, &windows[..windows.len() - 1]),
            Err(ModelSelectionError::RecoveryWindowSetMismatch)
        );
    }
}
