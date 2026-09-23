//! Interval coverage for recovered confidence/credible intervals.

use crate::ValidationError;
use crate::monte_carlo::{
    MonteCarloRecoveryMetricSummary, MonteCarloSummary, summarize_recovery_metric_replications,
    summarize_replications,
};

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

fn collapse_window_coverages(
    replication_window_coverages: &[Vec<f64>],
) -> Result<Vec<f64>, ValidationError> {
    if replication_window_coverages.iter().any(|windows| {
        windows.is_empty()
            || windows
                .iter()
                .any(|coverage| !coverage.is_finite() || !(0.0..=1.0).contains(coverage))
    }) {
        return Err(ValidationError::InvalidInput);
    }

    Ok(replication_window_coverages
        .iter()
        .map(|windows| windows.iter().sum::<f64>() / windows.len() as f64)
        .collect())
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
    if replication_window_coverages.len() < 2 {
        return Err(ValidationError::InvalidInput);
    }
    let per_replication_coverage = collapse_window_coverages(replication_window_coverages)?;
    summarize_replications(
        &per_replication_coverage,
        lower_percentile,
        upper_percentile,
    )
}

/// Collapse successful rolling-origin coverage within DGP replications while retaining failures.
///
/// `attempted_replication_count` is the unconditional number of scientifically
/// admissible DGP replications attempted. `successful_replication_window_coverages`
/// contains window-level coverage only for attempts where that metric was
/// numerically available. Each successful DGP is collapsed to one equal-window
/// mean before the existing recovery-metric owner computes between-DGP Monte
/// Carlo uncertainty and the unconditional failure denominator.
///
/// Structural experiment invalidity must be rejected by the owning workflow
/// before calling this function. Missing successful coverage here represents only
/// an owner-admitted numerical failure; it must not be used to launder malformed
/// split, identity, covariance, or interval geometry into the failure denominator.
///
/// All-failed and one-success experiments remain reportable. One successful DGP
/// retains its conditional coverage mean but does not fabricate a between-DGP
/// standard deviation or Monte Carlo standard error.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when a successful DGP has no windows,
/// any window coverage is non-finite or outside `[0, 1]`, the attempted count is
/// zero, or successful replications outnumber attempts. Percentile configuration
/// errors are propagated from the recovery-metric Monte Carlo owner.
pub fn summarize_windowed_coverage_recovery_replications(
    attempted_replication_count: usize,
    successful_replication_window_coverages: &[Vec<f64>],
    lower_percentile: f64,
    upper_percentile: f64,
) -> Result<MonteCarloRecoveryMetricSummary, ValidationError> {
    let successful_dgp_coverages =
        collapse_window_coverages(successful_replication_window_coverages)?;
    summarize_recovery_metric_replications(
        attempted_replication_count,
        &successful_dgp_coverages,
        lower_percentile,
        upper_percentile,
    )
}

/// Prospective interval-calibration design for one versioned scientific acceptance run.
///
/// This value records the design before the expensive DGP experiment is run. It
/// intentionally separates a practical psychometric coverage band from Monte
/// Carlo precision and from numerical-failure reporting. The design does not
/// define an acceptable failure-rate threshold and therefore cannot by itself
/// promote a full scientific or release claim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoverageCalibrationDesign {
    design_id: &'static str,
    attempted_dgp_count: usize,
    nominal_coverage: f64,
    practical_lower_coverage: f64,
    practical_upper_coverage: f64,
    maximum_monte_carlo_standard_error: f64,
}

impl CoverageCalibrationDesign {
    /// TEPP's prospectively declared nominal-95% coverage design, version 1.
    ///
    /// The practical `[0.91, 0.98]` band follows the Muthén and Muthén (2002)
    /// psychometric simulation convention. Ten thousand independent DGP
    /// attempts are predeclared so a complete successful sample of bounded
    /// `[0,1]` DGP-level coverage values has worst-case Monte Carlo standard
    /// error at most `0.005`. The actual owner-reported standard error remains
    /// authoritative when numerical failures reduce the successful sample.
    #[must_use]
    pub const fn tepp_nominal_95_v1() -> Self {
        Self {
            design_id: "tepp.coverage.nominal95.v1",
            attempted_dgp_count: 10_000,
            nominal_coverage: 0.95,
            practical_lower_coverage: 0.91,
            practical_upper_coverage: 0.98,
            maximum_monte_carlo_standard_error: 0.005,
        }
    }

