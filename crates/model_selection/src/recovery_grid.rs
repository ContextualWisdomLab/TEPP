//! Scientific-recovery binding for a predeclared candidate-`K` design.

use std::collections::BTreeSet;

use corpus_split::RollingOriginPartition;
use membership_core::MembershipNetwork;
use temporal_core::EventTime;
use topic_measurement::{ReferenceTopicTrainingFit, SparseMatrix};
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

/// One rolling-origin evaluation whose fitted candidate dimensions are retained
/// for comparison against the predeclared recovery design.
///
/// This is a scientific-acceptance composition value. It does not mint split,
/// source, Membership, relation, or fitted-model authority; those values remain
/// owned by the wrapped partition, training fits, and numerical inputs.
pub struct RollingOriginRecoveryEvaluation<'a> {
    predictive: RollingOriginPredictiveEvaluation<'a>,
    candidate_topic_counts: BTreeSet<usize>,
    candidate_fit_count: usize,
}

impl<'a> RollingOriginRecoveryEvaluation<'a> {
    /// Group one predictive window while retaining its fitted candidate dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`ModelSelectionError::DuplicateCandidateK`] when two supplied
    /// training fits represent the same fitted topic dimension.
    pub fn new(
        partition: &'a RollingOriginPartition,
        candidate_training_fits: &'a [ReferenceTopicTrainingFit],
        evaluation_document_ids: &'a [Uuid],
        evaluation_document_term: &'a SparseMatrix,
        evaluation_event_times: &'a [EventTime],
        evaluation_covariates: Option<&'a SparseMatrix>,
        memberships: &'a MembershipNetwork,
    ) -> Result<Self, ModelSelectionError> {
        let candidate_topic_counts = unique_candidate_topic_counts(
            candidate_training_fits.iter().map(|fit| {
                fit.reference_fit()
                    .model()
                    .topic_term_probabilities
                    .len()
            }),
            candidate_training_fits.len(),
        )?;

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
            candidate_topic_counts,
            candidate_fit_count: candidate_training_fits.len(),
        })
    }
}

/// Select candidate `K` only when every rolling-origin window covers the
/// predeclared scientific candidate grid exactly.
///
/// The declared grid comes from [`FittedCandidateKConfig`], not from successful
/// fits. Therefore a candidate that fails fitting and disappears from every
/// window cannot silently shrink the recovery design. Such a replication fails
/// closed and can be recorded as `None` by the caller before aggregation with
/// `selected_k_recovery_summary`.
///
/// This stricter path is for scientific recovery/validation. The generic
/// multi-window predictive selector retains its operational survivor semantics.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when there are no windows,
/// [`ModelSelectionError::PredictiveCandidateGridMismatch`] when any window's
/// fitted candidate dimensions differ from the declared grid, or propagates the
/// generic rolling-origin predictive selector's cutoff, identity, candidate, and
/// numerical failures.
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
    }

    let predictive: Vec<_> = evaluations
        .iter()
        .map(|evaluation| evaluation.predictive)
        .collect();
    select_rolling_origin_predictive_candidate_k_across_windows(&predictive)
}

#[cfg(test)]
mod tests {
    use super::unique_candidate_topic_counts;
    use crate::ModelSelectionError;

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
    }
}
