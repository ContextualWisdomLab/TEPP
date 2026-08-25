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
//! (JSS PDF re-opened 2026-08-23T05:12Z; form the evolved free
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
//! recovers the Driver §4.3 predetermined lagged covariance as
//! `trait + e^{a Δt} p_0 + (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T09:04Z; form the lagged free
//! first-occasion covariance first, then include the trait, then
//! include the TI extra variance, then add; trait and
//! `addedTIPREDVAR` do not decay with `e^{a Δt}`; free `T0VAR` `p_0`
//! is not that lagged map; setting `p_0 = −q / (2 a)` recovers the
//! stationary lagged map; stationary lagged covariance uses
//! `−q / (2 a)` in place of `p_0` and is not that lagged map when
//! `p_0` is free; evolving `trait + p_0 + (B / a)² v` as if it were
//! all state is not that lagged map; later-occasion variance
//! includes `Q_Δt` and is not that lagged map; as `Δt → ∞` with
//! stable `a < 0` the state term vanishes; as `Δt → 0+` the
//! composition approaches `trait + p_0 + (B / a)² v`),
//! recovers the Driver Eq. 5 of that predetermined lagged
//! covariance as `λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ`
//! (`MANIFESTVAR` does not enter; the predetermined lagged latent
//! covariance is not that observed covariance; predetermined later
//! observed variance includes `Q_Δt` and `θ` and is not that lagged
//! observed covariance; stationary lagged observed covariance is
//! not that observed covariance when `p_0` is free),
//! recovers the Driver §4.3 predetermined first-occasion variance as
//! `trait + p_0 + (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T10:03Z; form the free first-occasion
//! state variance first, then include the trait, then include the TI
//! extra variance, then add; trait and `addedTIPREDVAR` do not decay
//! and do not enter `Q_Δt`; free `T0VAR` `p_0` is not that
//! first-occasion map; setting `p_0 = −q / (2 a)` recovers the
//! stationary first-occasion map; stationary first-occasion variance
//! uses `−q / (2 a)` in place of `p_0` and is not that map when
//! `p_0` is free; lagged covariance decays the state and is not that
//! map; later-occasion variance includes `Q_Δt` and is not that map;
//! as `Δt → 0+` the lagged and later maps approach this composition),
//! recovers the Driver Eq. 5 of that predetermined first-occasion
//! variance as `λ²(trait + p_0 + (B / a)² v) + θ + ψ`
//! (`MANIFESTVAR` is not that first-occasion observed variance; the
//! predetermined first-occasion latent variance is not that observed
//! variance; stationary first-occasion observed variance is not that
//! observed variance when `p_0` is free; predetermined later
//! observed variance includes `Q_Δt` and is not that first-occasion
//! observed variance),
//! recovers the Driver §4.3 later-start lagged covariance of
//! predetermined `T0VAR` as
//! `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T10:27Z; `startoffset`; form the
//! later-start within-subject variance first, then lag that state,
//! then include the trait, then include the TI extra variance, then
//! add; trait and `addedTIPREDVAR` do not decay with `e^{a s}`;
//! first-occasion lagged covariance omits `e^{a s} Q_u` and is not
//! that map when `u > 0`; later-occasion variance includes `Q_u`
//! without lagging and is not that map; setting `p_0 = −q / (2 a)`
//! recovers the stationary lagged map; stationary lagged covariance
//! uses `−q / (2 a)` in place of `p_0` and is not that map when
//! `p_0` is free; evolving the later total as if it were all state
//! is not that map; as `u → 0+` the composition approaches
//! first-occasion lagged covariance; as `s → 0+` the composition
//! approaches later-occasion variance at `u`),
//! recovers the Driver Eq. 5 of that later-start lagged covariance as
//! `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`
//! (`MANIFESTVAR` does not enter; the later-start lagged latent
//! covariance is not that observed covariance; first-occasion lagged
//! observed covariance omits `e^{a s} Q_u` and is not that observed
//! covariance when `u > 0`; predetermined later observed variance
//! includes `Q_u` and `θ` and is not that later-start lagged
//! observed covariance; stationary lagged observed covariance is
//! not that observed covariance when `p_0` is free),
//! recovers the Driver §4.3 later-start later-occasion variance of
//! predetermined `T0VAR` as
//! `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T11:05Z; `startoffset`; Chapman–
//! Kolmogorov `Q_{u+s} = e^{2 a s} Q_u + Q_s`; form the later-start
//! within-subject variance first, then evolve that state, then
//! include the trait, then include the TI extra variance, then add;
//! trait and `addedTIPREDVAR` do not enter `Q_s`; later-occasion
//! variance at `u` omits `Q_s` and is not that map when `s > 0`;
//! later-start lagged covariance omits `Q_s` and is not that map;
//! setting `p_0 = −q / (2 a)` recovers the stationary later-occasion
//! map; stationary later-occasion variance uses `−q / (2 a)` in
//! place of `p_0` and is not that map when `p_0` is free; evolving
//! the later total as if it were all state is not that map;
//! later-occasion variance over the lag interval alone ignores
//! `startoffset` and is not that map when `u > 0`; as `u → 0+` the
//! composition approaches later-occasion variance over `s`; as
//! `s → 0+` the composition approaches later-occasion variance at
//! `u`),
//! recovers the Driver Eq. 5 of that later-start later-occasion
//! variance as
//! `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`
//! (`MANIFESTVAR` is not that later-start later-occasion observed
//! variance; the later-start later-occasion latent variance is not
//! that observed variance; predetermined later observed variance
//! omits `Q_s` and is not that observed variance when `s > 0`;
//! later-start lagged observed covariance omits `Q_s` and `θ` and is
//! not that observed variance; stationary later observed variance is
//! not that observed variance when `p_0` is free),
//! recovers the Driver p. 16 `discreteDRIFTstd` as `e^{a Δt}` after
//! forming strictly positive `asymDIFFUSION` `−q / (2 a)`
//! (JSS PDF re-opened 2026-08-23T11:40Z; footnote 4 standardises
//! `DRIFT` using only within-subject variance, not the total;
//! unstandardised `e^{a Δt}` is defined for growing `a ≥ 0` and for
//! zero diffusion, and is not `discreteDRIFTstd`; zero
//! `asymDIFFUSION` fails closed; the §7.1 trait-plus-state
//! autocorrelation `(trait + e^{a Δt} p + added) / (trait + p + added)`
//! uses `TRAITVAR` and is not `discreteDRIFTstd`; `TRAITVAR` is not
//! the standardisation variance),
//! recovers the Driver p. 16 `discreteDIFFUSIONstd` as
//! `Q_Δt / (−q / (2 a))` after forming strictly positive
//! `asymDIFFUSION` `−q / (2 a)`
//! (JSS PDF re-opened 2026-08-23T13:06Z; footnote 4 standardises
//! using only the relevant variance, not the total; process noise is
//! within-subject, so that variance is `asymDIFFUSION`;
//! unstandardised `Q_Δt` is defined for growing `a ≥ 0` and for
//! zero diffusion, and is not `discreteDIFFUSIONstd`; zero
//! `asymDIFFUSION` fails closed; the continuous standardisation
//! `q / (−q / (2 a)) = −2 a` is not `discreteDIFFUSIONstd`;
//! `Q_Δt / (trait + p + added)` uses `TRAITVAR` and is not
//! `discreteDIFFUSIONstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver p. 16 `DIFFUSIONstd` as
//! `q / (−q / (2 a)) = −2 a` after forming strictly positive
//! `asymDIFFUSION` `−q / (2 a)`
//! (JSS PDF re-opened 2026-08-23T13:20Z; footnote 4 standardises
//! using only the relevant variance, not the total; process noise is
//! within-subject, so that variance is `asymDIFFUSION`;
//! unstandardised `q` is defined for growing `a ≥ 0` and for zero
//! diffusion, and is not `DIFFUSIONstd`; zero `asymDIFFUSION` fails
//! closed; the discrete standardisation
//! `Q_Δt / (−q / (2 a)) = 1 − exp(2 a Δt)` is not `DIFFUSIONstd`;
//! `q / (trait + p + added)` uses `TRAITVAR` and is not
//! `DIFFUSIONstd`; `TRAITVAR` is not the standardisation variance),
//! recovers the Driver p. 16 `DRIFTstd` as the continuous auto-effect
//! after forming strictly positive `asymDIFFUSION` `−q / (2 a)`
//! (JSS PDF re-opened 2026-08-23T13:28Z; footnote 4 standardises
//! `DRIFT` using only within-subject variance, not the total;
//! unstandardised `a` is defined for growing `a ≥ 0` and for zero
//! diffusion, and is not `DRIFTstd`; zero `asymDIFFUSION` fails
//! closed; the discrete standardisation `e^{a Δt}` is not
//! `DRIFTstd`; `a p / (trait + p + added)` uses `TRAITVAR` and is
//! not `DRIFTstd`; `TRAITVAR` is not the standardisation variance),
//! recovers the Driver p. 16 `asymTIPREDEFFECTstd` as
//! `(-B / a) · √v / √(-q / (2 a))` after forming strictly positive
//! `asymDIFFUSION` and strictly positive predictor variance
//! (JSS PDF re-opened 2026-08-23T14:25Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is `TIPREDVAR` `v`; the affected variance is
//! `asymDIFFUSION`; unstandardised `-B / a` is defined for a zero
//! coefficient and for zero predictor variance, and is not
//! `asymTIPREDEFFECTstd`; zero `asymDIFFUSION` or zero `v` fails
//! closed; the finite-interval standardisation
//! `A^{-1}[e^{A Δt} − I] B · √v / √p` is not `asymTIPREDEFFECTstd`;
//! `(-B / a) · √v / √(trait + p + added)` uses `TRAITVAR` and is
//! not `asymTIPREDEFFECTstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver p. 16 finite-interval standardised
//! `TIPREDEFFECT` as `A^{-1}[e^{A Δt} − I] B · √v / √(-q / (2 a))`
//! after forming strictly positive `asymDIFFUSION` and strictly
//! positive predictor variance (JSS PDF re-opened 2026-08-24T01:20Z;
//! footnote 4 standardises using only the relevant variance, not the
//! total; the affecting variance is `TIPREDVAR` `v`; the affected
//! variance is `asymDIFFUSION`; unstandardised
//! `A^{-1}[e^{A Δt} − I] B` is defined for a zero coefficient and
//! for zero predictor variance, and is not the standardised
//! finite-interval map; zero `asymDIFFUSION` or zero `v` fails
//! closed; `asymTIPREDEFFECTstd` `(-B / a) · √v / √p` is the
//! `Δt → ∞` map and is not the finite-interval map;
//! `A^{-1}[e^{A Δt} − I] B · √v / √(trait + p + added)` uses
//! `TRAITVAR` and is not the finite-interval map; `TRAITVAR` is not
//! the standardisation variance),
//! recovers the Driver p. 16 `TIPREDEFFECTstd` as
//! `B · √v / √(-q / (2 a))` after forming strictly positive
//! `asymDIFFUSION` and strictly positive predictor variance
//! (JSS PDF re-opened 2026-08-23T16:21Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is `TIPREDVAR` `v`; the affected variance is
//! `asymDIFFUSION`; unstandardised `B` is defined for a zero
//! coefficient and for zero predictor variance, and is not
//! `TIPREDEFFECTstd`; zero `asymDIFFUSION` or zero `v` fails
//! closed; the asymptotic standardisation
//! `(-B / a) · √v / √p` is not `TIPREDEFFECTstd`;
//! the finite-interval standardisation
//! `A^{-1}[e^{A Δt} − I] B · √v / √p` is not `TIPREDEFFECTstd`;
//! `B · √v / √(trait + p + added)` uses `TRAITVAR` and is
//! not `TIPREDEFFECTstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver p. 16 `CINTstd` as
//! `κ / √(-q / (2 a))` after forming strictly positive
//! `asymDIFFUSION`
//! (JSS PDF re-opened 2026-08-23T17:10Z; footnote 4 standardises
//! using only the relevant variance, not the total; `CINT` is the
//! process intercept of individual, or average individual, dynamics,
//! so the relevant variance is `asymDIFFUSION`; unstandardised `κ`
//! is defined for growing `a ≥ 0` and for zero diffusion, and is
//! not `CINTstd`; zero `asymDIFFUSION` fails closed; the asymptotic
//! standardisation `(-κ / a) / √p` is not `CINTstd`; the
//! finite-interval standardisation
//! `A^{-1}[e^{A Δt} − I] κ / √p` is not `CINTstd`;
//! `κ / √(trait + p + added)` uses `TRAITVAR` and is
//! not `CINTstd`; `TRAITVAR` is not the standardisation
//! recovers the Driver Table 3 / p. 16 `T0TIPREDEFFECTstd` as
//! `t0_b · √v / √p_0` after forming strictly positive free `T0VAR`
//! and strictly positive predictor variance
//! (JSS PDF re-opened 2026-08-23T17:20Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is `TIPREDVAR` `v`; the affected variance is free
//! first-occasion `T0VAR` `p_0`, not `asymDIFFUSION`; unstandardised
//! `t0_b` is defined for a zero coefficient and for zero predictor
//! variance, and is not `T0TIPREDEFFECTstd`; zero `p_0` or zero `v`
//! fails closed; `T0` is event time, so a non-event clock fails
//! closed; free `T0VAR` does not require stable `a < 0`;
//! `B · √v / √(-q / (2 a))` is not `T0TIPREDEFFECTstd`;
//! `(-B / a) · √v / √p` is not `T0TIPREDEFFECTstd`;
//! `t0_b · √v / √(trait + p_0 + added)` uses `TRAITVAR` and is
//! not `T0TIPREDEFFECTstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver Table 3 / p. 16 `T0TDPREDEFFECTstd` as
//! `t0_m · √v_x / √p_0` after forming strictly positive free `T0VAR`
//! and strictly positive time-dependent predictor variance
//! (JSS PDF re-opened 2026-08-23T18:17Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is `TDPREDVAR` `v_x`, not `TIPREDVAR`; the affected
//! variance is free first-occasion `T0VAR` `p_0`, not
//! `asymDIFFUSION`; unstandardised `t0_m` is defined for a zero
//! coefficient and for zero predictor variance, and is not
//! `T0TDPREDEFFECTstd`; zero `p_0` or zero `v_x` fails closed;
//! `T0` is event time, so a non-event clock fails closed; free
//! `T0VAR` does not require stable `a < 0`; same numbers as
//! `T0TIPREDEFFECTstd` yield the same product and Table 3 names a
//! different matrix; `B · √v / √(-q / (2 a))` is not
//! `T0TDPREDEFFECTstd`; `t0_m · √v_x / √(trait + p_0 + added)` uses
//! `TRAITVAR` and is not `T0TDPREDEFFECTstd`; `TRAITVAR` is not the
//! standardisation variance),
//! recovers the Driver Table 3 / p. 16 / 2017-era
//! `addedT0TIPREDVAR` as `t0_b² v`
//! (JSS PDF re-opened 2026-08-23T18:20Z; 2017-era
//! `summary.ctsemFit.R` forms
//! `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` immediately
//! after `T0TIPREDEFFECTstd`; form `t0_b` first, then square, then
//! multiply by `v`; a zero coefficient or zero predictor variance is
//! exactly zero; free `T0TIPREDEFFECT` does not require `a < 0`;
//! `(B / a)² v` is `addedTIPREDVAR` and is not this first-occasion
//! map; `t0_b · √v / √p_0` is `T0TIPREDEFFECTstd` and is not this
//! variance; free `T0VAR` is not this extra TI variance; `TRAITVAR`
//! is not this extra TI variance),
//! recovers the Driver Eq. 5 of 2017-era
//! `addedT0TIPREDVAR` as `λ² t0_b² v`
//! (JSS PDF re-opened 2026-08-23T19:10Z; 2017-era
//! `summary.ctsemFit.R` forms the latent extra first; form
//! `t0_b² v` first, then `(λ extra) λ` with `θ = 0`; a zero
//! loading or zero extra is exactly zero; free `T0TIPREDEFFECT`
//! does not require `a < 0`; `t0_b² v` is the latent extra and is
//! not this observed extra; `λ² p_0 + θ` is first-occasion
//! observed variance and is not this extra; `λ² (B / a)² v` is
//! Eq. 5 of `addedTIPREDVAR` and is not this first-occasion
//! observed extra; `MANIFESTVAR` `θ` is not this extra),
//! recovers the Driver Eq. 5 of §7.2 `addedTIPREDVAR` as
//! `λ² (B / a)² v`
//! (JSS PDF re-opened 2026-08-23T19:23Z; 2017-era
//! `summary.ctsemFit.R` forms `addedTIPREDVAR` as
//! `asymTIPREDEFFECT %*% TIPREDVAR %*% t(asymTIPREDEFFECT)`; form
//! `(B / a)² v` first, then `(λ extra) λ` with `θ = 0`; a zero
//! loading or zero extra is exactly zero; lasting asymptotic extra
//! requires `a < 0`; `(B / a)² v` is the latent extra and is not
//! this observed extra; `λ² t0_b² v` is Eq. 5 of `addedT0TIPREDVAR`
//! and is not this extra; `λ² p + θ` is stationary observed
//! variance and is not this extra; `MANIFESTVAR` `θ` is not this
//! extra),
//! recovers the Driver p. 16 `TDPREDEFFECTstd` as
//! `m · √v / √(-q / (2 a))` after forming strictly positive
//! `asymDIFFUSION` and strictly positive predictor variance
//! (JSS PDF re-opened 2026-08-23T21:10Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is time-dependent predictor variance `v`; the affected
//! variance is `asymDIFFUSION`; unstandardised `M` is defined for a
//! zero coefficient and for zero predictor variance, and is not
//! `TDPREDEFFECTstd`; zero `asymDIFFUSION` or zero `v` fails
//! closed; `TIPREDEFFECTstd` `B · √v / √p` is not
//! `TDPREDEFFECTstd` even when `M = B`; the finite-interval
//! intercept-style standardisation
//! `A^{-1}[e^{A Δt} − I] M · √v / √p` is not `TDPREDEFFECTstd`;
//! `m · √v / √(trait + p + added)` uses `TRAITVAR` and is
//! not `TDPREDEFFECTstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver 2017-era `addedTIPREDVARstd` as
//! `extra / extra = 1` after strictly positive extra
//! (JSS PDF re-opened 2026-08-23T21:22Z; 2017-era
//! `summary.ctsemFit.R` forms
//! `solve(sqrt(diag(addedTIPREDVAR))) %&% addedTIPREDVAR`;
//! `OpenMx` `%&%` is the quadratic form; form `(B / a)² v`
//! first, then the ratio; zero extra fails closed; the
//! default `ridging = FALSE` does not add `0.0001`;
//! unstandardised `(B / a)² v` is not this correlation;
//! `λ² (B / a)² v` is Eq. 5 of the extra and is not this
//! correlation; `t0_b² v` is `addedT0TIPREDVAR` and is not
//! this asymptotic extra correlation; `TRAITVAR` is not
//! the standardisation variance; the printed 2-latent
//! `addedTIPREDVAR` 2.838 is not this scalar 1),
//! recovers the Driver Table 3 / p. 16 `T0TDPREDEFFECTstd` as
//! `t0_m · √v / √p_0` after forming strictly positive free `T0VAR`
//! and strictly positive time-dependent predictor variance
//! (JSS PDF re-opened 2026-08-23T21:34Z; footnote 4 standardises
//! using only the relevant variance, not the total; the affecting
//! variance is TD predictor variance `v`, not `TIPREDVAR`; the
//! affected variance is free first-occasion `T0VAR` `p_0`, not
//! `asymDIFFUSION`; unstandardised `t0_m` is defined for a zero
//! coefficient and for zero predictor variance, and is not
//! `T0TDPREDEFFECTstd`; zero `p_0` or zero `v` fails closed; `T0`
//! is event time, so a non-event clock fails closed; free `T0VAR`
//! does not require stable `a < 0`; `m · √v / √(-q / (2 a))` is
//! not `T0TDPREDEFFECTstd`; `t0_b · √v / √p_0` is not
//! `T0TDPREDEFFECTstd` even when `t0_m = t0_b`;
//! `t0_m · √v / √(trait + p_0 + added)` uses `TRAITVAR` and is
//! not `T0TDPREDEFFECTstd`; `TRAITVAR` is not the standardisation
//! variance),
//! recovers the Driver p. 16 `T0VARstd` as the correlation form
//! `solve(sqrt(diag(T0VAR))) %&% T0VAR` after forming strictly
//! positive free `T0VAR` (JSS PDF re-opened 2026-08-23T22:06Z;
//! 2017-era `summary.ctsemFit.R` forms that quadratic when
//! `verbose = TRUE`; `OpenMx` `%&%` is `t(A) %*% B %*% A`; the
//! default ridge is 0; the scalar map is `p_0 / p_0 = 1`;
//! unstandardised `p_0` is defined for a zero first-occasion
//! variance and is not `T0VARstd`; zero `p_0` fails closed; `T0`
//! is event time, so a non-event clock fails closed; free `T0VAR`
//! does not require stable `a < 0`; distinct positive `p_0`
//! recover the same 1; `t0_m · √v / √p_0` is not `T0VARstd`;
//! `t0_b² v` is not `T0VARstd`; `TRAITVAR` is not the
//! standardisation variance),
//! recovers the scalar analog of 2017-era `addedT0TIPREDVAR` for
//! the first-occasion TD coefficient as `t0_m² v` (JSS PDF
//! re-opened 2026-08-23T22:13Z; 2017-era `summary.ctsemFit.R` forms
//! `addedT0TIPREDVAR` as
//! `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` and comments
//! out `TDPREDVAR`; it does not form `addedT0TDPREDVAR`; Table 2
//! names `T0TDPREDCOV` the first-occasion covariance, not this
//! extra; Table 3 names `T0TIPREDEFFECT`, not a TD first-occasion
//! effect matrix; form `t0_m` first, then square, then multiply by
//! `v`; a zero coefficient or zero predictor variance is exactly
//! zero; `v < 0` fails closed; `T0` is event time, so a non-event
//! clock fails closed; free `t0_m` does not require stable
//! `a < 0`; `t0_b² v` is `addedT0TIPREDVAR` and is not this extra
//! even when `t0_m = t0_b`; `t0_m · √v / √p_0` is
//! `T0TDPREDEFFECTstd` and is not this variance; `T0TDPREDCOV` is
//! the covariance, not `t0_m² v`; free `T0VAR` is not this extra;
//! `TRAITVAR` is not this extra),
//! recovers the Driver p. 16 `TRAITVARstd` as the correlation form
//! `solve(sqrt(diag(TRAITVAR))) %&% TRAITVAR` after forming
//! strictly positive `TRAITVAR` (JSS PDF re-opened 2026-08-23T22:21Z;
//! Table 2, p. 12; §7.1, pp. 18–19; 2017-era `summary.ctsemFit.R`
//! forms that quadratic only when `TRAITVAR != 0` and `verbose =
//! TRUE`; `OpenMx` `%&%` is `t(A) %*% B %*% A`; unlike `T0VARstd`
//! there is no ridge addend; the scalar map is `trait / trait = 1`;
//! unstandardised `TRAITVAR` is defined for a zero trait and is
//! not `TRAITVARstd`; zero `TRAITVAR` fails closed; a non-event
//! clock fails closed; `TRAITVAR` does not require stable `a < 0`;
//! distinct positive `trait` recover the same 1; `T0VARstd`
//! recovers the same number and remains a distinct named quantity;
//! `t0_b² v` is not `TRAITVARstd`),
//! recovers the Driver p. 16 `MANIFESTTRAITVARstd` as the
//! correlation form `solve(sqrt(diag(MANIFESTTRAITVAR))) %&%
//! MANIFESTTRAITVAR` after forming strictly positive
//! `MANIFESTTRAITVAR` (JSS PDF re-opened 2026-08-23T22:28Z; Table 2,
//! p. 12; §7.1, p. 19; 2017-era `summary.ctsemFit.R` forms that
//! quadratic only when `MANIFESTTRAITVAR != 0` and `verbose = TRUE`;
//! `OpenMx` `%&%` is `t(A) %*% B %*% A`; unlike `TRAITVARstd` the
//! 2017-era source adds `diag(c(ridging), n.manifest)`; the default
//! ridge is 0 and is not this exact map; the scalar map is
//! `ψ / ψ = 1`; unstandardised `Ψ_τ` is defined for a zero
//! manifest trait and is not `MANIFESTTRAITVARstd`; zero
//! `MANIFESTTRAITVAR` fails closed; a non-event clock fails
//! closed; `MANIFESTTRAITVAR` does not require stable `a < 0`;
//! distinct positive `ψ` recover the same 1; `TRAITVARstd`
//! recovers the same number and remains a distinct named quantity;
//! `θ` is not `MANIFESTTRAITVARstd`),
//! recovers Eq. 5 of that analog extra as `λ² t0_m² v` (JSS PDF
//! re-opened 2026-08-23T22:26Z; form `t0_m² v` first, then
//! `(λ extra) λ` with `θ = 0`; a zero loading or zero extra is
//! exactly zero; `v < 0` fails closed; `T0` is event time, so a
//! non-event clock fails closed; free `t0_m` does not require
//! stable `a < 0`; `t0_m² v` is the latent extra, not this observed
//! extra; `λ² p_0 + θ` is first-occasion observed variance, not
//! this extra; `λ² t0_b² v` is Eq. 5 of `addedT0TIPREDVAR` and is
//! not this extra even when `t0_m = t0_b`; `MANIFESTVAR` `θ` is
//! not this extra),
//! recovers the Driver p. 16 `MANIFESTVARstd` as the
//! correlation form `solve(sqrt(diag(MANIFESTVAR))) %&%
//! MANIFESTVAR` after forming strictly positive `MANIFESTVAR`
//! (JSS PDF re-opened 2026-08-23T22:40Z; Table 2, p. 12; Eq. 5,
//! p. 5; 2017-era `summary.ctsemFit.R` forms that quadratic whenever
//! `verbose = TRUE`; `OpenMx` `%&%` is `t(A) %*% B %*% A`; unlike
//! `TRAITVARstd` the 2017-era source adds
//! `diag(c(ridging), n.manifest)`; the default ridge is 0 and is
//! not this exact map; the 2017-era `dimnames` assignment to
//! `latentNames` is a source bug and is not this exact map; the
//! scalar map is `θ / θ = 1`; unstandardised `Θ` is defined for a
//! zero residual and is not `MANIFESTVARstd`; zero `MANIFESTVAR`
//! makes `solve(sqrt(0))` fail and fails closed; a non-event clock
//! fails closed; `MANIFESTVAR` does not require stable `a < 0`;
//! distinct positive `θ` recover the same 1; `MANIFESTTRAITVARstd`
//! recovers the same number and remains a distinct named quantity;
//! `λ² Var(η) + θ` is not `MANIFESTVARstd`),
//! recovers the Driver p. 16 `TIPREDVARstd` as the
//! correlation form `solve(sqrt(diag(TIPREDVAR))) %&%
//! TIPREDVAR` after forming strictly positive `TIPREDVAR`
//! (JSS PDF re-opened 2026-08-23T22:53Z; Table 2, p. 12;
//! 2017-era `summary.ctsemFit.R` forms that quadratic whenever
//! `verbose = TRUE` and `n.TIpred > 0`; `OpenMx` `%&%` is
//! `t(A) %*% B %*% A`; unlike `TRAITVARstd` the 2017-era source
//! adds `diag(c(ridging), n.TIpred)`; the default ridge is 0 and
//! is not this exact map; `dimnames` are `TIpredNames`; the scalar
//! map is `v / v = 1`; unstandardised `v` is defined for a zero
//! predictor and is not `TIPREDVARstd`; zero `TIPREDVAR` makes
//! `solve(sqrt(0))` fail and fails closed; a non-event clock
//! fails closed; `TIPREDVAR` does not require stable `a < 0`;
//! distinct positive `v` recover the same 1; `MANIFESTVARstd`
//! recovers the same number and remains a distinct named quantity;
//! `(B / a)² v` is not `TIPREDVARstd`),
//! recovers the Driver p. 16 `asymDIFFUSIONstd` as the
//! correlation form `solve(sqrt(diag(asymDIFFUSION))) %&%
//! asymDIFFUSION` after forming strictly positive `asymDIFFUSION`
//! (JSS PDF re-opened 2026-08-23T23:02Z; p. 16; footnote 4; Eq. 4;
//! 2017-era `summary.ctsemFit.R` forms that quadratic whenever
//! `verbose = TRUE`; `OpenMx` `%&%` is `t(A) %*% B %*% A`; the
//! 2017-era source adds `diag(c(ridging), n.latent)`; the default
//! ridge is 0 and is not this exact map; `dimnames` are
//! `latentNames`; the scalar map is `p / p = 1` after
//! `p = −q / (2 a)`; unstandardised `p` is defined for a zero
//! process and is not `asymDIFFUSIONstd`; zero `q` makes
//! `solve(sqrt(0))` fail and fails closed; a non-event clock
//! fails closed; `a ≥ 0` fails closed; distinct positive `p`
//! recover the same 1; `TIPREDVARstd` recovers the same number
//! and remains a distinct named quantity; `DIFFUSIONstd`
//! `−2 a` is not `asymDIFFUSIONstd`),
//! recovers the Driver p. 16 `discreteCINTstd` as
//! `A^{-1}[e^{A Δt} − I] κ / √p` after forming strictly
//! positive `asymDIFFUSION` (JSS PDF re-opened 2026-08-24T05:20Z;
//! p. 16; footnote 4; Eq. 3; Table 2; 2017-era
//! `summary.ctsemFit.R` forms unstandardised `discreteCINT`
//! whenever `verbose = TRUE`; that source does not form a
//! `discreteCINTstd` matrix; the scalar map is the footnote 4
//! standardisation of that named discrete intercept; unstandardised
//! `discreteCINT` is defined for growing `a ≥ 0` and for zero
//! diffusion and is not `discreteCINTstd`; zero `q` fails closed;
//! a non-event clock fails closed; a non-positive event interval
//! fails closed; `a ≥ 0` fails closed; `κ / √p` does not depend
//! on `Δt` and is not this map; `(-κ / a) / √p` is not this map),
//! recovers the Driver p. 16 `asymCINTstd` as
//! `(-κ / a) / √p` after forming strictly
//! positive `asymDIFFUSION` (JSS PDF re-opened 2026-08-24T09:05Z;
//! p. 16; footnote 4; Eq. 3; Table 2; 2017-era
//! `summary.ctsemFit.R` forms unstandardised `asymCINT`
//! whenever `verbose = TRUE` as `-solve(DRIFT) %*% CINT`; that
//! source does not form an `asymCINTstd` matrix; the scalar map
//! is the footnote 4 standardisation of that named asymptotic
//! intercept; unstandardised `asymCINT` is defined for a zero
//! process and is not `asymCINTstd`; zero `q` fails closed;
//! a non-event clock fails closed; `a ≥ 0` fails closed;
//! `κ / √p` is not this total-change map; `discreteCINTstd`
//! depends on `Δt` and is not this map),
//! recovers the Driver p. 16 `T0MEANSstd` as
//! `μ_0 / √p_0` after forming strictly
//! positive free `T0VAR` (JSS PDF re-opened 2026-08-24T22:30Z;
//! p. 16; footnote 4; Table 2; 2017-era
//! `summary.ctsemFit.R` forms unstandardised `T0MEANS`
//! as `OpenMx::mxEval(T0MEANS, mxobj, compute=TRUE)`; that
//! source does not form a `T0MEANSstd` matrix; the scalar map
//! is the footnote 4 standardisation of that named first-occasion
//! mean; unstandardised `T0MEANS` is defined for a zero
//! first-occasion variance and is not `T0MEANSstd`; zero `p_0`
//! fails closed; a non-event clock fails closed; free `T0MEANS`
//! does not require `a < 0`; `T0VARstd` recovers the same
//! number when `μ_0 = √p_0` and remains a distinct named
//! quantity; `μ_0 / √asymDIFFUSION` uses process-dynamics
//! variance and is not this map),
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

