//! Interval coverage for recovered confidence/credible intervals.

use crate::ValidationError;
use crate::monte_carlo::{MonteCarloSummary, summarize_replications};

/// Empirical coverage of closed intervals `[lower, upper]` for truth values.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when vectors are empty, lengths
/// differ, bounds are non-finite, or any interval is inverted (`lower > upper`).
pub fn interval_coverage(
    truth: &[f64],
    lower: &[f64],
    upper: &[f64],
) -> Result<f64, ValidationError> {
    if truth.is_empty() || truth.len() != lower.len() || truth.len() != upper.len() {
        return Err(ValidationError::InvalidInput);
    }
    let mut covered = 0usize;
    for index in 0..truth.len() {
        let t = truth[index];
        let lo = lower[index];
        let hi = upper[index];
        if !t.is_finite() || !lo.is_finite() || !hi.is_finite() {
            return Err(ValidationError::InvalidInput);
        }
        if lo > hi {
            return Err(ValidationError::InvalidInput);
        }
        let low_ok = t >= lo;
        let high_ok = t <= hi;
        if low_ok && high_ok {
            covered += 1;
        }
    }
    Ok(covered as f64 / truth.len() as f64)
}

/// Collapse rolling-origin coverage within each DGP replication before Monte Carlo inference.
///
/// Each outer element is one independently generated DGP replication. Each inner
/// vector contains already-computed coverage proportions for the declared
/// rolling-origin windows within that replication. Windows are weighted equally
/// inside a DGP replication; the resulting per-DGP means are the only samples
/// passed to the Monte Carlo summary owner.
///
/// This contract prevents repeated document, coordinate, or expanding-window
/// intervals from inflating the Monte Carlo replication count. It does not assert
/// independence within a DGP replication and does not construct binomial/Wilson
/// intervals from the flattened interval count.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when fewer than two independent DGP
/// replications are supplied, any replication has no declared windows, or any
/// window coverage is non-finite or outside `[0, 1]`. Percentile configuration
/// errors are propagated from [`summarize_replications`].
pub fn summarize_windowed_coverage_replications(
    replication_window_coverages: &[Vec<f64>],
    lower_percentile: f64,
    upper_percentile: f64,
) -> Result<MonteCarloSummary, ValidationError> {
    if replication_window_coverages.len() < 2
        || replication_window_coverages.iter().any(|windows| {
            windows.is_empty()
                || windows
                    .iter()
                    .any(|coverage| !coverage.is_finite() || !(0.0..=1.0).contains(coverage))
        })
    {
        return Err(ValidationError::InvalidInput);
    }

    let per_replication_coverage: Vec<f64> = replication_window_coverages
        .iter()
        .map(|windows| windows.iter().sum::<f64>() / windows.len() as f64)
        .collect();
    summarize_replications(
        &per_replication_coverage,
        lower_percentile,
        upper_percentile,
    )
}

/// Wilson score lower/upper bounds for a binomial coverage proportion.
///
/// Returns `(lower, upper)` for the empirical coverage rate at the stated
/// normal critical value `z` (for example `1.96` for nominal 95%).
///
/// # Errors
///
/// Returns configuration errors for non-finite `z` or `z <= 0`, and input
/// errors for empty/invalid interval triples.
pub fn wilson_coverage_interval(
    truth: &[f64],
    lower: &[f64],
    upper: &[f64],
    z: f64,
) -> Result<(f64, f64), ValidationError> {
    if !z.is_finite() || z <= 0.0 {
        return Err(ValidationError::InvalidConfiguration);
    }
    let p = interval_coverage(truth, lower, upper)?;
    let n = truth.len() as f64;
    let z2 = z * z;
    if !z2.is_finite() {
        return Err(ValidationError::InvalidConfiguration);
    }
    let denominator = 1.0 + z2 / n;
    let center = p + z2 / (2.0 * n);
    let radical = (p * (1.0 - p) / n) + z2 / (4.0 * n * n);
    // With finite z² and coverage p in [0,1], Wilson terms remain finite.
    let margin = z * radical.sqrt();
    // radical and z are finite and non-negative; margin/bounds stay finite in [0,1].
    let low = ((center - margin) / denominator).clamp(0.0, 1.0);
    let high = ((center + margin) / denominator).clamp(0.0, 1.0);
    Ok((low, high))
}

