#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Within/between decomposition for longitudinal scores.
//!
//! Stable between-unit components cannot be scored as within-unit change.
//! Recovery reports computed component RMSE against known truth (ADR 0005).

mod component;
mod decompose;
mod error;
mod level;

/// One unit-specific within or between component.
pub use component::ComponentValue;
/// RMSE of recovered components against known truth.
pub use component::component_root_mean_square_error;
/// One occasion score for one unit.
pub use decompose::OccasionObservation;
/// Decompose occasion scores into unit means and within residuals.
pub use decompose::decompose_within_between;
/// Fail-closed longitudinal-decomposition errors.
pub use error::LongitudinalError;
/// Established longitudinal component level.
pub use level::ComponentLevel;
/// Refuse to treat a between-unit component as within-unit change.
pub use level::refuse_between_as_within_change;
