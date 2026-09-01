#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Longitudinal modeling primitives for temporal psychometrics.
//!
//! Stable between-unit components cannot be scored as within-unit change.
//! Event-time lagged associations require both marginal variances before a
//! covariance can be standardized as a correlation. Event-interval response
//! transforms live here rather than in a generic psychometric kernel. Recovery
//! reports computed component RMSE against known truth (ADR 0005).

mod association;
mod component;
mod decompose;
mod discrete_drift;
mod error;
mod event_time;
mod level;
mod temporal_association;

/// One unit-specific within or between component.
pub use component::ComponentValue;
/// RMSE of recovered components against known truth.
pub use component::component_root_mean_square_error;
/// One occasion score for one unit.
pub use decompose::OccasionObservation;
/// Decompose occasion scores into unit means and within residuals.
pub use decompose::decompose_within_between;
/// Recover scalar Driver p.16 `discreteDRIFTstd` on substantive event time.
pub use discrete_drift::recover_event_time_standardised_discrete_drift;
/// Refuse trait-plus-state association as `discreteDRIFTstd`.
pub use discrete_drift::refuse_trait_plus_state_association_as_standardised_discrete_drift;
/// Refuse trait variance as the drift standardisation variance.
pub use discrete_drift::refuse_trait_variance_as_standardisation_variance;
/// Refuse unstandardised `discreteDRIFT` as `discreteDRIFTstd`.
pub use discrete_drift::refuse_unstandardised_discrete_drift_as_standardised_discrete_drift;
/// Fail-closed longitudinal-modeling errors.
pub use error::LongitudinalError;
/// A finite, strictly positive interval admitted on substantive event time.
pub use event_time::EventTimeInterval;
/// Established longitudinal component level.
pub use level::ComponentLevel;
/// Refuse to treat a between-unit component as within-unit change.
pub use level::refuse_between_as_within_change;
/// Recover a valid event-time lagged correlation from covariance and both
/// marginal variances through the typed event-time boundary.
pub use temporal_association::recover_event_time_lagged_correlation;
