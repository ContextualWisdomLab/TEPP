//! Exact replication-identity binding for sharded coverage-calibration evidence.

use crate::{
    MonteCarloRecoveryMetricSummary, ValidationError,
    summarize_windowed_coverage_recovery_replications,
};

/// One declared coverage-calibration replication and its numerical outcome.
///
/// A missing coverage vector represents only an owner-admitted numerical
/// failure. Structural experiment invalidity must abort before constructing an
/// outcome so it cannot be hidden inside the attempted scientific denominator.
#[derive(Clone, Debug, PartialEq)]
pub struct CoverageCalibrationReplicationOutcome {
    replication_index: usize,
    window_coverages: Option<Vec<f64>>,
}

impl CoverageCalibrationReplicationOutcome {
    /// Construct a successful replication with its rolling-origin window coverage.
    #[must_use]
    pub const fn successful(replication_index: usize, window_coverages: Vec<f64>) -> Self {
        Self {
            replication_index,
            window_coverages: Some(window_coverages),
        }
    }

    /// Construct one owner-admitted numerical failure.
    #[must_use]
    pub const fn numerical_failure(replication_index: usize) -> Self {
        Self {
            replication_index,
            window_coverages: None,
        }
    }

    /// Zero-based identity in the prospectively declared replication schedule.
    #[must_use]
    pub const fn replication_index(&self) -> usize {
        self.replication_index
    }

    /// Successful rolling-origin coverage values, or `None` for numerical failure.
    #[must_use]
    pub fn window_coverages(&self) -> Option<&[f64]> {
        self.window_coverages.as_deref()
    }
}

/// Aggregate coverage only from an exact permutation of declared replication identities.
///
/// Input order is deliberately irrelevant. Outcomes are sorted by their declared
/// replication index before successful coverage values reach the existing
/// denominator-preserving Monte Carlo owner, so shard completion order cannot
/// perturb binary64 accumulation order. The identity set must be exactly
/// `0..attempted_replication_count`; duplicates, omissions, and out-of-range
/// identities fail closed.
///
/// Window validation, within-DGP collapse, unconditional failure arithmetic,
/// Monte Carlo standard errors, and empirical percentiles remain owned by
/// [`summarize_windowed_coverage_recovery_replications`].
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when no replications were declared,
/// the number or identity set of outcomes differs from the declared experiment,
/// or successful window coverage is structurally invalid. Returns
/// [`ValidationError::InvalidConfiguration`] for invalid percentile bounds.
pub fn summarize_indexed_windowed_coverage_recovery_replications(
    attempted_replication_count: usize,
    outcomes: &[CoverageCalibrationReplicationOutcome],
    lower_percentile: f64,
    upper_percentile: f64,
) -> Result<MonteCarloRecoveryMetricSummary, ValidationError> {
    if attempted_replication_count == 0 || outcomes.len() != attempted_replication_count {
        return Err(ValidationError::InvalidInput);
    }

    let mut ordered: Vec<_> = outcomes.iter().collect();
    ordered.sort_unstable_by_key(|outcome| outcome.replication_index);
    if ordered
        .iter()
        .enumerate()
        .any(|(expected, outcome)| outcome.replication_index != expected)
    {
        return Err(ValidationError::InvalidInput);
    }

    let successful_window_coverages: Vec<_> = ordered
        .into_iter()
        .filter_map(|outcome| outcome.window_coverages.clone())
        .collect();
    summarize_windowed_coverage_recovery_replications(
        attempted_replication_count,
        &successful_window_coverages,
        lower_percentile,
        upper_percentile,
    )
}
