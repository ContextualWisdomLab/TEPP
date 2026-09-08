//! Stable irregular residual log-rate API.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. This
//! module is only the crate-public facade and must not carry a second
//! floating-point implementation.

/// Crate-public irregular residual log-rate entry points.
///
/// Re-exports [`crate::irregular_residual::recover_centered_irregular_residual_log_rate`]
/// and [`crate::irregular_residual::recover_within_unit_irregular_residual_log_rate`].
pub use crate::irregular_residual::{
    recover_centered_irregular_residual_log_rate, recover_within_unit_irregular_residual_log_rate,
};
