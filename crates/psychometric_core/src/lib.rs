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
//! decomposes cluster-mean within/between OLS and the CWC contextual effect,
//! maps event-time discrete lags through the exact scalar exponential, maps
//! already-centered irregular residuals without re-centering, remaps discrete
//! lags across unequal event intervals through that log-rate, recovers the
//! exact scalar discrete effect of a constant predictor, recovers the
//! first-order discrete effect of a time-varying predictor with matched
//! sampling and constancy intervals, recovers the exact scalar discrete
//! process noise of Driver et al. (2017, Eq. 3), recovers the lagged
//! latent covariance and unconditional latent variance licensed by
//! their Eq. 3–4, recovers the scalar stationary within-subject
//! variance (Driver et al., 2017, Eq. 4 as `Δt → ∞`; §4.3; p. 16
//! `asymDIFFUSION`),
//! and refuses
//! latent-mean comparison below strong invariance.

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
/// Recovered within-cluster, between-cluster, and contextual OLS slopes.
pub use cluster_mean::WithinBetweenSlopes;
/// Kish effective sample size on psychometric weights.
pub use cluster_mean::kish_effective_sample_size;
/// Cluster-mean within/between OLS after CWC, plus the contextual effect.
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
/// Already-centered lagged residual pair with an irregular event interval.
pub use event_time::LaggedWithinResidual;
/// Map a discrete lag onto another event interval through the exact log-rate.
pub use event_time::map_discrete_lag_across_event_intervals;
/// Exact scalar discrete effect of a constant event-time predictor.
pub use event_time::recover_discrete_constant_predictor_effect;
/// Exact scalar forward map `φ = exp(a Δt)`.
pub use event_time::recover_discrete_lag_from_log_rate;
/// Noiseless scalar discrete lag `later / earlier`.
pub use event_time::recover_discrete_lag_one;
/// Exact scalar lagged latent covariance `A_Δt cov(η_{t-1})`.
pub use event_time::recover_discrete_lagged_latent_covariance;
/// Exact scalar discrete latent variance `A_Δt P A_Δt⊤ + Q_Δt`.
pub use event_time::recover_discrete_latent_variance;
/// Exact scalar discrete process noise `Q_Δt` on event time.
pub use event_time::recover_discrete_process_noise;
/// First-order discrete effect of a time-varying event-time predictor.
pub use event_time::recover_discrete_time_varying_predictor_effect;
/// Mean local log-rate on a sorted event-time series.
pub use event_time::recover_event_series_mean_log_rate;
/// Exact scalar pair `(φ, a)` on event time.
pub use event_time::recover_event_time_discrete_lag_and_log_rate;
/// Mean exact log-rate on already-centered irregular residuals.
pub use event_time::recover_irregular_centered_residual_log_rate;
/// Exact scalar inverse `a = ln(φ) / Δt`.
pub use event_time::recover_local_log_rate;
/// Exact scalar stationary within-subject variance `-q / (2 a)`.
pub use event_time::recover_stationary_latent_variance;
/// CWC-then-event-time local log-rate (not DSEM; not raw-process AR drift).
pub use event_time::recover_within_residual_event_time_log_rate;
/// Refuse the difference quotient as a continuous-time rate.
pub use event_time::refuse_difference_quotient_as_local_rate;
/// Refuse treating finite-interval `Q_Δt` as `asymDIFFUSION`.
pub use event_time::refuse_finite_interval_process_noise_as_stationary_variance;
/// Refuse pooling discrete lags from unequal event intervals.
pub use event_time::refuse_pooled_discrete_lag_across_unequal_intervals;
/// Refuse treating Driver Eq. 3 process noise as the unconditional variance.
pub use event_time::refuse_process_noise_as_unconditional_variance;
/// Refuse a time-varying predictor whose sampling and constancy intervals differ.
pub use event_time::refuse_unmatched_time_varying_predictor_interval;
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
