#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Posterior-aware psychometric input gates for ESEM/DSEM.
//!
//! Raw topic proportions are not unconstrained structural indicators. This
//! crate classifies constructs, admits mapped log-ratio/logistic-normal inputs,
//! distinguishes ALR from orthonormal ILR geometry, averages loading point
//! estimates across posterior draws on a CPU `f64` path without claiming Rubin
//! uncertainty pooling, combines draw-level OLS loadings with Rubin `T`,
//! decomposes cluster-mean within/between OLS, maps event-time discrete lags
//! through the exact scalar exponential, and refuses latent-mean comparison
//! below strong invariance.

mod causality;
mod cluster_mean;
mod construct;
mod error;
mod event_time;
mod indicator;
mod latent_mean;
mod loading;
mod plausible;
mod rubin_total;

/// A heuristic that is not causal identification.
pub use causality::CausalHeuristic;
/// Refuse a causal-effect claim from a non-identifying heuristic.
pub use causality::claim_causal_effect;
/// One clustered predictor–outcome pair.
pub use cluster_mean::ClusteredScore;
/// Recovered within-cluster and between-cluster OLS slopes.
pub use cluster_mean::WithinBetweenSlopes;
/// Kish effective sample size on psychometric weights.
pub use cluster_mean::kish_effective_sample_size;
/// Cluster-mean within/between OLS after CWC.
pub use cluster_mean::recover_cluster_mean_within_between_slopes;
/// Kish-weighted least-squares slope.
pub use cluster_mean::recover_kish_weighted_slope;
/// Higher-order construct class.
pub use construct::ConstructClass;
/// Permit latent-mean comparison only with invariance evidence.
pub use construct::compare_latent_means;
/// Refuse fit-driven reinterpretation as reflective.
pub use construct::interpret_as_reflective;
/// Fail-closed psychometric errors.
pub use error::PsychometricError;
/// One clustered event-time score.
pub use event_time::ClusteredEventScore;
/// Discrete lag-1 coefficient and local log-rate.
pub use event_time::DiscreteLagAndLogRate;
/// One event-time occasion.
pub use event_time::EventOccasion;
/// Clock on which a structural lag may be computed.
pub use event_time::LagClock;
/// Noiseless scalar discrete lag `later / earlier`.
pub use event_time::recover_discrete_lag_one;
/// Mean local log-rate on a sorted event-time series.
pub use event_time::recover_event_series_mean_log_rate;
/// Exact scalar pair `(φ, a)` on event time.
pub use event_time::recover_event_time_discrete_lag_and_log_rate;
/// Exact scalar inverse `a = ln(φ) / Δt`.
pub use event_time::recover_local_log_rate;
/// CWC-then-event-time local log-rate (not DSEM).
pub use event_time::recover_within_residual_event_time_log_rate;
/// Refuse the difference quotient as a continuous-time rate.
pub use event_time::refuse_difference_quotient_as_local_rate;
/// Indicator coordinate kind.
pub use indicator::IndicatorKind;
/// Pearson correlation on valid coordinates.
pub use indicator::pearson_correlation;
/// Refuse raw topic proportions as psychometric indicators.
pub use indicator::require_valid_indicator;
/// One group's factor-score and indicator series.
pub use latent_mean::GroupIndicatorSeries;
/// Two-group OLS invariance status for a mean comparison.
pub use latent_mean::MeanInvarianceStatus;
/// Two-group OLS measurement parameters and status.
pub use latent_mean::TwoGroupMeasurement;
/// Classify two-group OLS invariance.
pub use latent_mean::classify_two_group_ols_invariance;
/// Strong/strict-gated latent-mean difference.
pub use latent_mean::recover_strong_gated_latent_mean_difference;
/// Ordinary least-squares intercept, slope, and residual variance.
pub use loading::OrdinaryLeastSquaresFit;
/// Ordinary least-squares intercept and slope with residual variance.
pub use loading::ordinary_least_squares_fit;
/// Ordinary least-squares slope.
pub use loading::ordinary_least_squares_slope;
/// Recover one reflective loading.
pub use loading::recover_reflective_loading;
/// Arithmetic mean of posterior-draw point estimates.
pub use plausible::posterior_draw_point_estimate_mean;
/// Average OLS loading point estimates across posterior indicator draws.
pub use plausible::recover_loading_point_estimate_mean;
/// Rubin-combined OLS loading and total variance.
pub use rubin_total::RubinCombinedLoading;
/// Combine OLS loadings across draws with Rubin `T`.
pub use rubin_total::combine_draw_level_ols_loadings;
