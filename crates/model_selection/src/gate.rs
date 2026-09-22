//! Pareto admission and selection among statistically supported candidates.

use crate::{ModelCandidate, ModelSelectionError};

/// Monte Carlo recovery summary for selected topic counts.
///
/// RMSE and bias are conditional on successful replications. The total,
/// successful, and failed counts remain explicit so failed fits cannot disappear
/// from the scientific denominator. Monte Carlo standard errors use the usual
/// sample-variance estimator across successful replications; RMSE uncertainty
/// applies the delta method to the mean squared error and is exactly zero when
/// every successful replication recovers the truth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectedKRecoverySummary {
    truth_k: u32,
    replication_count: usize,
    success_count: usize,
    failure_count: usize,
    bias: f64,
    root_mean_square_error: f64,
    bias_monte_carlo_standard_error: f64,
    rmse_monte_carlo_standard_error: f64,
}

impl SelectedKRecoverySummary {
    /// Return the true data-generating topic count.
    #[must_use]
    pub const fn truth_k(self) -> u32 {
        self.truth_k
    }

    /// Return every attempted replication, including failed fits/selections.
    #[must_use]
    pub const fn replication_count(self) -> usize {
        self.replication_count
    }

    /// Return replications that produced an admissible selected `K`.
    #[must_use]
    pub const fn success_count(self) -> usize {
        self.success_count
    }

    /// Return attempted replications without an admissible selected `K`.
    #[must_use]
    pub const fn failure_count(self) -> usize {
        self.failure_count
    }

    /// Return the failed-replication share over all attempted replications.
    #[must_use]
    pub fn failure_rate(self) -> f64 {
        self.failure_count as f64 / self.replication_count as f64
    }

    /// Return mean selected-`K` error conditional on successful replications.
    #[must_use]
    pub const fn bias(self) -> f64 {
        self.bias
    }

    /// Return selected-`K` RMSE conditional on successful replications.
    #[must_use]
    pub const fn root_mean_square_error(self) -> f64 {
        self.root_mean_square_error
    }

    /// Return the Monte Carlo standard error of conditional selected-`K` bias.
    #[must_use]
    pub const fn bias_monte_carlo_standard_error(self) -> f64 {
        self.bias_monte_carlo_standard_error
    }

    /// Return the delta-method Monte Carlo standard error of conditional RMSE.
    #[must_use]
    pub const fn rmse_monte_carlo_standard_error(self) -> f64 {
        self.rmse_monte_carlo_standard_error
    }
}

/// Select the unique admissible `K` from a Pareto-filtered statistical front.
///
/// LLM-only candidates are ignored as recommenders and never become the
/// numerical optimum. Among non-dominated statistical candidates the gate
/// prefers higher Schwarz model-selection score, then smaller `K`; complexity
/// is applied while constructing the Pareto front. This gate does not claim a
/// held-out predictive likelihood unless one is separately measured upstream.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when no candidates are
/// supplied or [`ModelSelectionError::LlmVoteIsNotStatisticalAuthority`] when
/// every candidate is an LLM vote.
pub fn select_candidate_k(candidates: &[ModelCandidate]) -> Result<u32, ModelSelectionError> {
    if candidates.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    if candidates
        .iter()
        .all(|candidate| candidate.is_llm_vote_only())
    {
        return Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority);
    }

    let statistical: Vec<ModelCandidate> = candidates
        .iter()
        .copied()
        .filter(|candidate| candidate.is_statistically_supported())
        .collect();
    let mut front: Vec<ModelCandidate> = statistical
        .iter()
        .copied()
        .filter(|candidate| !statistical.iter().any(|other| other.dominates(*candidate)))
        .collect();
    front.sort_by(|left, right| {
        let score_order = right
            .schwarz_score()
            .partial_cmp(&left.schwarz_score())
            .unwrap_or(std::cmp::Ordering::Equal);
        if score_order != std::cmp::Ordering::Equal {
            return score_order;
        }
        left.candidate_k().cmp(&right.candidate_k())
    });
    Ok(front[0].candidate_k())
}