/// Refuse a causal-effect claim from a non-identifying heuristic.
pub use causality::claim_causal_effect;
/// A heuristic that is not causal identification.
pub use causality::CausalHeuristic;
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
/// One clustered predictor–outcome pair.
pub use cluster_mean::ClusteredScore;
/// Recovered within-cluster, between-cluster, and contextual OLS slopes.
pub use cluster_mean::WithinBetweenSlopes;
/// Permit latent-mean comparison only with invariance evidence.
pub use construct::compare_latent_means;
/// Refuse fit-driven reinterpretation as reflective.
pub use construct::interpret_as_reflective;
/// Higher-order construct class.
pub use construct::ConstructClass;
/// Fail-closed psychometric errors.
pub use error::PsychometricError;
/// Map a discrete lag onto another event interval through the exact log-rate.
pub use event_time::map_discrete_lag_across_event_intervals;
/// Exact scalar Table 2 `asymCINT` `-κ / a`.
pub use event_time::recover_asymptotic_continuous_intercept;
/// Exact scalar Eq. 5 of §7.2 `addedTIPREDVAR` `λ² (B / a)² v`.
pub use event_time::recover_asymptotic_time_independent_observed_variance;
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
/// Exact scalar Eq. 5 of the analog first-occasion TD extra `λ² t0_m² v`.
pub use event_time::recover_initial_time_dependent_observed_variance;
/// Exact scalar carried first-occasion `T0TDPREDEFFECT` `e^{A Δt} t0_m x0`.
pub use event_time::recover_initial_time_dependent_predictor_carry;
/// Exact scalar first-occasion `T0TDPREDEFFECT` shift `t0_m x0`.
pub use event_time::recover_initial_time_dependent_predictor_effect;
/// Exact scalar analog of 2017-era `addedT0TIPREDVAR` for the first-occasion TD coefficient `t0_m² v`.
pub use event_time::recover_initial_time_dependent_predictor_variance;
/// Exact scalar Eq. 5 of 2017-era `addedT0TIPREDVAR` `λ² t0_b² v`.
pub use event_time::recover_initial_time_independent_observed_variance;
/// Exact scalar carried first-occasion `T0TIPREDEFFECT` `e^{A Δt} t0_b z`.
pub use event_time::recover_initial_time_independent_predictor_carry;
/// Exact scalar first-occasion `T0TIPREDEFFECT` shift `t0_b z`.
pub use event_time::recover_initial_time_independent_predictor_effect;
/// Exact scalar 2017-era `addedT0TIPREDVAR` `t0_b² v`.
pub use event_time::recover_initial_time_independent_predictor_variance;
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
/// Exact scalar first-occasion variance of §4.3 predetermined `T0VAR` `trait + p_0 + (B / a)² v`.
pub use event_time::recover_predetermined_initial_latent_variance;
/// Exact scalar Eq. 5 of first-occasion §4.3 predetermined `T0VAR` `λ²(trait + p_0 + (B / a)² v) + θ + ψ`.
pub use event_time::recover_predetermined_initial_observed_variance;
/// Exact scalar lagged covariance of §4.3 predetermined `T0VAR` `trait + e^{a Δt} p_0 + (B / a)² v`.
pub use event_time::recover_predetermined_lagged_latent_covariance;
/// Exact scalar Eq. 5 of lagged §4.3 predetermined `T0VAR` `λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ`.
pub use event_time::recover_predetermined_lagged_observed_covariance;
/// Exact scalar later-start lagged covariance of §4.3 predetermined `T0VAR` `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v`.
pub use event_time::recover_predetermined_later_lagged_latent_covariance;
/// Exact scalar Eq. 5 of later-start lagged §4.3 predetermined `T0VAR` `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`.
pub use event_time::recover_predetermined_later_lagged_observed_covariance;
/// Exact scalar later-occasion variance of §4.3 predetermined `T0VAR` `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v`.
pub use event_time::recover_predetermined_later_latent_variance;
/// Exact scalar Eq. 5 of later-occasion §4.3 predetermined `T0VAR` `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`.
pub use event_time::recover_predetermined_later_observed_variance;
/// Exact scalar later-start later-occasion variance of §4.3 predetermined `T0VAR` `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v`.
pub use event_time::recover_predetermined_later_start_later_latent_variance;
/// Exact scalar Eq. 5 of later-start later-occasion §4.3 predetermined `T0VAR` `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`.
pub use event_time::recover_predetermined_later_start_later_observed_variance;
/// Exact scalar p. 16 `asymCINTstd` `(-κ / a) / √p` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_asymptotic_continuous_intercept;
/// Exact scalar p. 16 `asymDIFFUSIONstd` `p / p = 1` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_asymptotic_diffusion;
/// Exact scalar p. 16 `asymTIPREDEFFECTstd` `(-B / a) · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and `v`.
pub use event_time::recover_standardised_asymptotic_time_independent_predictor_effect;
/// Exact scalar 2017-era `addedTIPREDVARstd` `extra / extra = 1` after strictly positive extra.
pub use event_time::recover_standardised_asymptotic_time_independent_predictor_variance;
/// Exact scalar p. 16 `DIFFUSIONstd` `q / (−q / (2 a)) = −2 a` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_continuous_diffusion;
/// Exact scalar p. 16 `DRIFTstd` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_continuous_drift;
/// Exact scalar p. 16 `CINTstd` `κ / √(-q / (2 a))` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_continuous_intercept;
/// Exact scalar p. 16 `TDPREDEFFECTstd` `m · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and `v`.
pub use event_time::recover_standardised_continuous_time_dependent_predictor_effect;
/// Exact scalar p. 16 `TIPREDEFFECTstd` `B · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and `v`.
pub use event_time::recover_standardised_continuous_time_independent_predictor_effect;
/// Exact scalar p. 16 `discreteCINTstd` `A^{-1}[e^{A Δt} − I] κ / √p` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_discrete_continuous_intercept;
/// Exact scalar p. 16 `discreteDIFFUSIONstd` `Q_Δt / (−q / (2 a))` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_discrete_diffusion;
/// Exact scalar p. 16 `discreteDRIFTstd` `e^{a Δt}` after strictly positive `asymDIFFUSION`.
pub use event_time::recover_standardised_discrete_drift;
/// Exact scalar p. 16 finite-interval standardised `TIPREDEFFECT` `A^{-1}[e^{A Δt} − I] B · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION` and `v`.
pub use event_time::recover_standardised_discrete_time_independent_predictor_effect;
/// Exact scalar Table 3 / p. 16 `T0TDPREDEFFECTstd` `t0_m · √v_x / √p_0` after strictly positive free `T0VAR` and `TDPREDVAR`.
/// Exact scalar p. 16 `T0MEANSstd` `μ_0 / √p_0` after strictly positive free `T0VAR`.
pub use event_time::recover_standardised_initial_latent_mean;
/// Exact scalar p. 16 `T0VARstd` `p_0 / p_0 = 1` after strictly positive free `T0VAR`.
pub use event_time::recover_standardised_initial_latent_variance;
/// Exact scalar Table 3 / p. 16 `T0TDPREDEFFECTstd` `t0_m · √v / √p_0` after strictly positive free `T0VAR` and `v`.
pub use event_time::recover_standardised_initial_time_dependent_predictor_effect;
/// Exact scalar Table 3 / p. 16 `T0TIPREDEFFECTstd` `t0_b · √v / √p_0` after strictly positive free `T0VAR` and `v`.
pub use event_time::recover_standardised_initial_time_independent_predictor_effect;
/// Exact scalar p. 16 `MANIFESTTRAITVARstd` `ψ / ψ = 1` after strictly positive `MANIFESTTRAITVAR`.
pub use event_time::recover_standardised_manifest_trait_variance;
/// Exact scalar p. 16 `MANIFESTVARstd` `θ / θ = 1` after strictly positive `MANIFESTVAR`.
pub use event_time::recover_standardised_manifest_variance;
/// Exact scalar p. 16 `TIPREDVARstd` `v / v = 1` after strictly positive `TIPREDVAR`.
pub use event_time::recover_standardised_time_independent_predictor_variance;
/// Exact scalar p. 16 `TRAITVARstd` `trait / trait = 1` after strictly positive `TRAITVAR`.
pub use event_time::recover_standardised_trait_variance;
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
/// Refuse treating Table 2 `asymCINT` `/ √p` as p. 16 `discreteCINTstd`.
pub use event_time::refuse_asymptotic_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `TIPREDEFFECT` `B`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_coefficient;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `CINT`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_continuous_intercept;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as the finite-interval discrete increment.
pub use event_time::refuse_asymptotic_time_independent_effect_as_discrete_effect;
/// Refuse treating §7.2 `asymTIPREDEFFECT` as `M x`.
pub use event_time::refuse_asymptotic_time_independent_effect_as_time_dependent_impulse;
/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as the latent extra.
pub use event_time::refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance;
/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as Eq. 5 of `addedT0TIPREDVAR`.
pub use event_time::refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance;
/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as `MANIFESTVAR`.
pub use event_time::refuse_asymptotic_time_independent_observed_variance_as_measurement_error;
/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as 2017-era `addedTIPREDVARstd`.
pub use event_time::refuse_asymptotic_time_independent_observed_variance_as_standardised_asymptotic_time_independent_variance;
/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as stationary observed variance.
pub use event_time::refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance;
/// Refuse treating §7.2 `addedTIPREDVAR` as p. 16 `TIPREDVARstd`.
pub use event_time::refuse_asymptotic_time_independent_predictor_variance_as_standardised_time_independent_predictor_variance;
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
/// Refuse treating Eq. 5 of the analog first-occasion TD extra as first-occasion observed variance.
pub use event_time::refuse_initial_time_dependent_observed_variance_as_initial_observed_variance;
/// Refuse treating Eq. 5 of the analog first-occasion TD extra as the latent extra.
pub use event_time::refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance;
/// Refuse treating Eq. 5 of the analog first-occasion TD extra as Eq. 5 of `addedT0TIPREDVAR`.
pub use event_time::refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance;
/// Refuse treating Eq. 5 of the analog first-occasion TD extra as `MANIFESTVAR`.
pub use event_time::refuse_initial_time_dependent_observed_variance_as_measurement_error;
/// Refuse treating the first-occasion TD extra as free first-occasion `T0VAR`.
pub use event_time::refuse_initial_time_dependent_variance_as_initial_latent_variance;
/// Refuse treating the first-occasion TD extra as Table 2 `T0TDPREDCOV`.
pub use event_time::refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance;
/// Refuse treating the first-occasion TD extra as 2017-era `addedT0TIPREDVAR`.
pub use event_time::refuse_initial_time_dependent_variance_as_initial_time_independent_variance;
/// Refuse treating the first-occasion TD extra as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect;
/// Refuse treating the first-occasion TD extra as `TRAITVAR`.
pub use event_time::refuse_initial_time_dependent_variance_as_trait_variance;
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
/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as Eq. 5 of `addedTIPREDVAR`.
pub use event_time::refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance;
/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as first-occasion observed variance.
pub use event_time::refuse_initial_time_independent_observed_variance_as_initial_observed_variance;
/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as the latent extra.
pub use event_time::refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance;
/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as `MANIFESTVAR`.
pub use event_time::refuse_initial_time_independent_observed_variance_as_measurement_error;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as §7.2 `addedTIPREDVAR`.
pub use event_time::refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as free first-occasion `T0VAR`.
pub use event_time::refuse_initial_time_independent_variance_as_initial_latent_variance;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as 2017-era `addedTIPREDVARstd`.
pub use event_time::refuse_initial_time_independent_variance_as_standardised_asymptotic_time_independent_variance;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as p. 16 `T0VARstd`.
pub use event_time::refuse_initial_time_independent_variance_as_standardised_initial_latent_variance;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as Table 3 / p. 16 `T0TIPREDEFFECTstd`.
pub use event_time::refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as p. 16 `TRAITVARstd`.
pub use event_time::refuse_initial_time_independent_variance_as_standardised_trait_variance;
/// Refuse treating 2017-era `addedT0TIPREDVAR` as `TRAITVAR`.
pub use event_time::refuse_initial_time_independent_variance_as_trait_variance;
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
/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined first-occasion `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_initial_observed_variance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined lagged `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_lagged_observed_covariance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-start lagged predetermined `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_later_lagged_observed_covariance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined later-occasion `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_later_observed_variance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-start later-occasion predetermined `T0VAR`.
pub use event_time::refuse_measurement_error_as_predetermined_later_start_later_observed_variance;
/// Refuse treating `MANIFESTVAR` as p. 16 `MANIFESTTRAITVARstd`.
pub use event_time::refuse_measurement_error_as_standardised_manifest_trait_variance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of lagged §4.3 stationary `T0VAR`.
pub use event_time::refuse_measurement_error_as_stationary_lagged_observed_covariance;
/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-occasion §4.3 stationary `T0VAR`.
pub use event_time::refuse_measurement_error_as_stationary_later_observed_variance;
/// Refuse treating Driver Eq. 5 `Var(y)` as p. 16 `MANIFESTVARstd`.
pub use event_time::refuse_observed_variance_as_standardised_manifest_variance;
/// Refuse pooling discrete lags from unequal event intervals.
pub use event_time::refuse_pooled_discrete_lag_across_unequal_intervals;
/// Refuse treating predetermined first-occasion variance as free first-occasion `T0VAR`.
pub use event_time::refuse_predetermined_initial_latent_variance_as_initial_latent_variance;
/// Refuse treating predetermined first-occasion variance as predetermined lagged covariance.
pub use event_time::refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance;
/// Refuse treating predetermined first-occasion variance as predetermined later-occasion variance.
pub use event_time::refuse_predetermined_initial_latent_variance_as_later_latent_variance;
/// Refuse treating predetermined first-occasion variance as predetermined first-occasion observed variance.
pub use event_time::refuse_predetermined_initial_latent_variance_as_observed_variance;
/// Refuse treating predetermined first-occasion variance as stationary first-occasion `T0VAR`.
pub use event_time::refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance;
/// Refuse treating predetermined lagged covariance as the decayed total.
pub use event_time::refuse_predetermined_lagged_latent_covariance_as_decayed_total;
/// Refuse treating predetermined lagged covariance as free first-occasion `T0VAR`.
pub use event_time::refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance;
/// Refuse treating predetermined lagged covariance as predetermined later-occasion variance.
pub use event_time::refuse_predetermined_lagged_latent_covariance_as_later_latent_variance;
/// Refuse treating predetermined lagged covariance as predetermined lagged observed covariance.
pub use event_time::refuse_predetermined_lagged_latent_covariance_as_observed_covariance;
/// Refuse treating predetermined lagged covariance as lagged stationary `T0VAR`.
pub use event_time::refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance;
/// Refuse treating Eq. 5 of first-occasion lagged predetermined `T0VAR` as later-start lagged observed covariance.
pub use event_time::refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance;
/// Refuse treating later-start lagged covariance as the decayed later total.
pub use event_time::refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total;
/// Refuse treating later-start lagged covariance as later-occasion variance of predetermined `T0VAR`.
pub use event_time::refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance;
/// Refuse treating later-start lagged covariance as later-start lagged observed covariance.
pub use event_time::refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance;
/// Refuse treating later-start lagged covariance as first-occasion lagged covariance of predetermined `T0VAR`.
pub use event_time::refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance;
/// Refuse treating later-start lagged covariance as lagged stationary `T0VAR`.
pub use event_time::refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance;
/// Refuse treating Eq. 5 of later-start lagged predetermined `T0VAR` as later-start later-occasion observed variance.
pub use event_time::refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance;
/// Refuse treating predetermined later-occasion variance as the free discrete evolution of the total.
pub use event_time::refuse_predetermined_later_latent_variance_as_discrete_variance;
/// Refuse treating predetermined later-occasion variance as free first-occasion `T0VAR`.
pub use event_time::refuse_predetermined_later_latent_variance_as_initial_latent_variance;
/// Refuse treating predetermined later-occasion variance as predetermined later-occasion observed variance.
pub use event_time::refuse_predetermined_later_latent_variance_as_observed_variance;
/// Refuse treating predetermined later-occasion variance as later-occasion stationary `T0VAR`.
pub use event_time::refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance;
/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as predetermined first-occasion observed variance.
pub use event_time::refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance;
/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as predetermined lagged observed covariance.
pub use event_time::refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance;
/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as later-start lagged observed covariance.
pub use event_time::refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance;
/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as later-start later-occasion observed variance.
pub use event_time::refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance;
/// Refuse treating later-start later-occasion variance as the evolved later total.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total;
/// Refuse treating later-start later-occasion variance as later-occasion variance over the lag interval alone.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance;
/// Refuse treating later-start later-occasion variance as later-start lagged covariance of predetermined `T0VAR`.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance;
/// Refuse treating later-start later-occasion variance as later-occasion variance at the later start.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance;
/// Refuse treating later-start later-occasion variance as later-start later-occasion observed variance.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_observed_variance;
/// Refuse treating later-start later-occasion variance as later-occasion stationary `T0VAR`.
pub use event_time::refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance;
/// Refuse treating Driver Eq. 3 process noise as the unconditional variance.
pub use event_time::refuse_process_noise_as_unconditional_variance;
/// Refuse treating p. 16 `asymTIPREDEFFECTstd` as the finite-interval standardised `TIPREDEFFECT`.
pub use event_time::refuse_standardised_asymptotic_time_independent_effect_as_standardised_discrete_time_independent_effect;
/// Refuse treating p. 16 `asymCINTstd` as p. 16 `CINTstd`.
pub use event_time::refuse_standardised_asymptotic_continuous_intercept_as_standardised_continuous_intercept;
/// Refuse treating p. 16 `asymTIPREDEFFECTstd` as p. 16 `TIPREDEFFECTstd`.
pub use event_time::refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect;
/// Refuse treating p. 16 `asymTIPREDEFFECTstd` as Table 3 / p. 16 `T0TIPREDEFFECTstd`.
pub use event_time::refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect;
/// Refuse treating p. 16 `DIFFUSIONstd` `−2 a` as p. 16 `asymDIFFUSIONstd`.
pub use event_time::refuse_standardised_continuous_diffusion_as_standardised_asymptotic_diffusion;
/// Refuse treating continuous `DIFFUSION` standardisation `−2 a` as p. 16 `discreteDIFFUSIONstd`.
pub use event_time::refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion;
/// Refuse treating a finite-interval standardised `CINT` as p. 16 `CINTstd`.
pub use event_time::refuse_standardised_discrete_continuous_intercept_as_standardised_continuous_intercept;
/// Refuse treating p. 16 `TIPREDEFFECTstd` as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_dependent_effect;
/// Refuse treating `κ / √p` as p. 16 `asymCINTstd`.
pub use event_time::refuse_standardised_continuous_intercept_as_standardised_asymptotic_continuous_intercept;
/// Refuse treating `κ / √p` as p. 16 `discreteCINTstd`.
pub use event_time::refuse_standardised_continuous_intercept_as_standardised_discrete_continuous_intercept;
/// Refuse treating p. 16 `TDPREDEFFECTstd` as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect;
/// Refuse treating p. 16 `TIPREDEFFECTstd` as p. 16 `TDPREDEFFECTstd`.
pub use event_time::refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect;
/// Refuse treating p. 16 `TIPREDEFFECTstd` as Table 3 / p. 16 `T0TIPREDEFFECTstd`.
pub use event_time::refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect;
/// Refuse treating p. 16 `discreteCINTstd` as p. 16 `asymCINTstd`.
pub use event_time::refuse_standardised_discrete_continuous_intercept_as_standardised_asymptotic_continuous_intercept;
/// Refuse treating p. 16 `discreteDIFFUSIONstd` `1 − exp(2 a Δt)` as p. 16 `DIFFUSIONstd`.
pub use event_time::refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion;
/// Refuse treating p. 16 `discreteDRIFTstd` `e^{a Δt}` as p. 16 `DRIFTstd`.
pub use event_time::refuse_standardised_discrete_drift_as_standardised_continuous_drift;
/// Refuse treating intercept-style standardised `TDPREDEFFECT` as p. 16 `TDPREDEFFECTstd`.
pub use event_time::refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect;
/// Refuse treating a finite-interval standardised `TIPREDEFFECT` as p. 16 `asymTIPREDEFFECTstd`.
pub use event_time::refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect;
/// Refuse treating a finite-interval standardised `TIPREDEFFECT` as p. 16 `TIPREDEFFECTstd`.
pub use event_time::refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect;
/// Refuse treating p. 16 `T0VARstd` as p. 16 `T0MEANSstd`.
pub use event_time::refuse_standardised_initial_latent_variance_as_standardised_initial_latent_mean;
/// Refuse treating p. 16 `T0VARstd` as p. 16 `TRAITVARstd`.
pub use event_time::refuse_standardised_initial_latent_variance_as_standardised_trait_variance;
/// Refuse treating Table 3 / p. 16 `T0TDPREDEFFECTstd` as p. 16 `T0VARstd`.
pub use event_time::refuse_standardised_initial_time_dependent_effect_as_standardised_initial_latent_variance;
/// Refuse treating Table 3 / p. 16 `T0TIPREDEFFECTstd` as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect;
/// Refuse treating p. 16 `MANIFESTTRAITVARstd` as p. 16 `MANIFESTVARstd`.
pub use event_time::refuse_standardised_manifest_trait_variance_as_standardised_manifest_variance;
/// Refuse treating p. 16 `MANIFESTVARstd` as p. 16 `TIPREDVARstd`.
pub use event_time::refuse_standardised_manifest_variance_as_standardised_time_independent_predictor_variance;
/// Refuse treating p. 16 `TIPREDVARstd` as p. 16 `asymDIFFUSIONstd`.
pub use event_time::refuse_standardised_time_independent_predictor_variance_as_standardised_asymptotic_diffusion;
/// Refuse treating p. 16 `TRAITVARstd` as p. 16 `MANIFESTTRAITVARstd`.
pub use event_time::refuse_standardised_trait_variance_as_standardised_manifest_trait_variance;
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
/// Refuse treating Eq. 5 of §4.3 stationary `T0VAR` as predetermined first-occasion observed variance.
pub use event_time::refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance;
/// Refuse treating Eq. 5 of contemporaneous §4.3 stationary `T0VAR` as lagged observed covariance.
pub use event_time::refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as decayed total stationary variance.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as lagged observed covariance.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_observed_covariance;
/// Refuse treating lagged §4.3 stationary `T0VAR` as contemporaneous stationary `T0VAR`.
pub use event_time::refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance;
/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as predetermined lagged observed covariance.
pub use event_time::refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance;
/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as later-start lagged observed covariance of predetermined `T0VAR`.
pub use event_time::refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance;
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
/// Refuse treating Eq. 5 of later-occasion §4.3 stationary `T0VAR` as later-start later-occasion observed variance of predetermined `T0VAR`.
pub use event_time::refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance;
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
/// Refuse treating Driver §7.1 trait-contaminated asymptotic TI effect as p. 16 `asymTIPREDEFFECTstd`.
pub use event_time::refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect;
/// Refuse treating Driver §7.1 trait-contaminated continuous diffusion as p. 16 `DIFFUSIONstd`.
pub use event_time::refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion;
/// Refuse treating Driver §7.1 trait-contaminated continuous drift as p. 16 `DRIFTstd`.
pub use event_time::refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift;
/// Refuse treating Driver §7.1 trait-contaminated finite-interval TI effect as the p. 16 standardised finite-interval `TIPREDEFFECT`.
pub use event_time::refuse_trait_contaminated_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect;
/// Refuse treating Driver §7.1 trait-contaminated continuous intercept as p. 16 `CINTstd`.
pub use event_time::refuse_trait_contaminated_continuous_intercept_as_standardised_continuous_intercept;
/// Refuse treating Driver §7.1 trait-contaminated continuous TD effect as p. 16 `TDPREDEFFECTstd`.
pub use event_time::refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect;
/// Refuse treating Driver §7.1 trait-contaminated continuous TI effect as p. 16 `TIPREDEFFECTstd`.
pub use event_time::refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect;
/// Refuse treating Driver §7.1 trait-contaminated first-occasion TD effect as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect;
/// Refuse treating Driver §7.1 trait-contaminated first-occasion TI effect as Table 3 / p. 16 `T0TIPREDEFFECTstd`.
pub use event_time::refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect;
/// Refuse treating Driver §7.1 trait-contaminated process noise as p. 16 `discreteDIFFUSIONstd`.
pub use event_time::refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion;
/// Refuse treating Driver §7.1 trait-plus-state autocorrelation as p. 16 `discreteDRIFTstd`.
pub use event_time::refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift;
/// Refuse treating §4.3 trait-plus-state lagged covariance as lagged stationary `T0VAR`.
pub use event_time::refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance;
/// Refuse treating Driver §4.3 trait variance as process noise.
pub use event_time::refuse_trait_variance_as_process_noise;
/// Refuse treating Driver §4.3 trait variance as the p. 16 footnote 4 standardisation variance.
pub use event_time::refuse_trait_variance_as_standardisation_variance;
/// Refuse treating Driver §4.3 trait variance as `asymDIFFUSION`.
pub use event_time::refuse_trait_variance_as_stationary_within_subject;
/// Refuse a time-varying predictor whose sampling and constancy intervals differ.
pub use event_time::refuse_unmatched_time_varying_predictor_interval;
/// Refuse treating unstandardised `asymCINT` as p. 16 `asymCINTstd`.
pub use event_time::refuse_unstandardised_asymptotic_continuous_intercept_as_standardised_asymptotic_continuous_intercept;
/// Refuse treating unstandardised `asymDIFFUSION` as p. 16 `asymDIFFUSIONstd`.
pub use event_time::refuse_unstandardised_asymptotic_diffusion_as_standardised_asymptotic_diffusion;
/// Refuse treating unstandardised `asymTIPREDEFFECT` `-B / a` as p. 16 `asymTIPREDEFFECTstd`.
pub use event_time::refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect;
/// Refuse treating unstandardised `addedTIPREDVAR` `(B / a)² v` as 2017-era `addedTIPREDVARstd`.
pub use event_time::refuse_unstandardised_asymptotic_time_independent_variance_as_standardised_asymptotic_time_independent_variance;
/// Refuse treating unstandardised `DIFFUSION` as p. 16 `DIFFUSIONstd`.
pub use event_time::refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion;
/// Refuse treating unstandardised `DRIFT` as p. 16 `DRIFTstd`.
pub use event_time::refuse_unstandardised_continuous_drift_as_standardised_continuous_drift;
/// Refuse treating unstandardised `CINT` `κ` as p. 16 `CINTstd`.
pub use event_time::refuse_unstandardised_continuous_intercept_as_standardised_continuous_intercept;
/// Refuse treating unstandardised `TDPREDEFFECT` `M` as p. 16 `TDPREDEFFECTstd`.
pub use event_time::refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect;
/// Refuse treating unstandardised `TIPREDEFFECT` `B` as p. 16 `TIPREDEFFECTstd`.
pub use event_time::refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect;
/// Refuse treating unstandardised `discreteCINT` as p. 16 `discreteCINTstd`.
pub use event_time::refuse_unstandardised_discrete_continuous_intercept_as_standardised_discrete_continuous_intercept;
/// Refuse treating unstandardised `discreteDIFFUSION` as p. 16 `discreteDIFFUSIONstd`.
pub use event_time::refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion;
/// Refuse treating unstandardised `discreteDRIFT` as p. 16 `discreteDRIFTstd`.
pub use event_time::refuse_unstandardised_discrete_drift_as_standardised_discrete_drift;
/// Refuse treating unstandardised finite-interval `TIPREDEFFECT` `A^{-1}[e^{A Δt} − I] B` as the p. 16 standardised finite-interval map.
pub use event_time::refuse_unstandardised_discrete_time_independent_effect_as_standardised_discrete_time_independent_effect;
/// Indicator coordinate kind.
pub use indicator::IndicatorKind;
/// Refuse treating unstandardised `T0MEANS` as p. 16 `T0MEANSstd`.
pub use event_time::refuse_unstandardised_initial_latent_mean_as_standardised_initial_latent_mean;
/// Refuse treating unstandardised `T0VAR` as p. 16 `T0VARstd`.
pub use event_time::refuse_unstandardised_initial_latent_variance_as_standardised_initial_latent_variance;
/// Refuse treating unstandardised `T0TDPREDEFFECT` `t0_m` as Table 3 / p. 16 `T0TDPREDEFFECTstd`.
pub use event_time::refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect;
/// Refuse treating unstandardised `T0TIPREDEFFECT` `t0_b` as Table 3 / p. 16 `T0TIPREDEFFECTstd`.
pub use event_time::refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect;
/// Refuse treating unstandardised `MANIFESTTRAITVAR` as p. 16 `MANIFESTTRAITVARstd`.
pub use event_time::refuse_unstandardised_manifest_trait_variance_as_standardised_manifest_trait_variance;
/// Refuse treating unstandardised `MANIFESTVAR` as p. 16 `MANIFESTVARstd`.
pub use event_time::refuse_unstandardised_manifest_variance_as_standardised_manifest_variance;
/// Refuse treating unstandardised `TIPREDVAR` as p. 16 `TIPREDVARstd`.
pub use event_time::refuse_unstandardised_time_independent_predictor_variance_as_standardised_time_independent_predictor_variance;
/// Refuse treating unstandardised `TRAITVAR` as p. 16 `TRAITVARstd`.
pub use event_time::refuse_unstandardised_trait_variance_as_standardised_trait_variance;
/// Refuse treating `μ_0 / √asymDIFFUSION` as p. 16 `T0MEANSstd`.
pub use event_time::refuse_within_subject_scaled_initial_latent_mean_as_standardised_initial_latent_mean;
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
/// Pearson correlation on valid coordinates.
pub use indicator::pearson_correlation;
/// Refuse raw topic proportions as psychometric indicators.
pub use indicator::require_valid_indicator;
/// Indicator coordinate kind.
pub use indicator::IndicatorKind;
/// Classify two-group OLS invariance.
pub use latent_mean::classify_two_group_ols_invariance;
/// Strong/strict-gated latent-mean difference.
pub use latent_mean::recover_strong_gated_latent_mean_difference;
/// One group's factor-score and indicator series.
pub use latent_mean::GroupIndicatorSeries;
/// Two-group OLS invariance status for a mean comparison.
pub use latent_mean::MeanInvarianceStatus;
/// Two-group OLS measurement parameters and status.
pub use latent_mean::TwoGroupMeasurement;
/// Ordinary least-squares intercept and slope with residual variance.
pub use loading::ordinary_least_squares_fit;
/// Ordinary least-squares slope.
pub use loading::ordinary_least_squares_slope;
/// Recover one reflective loading.
pub use loading::recover_reflective_loading;
/// Ordinary least-squares intercept, slope, and residual variance.
pub use loading::OrdinaryLeastSquaresFit;
/// Arithmetic mean of posterior-draw point estimates.
pub use plausible::posterior_draw_point_estimate_mean;
/// Average OLS loading point estimates across posterior indicator draws.
pub use plausible::recover_loading_point_estimate_mean;
/// Combine OLS loadings across draws with Rubin `T`.
pub use rubin_total::combine_draw_level_ols_loadings;
/// Rubin-combined OLS loading and total variance.
pub use rubin_total::RubinCombinedLoading;
