//! Public facade for the canonical irregular residual log-rate composition.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. This
//! module preserves the crate-public API while keeping evidence-row order out
//! of the binary64 scientific result.

use crate::{LaggedWithinResidual, LongitudinalError, irregular_residual};

/// Recover the mean exact scalar log-rate on already-centered residual pairs.
///
/// The estimand is invariant to the caller's evidence-row order. Pairs are
/// therefore put in a canonical numeric order before the bounded Longitudinal
/// Modeling mean is evaluated. This changes no pair value or event interval;
/// it only prevents floating-point recurrence order from becoming scientific
/// evidence.
///
/// # Errors
///
/// Propagates the canonical irregular-residual admission and recovery errors.
pub fn recover_centered_irregular_residual_log_rate(
    lagged: &[LaggedWithinResidual],
) -> Result<f64, LongitudinalError> {
    let mut canonical = lagged.to_vec();
    canonical.sort_by(|left, right| {
        left.earlier_residual()
            .total_cmp(&right.earlier_residual())
            .then_with(|| left.later_residual().total_cmp(&right.later_residual()))
            .then_with(|| {
                left.event_interval()
                    .as_f64()
                    .total_cmp(&right.event_interval().as_f64())
            })
    });
    irregular_residual::recover_centered_irregular_residual_log_rate(&canonical)
}

pub use crate::irregular_residual::recover_within_unit_irregular_residual_log_rate;
