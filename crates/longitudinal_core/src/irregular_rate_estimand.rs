//! Versioned weighting policy for CWC irregular-residual log-rate summaries.
//!
//! Longitudinal units can contribute different numbers of consecutive event-time
//! pairs. That multiplicity is part of the estimand, not a numerical detail.
//! This module makes the currently supported lag-pair-average target explicit
//! and reports its unit/pair denominators. Equal-unit aggregation stays typed
//! but fail-closed until the reusable correctly-rounded finite-mean contract is
//! available from its canonical numerical owner.

use std::collections::BTreeMap;

use crate::irregular_residual::{
    EventTimedObservation, center_within_unit_event_lags, driver_same_sign_log_rate,
    same_sign_nonzero, scaled_compensated_mean,
};
use crate::LongitudinalError;

/// Versioned scientific weighting target for irregular residual log-rate summaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrregularRateEstimand {
    /// Every admitted consecutive lag pair receives equal weight.
    LagPairAverageV1,
    /// Each unit first receives one within-unit summary and then equal unit weight.
    ///
    /// This target is reserved but not yet numerically activated. Requesting it
    /// fails closed until TEPP can consume the released reusable finite-mean
    /// contract instead of adding another local generic mean implementation.
    UnitAverageV1,
}

impl IrregularRateEstimand {
    /// Stable external name of this estimand contract.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::LagPairAverageV1 => "tepp.irregular_rate.lag_pair_average.v1",
            Self::UnitAverageV1 => "tepp.irregular_rate.unit_average.v1",
        }
    }
}

/// Denominator-bearing evidence for one irregular residual log-rate summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IrregularRateSummary {
    estimand: IrregularRateEstimand,
    estimate: Option<f64>,
    candidate_units: usize,
    contributing_units: usize,
    candidate_pairs: usize,
    admitted_pairs: usize,
    sign_or_zero_refused_pairs: usize,
    nonrepresentable_rate_refused_pairs: usize,
}

impl IrregularRateSummary {
    /// Return the versioned weighting target used by this summary.
    #[must_use]
    pub const fn estimand(self) -> IrregularRateEstimand {
        self.estimand
    }

    /// Return the represented estimate, or `None` when no candidate pair was admissible.
    #[must_use]
    pub const fn estimate(self) -> Option<f64> {
        self.estimate
    }

    /// Return the number of units with at least two admitted event-time occasions.
    #[must_use]
    pub const fn candidate_units(self) -> usize {
        self.candidate_units
    }

    /// Return the number of candidate units contributing at least one admitted rate.
    #[must_use]
    pub const fn contributing_units(self) -> usize {
        self.contributing_units
    }

    /// Return the number of consecutive event-time pairs considered.
    #[must_use]
    pub const fn candidate_pairs(self) -> usize {
        self.candidate_pairs
    }

    /// Return the number of pairs admitted to the reported numerical estimate.
    #[must_use]
    pub const fn admitted_pairs(self) -> usize {
        self.admitted_pairs
    }

    /// Return pairs refused because a residual was zero or the pair changed sign.
    #[must_use]
    pub const fn sign_or_zero_refused_pairs(self) -> usize {
        self.sign_or_zero_refused_pairs
    }

    /// Return same-sign nonzero pairs whose represented log-rate was not admissible.
    #[must_use]
    pub const fn nonrepresentable_rate_refused_pairs(self) -> usize {
        self.nonrepresentable_rate_refused_pairs
    }

    /// Return the total number of candidate pairs refused by log-rate admission.
    #[must_use]
    pub const fn refused_pairs(self) -> usize {
        self.sign_or_zero_refused_pairs + self.nonrepresentable_rate_refused_pairs
    }
}

/// Recover an explicitly named irregular residual log-rate estimand with denominators.
///
/// `LagPairAverageV1` preserves the existing TEPP behavior: every admissible
/// consecutive pair receives equal weight. The returned evidence makes the
/// weighting population observable by carrying candidate/contributing units and
/// candidate/admitted/refused pairs. Zero or opposite-sign pairs and represented
/// same-sign pairs whose log-rate cannot be admitted have separate refusal
/// denominators instead of silently changing the weighting population.
///
/// `UnitAverageV1` is intentionally fail-closed. Equal-unit aggregation needs a
/// second finite-mean operation over within-unit summaries. Reusable finite mean
/// arithmetic is not owned by TEPP, so activation waits for an immutable released
/// owner contract and parity evidence.
///
/// # Errors
///
/// Propagates event-time/CWC admission failures. `UnitAverageV1` returns
/// [`LongitudinalError::InvalidTemporalTransformInput`] while the released
/// reusable mean contract is unavailable. The pair-average numerical mean can
/// return the same error if its currently shared compatibility arithmetic cannot
/// represent the final admitted-pair mean.
pub fn recover_within_unit_irregular_rate_summary(
    rows: &[EventTimedObservation],
    estimand: IrregularRateEstimand,
) -> Result<IrregularRateSummary, LongitudinalError> {
    if estimand == IrregularRateEstimand::UnitAverageV1 {
        return Err(LongitudinalError::InvalidTemporalTransformInput);
    }

    let lagged = center_within_unit_event_lags(rows)?;
    let pair_counts = consecutive_pair_counts(rows);
    let candidate_units = pair_counts.len();
    let candidate_pairs = lagged.len();
    let mut admitted_rates = Vec::with_capacity(candidate_pairs);
    let mut admitted_pairs = 0_usize;
    let mut sign_or_zero_refused_pairs = 0_usize;
    let mut nonrepresentable_rate_refused_pairs = 0_usize;
    let mut contributing_units = 0_usize;
    let mut offset = 0_usize;

    for pair_count in pair_counts.values().copied() {
        let end = offset + pair_count;
        let unit_pairs = &lagged[offset..end];
        let mut unit_contributed = false;
        for pair in unit_pairs {
            if !same_sign_nonzero(pair.earlier_residual(), pair.later_residual()) {
                sign_or_zero_refused_pairs += 1;
                continue;
            }
            match driver_same_sign_log_rate(
                pair.earlier_residual(),
                pair.later_residual(),
                pair.event_interval(),
            ) {
                Ok(rate) => {
                    admitted_rates.push(rate);
                    admitted_pairs += 1;
                    unit_contributed = true;
                }
                Err(LongitudinalError::InvalidTemporalTransformInput) => {
                    nonrepresentable_rate_refused_pairs += 1;
                }
                Err(error) => return Err(error),
            }
        }
        if unit_contributed {
            contributing_units += 1;
        }
        offset = end;
    }

    let refused_pairs = sign_or_zero_refused_pairs + nonrepresentable_rate_refused_pairs;
    debug_assert_eq!(offset, candidate_pairs);
    debug_assert_eq!(admitted_pairs + refused_pairs, candidate_pairs);

    let estimate = if admitted_rates.is_empty() {
        None
    } else {
        Some(scaled_compensated_mean(&admitted_rates)?)
    };

    Ok(IrregularRateSummary {
        estimand,
        estimate,
        candidate_units,
        contributing_units,
        candidate_pairs,
        admitted_pairs,
        sign_or_zero_refused_pairs,
        nonrepresentable_rate_refused_pairs,
    })
}

fn consecutive_pair_counts(rows: &[EventTimedObservation]) -> BTreeMap<u32, usize> {
    let mut occasion_counts = BTreeMap::<u32, usize>::new();
    for row in rows {
        *occasion_counts.entry(row.unit_index()).or_default() += 1;
    }
    occasion_counts
        .into_iter()
        .filter_map(|(unit, count)| (count >= 2).then_some((unit, count - 1)))
        .collect()
}
