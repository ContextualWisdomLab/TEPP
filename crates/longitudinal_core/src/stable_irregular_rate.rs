//! Stable irregular residual log-rate API.
//!
//! Numerical ownership lives in [`crate::irregular_residual`], alongside event
//! ordering, CWC admission, and the shared same-sign log-rate primitive. The
//! typed weighting/denominator contract is exported directly from the crate root
//! and is not re-exported through this private compatibility facade.

/// Legacy scalar irregular-rate entry points retained for compatibility.
pub use crate::irregular_residual::{
    recover_centered_irregular_residual_log_rate, recover_within_unit_irregular_residual_log_rate,
};
