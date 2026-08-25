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
//! `asymDIFFUSION`), recovers the Driver §4.3 trait-plus-state
//! variance and lagged covariance (`TRAITVAR` is not process noise
//! and not `asymDIFFUSION`), recovers the Driver Eq. 5 scalar
//! observed-indicator variance (`λ² Var(η) + θ` when
//! `MANIFESTTRAITVAR` is zero, else `λ² Var(η) + θ + ψ`; Table 2,
//! p. 12: `MANIFESTVAR` is `Θ`, not `Var(y)`; `MANIFESTTRAITVAR` is
//! not `MANIFESTVAR`; lagged observed covariance is
//! `λ² cov(η_t, η_{t-1}) + ψ` and does not include `Θ`; the
//! observed-indicator mean is `τ + λ μ` (`MANIFESTMEANS` is `τ`,
//! not `E(y)`; `CINT` is not `MANIFESTMEANS`; Equation 1
//! is the SDE), recovers the Driver Eq. 3 expected-value latent
//! mean `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ` (`T0MEANS` is not
//! `μ_t`; `CINT` is not the discrete increment), recovers the
//! Driver Eq. 5 of that evolved mean as `τ + λ μ_t` (the
//! first-occasion map `τ + λ μ_0` is not `E(y_t)`), recovers the
//! Driver Eq. 3 fourth-summand impulse `m x` (Table 2 `TDPREDEFFECT`
//! is `M`, not `CINT`, not `TIPREDEFFECT`, and not Voelkle Eq. 14),
//! recovers the Driver Eq. 5 of that contemporaneous impulse as
//! `τ + λ(μ_t + m x)` (`τ + λ μ_t` is not that observed mean;
//! `τ + λ(μ_t + e^{a(t−u)} m x)` is not that observed mean when
//! `u ≠ t`), recovers the Driver Eq. 1–2 within-interval impulse carry
//! `e^{A(t−u)} M x` for `t0 < u < t` (not the contemporaneous Dirac,
//! not `CINT`, not `TIPREDEFFECT`, and not Voelkle Eq. 14; §7.2
//! dissipation), recovers the Driver Eq. 5 of that carried latent
//! mean as `τ + λ(μ_t + e^{a(t−u)} m x)` (`τ + λ μ_t` is not that
//! observed mean), recovers the Driver Eq. 3 second-summand
//! time-independent predictor increment `A^{-1}[e^{A Δt} − I] B z`
//! (Table 2 `TIPREDEFFECT` is `B`, not `κ`, not `M`, and not Voelkle
//! Eq. 14; `B` is not that discrete increment), recovers the Driver
//! Eq. 5 of that increment as
//! `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` (`τ + λ μ_t` is not that
//! observed mean; `τ + λ(μ_t + m x)` is not that observed mean;
//! `τ + λ(μ_t + e^{a(t−u)} m x)` is not that observed mean when
//! `u ≠ t`), recovers the Driver Table 3 first-occasion
//! `T0TIPREDEFFECT` shift `t0_b z` and its Eq. 3 first-summand carry
//! `e^{A Δt} t0_b z` (`T0TIPREDEFFECT` is not `TIPREDEFFECT` `B`;
//! `t0_b z` is not `A^{-1}[e^{A Δt} − I] B z`; `e^{A Δt} t0_b z` is
//! not `t0_b z`), recovers the Driver Eq. 5 of that first-occasion
//! carry as `τ + λ(μ_t + e^{a Δt} t0_b z)` (`τ + λ μ_t` is not that
//! observed mean; `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not
//! that observed mean; `τ + λ(μ_t + m x)` is not that observed mean;
//! `τ + λ(μ_t + e^{a(t−u)} m x)` is not that observed mean when
//! `u ≠ t0`), recovers the Driver Table 3 first-occasion
//! `T0TDPREDEFFECT` shift `t0_m x0` and its Eq. 3 first-summand carry
//! `e^{A Δt} t0_m x0` (`T0TDPREDEFFECT` is not `TDPREDEFFECT` `M`;
//! `t0_m x0` is not `M x`; `e^{A Δt} t0_m x0` is not `t0_m x0`;
//! `e^{A Δt} t0_m x0` is not `e^{A(t−u)} M x` for `t0 < u < t`;
//! `t0_m x0` is not `t0_b z`; an impulse at `u ≤ t0` that used `M`
//! is already in `η(t0)` as `TDPREDEFFECT`, not as `T0TDPREDEFFECT`),
//! recovers the Driver Eq. 5 of that first-occasion TD carry as
//! `τ + λ(μ_t + e^{a Δt} t0_m x0)` (`τ + λ μ_t` is not that observed
//! mean; `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not that
//! observed mean; `τ + λ(μ_t + m x)` is not that observed mean;
//! `τ + λ(μ_t + e^{a(t−u)} m x)` is not that observed mean when
//! `u ≠ t0`; `τ + λ(μ_t + e^{a Δt} t0_b z)` is not that observed
//! mean),
//! recovers the Driver §7.2 level-change `CINT` setting `κ = −a m x`
//! (`a < 0` so `−κ / a = m x`; not the dissipating Dirac `m x`, not
//! a free `CINT`, not `A^{-1}[e^{A Δt} − I] B z`, and not the extra
//! near-zero-drift latent process also named in §7.2),
//! recovers the Driver Eq. 3 increment of that setting as
//! `(1 − e^{a Δt}) m x` (`(1 − e^{a Δt}) m x` is not `m x`, not `κ`,
//! and not `A^{-1}[e^{A Δt} − I] B z`),
//! recovers the Driver §7.2 extra near-zero-drift latent process
//! contribution `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)` (pp. 22–23;
//! identification `TDPREDEFFECT` on the extra process is 1; `ε < 0`;
//! printed extra `DRIFT` is `−0.000001`; not `κ = −a m x`, not
//! `(1 − e^{a Δt}) m x`, and not the dissipating Dirac `m x`),
//! recovers the Driver Eq. 5 of that extra-process contribution as
//! `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))` (Eq. 5,
//! p. 5; §7.2, pp. 22–23; JSS PDF re-opened 2026-08-21T06:12Z; the
//! extra process has `LAMBDA` 0 and is not an observed indicator;
//! `τ + λ μ_t` is not that observed mean; `τ + λ(μ_t + m x)` is not
//! that observed mean; the contribution is not `E(y_t)`; the
//! evolved-plus-contribution latent mean is not `E(y_t)`),
//! recovers the Driver §7.2 after-t0 extra-process `TDPREDEFFECT`
//! contribution as `a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a)`
//! for `t0 < u < t` (`T0TDPREDEFFECT` uses `Δt = t − t0` for both
//! the evolution and the extra drive; an impulse at `u = t0` is not
//! this map; an impulse at `u = t` has not yet driven the original
//! process; `e^{a(t−u)} m x` is a Dirac on the original process, not
//! this `DRIFT` drive),
//! recovers the Driver Eq. 5 of that after-t0 extra-process
//! contribution as
//! `τ + λ(μ_t + a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a))`
//! (JSS PDF re-opened 2026-08-21T06:32Z; the first-occasion
//! extra-process observed mean is not that observed mean when
//! `u ≠ t0`; `τ + λ μ_t` is not that observed mean; the impulse-carry
//! map is not that observed mean; the after-t0 contribution is not
//! `E(y_t)`; the evolved-plus-after-contribution latent mean is not
//! `E(y_t)`),
//! recovers the Driver §7.2 `asymTIPREDEFFECT` as `-B z / a`
//! (pp. 20–21; JSS PDF opened 2026-08-21T13:08Z; expected total
//! change in process means given a time-independent predictor;
//! `a < 0`; not the coefficient `B`, not
//! `A^{-1}[e^{A Δt} − I] B z`, not `CINT`, and not `M x`),
//! recovers the Driver §7.2 `addedTIPREDVAR` as `(B / a)² v`
//! (pp. 20–21; stable between-subject variance accounted for by a
//! time-independent predictor with variance `v`; not `TRAITVAR`,
//! not `asymDIFFUSION`, and not `-B z / a`),
//! recovers the Driver Table 2 `asymCINT` as `-κ / a`
//! (p. 12; Eq. 3 as `Δt → ∞`; JSS PDF opened 2026-08-21T16:13Z;
//! expected change in process means for a unit intercept; `a < 0`;
//! not `κ`, not `A^{-1}[e^{A Δt} − I] κ`, not `T0MEANS`, and not
//! `-B z / a`; p. 16 `T0MEANS` stationarity includes TI predictors;
//! that composition is not this intercept-only map),
//! recovers the Driver p. 16 / §4.3 stationary `T0MEANS` as
//! `-κ / a + −B z / a`
//! (constrained first-occasion mean; form the intercept
//! contribution first, then include the TI extra effect, then add;
//! not free `T0MEANS`, not `asymCINT` alone, not
//! `asymTIPREDEFFECT` alone, and not the finite-interval discrete
//! latent mean),
//! recovers the Driver Eq. 5 of that constrained mean as
//! `τ + λ(−κ / a + −B z / a)`
//! (§4.3, pp. 9–10; Eq. 5, p. 5; JSS PDF re-opened 2026-08-21T20:07Z;
//! form the stationary latent mean first, then `τ + λ` of that mean;
//! `τ + λ μ_0` is not that observed mean; `τ + λ(−κ / a)` is not
//! that observed mean when `B z ≠ 0`; `τ + λ μ_t` is not that
//! observed mean; `MANIFESTMEANS` is not `E(y_0)`; the constrained
//! latent mean is not `E(y_0)`),
//! recovers the Driver §4.3 / p. 16 stationary `T0VAR` as
//! `trait + −q / (2 a) + (B / a)² v`
//! (JSS PDF re-opened 2026-08-22T03:07Z; constrained first-occasion
//! variance; form the within-subject contribution first, then
//! include the trait, then include the TI extra variance, then add;
//! not free `T0VAR`, not `asymDIFFUSION` alone, not `TRAITVAR`
//! alone, not `addedTIPREDVAR` alone, and not the finite-interval
//! discrete latent variance),
//! recovers the Driver Eq. 5 of that constrained variance as
//! `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`
//! (§4.3, pp. 9–10; Eq. 5, p. 5; Table 2, p. 12; JSS PDF re-opened
//! 2026-08-22T03:20Z; form the stationary latent variance first,
//! then `λ² p + θ + ψ`; `λ² p_0` is not that observed variance;
//! `λ²(−q / (2 a)) + θ` is not that observed variance when
//! `TRAITVAR` or `addedTIPREDVAR` is nonzero; `MANIFESTVAR` is not
//! `Var(y_0)`; the constrained latent variance is not `Var(y_0)`),
//! recovers the Driver Eq. 3–4 lagged covariance of that constrained
//! process as `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v`
//! (JSS PDF re-opened 2026-08-22T19:13Z; form the lagged
//! within-subject covariance first, then include the trait, then
//! include the TI extra variance, then add; trait and
//! `addedTIPREDVAR` do not decay with `e^{a Δt}`; contemporaneous
//! `T0VAR` is not that lagged map; decaying the constrained total
//! as if it were all state is not that lagged map),
//! recovers the Driver Eq. 5 of that lagged covariance as
//! `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ`
//! (`Θ` does not enter; contemporaneous `Var(y_0)` is not that
//! lagged observed covariance; the lagged latent covariance is not
//! that observed covariance),
//! recovers the Driver Eq. 3–4 later-occasion variance of that
//! constrained process as
//! `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v`
//! (JSS PDF re-opened 2026-08-22T23:12Z; form the evolved
//! within-subject variance first, then include the trait, then
//! include the TI extra variance, then add; trait and
//! `addedTIPREDVAR` do not enter `Q_Δt`; under stationarity that
//! composition equals contemporaneous `T0VAR`; evolving the
//! constrained total as if it were all state is not that later
//! map; the lagged covariance omits `Q_Δt` and is not that later
//! map; `Q_Δt` is not that later map),
//! recovers the Driver Eq. 5 of that later-occasion variance as
//! `λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`
//! (the lagged observed covariance omits `Q_Δt` and `θ`;
//! `MANIFESTVAR` is not that later observed variance; the
//! later-occasion latent variance is not that observed variance),
//! recovers the Driver §4.3 predetermined later-occasion variance as
//! `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T20:20Z; form the evolved free
//! first-occasion variance first, then include the trait, then
//! include the TI extra variance, then add; trait and
//! `addedTIPREDVAR` do not enter `Q_Δt`; free `T0VAR` `p_0` is not
//! that later map; setting `p_0 = −q / (2 a)` recovers the
//! stationary later-occasion map; stationary later variance uses
//! `−q / (2 a)` in place of `p_0` and is not that later map when
//! `p_0` is free; evolving `trait + p_0 + (B / a)² v` as if it were
//! all state is not that later map; as `Δt → ∞` with stable `a < 0`
//! the composition approaches contemporaneous stationary `T0VAR`;
//! as `Δt → 0+` the composition approaches
//! `trait + p_0 + (B / a)² v`; nonzero diffusion with `a ≥ 0` is a
//! growing process and is kept),
//! recovers the Driver Eq. 5 of that predetermined later-occasion
//! variance as
//! `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`
//! (`MANIFESTVAR` is not that later observed variance; the
//! predetermined later-occasion latent variance is not that observed
//! variance; stationary later observed variance is not that observed
//! variance when `p_0` is free),
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
/// Typed invariance evidence required before a latent-mean comparison.
pub use construct::LatentMeanComparisonEvidence;
/// Permit latent-mean comparison only on strong/strict typed evidence.
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
/// Exact scalar Table 2 `asymCINT` `-κ / a`.
pub use event_time::recover_asymptotic_continuous_intercept;
/// Exact scalar §7.2 `asymTIPREDEFFECT` `-B z / a`.
pub use event_time::recover_asymptotic_time_independent_predictor_effect;
/// Exact scalar §7.2 `addedTIPREDVAR` `(B / a)² v`.
pub use event_time::recover_asymptotic_time_independent_predictor_variance;
/// Exact scalar discrete effect of a constant event-time predictor.
pub use event_time::recover_discrete_constant_predictor_effect;
/// Exact scalar discrete intercept increment `A^{-1}[e^{A Δt} − I] κ`.
pub use event_time::recover_discrete_continuous_intercept_effect;
/// Exact scalar forward map `φ = exp(a Δt)`.
pub use event_time::recover_discrete_lag_from_log_rate;
/// Noiseless scalar discrete lag `later / earlier`.
pub use event_time::recover_discrete_lag_one;
/// Exact scalar lagged latent covariance `A_Δt cov(η_{t-1})`.
pub use event_time::recover_discrete_lagged_latent_covariance;
/// Exact scalar discrete latent mean `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`.
pub use event_time::recover_discrete_latent_mean;
/// Exact scalar evolved latent mean plus a §7.2 extra-process contribution.
pub use event_time::recover_discrete_latent_mean_with_extra_process;
/// Exact scalar evolved latent mean plus a §7.2 extra-process contribution after t0.
pub use event_time::recover_discrete_latent_mean_with_extra_process_after;
/// Exact scalar evolved latent mean plus a contemporaneous impulse.
pub use event_time::recover_discrete_latent_mean_with_impulse;
/// Exact scalar evolved latent mean plus a within-interval impulse carry.
pub use event_time::recover_discrete_latent_mean_with_impulse_carry;
/// Exact scalar evolved latent mean plus a first-occasion TD predictor.
pub use event_time::recover_discrete_latent_mean_with_initial_time_dependent_predictor;
/// Exact scalar evolved latent mean plus a first-occasion TI predictor.
pub use event_time::recover_discrete_latent_mean_with_initial_time_independent_predictor;
/// Exact scalar evolved latent mean plus a time-independent predictor.
pub use event_time::recover_discrete_latent_mean_with_time_independent_predictor;
/// Exact scalar discrete latent variance `A_Δt P A_Δt⊤ + Q_Δt`.
pub use event_time::recover_discrete_latent_variance;
/// Exact scalar discrete observed mean `τ + λ μ_t` from Eq. 3 then Eq. 5.
pub use event_time::recover_discrete_observed_mean;
/// Exact scalar discrete observed mean of a §7.2 extra-process contribution.
pub use event_time::recover_discrete_observed_mean_with_extra_process;
/// Exact scalar discrete observed mean of a §7.2 extra-process contribution after t0.
pub use event_time::recover_discrete_observed_mean_with_extra_process_after;
/// Exact scalar discrete observed mean of a contemporaneous impulse.
pub use event_time::recover_discrete_observed_mean_with_impulse;
/// Exact scalar discrete observed mean of a within-interval impulse carry.
pub use event_time::recover_discrete_observed_mean_with_impulse_carry;
/// Exact scalar discrete observed mean of a first-occasion TD predictor.
pub use event_time::recover_discrete_observed_mean_with_initial_time_dependent_predictor;
/// Exact scalar discrete observed mean of a first-occasion TI predictor.
pub use event_time::recover_discrete_observed_mean_with_initial_time_independent_predictor;
/// Exact scalar discrete observed mean of a time-independent predictor.
pub use event_time::recover_discrete_observed_mean_with_time_independent_predictor;
/// Exact scalar discrete process noise `Q_Δt` on event time.
pub use event_time::recover_discrete_process_noise;
/// Exact scalar discrete `TIPREDEFFECT` increment `A^{-1}[e^{A Δt} − I] B z`.
pub use event_time::recover_discrete_time_independent_predictor_effect;
/// First-order discrete effect of a time-varying event-time predictor.
pub use event_time::recover_discrete_time_varying_predictor_effect;
/// Mean local log-rate on a sorted event-time series.
pub use event_time::recover_event_series_mean_log_rate;
/// Exact scalar pair `(φ, a)` on event time.
pub use event_time::recover_event_time_discrete_lag_and_log_rate;
/// Exact scalar carried first-occasion `T0TDPREDEFFECT` `e^{A Δt} t0_m x0`.
pub use event_time::recover_initial_time_dependent_predictor_carry;
/// Exact scalar first-occasion `T0TDPREDEFFECT` shift `t0_m x0`.
pub use event_time::recover_initial_time_dependent_predictor_effect;
/// Exact scalar carried first-occasion `T0TIPREDEFFECT` `e^{A Δt} t0_b z`.
pub use event_time::recover_initial_time_independent_predictor_carry;
/// Exact scalar first-occasion `T0TIPREDEFFECT` shift `t0_b z`.
pub use event_time::recover_initial_time_independent_predictor_effect;
/// Mean exact log-rate on already-centered irregular residuals.
pub use event_time::recover_irregular_centered_residual_log_rate;
/// Exact scalar §7.2 level-change `CINT` `κ = −a m x`.
pub use event_time::recover_level_change_continuous_intercept;
/// Exact scalar Eq. 3 increment of that `CINT` `(1 − e^{a Δt}) m x`.
pub use event_time::recover_level_change_discrete_increment;
/// Exact scalar §7.2 extra-process contribution `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`.
pub use event_time::recover_level_change_extra_process_contribution;
/// Exact scalar §7.2 extra-process contribution after t0 on `t − u`.
pub use event_time::recover_level_change_extra_process_contribution_after;
/// Exact scalar inverse `a = ln(φ) / Δt`.
pub use event_time::recover_local_log_rate;
/// Exact scalar lagged observed-indicator covariance `λ² cov(η) + ψ`.
pub use event_time::recover_manifest_lagged_observed_covariance;
/// Exact scalar observed-indicator mean `τ + λ μ`.
pub use event_time::recover_manifest_observed_mean;
/// Exact scalar observed-indicator variance `λ² Var(η) + θ`.
pub use event_time::recover_manifest_observed_variance;
/// Exact scalar observed-indicator variance `λ² Var(η) + θ + ψ`.
pub use event_time::recover_manifest_trait_plus_state_observed_variance;
/// Exact scalar later-occasion variance of §4.3 predetermined `T0VAR` `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v`.
pub use event_time::recover_predetermined_later_latent_variance;
/// Exact scalar Eq. 5 of later-occasion §4.3 predetermined `T0VAR` `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`.
pub use event_time::recover_predetermined_later_observed_variance;
/// Exact scalar p. 16 stationary `T0MEANS` `-κ / a + −B z / a`.
pub use event_time::recover_stationary_initial_latent_mean;
/// Exact scalar §4.3 / p. 16 stationary `T0VAR` `trait + −q / (2 a) + (B / a)² v`.
pub use event_time::recover_stationary_initial_latent_variance;
/// Exact scalar Eq. 5 of §4.3 stationary `T0MEANS` `τ + λ(−κ / a + −B z / a)`.
pub use event_time::recover_stationary_initial_observed_mean;
/// Exact scalar Eq. 5 of §4.3 stationary `T0VAR` `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`.
pub use event_time::recover_stationary_initial_observed_variance;
/// Exact scalar lagged covariance of §4.3 stationary `T0VAR` `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v`.
pub use event_time::recover_stationary_lagged_latent_covariance;
/// Exact scalar Eq. 5 of lagged §4.3 stationary `T0VAR` `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ`.
pub use event_time::recover_stationary_lagged_observed_covariance;
/// Exact scalar stationary within-subject variance `-q / (2 a)`.
pub use event_time::recover_stationary_latent_variance;
/// Exact scalar later-occasion variance of §4.3 stationary `T0VAR` `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v`.
pub use event_time::recover_stationary_later_latent_variance;
/// Exact scalar Eq. 5 of later-occasion §4.3 stationary `T0VAR` `λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`.
pub use event_time::recover_stationary_later_observed_variance;
/// Exact scalar contemporaneous `TDPREDEFFECT` impulse `m x`.
pub use event_time::recover_time_dependent_predictor_impulse;
/// Exact scalar within-interval `TDPREDEFFECT` carry `e^{A(t−u)} M x`.
pub use event_time::recover_time_dependent_predictor_impulse_carry;
/// Exact scalar trait-plus-state lagged covariance.
pub use event_time::recover_trait_plus_state_lagged_covariance;
/// Exact scalar trait-plus-state latent variance.
pub use event_time::recover_trait_plus_state_latent_variance;
/// CWC-then-event-time local log-rate (not DSEM; not raw-process AR drift).
pub use event_time::recover_within_residual_event_time_log_rate;
/// Refuse treating the after-t0 extra-process contribution as `E(y_t)`.
pub use event_time::refuse_after_extra_process_contribution_as_observed_mean;
/// Refuse treating the evolved-plus-after-contribution latent mean as `E(y_t)`.
pub use event_time::refuse_after_extra_process_latent_mean_as_observed_mean;
/// Refuse treating Table 2 `asymCINT` as `asymTIPREDEFFECT`.
pub use event_time::refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect;
/// Refuse treating Table 2 `asymCINT` as `CINT`.
pub use event_time::refuse_asymptotic_continuous_intercept_as_continuous_intercept;
/// Refuse treating Table 2 `asymCINT` as the finite-interval discrete increment.
pub use event_time::refuse_asymptotic_continuous_intercept_as_discrete_increment;
/// Refuse treating Table 2 `asymCINT` as `T0MEANS`.
pub use event_time::refuse_asymptotic_continuous_intercept_as_initial_latent_mean;
/// Refuse treating `τ + λ(−κ / a)` as Eq. 5 of §4.3 stationary `T0MEANS`.
pub use event_time::refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `TIPREDEFFECT` `B`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_coefficient;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `CINT`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_continuous_intercept;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as the finite-interval discrete increment.
pub use event_time::refuse_asymptotic_time_independent_effect_as_discrete_effect;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `M x`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_time_dependent_impulse;
/// Refuse treating §7.2 `addedTIPREDVAR` as `asymTIPREDEFFECT`.
pub use event_time::refuse_asymptotic_time_independent_variance_as_asymptotic_effect;
/// Refuse treating §7.2 `addedTIPREDVAR` as `asymDIFFUSION`.
pub use event_time::refuse_asymptotic_time_independent_variance_as_stationary_within_subject;
/// Refuse treating §7.2 `addedTIPREDVAR` as `TRAITVAR`.
pub use event_time::refuse_asymptotic_time_independent_variance_as_trait_variance;
/// Refuse treating Driver Table 2 `CINT` as the discrete mean increment.
pub use event_time::refuse_continuous_intercept_as_discrete_mean_increment;
/// Refuse treating Driver Table 2 `CINT` as `T0MEANS`.
pub use event_time::refuse_continuous_intercept_as_initial_latent_mean;
/// Refuse treating Driver Table 2 `CINT` as `MANIFESTMEANS`.
pub use event_time::refuse_continuous_intercept_as_manifest_means;
/// Refuse the difference quotient as a continuous-time rate.
pub use event_time::refuse_difference_quotient_as_local_rate;
/// Refuse treating evolved `τ + λ μ_t` as the after-t0 extra-process observed mean.
pub use event_time::refuse_evolved_observed_mean_as_after_extra_process_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the extra-process observed mean.
pub use event_time::refuse_evolved_observed_mean_as_extra_process_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the impulse-carry observed mean.
pub use event_time::refuse_evolved_observed_mean_as_impulse_carry_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the contemporaneous-impulse observed mean.
pub use event_time::refuse_evolved_observed_mean_as_impulse_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the first-occasion TD-predictor observed mean.
pub use event_time::refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the first-occasion TI-predictor observed mean.
pub use event_time::refuse_evolved_observed_mean_as_initial_time_independent_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as Eq. 5 of §4.3 stationary `T0MEANS`.
pub use event_time::refuse_evolved_observed_mean_as_stationary_initial_observed_mean;
/// Refuse treating evolved `τ + λ μ_t` as the time-independent-predictor observed mean.
pub use event_time::refuse_evolved_observed_mean_as_time_independent_observed_mean;
/// Refuse treating evolved `λ² Var(η_t) + θ` as Eq. 5 of §4.3 stationary `T0VAR`.
pub use event_time::refuse_evolved_observed_variance_as_stationary_initial_observed_variance;
/// Refuse treating the §7.2 extra-process contribution as `E(y_t)`.
pub use event_time::refuse_extra_process_contribution_as_observed_mean;
/// Refuse treating the evolved-plus-contribution latent mean as `E(y_t)`.
pub use event_time::refuse_extra_process_latent_mean_as_observed_mean;
/// Refuse treating the first-occasion extra-process observed mean as the after-t0 extra-process observed mean.
pub use event_time::refuse_extra_process_observed_mean_as_after_extra_process_observed_mean;
/// Refuse treating finite-interval `Q_Δt` as `asymDIFFUSION`.
pub use event_time::refuse_finite_interval_process_noise_as_stationary_variance;
/// Refuse treating impulse-carry `τ + λ(μ_t + e^{a(t−u)} m x)` as the after-t0 extra-process observed mean.
pub use event_time::refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean;
/// Refuse treating impulse-carry `τ + λ(μ_t + e^{a(t−u)} m x)` as the first-occasion TD-predictor observed mean.
pub use event_time::refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean;
/// Refuse treating impulse-carry `τ + λ(μ_t + e^{a(t−u)} m x)` as the first-occasion TI-predictor observed mean.
pub use event_time::refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean;
/// Refuse treating impulse-carry `τ + λ(μ_t + e^{a(t−u)} m x)` as the time-independent-predictor observed mean.
pub use event_time::refuse_impulse_carry_observed_mean_as_time_independent_observed_mean;
/// Refuse treating contemporaneous `τ + λ(μ_t + m x)` as the extra-process observed mean.
pub use event_time::refuse_impulse_observed_mean_as_extra_process_observed_mean;
/// Refuse treating contemporaneous `τ + λ(μ_t + m x)` as the impulse-carry observed mean.
pub use event_time::refuse_impulse_observed_mean_as_impulse_carry_observed_mean;
/// Refuse treating contemporaneous `τ + λ(μ_t + m x)` as the first-occasion TD-predictor observed mean.
pub use event_time::refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean;
/// Refuse treating contemporaneous `τ + λ(μ_t + m x)` as the first-occasion TI-predictor observed mean.
pub use event_time::refuse_impulse_observed_mean_as_initial_time_independent_observed_mean;
/// Refuse treating contemporaneous `τ + λ(μ_t + m x)` as the time-independent-predictor observed mean.
pub use event_time::refuse_impulse_observed_mean_as_time_independent_observed_mean;
/// Refuse treating Driver Table 2 `T0MEANS` as the evolved latent mean.
pub use event_time::refuse_initial_latent_mean_as_evolved_mean;
/// Refuse treating first-occasion `τ + λ μ_0` as `E(y_t)`.
pub use event_time::refuse_initial_observed_mean_as_evolved_observed_mean;
/// Refuse treating `τ + λ μ_0` as Eq. 5 of §4.3 stationary `T0MEANS`.
pub use event_time::refuse_initial_observed_mean_as_stationary_initial_observed_mean;
/// Refuse treating `λ² p_0 + θ` as Eq. 5 of §4.3 stationary `T0VAR`.
pub use event_time::refuse_initial_observed_variance_as_stationary_initial_observed_variance;
/// Refuse treating the Eq. 3 `T0TDPREDEFFECT` carry as the within-interval impulse carry.
pub use event_time::refuse_initial_time_dependent_carry_as_impulse_carry;
/// Refuse treating the Eq. 3 `T0TDPREDEFFECT` carry as the first-occasion shift.
pub use event_time::refuse_initial_time_dependent_carry_as_initial_effect;
/// Refuse treating Driver Table 3 `T0TDPREDEFFECT` as the first-occasion shift.
pub use event_time::refuse_initial_time_dependent_coefficient_as_initial_effect;
/// Refuse treating the Table 3 first-occasion TD shift as `M x`.
pub use event_time::refuse_initial_time_dependent_effect_as_contemporaneous_impulse;
/// Refuse treating the Table 3 first-occasion TD shift as `CINT`.
pub use event_time::refuse_initial_time_dependent_effect_as_continuous_intercept;
/// Refuse treating the Table 3 first-occasion TD shift as the Table 3 TI shift.
pub use event_time::refuse_initial_time_dependent_effect_as_initial_time_independent_effect;
/// Refuse treating the Table 3 first-occasion TD shift as the Eq. 3 process increment.
pub use event_time::refuse_initial_time_dependent_effect_as_process_increment;
/// Refuse treating the Eq. 3 `T0TIPREDEFFECT` carry as the first-occasion shift.
pub use event_time::refuse_initial_time_independent_carry_as_initial_effect;
/// Refuse treating Driver Table 3 `T0TIPREDEFFECT` as the first-occasion shift.
pub use event_time::refuse_initial_time_independent_coefficient_as_initial_effect;
/// Refuse treating the Table 3 first-occasion TI shift as `CINT`.
pub use event_time::refuse_initial_time_independent_effect_as_continuous_intercept;
/// Refuse treating the Table 3 first-occasion TI shift as the Eq. 3 process increment.
pub use event_time::refuse_initial_time_independent_effect_as_process_increment;
/// Refuse treating the Table 3 first-occasion TI shift as `M x`.
pub use event_time::refuse_initial_time_independent_effect_as_time_dependent_impulse;
/// Refuse treating first-occasion TI observed mean as the first-occasion TD observed mean.
pub use event_time::refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean;
/// Refuse treating Driver Eq. 3–4 lagged latent covariance as `cov(y_t, y_{t-1})`.
pub use event_time::refuse_latent_lagged_covariance_as_observed_covariance;
/// Refuse treating Driver Eq. 5 latent mean as `E(y)`.
pub use event_time::refuse_latent_mean_as_observed_mean;
/// Refuse treating Driver Eq. 5 latent variance as `Var(y)`.
pub use event_time::refuse_latent_variance_as_observed_variance;
/// Refuse treating the §7.2 extra-process contribution as the contemporaneous Dirac.
pub use event_time::refuse_level_change_extra_process_as_impulse;
/// Refuse treating the §7.2 extra-process contribution as the Eq. 3 level-change increment.
pub use event_time::refuse_level_change_extra_process_as_increment;
/// Refuse treating the §7.2 extra-process contribution as the level-change `CINT`.
pub use event_time::refuse_level_change_extra_process_as_intercept;
/// Refuse treating the §7.2 level-change CINT increment as the contemporaneous Dirac.
pub use event_time::refuse_level_change_increment_as_impulse;
/// Refuse treating the §7.2 level-change CINT increment as `CINT`.
pub use event_time::refuse_level_change_increment_as_intercept;
/// Refuse treating the §7.2 level-change CINT increment as the Eq. 3 process increment.
pub use event_time::refuse_level_change_increment_as_process_increment;
/// Refuse treating Driver §7.2 level-change `CINT` as a free `CINT`.
pub use event_time::refuse_level_change_intercept_as_free_continuous_intercept;
/// Refuse treating Driver §7.2 level-change `CINT` as the contemporaneous Dirac.
pub use event_time::refuse_level_change_intercept_as_impulse;
/// Refuse treating Driver §7.2 level-change `CINT` as the Eq. 3 process increment.
pub use event_time::refuse_level_change_intercept_as_process_increment;
/// Refuse treating Driver Eq. 5 `MANIFESTMEANS` as `E(y)`.
pub use event_time::refuse_manifest_means_as_observed_mean;
/// Refuse treating Driver Eq. 5 `MANIFESTTRAITVAR` as `MANIFESTVAR`.
pub use event_time::refuse_manifest_trait_variance_as_measurement_error;
/// Refuse treating Driver Eq. 5 measurement error as lagged observed covariance.
pub use event_time::refuse_measurement_error_as_lagged_observed_covariance;
/// Refuse treating Driver Eq. 5 measurement error as `Var(y)`.
pub use event_time::refuse_measurement_error_as_observed_variance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined later-occasion `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_later_observed_variance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of lagged §4.3 stationary `T0VAR`.
pub use event_time::refuse_measurement_error_as_stationary_lagged_observed_covariance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-occasion §4.3 stationary `T0VAR`.
pub use event_time::refuse_measurement_error_as_stationary_later_observed_variance;
/// Refuse pooling discrete lags from unequal event intervals.
pub use event_time::refuse_pooled_discrete_lag_across_unequal_intervals;
/// Refuse treating predetermined later-occasion variance as the free discrete evolution of the total.
pub use event_time::refuse_predetermined_later_latent_variance_as_discrete_variance;
/// Refuse treating predetermined later-occasion variance as free first-occasion `T0VAR`.
pub use event_time::refuse_predetermined_later_latent_variance_as_initial_latent_variance;
/// Refuse treating predetermined later-occasion variance as predetermined later-occasion observed variance.
pub use event_time::refuse_predetermined_later_latent_variance_as_observed_variance;
/// Refuse treating predetermined later-occasion variance as later-occasion stationary `T0VAR`.
pub use event_time::refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance;
/// Refuse treating Driver Eq. 3 process noise as the unconditional variance.
pub use event_time::refuse_process_noise_as_unconditional_variance;
/// Refuse treating p. 16 stationary `T0MEANS` as `asymCINT`.
pub use event_time::refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept;
/// Refuse treating p. 16 stationary `T0MEANS` as `asymTIPREDEFFECT`.
pub use event_time::refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect;
/// Refuse treating p. 16 stationary `T0MEANS` as a finite-interval discrete mean.
pub use event_time::refuse_stationary_initial_latent_mean_as_discrete_mean;
/// Refuse treating p. 16 stationary `T0MEANS` as free `T0MEANS`.
pub use event_time::refuse_stationary_initial_latent_mean_as_initial_latent_mean;
/// Refuse treating §4.3 stationary `T0MEANS` as `E(y_0)`.
pub use event_time::refuse_stationary_initial_latent_mean_as_observed_mean;
/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as `addedTIPREDVAR`.
pub use event_time::refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance;
/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as a finite-interval discrete variance.
pub use event_time::refuse_stationary_initial_latent_variance_as_discrete_variance;
/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as free `T0VAR`.
pub use event_time::refuse_stationary_initial_latent_variance_as_initial_latent_variance;
/// Refuse treating §4.3 stationary `T0VAR` as `Var(y_0)`.
pub use event_time::refuse_stationary_initial_latent_variance_as_observed_variance;
/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as `asymDIFFUSION`.
pub use event_time::refuse_stationary_initial_latent_variance_as_stationary_within_subject;
/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as `TRAITVAR`.
pub use event_time::refuse_stationary_initial_latent_variance_as_trait_variance;
/// Refuse treating Eq. 5 of §4.3 stationary `T0MEANS` as `MANIFESTMEANS`.
pub use event_time::refuse_stationary_initial_observed_mean_as_manifest_means;
/// Refuse treating Eq. 5 of §4.3 stationary `T0VAR` as `MANIFESTVAR`.
pub use event_time::refuse_stationary_initial_observed_variance_as_measurement_error;
/// Refuse treating Eq. 5 of contemporaneous §4.3 stationary `T0VAR` as lagged observed covariance.
pub use event_time::refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as decayed total stationary variance.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as lagged observed covariance.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_observed_covariance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as contemporaneous stationary `T0VAR`.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance;
/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as later-occasion observed variance.
pub use event_time::refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance;
/// Refuse treating later-occasion §4.3 stationary `T0VAR` as the free discrete evolution of the constrained total.
pub use event_time::refuse_stationary_later_latent_variance_as_discrete_variance;
/// Refuse treating later-occasion §4.3 stationary `T0VAR` as lagged covariance.
pub use event_time::refuse_stationary_later_latent_variance_as_lagged_covariance;
/// Refuse treating later-occasion §4.3 stationary `T0VAR` as later-occasion observed variance.
pub use event_time::refuse_stationary_later_latent_variance_as_observed_variance;
/// Refuse treating later-occasion §4.3 stationary `T0VAR` as finite-interval process noise.
pub use event_time::refuse_stationary_later_latent_variance_as_process_noise;
/// Refuse treating Eq. 5 of later-occasion §4.3 stationary `T0VAR` as predetermined later-occasion observed variance.
pub use event_time::refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance;
/// Refuse treating Eq. 5 of `asymDIFFUSION` as Eq. 5 of §4.3 stationary `T0VAR`.
pub use event_time::refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance;
/// Refuse treating Driver Eq. 3 `TDPREDEFFECT` impulse as `CINT`.
pub use event_time::refuse_time_dependent_impulse_as_continuous_intercept;
/// Refuse treating Driver Eq. 3 impulse as `TIPREDEFFECT`.
pub use event_time::refuse_time_dependent_impulse_as_time_independent_effect;
/// Refuse treating Driver Eq. 3 impulse as Voelkle Eq. 14.
pub use event_time::refuse_time_dependent_impulse_as_time_varying_discrete_effect;
/// Refuse treating Driver Eq. 1–2 impulse carry as the contemporaneous Dirac.
pub use event_time::refuse_time_dependent_impulse_carry_as_contemporaneous_impulse;
/// Refuse treating Driver Eq. 1–2 impulse carry as `CINT`.
pub use event_time::refuse_time_dependent_impulse_carry_as_continuous_intercept;
/// Refuse treating Driver Eq. 1–2 impulse carry as `TIPREDEFFECT`.
pub use event_time::refuse_time_dependent_impulse_carry_as_time_independent_effect;
/// Refuse treating Driver Eq. 1–2 impulse carry as Voelkle Eq. 14.
pub use event_time::refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect;
/// Refuse treating Driver Table 2 `TIPREDEFFECT` as the discrete increment.
pub use event_time::refuse_time_independent_coefficient_as_discrete_effect;
/// Refuse treating Driver Eq. 3 `TIPREDEFFECT` increment as `CINT`.
pub use event_time::refuse_time_independent_effect_as_continuous_intercept;
/// Refuse treating Driver Eq. 3 `TIPREDEFFECT` increment as `M x`.
pub use event_time::refuse_time_independent_effect_as_time_dependent_impulse;
/// Refuse treating Driver Eq. 3 `TIPREDEFFECT` increment as Voelkle Eq. 14.
pub use event_time::refuse_time_independent_effect_as_time_varying_discrete_effect;
/// Refuse treating process-increment `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` as the first-occasion TD-predictor observed mean.
pub use event_time::refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean;
/// Refuse treating process-increment `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` as the first-occasion TI-predictor observed mean.
pub use event_time::refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean;
/// Refuse treating §4.3 trait-plus-state lagged covariance as lagged stationary `T0VAR`.
pub use event_time::refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance;
/// Refuse treating Driver §4.3 trait variance as process noise.
pub use event_time::refuse_trait_variance_as_process_noise;
/// Refuse treating Driver §4.3 trait variance as `asymDIFFUSION`.
pub use event_time::refuse_trait_variance_as_stationary_within_subject;
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
