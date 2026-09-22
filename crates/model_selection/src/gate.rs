//! Pareto admission and selection among statistically supported candidates.

use crate::{ModelCandidate, ModelSelectionError};

/// Monte Carlo recovery summary for selected topic counts.
///
/// RMSE and bias are conditional on successful replications. The total,
/// successful, and failed counts remain explicit so failed fits cannot disappear
/// from the scientific denominator. Conditional recovery measures are optional:
/// zero successful replications preserve the failure evidence with no fabricated
/// bias/RMSE, while one successful replication can report bias/RMSE but not an
/// empirical Monte Carlo standard error. Failure-rate Monte Carlo uncertainty
/// continues to use every attempted replication. With at least two successful
/// replications, bias uses the usual sample-variance estimator and RMSE uncertainty
/// applies the delta method; RMSE MCSE is exactly zero when every successful
/// replication recovers the truth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectedKRecoverySummary {
    truth_k: u32,
    replication_count: usize,
    success_count: usize,
    failure_count: usize,
    failure_rate_monte_carlo_standard_error: f64,
    bias: Option<f64>,
    root_mean_square_error: Option<f64>,
    bias_monte_carlo_standard_error: Option<f64>,
    rmse_monte_carlo_standard_error: Option<f64>,
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

    /// Return the Monte Carlo standard error of the empirical failure rate.
    ///
    /// This uses every attempted replication as the Bernoulli denominator:
    /// `sqrt(p_hat * (1 - p_hat) / R)`, where `R` is the attempted replication
    /// count. It is simulation uncertainty for the observed failure proportion,
    /// not a confidence interval or uncertainty conditional on successful fits.
    #[must_use]
    pub const fn failure_rate_monte_carlo_standard_error(self) -> f64 {
        self.failure_rate_monte_carlo_standard_error
    }

    /// Return mean selected-`K` error conditional on at least one success.
    #[must_use]
    pub const fn bias(self) -> Option<f64> {
        self.bias
    }

    /// Return selected-`K` RMSE conditional on at least one success.
    #[must_use]
    pub const fn root_mean_square_error(self) -> Option<f64> {
        self.root_mean_square_error
    }

    /// Return the Monte Carlo standard error of conditional selected-`K` bias.
    ///
    /// At least two successful replications are required for an empirical sample
    /// variance; otherwise this returns `None` while the unconditional failure
    /// denominator remains available.
    #[must_use]
    pub const fn bias_monte_carlo_standard_error(self) -> Option<f64> {
        self.bias_monte_carlo_standard_error
    }

    /// Return the delta-method Monte Carlo standard error of conditional RMSE.
    ///
    /// At least two successful replications are required; otherwise this returns
    /// `None` rather than manufacturing zero or NaN.
    #[must_use]
    pub const fn rmse_monte_carlo_standard_error(self) -> Option<f64> {
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
/// failure denominator required for scientific acceptance. The failure-rate
/// Monte Carlo standard error uses all attempted replications. Zero-success
/// experiments still return a summary with unavailable conditional recovery
/// measures; one-success experiments return conditional bias/RMSE while their
/// empirical MCSE remains unavailable. At least two successes are required only
/// for the success-conditional Monte Carlo standard errors.
///
/// This function does not itself establish temporal leakage safety: callers
/// must obtain each replication from the canonical rolling-origin owner path.
///
/// # Errors
///
/// Returns [`ModelSelectionError::EmptyCandidateSet`] when no replication was
/// attempted, [`ModelSelectionError::NonPositiveCandidateK`] when `truth_k` is
/// less than two, or [`ModelSelectionError::InvalidDiagnostic`] when any successful
/// selected `K` is less than two.
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

    let success_count = residuals.len();
    let replication_count = selected.len();
    let replication_count_f64 = replication_count as f64;
    let failure_rate = failure_count as f64 / replication_count_f64;
    let failure_rate_monte_carlo_standard_error =
        (failure_rate * (1.0 - failure_rate) / replication_count_f64).sqrt();

    let (bias, root_mean_square_error, bias_monte_carlo_standard_error, rmse_monte_carlo_standard_error) =
        if residuals.is_empty() {
            (None, None, None, None)
        } else {
            let n = success_count as f64;
            let bias = residuals.iter().sum::<f64>() / n;
            let mean_squared_error = residuals.iter().map(|value| value * value).sum::<f64>() / n;
            let root_mean_square_error = mean_squared_error.sqrt();

            if success_count < 2 {
                (Some(bias), Some(root_mean_square_error), None, None)
            } else {
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
                let mse_monte_carlo_standard_error =
                    (squared_error_sample_variance / n).sqrt();
                let rmse_monte_carlo_standard_error = if root_mean_square_error == 0.0 {
                    0.0
                } else {
                    mse_monte_carlo_standard_error / (2.0 * root_mean_square_error)
                };
                (
                    Some(bias),
                    Some(root_mean_square_error),
                    Some(bias_monte_carlo_standard_error),
                    Some(rmse_monte_carlo_standard_error),
                )
            }
        };

    Ok(SelectedKRecoverySummary {
        truth_k,
        replication_count,
        success_count,
        failure_count,
        failure_rate_monte_carlo_standard_error,
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