/// Summarize selected-`K` recovery without dropping failed replications.
///
/// Each `Some(K)` is one successful replication and `None` is one failed
/// fitting/selection attempt. Bias and RMSE are therefore conditional on
/// success while [`SelectedKRecoverySummary::failure_count`] and
/// [`SelectedKRecoverySummary::failure_rate`] preserve the unconditional
/// failure denominator required for scientific acceptance. At least two
/// successful replications are required because a one-run result cannot carry
/// an empirical Monte Carlo standard error.
///
/// This function does not itself establish temporal leakage safety: callers
/// must obtain each replication from the canonical rolling-origin owner path.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when no replication was
/// attempted, [`ModelSelectionError::NonPositiveCandidateK`] when `truth_k` is
/// less than two, [`ModelSelectionError::InvalidDiagnostic`] when any successful
/// selected `K` is less than two, or
/// [`ModelSelectionError::InsufficientRecoveryReplications`] when fewer than two
/// successful replications remain.
pub fn selected_k_recovery_summary(
    selected: &[Option<u32>],
    truth_k: u32,
) -> Result<SelectedKRecoverySummary, ModelSelectionError> {
    if selected.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    if truth_k < 2 {
        return Err(ModelSelectionError::NonPositiveCandidateK);
    }

    let mut residuals = Vec::new();
    let mut failure_count = 0_usize;
    for outcome in selected {
        match outcome {
            Some(selected_k) if *selected_k >= 2 => {
                residuals.push(f64::from(*selected_k) - f64::from(truth_k));
            }
            Some(_) => return Err(ModelSelectionError::InvalidDiagnostic),
            None => failure_count += 1,
        }
    }
    if residuals.len() < 2 {
        return Err(ModelSelectionError::InsufficientRecoveryReplications);
    }

    let success_count = residuals.len();
    let replication_count = selected.len();
    let n = success_count as f64;
    let bias = residuals.iter().sum::<f64>() / n;
    let mean_squared_error = residuals.iter().map(|value| value * value).sum::<f64>() / n;
    let root_mean_square_error = mean_squared_error.sqrt();
    let sample_denominator = (success_count - 1) as f64;
    let residual_sample_variance = residuals
        .iter()
        .map(|value| (value - bias).powi(2))
        .sum::<f64>()
        / sample_denominator;
    let bias_monte_carlo_standard_error = (residual_sample_variance / n).sqrt();
    let squared_error_sample_variance = residuals
        .iter()
        .map(|value| ((value * value) - mean_squared_error).powi(2))
        .sum::<f64>()
        / sample_denominator;
    let mse_monte_carlo_standard_error = (squared_error_sample_variance / n).sqrt();
    let rmse_monte_carlo_standard_error = if root_mean_square_error == 0.0 {
        0.0
    } else {
        mse_monte_carlo_standard_error / (2.0 * root_mean_square_error)
    };

    if !bias.is_finite()
        || !root_mean_square_error.is_finite()
        || !bias_monte_carlo_standard_error.is_finite()
        || !rmse_monte_carlo_standard_error.is_finite()
    {
        return Err(ModelSelectionError::InvalidDiagnostic);
    }

    Ok(SelectedKRecoverySummary {
        truth_k,
        replication_count,
        success_count,
        failure_count,
        bias,
        root_mean_square_error,
        bias_monte_carlo_standard_error,
        rmse_monte_carlo_standard_error,
    })
}

/// RMSE of selected `K` replications against a known-truth topic count.
///
/// This compatibility helper does not retain failed-replication denominators or
/// Monte Carlo uncertainty. Scientific acceptance should use
/// [`selected_k_recovery_summary`] instead.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when `selected` is
/// empty, [`ModelSelectionError::NonPositiveCandidateK`] when `truth_k` is
/// less than two, or [`ModelSelectionError::InvalidDiagnostic`] when a
/// selected replication is less than two.
pub fn selected_k_root_mean_square_error(
    selected: &[u32],
    truth_k: u32,
) -> Result<f64, ModelSelectionError> {
    if selected.is_empty() {
        return Err(ModelSelectionError::EmptyCandidateSet);
    }
    if truth_k < 2 {
        return Err(ModelSelectionError::NonPositiveCandidateK);
    }
    let mut sum_squares = 0.0_f64;
    for selected_k in selected {
        if *selected_k < 2 {
            return Err(ModelSelectionError::InvalidDiagnostic);
        }
        let residual = f64::from(*selected_k) - f64::from(truth_k);
        sum_squares += residual * residual;
    }
    Ok((sum_squares / selected.len() as f64).sqrt())
}

#[cfg(test)]
mod tests {
    use super::{select_candidate_k, selected_k_root_mean_square_error};
    use crate::{ModelCandidate, ModelSelectionError};

    #[test]
    fn gate_helpers_cover_local_branches() {
        let a = ModelCandidate::statistical(2, -30.0, 8.0).expect("a");
        let b = ModelCandidate::statistical(4, -30.0, 8.0).expect("b");
        assert_eq!(
            select_candidate_k(&[]),
            Err(ModelSelectionError::EmptyCandidateSet)
        );
        assert_eq!(select_candidate_k(&[a, b]).expect("tie"), 2);
        let higher_score = ModelCandidate::statistical(8, -20.0, 9.0).expect("score");
        assert_eq!(
            select_candidate_k(&[a, higher_score]).expect("score tie-break"),
            8
        );

        assert_eq!(
            selected_k_root_mean_square_error(&[], 4),
            Err(ModelSelectionError::EmptyCandidateSet)
        );
        assert_eq!(
            selected_k_root_mean_square_error(&[4], 1),
            Err(ModelSelectionError::NonPositiveCandidateK)
        );
        assert_eq!(
            selected_k_root_mean_square_error(&[1], 4),
            Err(ModelSelectionError::InvalidDiagnostic)
        );
        assert!(
            selected_k_root_mean_square_error(&[4], 4)
                .expect("valid rmse")
                .abs()
                < f64::EPSILON
        );
        assert_eq!(
            select_candidate_k(&[ModelCandidate::llm_vote_only(3).expect("valid llm candidate")]),
            Err(ModelSelectionError::LlmVoteIsNotStatisticalAuthority)
        );
    }
}
