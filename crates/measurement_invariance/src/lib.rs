#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Explicit invariance status and identified loading recovery for shared metric meaning.
//!
//! Configural structure alone cannot license shared metric meaning. Metric and
//! scalar status may; recovery reports computed loading RMSE against known
//! truth only when group × indicator × factor parameter identities match
//! (ADR 0004/0005).

mod error;
mod loading;
mod status;

/// Fail-closed measurement-invariance errors.
pub use error::InvarianceError;
/// One group-, indicator-, and factor-identified loading.
pub use loading::GroupLoading;
/// RMSE of recovered loadings against known truth.
pub use loading::loading_root_mean_square_error;
/// Established invariance status.
pub use status::InvarianceLevel;
/// Require an invariance status strong enough for shared metric meaning.
pub use status::require_shared_metric_meaning;
