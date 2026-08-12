//! Monte Carlo aggregation of recovery metrics.

use crate::ValidationError;

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

/// Aggregate Monte Carlo metric replications with percentile bounds.
///
/// Percentiles use the inclusive nearest-rank method on sorted finite samples.
///
/// # Errors
///
/// Returns input errors for empty/non-finite samples and configuration errors
/// for invalid percentile bounds.
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
    let n = sorted.len() as f64;
    let mean = sorted.iter().sum::<f64>() / n;
    let standard_deviation = if sorted.len() == 1 {
        0.0
    } else {
        let variance = sorted
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / (n - 1.0);
        variance.sqrt()
    };
    let standard_error = standard_deviation / n.sqrt();
    Ok(MonteCarloSummary {
        replication_count: sorted.len(),
        mean,
        standard_deviation,
        standard_error,
        percentile_lower: nearest_rank(&sorted, lower_percentile),
        percentile_upper: nearest_rank(&sorted, upper_percentile),
    })
}

/// SE-aware acceptance: accept when `|estimate − target| ≤ k · se`.
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
    Ok((estimate - target).abs() <= k * standard_error)
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
    use super::{accept_within_standard_errors, summarize_replications};
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
    }
}
