//! Monte Carlo aggregation of recovery metrics.

use crate::ValidationError;
use crate::input::require_finite;

/// Summary of Monte Carlo replications for a scalar metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonteCarloSummary {
    /// Number of finite replications retained.
    pub replication_count: usize,
    /// Sample mean.
    pub mean: f64,
    /// Sample standard deviation (`n − 1` denominator).
    pub standard_deviation: f64,
    /// Standard error of the mean.
    pub standard_error: f64,
    /// Inclusive empirical percentile lower bound.
    pub percentile_lower: f64,
    /// Inclusive empirical percentile upper bound.
    pub percentile_upper: f64,
}

impl MonteCarloSummary {
    /// Validate structural invariants for a Monte Carlo summary payload.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidInput`] when counts or numeric fields
    /// violate the summary contract.
    pub fn validate(self) -> Result<Self, ValidationError> {
        if self.replication_count == 0 {
            return Err(ValidationError::InvalidInput);
        }
        for value in [
            self.mean,
            self.standard_deviation,
            self.standard_error,
            self.percentile_lower,
            self.percentile_upper,
        ] {
            if !value.is_finite() {
                return Err(ValidationError::InvalidInput);
            }
        }
        if self.standard_deviation < 0.0 || self.standard_error < 0.0 {
            return Err(ValidationError::InvalidInput);
        }
        if self.percentile_lower > self.percentile_upper {
            return Err(ValidationError::InvalidInput);
        }
        Ok(self)
    }
}

/// Aggregate Monte Carlo metric replications with percentile bounds.
///
/// Percentiles use the inclusive nearest-rank method on sorted finite samples.
/// Mean and variance use Welford accumulation so large finite samples do not
/// overflow intermediate sums.
///
/// # Errors
///
/// Returns input errors for empty/non-finite samples or non-finite summaries,
/// and configuration errors for invalid percentile bounds.
///
/// # Panics
///
/// Does not panic: samples are pre-validated as finite before sorting.
pub fn summarize_replications(
    samples: &[f64],
    lower_percentile: f64,
    upper_percentile: f64,
) -> Result<MonteCarloSummary, ValidationError> {
    if samples.is_empty() || samples.iter().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidInput);
    }
    if !(0.0..=1.0).contains(&lower_percentile)
        || !(0.0..=1.0).contains(&upper_percentile)
        || lower_percentile > upper_percentile
    {
        return Err(ValidationError::InvalidConfiguration);
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let (mean, m2, count) = welford_moments(&sorted)?;
    let n = count as f64;
    let standard_deviation = if count == 1 {
        0.0
    } else {
        require_finite((m2 / (n - 1.0)).sqrt())?
    };
    let standard_error = require_finite(standard_deviation / n.sqrt())?;
    let summary = MonteCarloSummary {
        replication_count: sorted.len(),
        mean: require_finite(mean)?,
        standard_deviation,
        standard_error,
        percentile_lower: nearest_rank(&sorted, lower_percentile),
        percentile_upper: nearest_rank(&sorted, upper_percentile),
    };
    summary.validate()
}

/// SE-aware acceptance: accept when `|estimate − target| ≤ k · se`.
///
/// Comparison scales all terms by a shared finite magnitude so opposite-sign
/// extremes do not overflow both sides of the inequality to infinity.
///
/// # Errors
///
/// Returns input errors for non-finite values and configuration errors for
/// `k < 0` or negative `standard_error`.
pub fn accept_within_standard_errors(
    estimate: f64,
    target: f64,
    standard_error: f64,
    k: f64,
) -> Result<bool, ValidationError> {
    if ![estimate, target, standard_error, k]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(ValidationError::InvalidInput);
    }
    if k < 0.0 || standard_error < 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    if standard_error == 0.0 {
        // Exact recovery only: zero SE admits no estimation residual.
        return Ok(estimate.total_cmp(&target).is_eq());
    }
    let scale = estimate
        .abs()
        .max(target.abs())
        .max(standard_error)
        .max(1.0);
    let scaled_error = (estimate / scale) - (target / scale);
    let scaled_bound = k * (standard_error / scale);
    // scale is at least 1.0 and all inputs are finite, so scaled terms are finite.
    Ok(scaled_error.abs() <= scaled_bound)
}

