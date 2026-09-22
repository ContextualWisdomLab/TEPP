//! Consumer binding between admitted rolling-origin partitions and predictive rows.

use std::collections::BTreeSet;

use corpus_split::RollingOriginPartition;
use membership_core::MembershipNetwork;
use temporal_core::EventTime;
use topic_measurement::{ReferenceTopicTrainingFit, SparseMatrix};
use uuid::Uuid;

use crate::ModelSelectionError;

fn candidate_k_from_topic_count(topic_count: usize) -> Result<u32, ModelSelectionError> {
    if topic_count < 2 {
        return Err(ModelSelectionError::NonPositiveCandidateK);
    }
    u32::try_from(topic_count).map_err(|_| ModelSelectionError::InvalidDiagnostic)
}

fn predictive_candidate_is_better(best: (u32, f64), candidate: (u32, f64)) -> bool {
    candidate.1 > best.1 || (candidate.1 == best.1 && candidate.0 < best.0)
}

/// Score one admitted rolling-origin evaluation partition under a fixed training fit.
///
/// The training fit's exact document identity set must match the admitted
/// partition's training set, and the caller-order evaluation identities must be
/// duplicate-free and equal the partition's evaluation set. This prevents a
/// valid cutoff/leakage receipt for one partition from being rebound to
/// dimension-compatible numerical rows from another partition.
///
/// The numerical quantity is the sum of #684's per-document fixed-training
/// prevalence-mean predictive log likelihoods. It is not STM document-completion
/// likelihood, the in-sample Schwarz score, or a new split/cutoff authority.
///
/// # Errors
///
/// Returns [`ModelSelectionError::PartitionInputMismatch`] when training or
/// evaluation identities do not exactly match the admitted partition. Returns
/// [`ModelSelectionError::InvalidDiagnostic`] when the topic-measurement owner
/// rejects evaluation geometry/coordinates/counts or when finite per-document
/// scores overflow while aggregating the partition diagnostic.
pub fn rolling_origin_prevalence_mean_predictive_log_likelihood(
    partition: &RollingOriginPartition,
    training_fit: &ReferenceTopicTrainingFit,
    evaluation_document_ids: &[Uuid],
    evaluation_document_term: &SparseMatrix,
    evaluation_event_times: &[EventTime],
    evaluation_covariates: Option<&SparseMatrix>,
    memberships: &MembershipNetwork,
) -> Result<f64, ModelSelectionError> {
    let fitted_training_ids: BTreeSet<_> = training_fit
        .reference_fit()
        .input()
        .document_ids()
        .iter()
        .copied()
        .collect();
    if fitted_training_ids != *partition.training_document_ids() {
        return Err(ModelSelectionError::PartitionInputMismatch);
    }

    let evaluation_ids: BTreeSet<_> = evaluation_document_ids.iter().copied().collect();
    if evaluation_ids.len() != evaluation_document_ids.len()
        || evaluation_ids != *partition.evaluation_document_ids()
    {
        return Err(ModelSelectionError::PartitionInputMismatch);
    }

    let scores = training_fit
        .prevalence_mean_predictive_log_likelihoods(
            evaluation_document_ids,
            evaluation_document_term,
            evaluation_event_times,
            evaluation_covariates,
            memberships,
        )
        .map_err(|_| ModelSelectionError::InvalidDiagnostic)?;
    let diagnostic = scores.into_iter().sum::<f64>();
    if !diagnostic.is_finite() {
        return Err(ModelSelectionError::InvalidDiagnostic);
    }
    Ok(diagnostic)
}

/// Select candidate `K` from one admitted rolling-origin predictive partition.
///
/// Every candidate is an owner-issued [`ReferenceTopicTrainingFit`]. Its topic
/// dimension defines `K`; callers cannot attach a detached label. Each candidate
/// is scored only through [`rolling_origin_prevalence_mean_predictive_log_likelihood`],
/// so the exact training/evaluation identities and frozen training prevalence
/// basis remain enforced. The highest finite predictive score wins, with the
/// smaller `K` winning an exact tie. Candidate order therefore has no authority.
///
/// This gate is intentionally separate from the in-sample Schwarz selector. The
/// score is a prevalence-mean fixed-training predictive log likelihood, not STM
/// document-completion likelihood and not a complete multi-window recovery
/// acceptance result.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when no fitted candidate
/// is supplied, [`ModelSelectionError::DuplicateCandidateK`] when two candidates
/// have the same fitted topic dimension, and propagates the fail-closed partition
/// or numerical diagnostic errors from the rolling-origin predictive scorer.
pub fn select_rolling_origin_predictive_candidate_k(
    partition: &RollingOriginPartition,
    candidate_training_fits: &[ReferenceTopicTrainingFit],
    evaluation_document_ids: &[Uuid],
    evaluation_document_term: &SparseMatrix,
    evaluation_event_times: &[EventTime],
    evaluation_covariates: Option<&SparseMatrix>,
    memberships: &MembershipNetwork,
) -> Result<u32, ModelSelectionError> {
    let mut candidates = candidate_training_fits.iter();
    let first_fit = candidates
        .next()
        .ok_or(ModelSelectionError::EmptyCandidateSet)?;
    let first_k = candidate_k_from_topic_count(
        first_fit
            .reference_fit()
            .model()
            .topic_term_probabilities
            .len(),
    )?;
    let first_score = rolling_origin_prevalence_mean_predictive_log_likelihood(
        partition,
        first_fit,
        evaluation_document_ids,
        evaluation_document_term,
        evaluation_event_times,
        evaluation_covariates,
        memberships,
    )?;
    let mut seen_candidate_k = BTreeSet::from([first_k]);
    let mut best = (first_k, first_score);

    for training_fit in candidates {
        let candidate_k = candidate_k_from_topic_count(
            training_fit
                .reference_fit()
                .model()
                .topic_term_probabilities
                .len(),
        )?;
        if !seen_candidate_k.insert(candidate_k) {
            return Err(ModelSelectionError::DuplicateCandidateK);
        }
        let score = rolling_origin_prevalence_mean_predictive_log_likelihood(
            partition,
            training_fit,
            evaluation_document_ids,
            evaluation_document_term,
            evaluation_event_times,
            evaluation_covariates,
            memberships,
        )?;
        let candidate = (candidate_k, score);
        if predictive_candidate_is_better(best, candidate) {
            best = candidate;
        }
    }

    Ok(best.0)
}

#[cfg(test)]
mod tests {
    use super::{candidate_k_from_topic_count, predictive_candidate_is_better};
    use crate::ModelSelectionError;

    #[test]
    fn topic_count_conversion_is_fail_closed() {
        assert_eq!(
            candidate_k_from_topic_count(1),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
        assert_eq!(candidate_k_from_topic_count(2), Ok(2));
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            candidate_k_from_topic_count(u32::MAX as usize + 1),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
    }

    #[test]
    fn predictive_candidate_ordering_covers_score_and_tie_rules() {
        assert!(predictive_candidate_is_better((2, -20.0), (3, -10.0)));
        assert!(!predictive_candidate_is_better((2, -10.0), (3, -20.0)));
        assert!(predictive_candidate_is_better((3, -10.0), (2, -10.0)));
        assert!(!predictive_candidate_is_better((2, -10.0), (3, -10.0)));
    }
}
