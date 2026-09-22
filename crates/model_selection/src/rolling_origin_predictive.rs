//! Consumer binding between admitted rolling-origin partitions and predictive rows.

use std::collections::{BTreeMap, BTreeSet};

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

fn select_best_predictive_candidate(scores: &[(u32, f64)]) -> Result<u32, ModelSelectionError> {
    let mut scores = scores.iter().copied();
    let mut best = scores.next().ok_or(ModelSelectionError::EmptyCandidateSet)?;
    for candidate in scores {
        if predictive_candidate_is_better(best, candidate) {
            best = candidate;
        }
    }
    Ok(best.0)
}

fn add_predictive_score(total: &mut f64, score: f64) -> Result<(), ModelSelectionError> {
    *total += score;
    if total.is_finite() {
        Ok(())
    } else {
        Err(ModelSelectionError::InvalidDiagnostic)
    }
}

/// Borrowed numerical payload for one admitted rolling-origin evaluation window.
///
/// This value groups the split owner, candidate training fits, and caller-order
/// evaluation rows so multi-window selection cannot accidentally zip unrelated
/// parallel slices. It is only a borrowed composition view: construction does
/// not mint source, Membership, relation, or cutoff authority. Those invariants
/// are checked by the owning partition and predictive scorer when consumed.
#[derive(Clone, Copy)]
pub struct RollingOriginPredictiveEvaluation<'a> {
    partition: &'a RollingOriginPartition,
    candidate_training_fits: &'a [ReferenceTopicTrainingFit],
    evaluation_document_ids: &'a [Uuid],
    evaluation_document_term: &'a SparseMatrix,
    evaluation_event_times: &'a [EventTime],
    evaluation_covariates: Option<&'a SparseMatrix>,
    memberships: &'a MembershipNetwork,
}

impl<'a> RollingOriginPredictiveEvaluation<'a> {
    /// Group one admitted partition with its candidate fits and evaluation rows.
    #[must_use]
    pub const fn new(
        partition: &'a RollingOriginPartition,
        candidate_training_fits: &'a [ReferenceTopicTrainingFit],
        evaluation_document_ids: &'a [Uuid],
        evaluation_document_term: &'a SparseMatrix,
        evaluation_event_times: &'a [EventTime],
        evaluation_covariates: Option<&'a SparseMatrix>,
        memberships: &'a MembershipNetwork,
    ) -> Self {
        Self {
            partition,
            candidate_training_fits,
            evaluation_document_ids,
            evaluation_document_term,
            evaluation_event_times,
            evaluation_covariates,
            memberships,
        }
    }
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

fn predictive_candidate_scores(
    evaluation: RollingOriginPredictiveEvaluation<'_>,
) -> Result<Vec<(u32, f64)>, ModelSelectionError> {
    if evaluation.candidate_training_fits.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    let mut seen_candidate_k = BTreeSet::new();
    let mut scores = Vec::with_capacity(evaluation.candidate_training_fits.len());
    for training_fit in evaluation.candidate_training_fits {
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
            evaluation.partition,
            training_fit,
            evaluation.evaluation_document_ids,
            evaluation.evaluation_document_term,
            evaluation.evaluation_event_times,
            evaluation.evaluation_covariates,
            evaluation.memberships,
        )?;
        scores.push((candidate_k, score));
    }
    Ok(scores)
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
    let evaluation = RollingOriginPredictiveEvaluation::new(
        partition,
        candidate_training_fits,
        evaluation_document_ids,
        evaluation_document_term,
        evaluation_event_times,
        evaluation_covariates,
        memberships,
    );
    let scores = predictive_candidate_scores(evaluation)?;
    select_best_predictive_candidate(&scores)
}

/// Select candidate `K` from predictive evidence accumulated across windows.
///
/// Adjacent partitions must form one contiguous chronological sequence: each
/// next training cutoff equals the previous evaluation cutoff. Every window must
/// expose the same unique fitted candidate-K set. Each `(window, K)` score is
/// obtained through the same partition-bound fixed-training scorer used by the
/// one-window gate, then finite log likelihoods are summed by `K`. The largest
/// aggregate wins with smaller `K` on an exact tie.
///
/// This is score aggregation across admitted rolling-origin windows, not a vote
/// over per-window winners, not the in-sample Schwarz criterion, and not STM
/// document-completion likelihood. It also does not itself establish realistic
/// repeated-simulation recovery acceptance.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] for no windows or a window
/// without fitted candidates; [`ModelSelectionError::RollingOriginWindowMismatch`]
/// for a noncontiguous window sequence; [`ModelSelectionError::DuplicateCandidateK`]
/// for repeated fitted dimensions within a window;
/// [`ModelSelectionError::PredictiveCandidateSetMismatch`] when candidate-K sets
/// differ across windows; or [`ModelSelectionError::InvalidDiagnostic`] when
/// finite per-window scores overflow during cross-window aggregation. Partition
/// and numerical input failures propagate from the one-window scorer.
pub fn select_rolling_origin_predictive_candidate_k_across_windows(
    evaluations: &[RollingOriginPredictiveEvaluation<'_>],
) -> Result<u32, ModelSelectionError> {
    let first = *evaluations
        .first()
        .ok_or(ModelSelectionError::EmptyCandidateSet)?;
    for pair in evaluations.windows(2) {
        if pair[0].partition.window().test_cutoff != pair[1].partition.window().train_cutoff {
            return Err(ModelSelectionError::RollingOriginWindowMismatch);
        }
    }

    let first_scores = predictive_candidate_scores(first)?;
    let expected_candidate_k: BTreeSet<_> =
        first_scores.iter().map(|(candidate_k, _)| *candidate_k).collect();
    let mut totals: BTreeMap<u32, f64> = first_scores.into_iter().collect();

    for evaluation in evaluations.iter().copied().skip(1) {
        let scores = predictive_candidate_scores(evaluation)?;
        let current_candidate_k: BTreeSet<_> =
            scores.iter().map(|(candidate_k, _)| *candidate_k).collect();
        if current_candidate_k != expected_candidate_k {
            return Err(ModelSelectionError::PredictiveCandidateSetMismatch);
        }
        for (candidate_k, score) in scores {
            let Some(total) = totals.get_mut(&candidate_k) else {
                return Err(ModelSelectionError::PredictiveCandidateSetMismatch);
            };
            add_predictive_score(total, score)?;
        }
    }

    let totals: Vec<_> = totals.into_iter().collect();
    select_best_predictive_candidate(&totals)
}

#[cfg(test)]
mod tests {
    use super::{
        add_predictive_score, candidate_k_from_topic_count, predictive_candidate_is_better,
        select_best_predictive_candidate,
    };
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
        assert_eq!(
            select_best_predictive_candidate(&[]),
            Err(ModelSelectionError::EmptyCandidateSet)
        );
        assert_eq!(
            select_best_predictive_candidate(&[(3, -10.0), (2, -10.0)]),
            Ok(2)
        );
    }

    #[test]
    fn predictive_score_aggregation_fails_closed_on_overflow() {
        let mut total = 1.0;
        assert_eq!(add_predictive_score(&mut total, 2.0), Ok(()));
        assert_eq!(total, 3.0);
        let mut extreme = f64::MAX;
        assert_eq!(
            add_predictive_score(&mut extreme, f64::MAX),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
    }
}