#[cfg(test)]
mod tests {
    use super::{
        interval_coverage, summarize_windowed_coverage_replications, wilson_coverage_interval,
    };
    use crate::ValidationError;

    #[test]
    fn coverage_and_wilson_bounds_are_oracle_correct() {
        let truth = [0.0, 1.0, 2.0, 3.0];
        let lower = [-0.5, 0.5, 1.5, 4.0];
        let upper = [0.5, 1.5, 2.5, 5.0];
        // first three covered, last not → 0.75
        assert!((interval_coverage(&truth, &lower, &upper).expect("cov") - 0.75).abs() < 1e-12);
        let (lo, hi) = wilson_coverage_interval(&truth, &lower, &upper, 1.96).expect("wilson");
        assert!(lo <= 0.75);
        assert!(0.75 <= hi);
        assert_eq!(
            interval_coverage(&[], &[], &[]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[2.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[0.0, 1.0], &[2.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[1.0], &[0.0], &[2.0, 3.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[f64::NAN], &[0.0], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[0.5], &[f64::NAN], &[1.0]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            interval_coverage(&[0.5], &[0.0], &[f64::INFINITY]),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 0.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, -1.0),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::NAN),
            Err(ValidationError::InvalidConfiguration)
        );
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::MAX),
            Err(ValidationError::InvalidConfiguration)
        );
        // Finite z whose scaled Wilson terms still overflow.
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 1e200),
            Err(ValidationError::InvalidConfiguration)
        );
        // uncovered: above interval and below interval
        let miss_high = interval_coverage(&[0.0], &[-2.0], &[-1.0]).expect("miss high");
        assert!((miss_high - 0.0).abs() < 1e-12);
        let miss_low = interval_coverage(&[0.0], &[1.0], &[2.0]).expect("miss low");
        assert!((miss_low - 0.0).abs() < 1e-12);
    }

    #[test]
    fn wilson_nonfinite_guards() {
        let truth = [0.0];
        let lower = [-1.0];
        let upper = [1.0];
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, f64::MAX),
            Err(ValidationError::InvalidConfiguration)
        );
        // Finite z whose scaled Wilson terms still overflow.
        assert_eq!(
            wilson_coverage_interval(&truth, &lower, &upper, 1e200),
            Err(ValidationError::InvalidConfiguration)
        );
    }

    #[test]
    fn windowed_coverage_collapses_within_dgp_before_monte_carlo() {
        let summary = summarize_windowed_coverage_replications(
            &[vec![1.0, 0.5], vec![0.5, 0.0]],
            0.025,
            0.975,
        )
        .expect("DGP-clustered coverage summary");

        assert_eq!(summary.replication_count, 2);
        assert!((summary.mean - 0.5).abs() < 1.0e-12);
        assert!((summary.percentile_lower - 0.25).abs() < 1.0e-12);
        assert!((summary.percentile_upper - 0.75).abs() < 1.0e-12);
    }

    #[test]
    fn windowed_coverage_rejects_nonindependent_or_invalid_replication_geometry() {
        assert_eq!(
            summarize_windowed_coverage_replications(&[vec![0.95]], 0.025, 0.975),
            Err(ValidationError::InvalidInput)
        );
        assert_eq!(
            summarize_windowed_coverage_replications(&[vec![0.9], vec![]], 0.025, 0.975),
            Err(ValidationError::InvalidInput)
        );
        for invalid in [f64::NAN, -0.1, 1.1] {
            assert_eq!(
                summarize_windowed_coverage_replications(
                    &[vec![0.9], vec![invalid]],
                    0.025,
                    0.975,
                ),
                Err(ValidationError::InvalidInput)
            );
        }
        assert_eq!(
            summarize_windowed_coverage_replications(
                &[vec![0.9], vec![0.8]],
                0.9,
                0.1,
            ),
            Err(ValidationError::InvalidConfiguration)
        );
    }
}
