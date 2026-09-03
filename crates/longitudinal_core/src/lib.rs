#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Longitudinal modeling primitives for temporal psychometrics.
//!
//! Stable between-unit components cannot be scored as within-unit change.
//! Event-time lagged associations require both marginal variances before a
//! covariance can be standardized as a correlation. Event-interval response
//! transforms and scalar diffusion standardisation candidates live here rather
//! than in a generic psychometric kernel. Person-mean centering of a time-related
//! series is not raw-process drift (Curran & Bauer, 2011). Recovery reports
//! computed component RMSE against known truth (ADR 0005).

mod association;
mod component;
mod decompose;
mod diffusion;
mod discrete_drift;
mod error;
mod event_time;
mod irregular_residual;
mod level;
mod occasion_mean;
mod stable_irregular_rate;
mod stationary;
mod temporal_association;

/// One unit-specific within or between component.
pub use component::ComponentValue;
/// RMSE of recovered components against known truth.
pub use component::component_root_mean_square_error;
/// One occasion score for one unit.
pub use decompose::OccasionObservation;
/// Decompose occasion scores into unit means and within residuals.
pub use decompose::decompose_within_between;
/// Recover the scalar research-candidate continuous diffusion standardisation on event time.
pub use diffusion::recover_event_time_standardised_continuous_diffusion;
/// Recover the scalar research-candidate discrete diffusion standardisation on event time.
pub use diffusion::recover_event_time_standardised_discrete_diffusion;
/// Refuse continuous standardised diffusion as discrete standardised diffusion.
pub use diffusion::refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion;
/// Refuse total-variance scaling as relevant-variance diffusion standardisation.
pub use diffusion::refuse_total_variance_scaled_diffusion_as_standardised_diffusion;
/// Refuse unstandardised diffusion as standardised diffusion.
pub use diffusion::refuse_unstandardised_diffusion_as_standardised_diffusion;
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
/// One unit's score at one event-time occasion.
pub use irregular_residual::EventTimedObservation;
/// One already-formed lagged within residual pair on event time.
pub use irregular_residual::LaggedWithinResidual;
/// Cluster-mean-center consecutive event-time lags inside each unit.
pub use irregular_residual::center_within_unit_event_lags;
/// Refuse treating a CWC residual log-rate as raw-process AR drift.
pub use irregular_residual::refuse_cwc_residual_log_rate_as_raw_process_drift;
/// Established longitudinal component level.
pub use level::ComponentLevel;
/// Refuse to treat a between-unit component as within-unit change.
pub use level::refuse_between_as_within_change;
/// Form consecutive event-time lags after subtracting each occasion's group mean.
pub use occasion_mean::center_occasion_mean_event_lags;
/// Recover the exact scalar log-rate of occasion-mean residuals.
pub use occasion_mean::recover_occasion_mean_centered_irregular_residual_log_rate;
/// Refuse treating occasion-mean residual lag as within-person change.
pub use occasion_mean::refuse_occasion_mean_centered_log_rate_as_within_person_lag;
/// Mean exact scalar log-rate on already-centered residuals with stable count weighting.
pub use stable_irregular_rate::recover_centered_irregular_residual_log_rate;
/// Pairwise-mean exact log-rate after CWC with stable count weighting.
pub use stable_irregular_rate::recover_within_unit_irregular_residual_log_rate;
/// Recover the scalar Driver p.16 unstandardised stationary within-person variance.
pub use stationary::recover_stationary_within_variance;
/// Recover a valid event-time lagged correlation from covariance and both
/// marginal variances through the typed event-time boundary.
pub use temporal_association::recover_event_time_lagged_correlation;
