//! Public facade for the canonical irregular residual log-rate composition.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. This
//! module preserves the crate-public API introduced during the overflow repair
//! without maintaining a second floating-point implementation that can drift
//! from the Longitudinal Modeling estimand.

use crate::LongitudinalError;
use crate::irregular_residual::{self, EventTimedObservation, LaggedWithinResidual};

/// Pairwise-mean exact log-rate after CWC on irregular event intervals.
///
/// This is a public facade over the canonical Longitudinal Modeling
/// implementation. Temporal admission, same-sign log-domain fallback, and
/// overflow-safe count weighting are therefore identical to the internal
/// composition used by the bounded context.
///
/// # Errors
///
/// Propagates the canonical temporal-admission and numerical errors from
/// [`irregular_residual::recover_within_unit_irregular_residual_log_rate`].
pub fn recover_within_unit_irregular_residual_log_rate(
    rows: &[EventTimedObservation],
) -> Result<f64, LongitudinalError> {
    irregular_residual::recover_within_unit_irregular_residual_log_rate(rows)
}

/// Mean exact scalar log-rate on already-centered residuals.
///
/// This facade delegates to the single canonical implementation. Finite,
/// nonzero same-sign residuals use the direct positive ratio logarithm when it
/// is representable and the equivalent log-domain difference otherwise; a
/// representable final mean is not rejected only because an intermediate
/// ratio or retained-count product overflows or underflows.
///
/// # Errors
///
/// Propagates payload, sign/zero, event-interval, and representability errors
/// from [`irregular_residual::recover_centered_irregular_residual_log_rate`].
pub fn recover_centered_irregular_residual_log_rate(
    pairs: &[LaggedWithinResidual],
) -> Result<f64, LongitudinalError> {
    irregular_residual::recover_centered_irregular_residual_log_rate(pairs)
}
