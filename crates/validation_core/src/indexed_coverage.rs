//! Exact replication-identity binding for sharded coverage-calibration evidence.

use sha2::{Digest, Sha256};

use crate::{
    MonteCarloRecoveryMetricSummary, ValidationError,
    summarize_windowed_coverage_recovery_replications,
};

const INDEXED_COVERAGE_OUTCOME_FINGERPRINT_DOMAIN: &[u8] =
    b"tepp.validation.indexed-coverage-outcomes.v1\0";
const HEX: &[u8; 16] = b"0123456789abcdef";

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

/// Compute a deterministic SHA-256 fingerprint of the exact indexed outcome ledger.
///
/// The digest is independent of shard/completion input order because outcomes are
/// first validated as an exact permutation of `0..attempted_replication_count`
/// and then hashed in declared replication order. The canonical byte stream is
/// domain-separated and includes the attempted count, each zero-based identity,
/// success/failure state, successful window count, and every coverage value's
/// exact IEEE-754 binary64 bits. This function binds provenance only; scientific
/// validation of window coverage values remains with the aggregation owner.
///
/// # Errors
///
/// Returns [`ValidationError::InvalidInput`] when no replications were declared,
/// the outcome count differs from the declaration, the identity set is not an
/// exact permutation, or a platform-sized count cannot be represented as `u64`.
pub fn indexed_coverage_outcome_fingerprint(
    attempted_replication_count: usize,
    outcomes: &[CoverageCalibrationReplicationOutcome],
) -> Result<String, ValidationError> {
    let ordered = ordered_outcomes(attempted_replication_count, outcomes)?;
    let attempted = u64::try_from(attempted_replication_count)
        .map_err(|_| ValidationError::InvalidInput)?;

    let mut hasher = Sha256::new();
    hasher.update(INDEXED_COVERAGE_OUTCOME_FINGERPRINT_DOMAIN);
    hasher.update(attempted.to_be_bytes());

    for outcome in ordered {
        let replication_index = u64::try_from(outcome.replication_index)
            .map_err(|_| ValidationError::InvalidInput)?;
        hasher.update(replication_index.to_be_bytes());
        match &outcome.window_coverages {
            Some(window_coverages) => {
                hasher.update([1]);
                let window_count = u64::try_from(window_coverages.len())
                    .map_err(|_| ValidationError::InvalidInput)?;
                hasher.update(window_count.to_be_bytes());
                for coverage in window_coverages {
                    hasher.update(coverage.to_bits().to_be_bytes());
                }
            }
            None => hasher.update([0]),
        }
    }

    Ok(lower_hex(hasher.finalize().as_slice()))
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
    let ordered = ordered_outcomes(attempted_replication_count, outcomes)?;
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

fn ordered_outcomes<'a>(
    attempted_replication_count: usize,
    outcomes: &'a [CoverageCalibrationReplicationOutcome],
) -> Result<Vec<&'a CoverageCalibrationReplicationOutcome>, ValidationError> {
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
    Ok(ordered)
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