    /// Stable versioned identity for the prospective design.
    #[must_use]
    pub const fn design_id(self) -> &'static str {
        self.design_id
    }

    /// Number of independent DGP replications declared before execution.
    #[must_use]
    pub const fn attempted_dgp_count(self) -> usize {
        self.attempted_dgp_count
    }

    /// Nominal interval coverage targeted by the estimator.
    #[must_use]
    pub const fn nominal_coverage(self) -> f64 {
        self.nominal_coverage
    }

    /// Lower practical coverage bound declared before execution.
    #[must_use]
    pub const fn practical_lower_coverage(self) -> f64 {
        self.practical_lower_coverage
    }

    /// Upper practical coverage bound declared before execution.
    #[must_use]
    pub const fn practical_upper_coverage(self) -> f64 {
        self.practical_upper_coverage
    }

    /// Maximum accepted Monte Carlo standard error of the successful DGP coverage mean.
    #[must_use]
    pub const fn maximum_monte_carlo_standard_error(self) -> f64 {
        self.maximum_monte_carlo_standard_error
    }
}

/// Assessment of one denominator-preserving coverage summary against a prospective design.
///
/// A passing conditional calibration assessment is not a convergence, robustness,
/// scientific-promotion, or release decision. Numerical failures remain visible
/// through the attempted/success/failure fields and require their own owner policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoverageCalibrationAssessment {
    attempted_replication_count: usize,
    successful_replication_count: usize,
    failure_count: usize,
    failure_rate: f64,
    coverage_mean: Option<f64>,
    coverage_monte_carlo_standard_error: Option<f64>,
    coverage_within_practical_band: bool,
    monte_carlo_precision_sufficient: bool,
}

impl CoverageCalibrationAssessment {
    /// Unconditional attempted DGP count.
    #[must_use]
    pub const fn attempted_replication_count(self) -> usize {
        self.attempted_replication_count
    }

    /// DGP count with numerically available coverage.
    #[must_use]
    pub const fn successful_replication_count(self) -> usize {
        self.successful_replication_count
    }

    /// Owner-admitted numerical failures among attempted DGP replications.
    #[must_use]
    pub const fn failure_count(self) -> usize {
        self.failure_count
    }

    /// Unconditional numerical failure rate.
    #[must_use]
    pub const fn failure_rate(self) -> f64 {
        self.failure_rate
    }

    /// Conditional mean DGP-level coverage, when at least one replication succeeded.
    #[must_use]
    pub const fn coverage_mean(self) -> Option<f64> {
        self.coverage_mean
    }

    /// Between-DGP Monte Carlo standard error, available only with at least two successes.
    #[must_use]
    pub const fn coverage_monte_carlo_standard_error(self) -> Option<f64> {
        self.coverage_monte_carlo_standard_error
    }

    /// Whether the conditional coverage mean lies inside the prospective practical band.
    #[must_use]
    pub const fn coverage_within_practical_band(self) -> bool {
        self.coverage_within_practical_band
    }

    /// Whether between-DGP coverage uncertainty meets the prospective precision target.
    #[must_use]
    pub const fn monte_carlo_precision_sufficient(self) -> bool {
        self.monte_carlo_precision_sufficient
    }

    /// Whether this summary supports the **conditional coverage-calibration** claim.
    ///
    /// This deliberately does not judge the numerical failure rate. A caller must
    /// not treat `true` as estimator robustness, full scientific promotion, or
    /// release authority.
    #[must_use]
    pub const fn supports_calibration_claim(self) -> bool {
        self.coverage_within_practical_band && self.monte_carlo_precision_sufficient
    }
}

/// Assess denominator-preserving coverage evidence against one prospective design.
///
/// The attempted DGP count must match the design exactly, preventing a caller
/// from shrinking or extending the experiment after seeing outcomes while still
/// claiming the same design identity. Coverage and its Monte Carlo standard
/// error are read only from the owner summary; this function does not recompute
/// window/document/coordinate arithmetic or remove failed attempts.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when the summary's attempted DGP
/// count differs from the prospective design. Structural experiment invalidity
/// must already have failed closed before a recovery summary is minted.
pub fn assess_coverage_calibration(
    design: &CoverageCalibrationDesign,
    summary: &MonteCarloRecoveryMetricSummary,
) -> Result<CoverageCalibrationAssessment, ValidationError> {
    if summary.attempted_replication_count() != design.attempted_dgp_count {
        return Err(ValidationError::InvalidInput);
    }

    let coverage_mean = summary.successful_metric_mean();
    let coverage_monte_carlo_standard_error = summary
        .successful_metric_summary()
        .map(|metric| metric.standard_error);
    let coverage_within_practical_band = coverage_mean.is_some_and(|mean| {
        (design.practical_lower_coverage..=design.practical_upper_coverage).contains(&mean)
    });
    let monte_carlo_precision_sufficient = coverage_monte_carlo_standard_error
        .is_some_and(|standard_error| {
            standard_error <= design.maximum_monte_carlo_standard_error
        });

    Ok(CoverageCalibrationAssessment {
        attempted_replication_count: summary.attempted_replication_count(),
        successful_replication_count: summary.successful_replication_count(),
        failure_count: summary.failure_count(),
        failure_rate: summary.failure_rate(),
        coverage_mean,
        coverage_monte_carlo_standard_error,
        coverage_within_practical_band,
        monte_carlo_precision_sufficient,
    })
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
