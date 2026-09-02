//! Public facade for the canonical irregular residual log-rate composition.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. This
//! module preserves the crate-public API while re-exporting the canonical
//! function identities instead of maintaining wrapper functions that can drift
//! from the Longitudinal Modeling estimand.

pub use crate::irregular_residual::{
    recover_centered_irregular_residual_log_rate,
    recover_within_unit_irregular_residual_log_rate,
};
