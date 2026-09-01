#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Longitudinal modeling primitives for temporal psychometrics.
//!
//! Stable between-unit components cannot be scored as within-unit change.
//! Event-time lagged associations require both marginal variances before a
//! covariance can be standardized as a correlation. Recovery reports computed
//! component RMSE against known truth (ADR 0005).

mod association;
mod component;
mod decompose;
mod error;
mod level;

/// Recover a valid event-time lagged correlation from covariance and both
/// marginal variances.
pub use association::recover_event_time_lagged_correlation;
/// One unit-specific within or between component.
pub use component::ComponentValue;
/// RMSE of recovered components against known truth.
pub use component::component_root_mean_square_error;
/// One occasion score for one unit.
pub use decompose::OccasionObservation;
/// Decompose occasion scores into unit means and within residuals.
pub use decompose::decompose_within_between;
/// Fail-closed longitudinal-modeling errors.
pub use error::LongitudinalError;
/// Established longitudinal component level.
pub use level::ComponentLevel;
/// Refuse to treat a between-unit component as within-unit change.
pub use level::refuse_between_as_within_change;