/// Welford one-pass mean and sum of squared deviations.
fn welford_moments(samples: &[f64]) -> Result<(f64, f64, usize), ValidationError> {
    let mut mean = 0.0_f64;
    let mut m2 = 0.0_f64;
    let mut count = 0_usize;
    for value in samples {
        count += 1;
        let delta = value - mean;
        mean += delta / count as f64;
        if !mean.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        let delta2 = value - mean;
        m2 += delta * delta2;
    }
    Ok((mean, m2, count))
}

#[allow(clippy::cast_possible_truncation)]
fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = (percentile * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::{MonteCarloSummary, accept_within_standard_errors, summarize_replications};
    use crate::ValidationError;

    #[test]
    fn monte_carlo_summary_and_acceptance_gates() {
        let samples = [1.0, 2.0, 3.0, 4.0];
        let summary = summarize_replications(&samples, 0.25, 0.75).expect("sum");
        assert_eq!(summary.replication_count, 4);
        assert!((summary.mean - 2.5).abs() < 1e-12);
        assert!(summary.standard_deviation > 0.0);
        assert!(summary.standard_error > 0.0);
        assert!((summary.percentile_lower - 1.0).abs() < 1e-12);
        assert!((summary.percentile_upper - 3.0).abs() < 1e-12);
        let single = summarize_replications(&[2.0], 0.0, 1.0).expect("one");
        assert!((single.standard_deviation - 0.0).abs() < 1e-12);
        assert!((single.percentile_lower - 2.0).abs() < 1e-12);
        assert!(accept_within_standard_errors(1.0, 1.0, 0.1, 1.0).expect("acc"));
        assert!(!accept_within_standard_errors(1.0, 2.0, 0.1, 1.0).expect("rej"));
        assert_eq!(
            summarize_replications(&[], 0.1, 0.9),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            summarize_replications(&[f64::NAN], 0.1, 0.9),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            summarize_replications(&[1.0], 0.9, 0.1),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            summarize_replications(&[1.0], -0.1, 0.5),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            summarize_replications(&[1.0], 0.0, 1.1),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, 1.0, -0.1, 1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, 1.0, 0.1, -1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            accept_within_standard_errors(f64::NAN, 1.0, 0.1, 1.0),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            accept_within_standard_errors(1.0, f64::INFINITY, 0.1, 1.0),
            Err(ValidationError::InvalidInput)
        );
        // equal percentiles
        let edge = summarize_replications(&[1.0, 2.0], 0.5, 0.5).expect("eq");
        assert!((edge.percentile_lower - edge.percentile_upper).abs() < 1e-12);
        // Large finite samples must not overflow the summary path.
        let large = summarize_replications(&[f64::MAX, f64::MAX], 0.0, 1.0).expect("large");
        assert!((large.mean - f64::MAX).abs() < 1.0);
        assert!((large.standard_deviation - 0.0).abs() < 1e-12);
        assert!(
            !accept_within_standard_errors(f64::MAX, -f64::MAX, f64::MAX, 1.5).expect("scaled")
        );
    }

    #[test]
    fn nonfinite_acceptance_and_summary_validate() {
        assert!(accept_within_standard_errors(1.0, 1.0, 0.0, 1.0).expect("eq"));
        assert!(!accept_within_standard_errors(1.0, 2.0, 0.0, 1.0).expect("neq"));
        assert_eq!(
            summarize_replications(&[f64::MAX, -f64::MAX], 0.0, 1.0),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 0,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: f64::NAN,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: -0.1,
                standard_error: 0.0,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: -0.1,
                percentile_lower: 0.0,
                percentile_upper: 1.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            MonteCarloSummary {
                replication_count: 2,
                mean: 0.0,
                standard_deviation: 0.0,
                standard_error: 0.0,
                percentile_lower: 1.0,
                percentile_upper: 0.0,
            }
            .validate(),
            Err(ValidationError::InvalidInput)
        );
    }
}
