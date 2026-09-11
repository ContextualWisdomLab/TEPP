//! Stable irregular residual log-rate API.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. The
//! weighting/denominator contract lives in [`crate::irregular_rate_estimand`].
//! This module is only the crate-public facade and must not carry a second
//! floating-point implementation.

/// Versioned, denominator-bearing irregular-rate estimand entry points.
pub use crate::irregular_rate_estimand::{
    IrregularRateEstimand, IrregularRateSummary, recover_within_unit_irregular_rate_summary,
};
/// Legacy scalar irregular-rate entry points retained for compatibility.
pub use crate::irregular_residual::{
    recover_centered_irregular_residual_log_rate, recover_within_unit_irregular_residual_log_rate,
};
