#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Explicit invariance status and group-loading recovery for shared meaning.
//!
//! Configural structure alone cannot license shared metric meaning. Metric and
//! scalar status may; recovery reports computed loading RMSE against known
//! truth (ADR 0004/0005).

mod error;
mod loading;
mod status;

/// Fail-closed measurement-invariance errors.
pub use error::InvarianceError;
/// One group-specific loading.
pub use loading::GroupLoading;
/// RMSE of recovered loadings against known truth.
pub use loading::loading_root_mean_square_error;
/// Established invariance status.
pub use status::InvarianceLevel;
/// Refuse to treat a weaker status as shared meaning.
pub use status::refuse_noninvariant_as_shared_meaning;
