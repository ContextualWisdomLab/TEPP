//! Event-time discrete lag-1 and exact scalar local log-rate.
//!
//! Voelkle, Oud, Davidov, and Schmidt (2012, Eq. 7; ZORA accepted
//! manuscript, Continuous Time Modeling p. 16 and Appendix B) and Driver,
//! Oud, and Voelkle (2017, Eq. 3) map the continuous-time drift by
//! `A*(Δt) = exp(A Δt)`. The noiseless scalar inverse is
//! `a = ln(φ) / Δt` with `φ = A*(Δt)`. The forward map is
//! `φ(Δt) = exp(a Δt)`. Discrete lags from unequal event intervals are
//! not one coefficient; they map through `a` first. The exact scalar
//! discrete effect of a constant predictor is Voelkle et al. (2012,
//! Eq. 12). The discrete effect of a time-varying predictor whose
//! sampling interval equals its constancy interval is their Eq. 14.
//! The exact scalar discrete process noise is the closed form of
//! Driver, Oud, and Voelkle (2017, Eq. 3–4, pp. 4–5) `Q_Δt`.
//! Equation 3 writes `η(t) = exp(A Δt) η(t0) + … +` the stochastic
//! integral. Equation 4 writes that the integral exhibits covariance
//! `Q_Δt`. The homogeneous-process consequence (`ξ`, `z` given) is
//! `Q_Δt = cov(η_ti | η_{t-1,i})` and
//! `cov(η_ti, η_{t-1,i}) = A_Δt cov(η_{t-1,i})`. The law of total
//! variance on that pair is
//! `Var(η_ti) = A_Δt Var(η_{t-1,i}) A_Δt⊤ + Q_Δt`. As `Δt → ∞` with
//! stable `a < 0`, Eq. 4 and the JSS `asymDIFFUSION` summary (p. 16;
//! §4.3 T0VAR stationarity) give the scalar Lyapunov solution
//! `-q / (2 a)`. Section 4.3 (p. 9) then adds a stable trait process
//! with `DRIFT` and `DIFFUSION` fixed to zero. That `TRAITVAR` is
//! time-invariant between-subject variance; it is not process noise
//! and not `asymDIFFUSION`. Equation 1 (p. 4) is the latent SDE.
//! Equation 5 (p. 5) writes `y_i(t) = τ_i + Λ η_i(t) + ε_i(t)` with
//! `ε ~ N(0, Θ)` and `τ_i ~ N(μ_τ, Ψ_τ)`. Table 2 (p. 12) names
//! `Θ` `MANIFESTVAR` and `Ψ_τ` `MANIFESTTRAITVAR`. The JSS summary
//! (p. 16) restates those names; it is not the measurement equation.
//! The scalar observed variance is `λ² Var(η) + θ` when `Ψ_τ = 0`
//! and `λ² Var(η) + θ + ψ` otherwise. `MANIFESTVAR` is not `Var(y)`,
//! `MANIFESTTRAITVAR` is not `MANIFESTVAR`, `TRAITVAR` is latent
//! (scaled by `λ²`), and `Var(η)` is not `Var(y)`. The lagged
//! observed covariance is `λ² cov(η_t, η_{t-1}) + ψ`; `Θ` does not
//! enter. The scalar observed mean is `τ + λ μ` (Table 2, p. 12:
//! `MANIFESTMEANS` is `τ`, not `E(y)`; `CINT` is `κ`, not `τ`;
//! `T0MEANS` is the initial latent mean, not `E(y)`). Equation 3's
//! expected-value map is `μ_t = exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`
//! (`T0MEANS` is not `μ_t`; `CINT` is not that discrete increment). Equation 3's
//! fourth summand is the contemporaneous Dirac impulse `M x` (Table 2
//! `TDPREDEFFECT` is `M`, not `CINT`, not `TIPREDEFFECT`, and not
//! Voelkle et al., 2012, Eq. 14). Equations 1–2 plus the Eq. 3
//! exponential map a time-dependent impulse that occurred strictly
//! inside `(t0, t)` as `e^{A(t−u)} M x` (`t0 < u < t`). That carry
//! is not the contemporaneous Dirac, not `CINT`, not `TIPREDEFFECT`,
//! and not Voelkle Eq. 14. Equation 5 of that carried latent mean
//! is `τ + λ(μ_t + e^{a(t−u)} m x)` (`τ + λ μ_t` is not that
//! observed mean). Section 7.2 calls this dissipation back to
//! the process mean. Equation 3's second summand also
//! maps the time-independent predictor as `A^{-1}[e^{A Δt} − I] B z`
//! (Table 2 `TIPREDEFFECT` is `B`, not `κ`, not `M`, and not Voelkle
//! Eq. 14). Section 7.2 (pp. 20–21; JSS PDF opened 2026-08-21T13:08Z)
//! names `asymTIPREDEFFECT` the expected total change in process
//! means given a unit increase on a time-independent predictor. The
//! scalar map is `-B z / a` for stable `a < 0`. That total change is
//! not the coefficient `B`, not the finite-interval increment
//! `A^{-1}[e^{A Δt} − I] B z`, not `CINT`, and not `M x`. Section 7.2
//! then names `addedTIPREDVAR` the stable between-subject variance
//! accounted for by those predictors. The scalar map is `(B / a)² v`
//! for predictor variance `v ≥ 0`. That variance is not `TRAITVAR`,
//! not `asymDIFFUSION`, and not the expected total change `-B z / a`.
//! Table 2 (p. 12) names `asymCINT` the asymptotic (`Δt = ∞`)
//! expected change in processes for a 1 unit change in intercept
//! (`CINT`). Equation 3 maps a finite event interval as
//! `A^{-1}[e^{A Δt} − I] κ`. For stable `a < 0` that `Δt → ∞` limit
//! is `-κ / a`. A unit intercept is `-1 / a`. That intercept
//! contribution is not `κ`, not the finite-interval increment, not
//! `T0MEANS`, and not `asymTIPREDEFFECT` `-B z / a`. Page 16 notes
//! that a `T0MEANS` stationarity constraint includes time-independent
//! predictors; that composition is not this intercept-only map.
//! Page 16 constrains `T0MEANS` to the model-implied values using
//! `T0MEANSbase` / `T0MEANSfree`. Those constraints include extra
//! effects due to time-independent predictors (`asymTIPREDEFFECT`).
//! The scalar composition is `-κ / a + −B z / a` for stable `a < 0`.
//! Form the intercept contribution first, then include the TI extra
//! effect, then add. That constrained first-occasion mean is not free
//! `T0MEANS`, not `asymCINT` alone, not `asymTIPREDEFFECT` alone, and
//! not the finite-interval discrete latent mean.
//! Equation 5 of that constrained first-occasion mean is
//! `τ + λ(−κ / a + −B z / a)` (§4.3, pp. 9–10; Eq. 5, p. 5; JSS PDF
//! re-opened 2026-08-21T20:07Z). Form the stationary latent mean
//! first, then `τ + λ` of that mean. `τ + λ μ_0` for free `T0MEANS`
//! is not that composition. `τ + λ(−κ / a)` is not that composition
//! when `B z ≠ 0`. `τ + λ μ_t` is not that composition.
//! Section 4.3 (pp. 9–10; JSS PDF re-opened 2026-08-22T03:07Z)
//! constrains `T0VAR` to the model-predicted variance when
//! `stationary` includes `"T0VAR"`. Page 16 names `asymDIFFUSION`
//! the total within-subject variance `-q / (2 a)`. Section 4.3
//! (p. 9) adds `TRAITVAR`. Section 7.2 (pp. 20–21) names
//! `addedTIPREDVAR` the stable between-subject variance accounted
//! for by time-independent predictors, `(B / a)² v`. The scalar
//! composition is `trait + −q / (2 a) + (B / a)² v` for stable
//! `a < 0` when the process or TI contribution is nonzero. Form the
//! within-subject contribution first, then include the trait, then
//! include the TI extra variance, then add. That constrained
//! first-occasion variance is not free `T0VAR`, not
//! `asymDIFFUSION` alone, not `TRAITVAR` alone, not
//! `addedTIPREDVAR` alone, and not the finite-interval discrete
//! latent variance `exp(2 a Δt) p + Q_Δt`. The printed 2-latent
//! `addedTIPREDVAR` 2.838 is not this scalar map.
//! Equation 5 of that constrained first-occasion variance is
//! `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` (§4.3, pp. 9–10;
//! Eq. 5, p. 5; Table 2, p. 12; JSS PDF re-opened 2026-08-22T03:20Z).
//! Form the stationary latent variance first, then `λ² p + θ + ψ`.
//! `λ² p` for free `T0VAR` is not that composition.
//! `λ²(−q / (2 a)) + θ` is not that composition when `TRAITVAR` or
//! `addedTIPREDVAR` is nonzero. `MANIFESTVAR` is not `Var(y_0)`.
//! The constrained latent variance is not `Var(y_0)`.
//! The lagged covariance of that stationary process is
//! `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` (Eq. 3–4 of §4.3
//! `T0VAR`; JSS PDF re-opened 2026-08-22T19:13Z). Form the lagged
//! within-subject covariance first, then include the trait, then
//! include the TI extra variance, then add. Trait variance and
//! `addedTIPREDVAR` are time-invariant between-subject and do not
//! decay with `e^{a Δt}`. Evolving the constrained total as if it
//! were all state is not that lagged map. Contemporaneous
//! `T0VAR` is not that lagged map. The interval must be a strictly
//! positive event interval. Equation 5 of that lagged covariance is
//! `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ`. Independent
//! `ε_t` does not enter. `θ` is not that lagged observed
//! covariance. Contemporaneous `Var(y_0)` is not that lagged
//! observed covariance. The lagged latent covariance is not the
//! lagged observed covariance.
//! The later-occasion variance of that stationary process is
//! `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v` (Eq. 3–4
//! of §4.3 `T0VAR`; JSS PDF re-opened 2026-08-22T23:12Z). Form
//! the evolved within-subject variance first, then include the
//! trait, then include the TI extra variance, then add. Trait
//! variance and `addedTIPREDVAR` do not enter `Q_Δt`. Under
//! stationarity that composition equals contemporaneous `T0VAR`.
//! Evolving the constrained total as if it were all state is not
//! that later-occasion map. The lagged covariance omits `Q_Δt`
//! and is not that later-occasion map. `Q_Δt` is not that map.
//! Equation 5 of that later-occasion variance is
//! `λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`.
//! The lagged observed covariance omits `Q_Δt` and `θ`. `θ` is
//! not that later-occasion observed variance. The later-occasion
//! latent variance is not that observed variance.
//! The later-occasion variance of §4.3 predetermined `T0VAR` is
//! `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v` (Eq. 3–4 of §4.3
//! predetermined first occasion; JSS PDF re-opened 2026-08-23T05:12Z).
//! Form the evolved free first-occasion variance first, then include
//! the trait, then include the TI extra variance, then add. Trait
//! variance and `addedTIPREDVAR` do not enter `Q_Δt`. Free `T0VAR`
//! `p_0` is not the later-occasion map. Setting `p_0 = −q / (2 a)`
//! recovers the stationary later-occasion map. Stationary later
//! variance uses `−q / (2 a)` in place of `p_0` and is not this map
//! when `p_0` is free. Evolving `trait + p_0 + (B / a)² v` as if it
//! were all state is not this map. As `Δt → ∞` with stable `a < 0`
//! the carried `p_0` vanishes and `Q_Δt` approaches `−q / (2 a)`, so
//! the composition approaches contemporaneous stationary `T0VAR`.
//! As `Δt → 0+` the composition approaches
//! `trait + p_0 + (B / a)² v`. Nonzero diffusion with `a ≥ 0` is a
//! growing process and is kept. Equation 5 of that predetermined
//! later-occasion variance is
//! `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`. `θ` is
//! not that later-occasion observed variance. The predetermined
//! later-occasion latent variance is not that observed variance.
//! Stationary later observed variance is not that observed variance
//! when `p_0` is free.
//! The lagged covariance of §4.3 predetermined `T0VAR` is
//! `trait + e^{a Δt} p_0 + (B / a)² v` (Eq. 3–4 of §4.3
//! predetermined first occasion; JSS PDF re-opened 2026-08-23T09:04Z).
//! Form the lagged free first-occasion covariance first, then include
//! the trait, then include the TI extra variance, then add. Trait
//! variance and `addedTIPREDVAR` do not decay with `e^{a Δt}`. Free
//! `T0VAR` `p_0` is not the lagged map. Setting `p_0 = −q / (2 a)`
//! recovers the stationary lagged map. Stationary lagged covariance
//! uses `−q / (2 a)` in place of `p_0` and is not this map when
//! `p_0` is free. Evolving `trait + p_0 + (B / a)² v` as if it were
//! all state is not this map. The later-occasion map includes
//! `Q_Δt` and `e^{2 a Δt} p_0` and is not this map. As `Δt → ∞`
//! with stable `a < 0` the state term vanishes. As `Δt → 0+` the
//! composition approaches `trait + p_0 + (B / a)² v`. Equation 5 of
//! that predetermined lagged covariance is
//! `λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ`. Independent `ε_t`
//! does not enter. `θ` is not that lagged observed covariance. The
//! predetermined lagged latent covariance is not that observed
//! covariance. Predetermined later observed variance includes
//! `Q_Δt` and `θ` and is not that lagged observed covariance.
//! Stationary lagged observed covariance is not that observed
//! covariance when `p_0` is free.
//! The first-occasion variance of §4.3 predetermined `T0VAR` is
//! `trait + p_0 + (B / a)² v` (JSS PDF re-opened 2026-08-23T10:03Z).
//! Form the free first-occasion state variance first, then include
//! the trait, then include the TI extra variance, then add. Setting
//! `p_0 = −q / (2 a)` recovers the stationary first-occasion map.
//! Equation 5 of that first-occasion variance is
//! `λ²(trait + p_0 + (B / a)² v) + θ + ψ`.
//! The later-start lagged covariance of §4.3 predetermined `T0VAR`
//! is `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v` (Eq. 3–4
//! of §4.3 later start / `startoffset`; JSS PDF re-opened
//! 2026-08-23T10:27Z). Form the later-start within-subject variance
//! first, then lag that state, then include the trait, then include
//! the TI extra variance, then add. Trait variance and
//! `addedTIPREDVAR` do not decay with `e^{a s}`. First-occasion
//! lagged covariance omits `e^{a s} Q_u`. Later-occasion variance
//! does not lag that later state. Setting `p_0 = −q / (2 a)`
//! recovers the stationary lagged map. As `u → 0+` the composition
//! approaches first-occasion lagged covariance. As `s → 0+` the
//! composition approaches later-occasion variance at `u`. Equation 5
//! of that later-start lagged covariance is
//! `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`.
//! Independent `ε_t` does not enter. `θ` is not that lagged observed
//! covariance. The later-start lagged latent covariance is not that
//! observed covariance. First-occasion lagged observed covariance
//! omits `e^{a s} Q_u`. Predetermined later observed variance
//! includes `Q_u` and `θ`. Stationary lagged observed covariance is
//! not that observed covariance when `p_0` is free.
//! Table 3 (p. 13) names a different matrix
//! `T0TIPREDEFFECT` for time-independent predictors on latents at
//! `T0`. The scalar first-occasion shift is `t0_b z`. Equation 3's
//! first summand carries that shift as `e^{A Δt} t0_b z`. That carry
//! is not `t0_b z`, not `A^{-1}[e^{A Δt} − I] B z`, not `CINT`, and
//! not `M x`. Equation 5 of that carried first-occasion shift is
//! `τ + λ(μ_t + e^{a Δt} t0_b z)` (`τ + λ μ_t` is not that observed
//! mean; `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not that
//! observed mean). Table 3 also names a different matrix
//! `T0TDPREDEFFECT` for time-dependent predictors on latents at
//! `T0`. The scalar first-occasion shift is `t0_m x0`. Equation 3's
//! first summand carries that shift as `e^{A Δt} t0_m x0`. That
//! carry is not `t0_m x0`, not `M x`, not `e^{A(t−u)} M x` for
//! `t0 < u < t`, not `t0_b z`, not `A^{-1}[e^{A Δt} − I] B z`, and
//! not `CINT`. An impulse at `u ≤ t0` that used `M` is already in
//! `η(t0)` as `TDPREDEFFECT`, not as `T0TDPREDEFFECT`. Equation 5 of
//! that carried first-occasion TD shift is
//! `τ + λ(μ_t + e^{a Δt} t0_m x0)` (`τ + λ μ_t` is not that
//! observed mean; `τ + λ(μ_t + e^{a Δt} t0_b z)` is not that
//! observed mean). The JSS §7.2 lasting level change sets
//! `CINT` to `TDPREDEFFECT * −DRIFT` (`κ = −a m x`; `a < 0` so
//! `−κ / a = m x`). That `CINT` setting is not the dissipating
//! Dirac, not a free `CINT`, and not the extra near-zero-drift
//! latent process also named in §7.2. Equation 3 maps that
//! intercept as `(1 − e^{a Δt}) m x` (`(1 − e^{a Δt}) m x` is not
//! `m x`, not `κ`, and not `A^{-1}[e^{A Δt} − I] B z`). Section 7.2
//! (pp. 22–23) then specifies a lasting level change by an extra
//! latent process: `T0MEANS`, `CINT`, `T0VAR`, `DIFFUSION`, and
//! `TRAITVAR` of that process are fixed to 0; `TDPREDEFFECT` on it
//! is fixed to 1; its `DRIFT` diagonal is very close to 0 (printed
//! example `−0.000001`; precisely 0 causes computational problems);
//! and its effect on the original process is the `DRIFT` coupling
//! `a_{ηξ}`. After a unit identification impulse the scalar
//! contribution is `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`
//! (`ε = a` is `a_{ηξ} x Δt e^{a Δt}`). That contribution is not
//! `κ = −a m x`, not `(1 − e^{a Δt}) m x`, and not the dissipating
//! Dirac `m x`. `ε ≥ 0` fails closed. Equation 5 of that extra-process
//! contribution is
//! `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))` (the extra
//! process has `LAMBDA` 0 and is not an observed indicator; `τ + λ μ_t`
//! is not that observed mean; `τ + λ(μ_t + m x)` is not that observed
//! mean; the contribution is not `E(y_t)`; the evolved-plus-contribution
//! latent mean is not `E(y_t)`). `T0TDPREDEFFECT` on the extra process
//! begins at `t = 0` and uses `Δt = t − t0` for both the original-process
//! evolution and the extra drive. `TDPREDEFFECT` after `t0` uses
//! `t − u` with `t0 < u < t` for the extra drive while `μ_t` still
//! uses `Δt`. Equation 5 of that after-t0 contribution is
//! `τ + λ(μ_t + a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a))` (the
//! first-occasion extra-process observed mean is not that observed
//! mean when `u ≠ t0`; `e^{a(t−u)} m x` is a Dirac on the original
//! process, not this `DRIFT` drive). Equation 5 of 2017-era
//! `addedT0TIPREDVAR` is `λ² t0_b² v` (Table 3 / p. 16 /
//! 2017-era `summary.ctsemFit.R`; JSS PDF re-opened
//! 2026-08-23T19:10Z). Form `t0_b² v` first, then `(λ extra) λ`
//! with `θ = 0`. A zero loading or zero extra is exactly zero.
//! `t0_b² v` is the latent extra, not the observed extra.
//! `λ² p_0 + θ` is first-occasion observed variance, not this
//! extra. `λ² (B / a)² v` is Eq. 5 of `addedTIPREDVAR`, not this
//! first-occasion observed extra. `MANIFESTVAR` `θ` is not this
//! extra. Free `T0TIPREDEFFECT` does not require `a < 0`. Equation 5
//! of §7.2 `addedTIPREDVAR` is `λ² (B / a)² v` (Table 2 / §7.2 /
//! 2017-era `summary.ctsemFit.R`; JSS PDF re-opened
//! 2026-08-23T19:23Z). Form `(B / a)² v` first, then `(λ extra) λ`
//! with `θ = 0`. A zero loading or zero extra is exactly zero.
//! Lasting asymptotic extra requires `a < 0`. `(B / a)² v` is the
//! latent extra, not the observed extra. `λ² t0_b² v` is
//! first-occasion extra observed TI variance, not this extra.
//! `λ² p + θ` is stationary observed variance, not this extra.
//! `MANIFESTVAR` `θ` is not this extra. Page 16 `TDPREDEFFECTstd` is
//! `m · √v / √(-q / (2 a))` after strictly positive `asymDIFFUSION`
//! and strictly positive TD predictor variance (JSS PDF re-opened
//! 2026-08-23T21:10Z). Unstandardised `M` is not `TDPREDEFFECTstd`.
//! `TIPREDEFFECTstd` is not `TDPREDEFFECTstd` even when `M = B`.
//! Table 3 / p. 16 `T0TDPREDEFFECTstd` is `t0_m · √v / √p_0` after
//! strictly positive free `T0VAR` and strictly positive TD predictor
//! variance (JSS PDF re-opened 2026-08-23T21:34Z). Unstandardised
//! `t0_m` is not `T0TDPREDEFFECTstd`. `TDPREDEFFECTstd` uses
//! `asymDIFFUSION` and is not this first-occasion map.
//! `T0TIPREDEFFECTstd` is not `T0TDPREDEFFECTstd` even when
//! `t0_m = t0_b`. Free `T0VAR` does not require `a < 0`. The scalar
//! analog of 2017-era `addedT0TIPREDVAR` for that first-occasion TD
//! coefficient is `t0_m² v` (JSS PDF re-opened 2026-08-23T22:13Z;
//! 2017-era `summary.ctsemFit.R` comments out `TDPREDVAR` and does
//! not form `addedT0TDPREDVAR`; Table 2 names `T0TDPREDCOV` the
//! covariance between latents at `T0` and time-dependent
//! predictors, not this extra; Table 3 names `T0TIPREDEFFECT`, not
//! a TD first-occasion effect matrix). Form `t0_m` first, then
//! square, then multiply by `v`. A zero coefficient or zero
//! predictor variance is exactly zero. `t0_b² v` is
//! `addedT0TIPREDVAR` and is not this extra even when
//! `t0_m = t0_b`. `t0_m · √v / √p_0` is `T0TDPREDEFFECTstd` and is
//! not this variance. `T0TDPREDCOV` is the covariance, not
//! `t0_m² v`. Free `T0VAR` is the first-occasion state, not the
//! extra. `TRAITVAR` is a zero-drift latent process, not this extra.
//! Equation 5 of that analog extra is `λ² t0_m² v` (Eq. 5, p. 5;
//! Table 2, p. 12; JSS PDF re-opened 2026-08-23T22:26Z). Form
//! `t0_m² v` first, then `(λ extra) λ` with `θ = 0`. A zero loading
//! or zero extra is exactly zero. `v < 0` fails closed. A non-event
//! clock fails closed. Free `t0_m` does not require `a < 0`.
//! `t0_m² v` is the latent extra, not the observed extra.
//! `λ² p_0 + θ` is first-occasion observed variance, not this extra.
//! `λ² t0_b² v` is Eq. 5 of `addedT0TIPREDVAR` and is not this extra
//! even when `t0_m = t0_b`. `MANIFESTVAR` `θ` is not this extra.
//! The JSS article
//! has no numbered §2.2 (2.1 is Continuous time and SEM; §3 follows).
//! The difference quotient `(x(t+Δt) − x(t)) / Δt` (their
//! Eqs. 3–4) is refused. This is not DSEM and not a matrix `expm`.

use std::collections::BTreeMap;

use crate::error::PsychometricError;
use crate::indicator::require_finite;

/// Clock on which a structural lag may be computed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LagClock {
    /// Event / valid time. The only clock that licenses a structural lag.
    EventTime,
    /// System / transaction time.
    SystemTime,
    /// Assertion time.
    AssertionTime,
    /// Document time.
    DocumentTime,
    /// Availability time.
    AvailabilityTime,
    /// Knowledge cutoff.
    KnowledgeCutoff,
}

impl LagClock {
    /// Stable wire name for the lag clock.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EventTime => "event_time",
            Self::SystemTime => "system_time",
            Self::AssertionTime => "assertion_time",
            Self::DocumentTime => "document_time",
            Self::AvailabilityTime => "availability_time",
            Self::KnowledgeCutoff => "knowledge_cutoff",
        }
    }

    /// Return whether this clock may carry a discrete lag or local log-rate.
    #[must_use]
    pub const fn admits_structural_lag(self) -> bool {
        matches!(self, Self::EventTime)
    }
}

/// One occasion on an event-time series.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventOccasion {
    /// Event / valid time of the observation.
    pub event_time: f64,
    /// Already-mapped score.
    pub score: f64,
}

/// One clustered event-time score used for CWC-then-lag recovery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusteredEventScore {
    /// Cluster identity.
    pub cluster_key: u64,
    /// Event / valid time.
    pub event_time: f64,
    /// Already-mapped score.
    pub score: f64,
}

/// One already-centered lagged residual pair on event time.
///
/// `earlier_residual` and `later_residual` are within residuals the caller
/// already formed. This type is not a raw score and is not re-centered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaggedWithinResidual {
    /// Earlier within residual.
    pub earlier_residual: f64,
    /// Later within residual.
    pub later_residual: f64,
    /// Strictly positive event-time interval. Intervals may be irregular.
    pub event_delta: f64,
}

/// Discrete lag-1 coefficient and its exact local log-rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscreteLagAndLogRate {
    /// Discrete-time lag `φ = exp(a Δt)`.
    pub discrete_lag: f64,
    /// Local log-rate `a = ln(φ) / Δt`.
    pub log_rate: f64,
    /// Positive event-time interval.
    pub event_delta: f64,
}

/// Recover the noiseless scalar discrete lag `later / earlier`.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when either score is
/// non-finite or the earlier score is zero (the ratio is undefined).
pub fn recover_discrete_lag_one(earlier: f64, later: f64) -> Result<f64, PsychometricError> {
    if !earlier.is_finite() || !later.is_finite() || earlier == 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    require_finite(later / earlier)
}

/// Map a discrete lag through the exact scalar exponential inverse.
///
/// `a = ln(φ) / Δt`. The clock must be event time. `φ` must be strictly
/// positive so the real logarithm exists.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when the
/// discrete lag is non-finite or not strictly positive.
pub fn recover_local_log_rate(
    discrete_lag: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !discrete_lag.is_finite() || discrete_lag <= 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    require_finite(discrete_lag.ln() / event_delta)
}

/// Exact scalar forward map `φ(Δt) = exp(a Δt)` (Voelkle et al., 2012, Eq. 7).
///
/// This is the inverse of [`recover_local_log_rate`]. It is the scalar case of
/// `A*(Δt) = exp(A Δt)`, not a matrix `expm`.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when the
/// log-rate is non-finite or the exponential overflows or underflows to zero.
/// Binary64 `exp` of a large negative argument is `+0`, which is not a
/// discrete lag: the inverse `a = ln(φ) / Δt` requires `φ > 0`.
pub fn recover_discrete_lag_from_log_rate(
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let discrete_lag = (log_rate * event_delta).exp();
    if !discrete_lag.is_finite() || discrete_lag <= 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    Ok(discrete_lag)
}

/// Exact scalar p. 16 `discreteDRIFTstd` after strictly positive
/// `asymDIFFUSION`.
///
/// Driver, Oud, and Voelkle (2017, p. 16; Eq. 3, p. 5; footnote 4;
/// §7.1, pp. 18–19; JSS PDF re-opened 2026-08-23T11:40Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print `discreteDRIFT` as the unstandardised discrete-time equivalent
/// of `DRIFT` for a chosen event interval, `expm(DRIFT Δt)`. Equation 3
/// writes that map. Page 16 then prints `discreteDRIFTstd` for
/// `Δt = 1` when a standardised matrix is appropriate. Footnote 4:
/// standardisations use only the relevant variance, not the total. For
/// `DRIFT`, that relevant variance is the within-subject variance
/// `asymDIFFUSION`, because `DRIFT` is intended to represent
/// individual, or average individual, temporal dynamics. The scalar
/// Lyapunov solution is `−q / (2 a)` for stable `a < 0`. Form that
/// strictly positive within-subject variance first, then
/// `φ = exp(a Δt)`. In the scalar stationary case the within-subject
/// SD ratio is 1, so the standardised auto-effect equals the
/// unstandardised discrete lag numerically; those remain distinct
/// named quantities. Unstandardised `e^{a Δt}` is defined for growing
/// `a ≥ 0` and for zero diffusion; standardised `DRIFT` is not.
/// Zero `asymDIFFUSION` has no positive SD and fails closed.
/// Section 7.1 warns that omitting trait variance confounds between-
/// and within-person information. The trait-plus-state autocorrelation
/// `(trait + e^{a Δt} p + added) / (trait + p + added)` uses the
/// total, not `asymDIFFUSION`, and is not this map when `TRAITVAR`
/// is nonzero. `TRAITVAR` is not the standardisation variance. This
/// is not a Kalman filter, not a matrix `expm`, not DSEM, and not
/// ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`] and
/// [`recover_discrete_lag_from_log_rate`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or the exponential overflows or underflows to zero.
pub fn recover_standardised_discrete_drift(
    continuous_diffusion: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance,
        );
    }
    recover_discrete_lag_from_log_rate(log_rate, event_delta, clock)
}

/// Refuse treating unstandardised `discreteDRIFT` as p. 16
/// `discreteDRIFTstd`.
///
/// `e^{a Δt}` is defined for growing and zero-diffusion processes.
/// Footnote 4 `discreteDRIFTstd` requires strictly positive
/// `asymDIFFUSION`. Equal numbers in the scalar stationary case are
/// still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift`].
pub fn refuse_unstandardised_discrete_drift_as_standardised_discrete_drift(
    unstandardised_discrete_drift: f64,
    standardised_discrete_drift: f64,
) -> Result<f64, PsychometricError> {
    let _ = (unstandardised_discrete_drift, standardised_discrete_drift);
    Err(PsychometricError::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift)
}

/// Refuse treating Driver §7.1 trait-plus-state autocorrelation as
/// p. 16 `discreteDRIFTstd`.
///
/// `(trait + e^{a Δt} p + added) / (trait + p + added)` mixes
/// between-subject `TRAITVAR` into the auto-effect. Footnote 4
/// standardises `DRIFT` using only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift`].
pub fn refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift(
    trait_plus_state_autocorrelation: f64,
    standardised_discrete_drift: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_plus_state_autocorrelation,
        standardised_discrete_drift,
    );
    Err(PsychometricError::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift)
}

/// Refuse treating Driver §4.3 trait variance as the p. 16 footnote 4
/// standardisation variance.
///
/// `TRAITVAR` is time-invariant between-subject variance. Footnote 4
/// uses only within-subject `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitVarianceIsNotStandardisationVariance`].
pub fn refuse_trait_variance_as_standardisation_variance(
    trait_variance: f64,
    within_subject_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (trait_variance, within_subject_variance);
    Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
}

/// Exact scalar p. 16 `discreteDIFFUSIONstd` after strictly positive
/// `asymDIFFUSION`.
///
/// Driver, Oud, and Voelkle (2017, p. 16; Eq. 3–4, pp. 4–5; footnote 4;
/// §7.1, pp. 18–19; JSS PDF re-opened 2026-08-23T13:06Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print discrete-time transformations for a chosen event interval
/// (`discreteDRIFT`, `discreteDIFFUSION`) and, when appropriate,
/// standardised matrices with the suffix `std`. Footnote 4:
/// standardisations use only the relevant variance, not the total.
/// Process noise is within-subject stochastic input, so that relevant
/// variance is `asymDIFFUSION`, the same footnote 4 variance used
/// for `DRIFT`. The scalar Lyapunov solution is `−q / (2 a)` for
/// stable `a < 0`. Form that strictly positive within-subject
/// variance first, then `Q_Δt` from Equation 4, then
/// `Q_Δt / (−q / (2 a))`. In the scalar stationary case that ratio
/// equals `1 − exp(2 a Δt)`. Unstandardised `Q_Δt` is defined for
/// growing `a ≥ 0` and for zero diffusion; standardised `DIFFUSION`
/// is not. Zero `asymDIFFUSION` has no positive SD and fails closed.
/// The continuous standardisation `q / (−q / (2 a)) = −2 a` is not
/// the discrete map. Section 7.1 warns that omitting trait variance
/// confounds between- and within-person information.
/// `Q_Δt / (trait + p + added)` uses the total, not
/// `asymDIFFUSION`, and is not this map when `TRAITVAR` is nonzero.
/// `TRAITVAR` is not the standardisation variance. This is not a
/// Kalman filter, not a matrix `expm`, not DSEM, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`] and
/// [`recover_discrete_process_noise`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or the mapped ratio is non-finite.
pub fn recover_standardised_discrete_diffusion(
    continuous_diffusion: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance,
        );
    }
    let process_noise =
        recover_discrete_process_noise(continuous_diffusion, log_rate, event_delta, clock)?;
    require_finite(process_noise / within)
}

/// Refuse treating unstandardised `discreteDIFFUSION` as p. 16
/// `discreteDIFFUSIONstd`.
///
/// `Q_Δt` is defined for growing and zero-diffusion processes.
/// Footnote 4 `discreteDIFFUSIONstd` requires strictly positive
/// `asymDIFFUSION`. Equal numbers would still be distinct named
/// quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion`].
pub fn refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion(
    unstandardised_discrete_diffusion: f64,
    standardised_discrete_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_discrete_diffusion,
        standardised_discrete_diffusion,
    );
    Err(PsychometricError::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion)
}

/// Refuse treating continuous `DIFFUSION` standardisation as p. 16
/// `discreteDIFFUSIONstd`.
///
/// `q / (−q / (2 a)) = −2 a` standardises the continuous diffusion
/// against `asymDIFFUSION`. Footnote 4 `discreteDIFFUSIONstd` is
/// `Q_Δt / (−q / (2 a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion`].
pub fn refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion(
    standardised_continuous_diffusion: f64,
    standardised_discrete_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        standardised_continuous_diffusion,
        standardised_discrete_diffusion,
    );
    Err(PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion)
}

/// Refuse treating Driver §7.1 trait-contaminated process noise as
/// p. 16 `discreteDIFFUSIONstd`.
///
/// `Q_Δt / (trait + p + added)` mixes between-subject `TRAITVAR`
/// into the process-noise ratio. Footnote 4 standardises `DIFFUSION`
/// using only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion`].
pub fn refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion(
    trait_contaminated_process_noise: f64,
    standardised_discrete_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_process_noise,
        standardised_discrete_diffusion,
    );
    Err(PsychometricError::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion)
}

/// Exact scalar p. 16 `DIFFUSIONstd` after strictly positive
/// `asymDIFFUSION`.
///
/// Driver, Oud, and Voelkle (2017, p. 16; Eq. 4, p. 5; footnote 4;
/// §7.1, pp. 18–19; JSS PDF re-opened 2026-08-23T13:20Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print continuous-time parameters (e.g., `DRIFT`, `DIFFUSION`) and,
/// when appropriate, standardised matrices with the suffix `std`.
/// Footnote 4: standardisations use only the relevant variance, not
/// the total. Process noise is within-subject stochastic input, so
/// that relevant variance is `asymDIFFUSION`, the same footnote 4
/// variance used for `DRIFT`. The scalar Lyapunov solution is
/// `−q / (2 a)` for stable `a < 0`. Form that strictly positive
/// within-subject variance first, then `q / (−q / (2 a))`. In the
/// scalar stationary case that ratio equals `−2 a` and does not
/// depend on `q` once `q > 0`. Unstandardised `q` is defined for
/// growing `a ≥ 0` and for zero diffusion; standardised `DIFFUSION`
/// is not. Zero `asymDIFFUSION` has no positive SD and fails closed.
/// The discrete standardisation `Q_Δt / (−q / (2 a)) = 1 − exp(2 a Δt)`
/// depends on the event interval and is not this continuous map.
/// Section 7.1 warns that omitting trait variance confounds between-
/// and within-person information. `q / (trait + p + added)` uses the
/// total, not `asymDIFFUSION`, and is not this map when `TRAITVAR`
/// is nonzero. `TRAITVAR` is not the standardisation variance. This
/// is not a Kalman filter, not a matrix `expm`, not DSEM, and not
/// ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or the mapped ratio is non-finite.
pub fn recover_standardised_continuous_diffusion(
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance,
        );
    }
    require_finite(continuous_diffusion / within)
}

/// Refuse treating unstandardised `DIFFUSION` as p. 16
/// `DIFFUSIONstd`.
///
/// `q` is defined for growing and zero-diffusion processes.
/// Footnote 4 `DIFFUSIONstd` requires strictly positive
/// `asymDIFFUSION`. Equal numbers would still be distinct named
/// quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion`].
pub fn refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion(
    unstandardised_continuous_diffusion: f64,
    standardised_continuous_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_continuous_diffusion,
        standardised_continuous_diffusion,
    );
    Err(PsychometricError::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion)
}

/// Refuse treating p. 16 `discreteDIFFUSIONstd` as p. 16
/// `DIFFUSIONstd`.
///
/// `Q_Δt / (−q / (2 a)) = 1 − exp(2 a Δt)` depends on the event
/// interval. Footnote 4 `DIFFUSIONstd` is `q / (−q / (2 a)) = −2 a`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion`].
pub fn refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion(
    standardised_discrete_diffusion: f64,
    standardised_continuous_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        standardised_discrete_diffusion,
        standardised_continuous_diffusion,
    );
    Err(PsychometricError::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion)
}

/// Refuse treating Driver §7.1 trait-contaminated continuous
/// diffusion as p. 16 `DIFFUSIONstd`.
///
/// `q / (trait + p + added)` mixes between-subject `TRAITVAR` into
/// the continuous-diffusion ratio. Footnote 4 standardises
/// `DIFFUSION` using only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion`].
pub fn refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion(
    trait_contaminated_continuous_diffusion: f64,
    standardised_continuous_diffusion: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_continuous_diffusion,
        standardised_continuous_diffusion,
    );
    Err(PsychometricError::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion)
}

/// Exact scalar p. 16 `DRIFTstd` after strictly positive
/// `asymDIFFUSION`.
///
/// Driver, Oud, and Voelkle (2017, p. 16; Eq. 1, p. 4; footnote 4;
/// §7.1, pp. 18–19; JSS PDF re-opened 2026-08-23T13:28Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print continuous-time parameters (e.g., `DRIFT`) and, when
/// appropriate, standardised matrices with the suffix `std`.
/// Footnote 4: standardisations use only the relevant variance, not
/// the total. For `DRIFT`, that relevant variance is the
/// within-subject variance `asymDIFFUSION`, because `DRIFT` is
/// intended to represent individual, or average individual, temporal
/// dynamics. The scalar Lyapunov solution is `−q / (2 a)` for
/// stable `a < 0`. Form that strictly positive within-subject
/// variance first. In the scalar stationary case the within-subject
/// SD ratio is 1, so the standardised auto-effect equals the
/// unstandardised log-rate numerically; those remain distinct named
/// quantities. Unstandardised `a` is defined for growing `a ≥ 0` and
/// for zero diffusion; standardised `DRIFT` is not. Zero
/// `asymDIFFUSION` has no positive SD and fails closed. The discrete
/// standardisation `e^{a Δt}` depends on the event interval and is
/// not this continuous map. Section 7.1 warns that omitting trait
/// variance confounds between- and within-person information. The
/// instantaneous mixed auto-effect `a p / (trait + p + added)` uses
/// the total, not `asymDIFFUSION`, and is not this map when
/// `TRAITVAR` is nonzero. `TRAITVAR` is not the standardisation
/// variance. This is not a Kalman filter, not a matrix `expm`, not
/// DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite.
pub fn recover_standardised_continuous_drift(
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance,
        );
    }
    require_finite(log_rate)
}

/// Refuse treating unstandardised `DRIFT` as p. 16 `DRIFTstd`.
///
/// `a` is defined for growing and zero-diffusion processes.
/// Footnote 4 `DRIFTstd` requires strictly positive
/// `asymDIFFUSION`. Equal numbers in the scalar stationary case are
/// still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift`].
pub fn refuse_unstandardised_continuous_drift_as_standardised_continuous_drift(
    unstandardised_continuous_drift: f64,
    standardised_continuous_drift: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_continuous_drift,
        standardised_continuous_drift,
    );
    Err(PsychometricError::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift)
}

/// Refuse treating p. 16 `discreteDRIFTstd` as p. 16 `DRIFTstd`.
///
/// `e^{a Δt}` depends on the event interval. Footnote 4 `DRIFTstd`
/// is the continuous auto-effect after strictly positive
/// `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift`].
pub fn refuse_standardised_discrete_drift_as_standardised_continuous_drift(
    standardised_discrete_drift: f64,
    standardised_continuous_drift: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_discrete_drift, standardised_continuous_drift);
    Err(PsychometricError::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift)
}

/// Refuse treating Driver §7.1 trait-contaminated continuous drift
/// as p. 16 `DRIFTstd`.
///
/// `a p / (trait + p + added)` mixes between-subject `TRAITVAR`
/// into the continuous auto-effect. Footnote 4 standardises `DRIFT`
/// using only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift`].
pub fn refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift(
    trait_contaminated_continuous_drift: f64,
    standardised_continuous_drift: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_continuous_drift,
        standardised_continuous_drift,
    );
    Err(PsychometricError::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift)
}

/// Exact scalar p. 16 `asymTIPREDEFFECTstd` after strictly positive
/// `asymDIFFUSION` and strictly positive predictor variance.
///
/// Driver, Oud, and Voelkle (2017, p. 16; §7.2, pp. 20–21; Eq. 3,
/// p. 5; Table 2, p. 12; footnote 4; JSS PDF re-opened
/// 2026-08-23T14:25Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print continuous-time parameters and, when appropriate,
/// standardised matrices with the suffix `std`. Section 7.2 names
/// `asymTIPREDEFFECT` the expected total change in process means
/// given a unit increase on a time-independent predictor. The scalar
/// map is `-B / a` for stable `a < 0`. Footnote 4: standardisations
/// use only the relevant variance, not the total. The affecting
/// variance is predictor variance `TIPREDVAR` `v`. The affected
/// variance is within-subject `asymDIFFUSION` `-q / (2 a)`, because
/// the process dynamics are individual, or average individual,
/// temporal dynamics. Form strictly positive `asymDIFFUSION` first,
/// then strictly positive `v`, then the unit asymptotic effect, then
/// `(-B / a) · √v / √(-q / (2 a))`. Unstandardised `-B / a` is
/// defined for a zero coefficient and for zero predictor variance;
/// standardised `asymTIPREDEFFECT` is not. Zero `asymDIFFUSION` or
/// zero `v` has no positive SD and fails closed. The finite-interval
/// standardisation `A^{-1}[e^{A Δt} − I] B · √v / √p` depends on the
/// event interval and is not this `Δt → ∞` map. Section 7.1 warns
/// that omitting trait variance confounds between- and within-person
/// information. `(-B / a) · √v / √(trait + p + added)` uses the
/// total, not `asymDIFFUSION`, and is not this map when `TRAITVAR`
/// is nonzero. `TRAITVAR` is not the standardisation variance. This
/// is not a Kalman filter, not a matrix `expm`, not DSEM, and not
/// ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`] and
/// [`recover_asymptotic_time_independent_predictor_effect`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero,
/// [`PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance`]
/// when predictor variance is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, predictor variance is negative, or the product
/// overflows.
pub fn recover_standardised_asymptotic_time_independent_predictor_effect(
    time_independent_effect: f64,
    predictor_variance: f64,
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance,
        );
    }
    if !predictor_variance.is_finite() || predictor_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if predictor_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance,
        );
    }
    let unit_effect = recover_asymptotic_time_independent_predictor_effect(
        time_independent_effect,
        1.0,
        log_rate,
        clock,
    )?;
    let process_sd = within.sqrt();
    let predictor_sd = predictor_variance.sqrt();
    let ratio = require_finite(predictor_sd / process_sd)?;
    require_finite(unit_effect * ratio)
}

/// Refuse treating unstandardised `asymTIPREDEFFECT` as p. 16
/// `asymTIPREDEFFECTstd`.
///
/// `-B / a` is defined for a zero coefficient and for zero
/// predictor variance. Footnote 4 `asymTIPREDEFFECTstd` requires
/// strictly positive `asymDIFFUSION` and strictly positive `v`.
/// Equal numbers when `v = p` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect`].
pub fn refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
    unstandardised_asymptotic_effect: f64,
    standardised_asymptotic_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_asymptotic_effect,
        standardised_asymptotic_effect,
    );
    Err(PsychometricError::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
}

/// Refuse treating a finite-interval standardised `TIPREDEFFECT`
/// as p. 16 `asymTIPREDEFFECTstd`.
///
/// `A^{-1}[e^{A Δt} − I] B · √v / √p` depends on the event
/// interval. Footnote 4 `asymTIPREDEFFECTstd` is
/// `(-B / a) · √v / √(-q / (2 a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect`].
pub fn refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
    standardised_discrete_effect: f64,
    standardised_asymptotic_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_discrete_effect, standardised_asymptotic_effect);
    Err(PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
}

/// Refuse treating Driver §7.1 trait-contaminated asymptotic
/// time-independent predictor effect as p. 16
/// `asymTIPREDEFFECTstd`.
///
/// `(-B / a) · √v / √(trait + p + added)` mixes between-subject
/// `TRAITVAR` into the affected SD. Footnote 4 standardises using
/// only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect`].
pub fn refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
    trait_contaminated_asymptotic_effect: f64,
    standardised_asymptotic_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_asymptotic_effect,
        standardised_asymptotic_effect,
    );
    Err(PsychometricError::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect)
}

/// Exact scalar p. 16 `TIPREDEFFECTstd` after strictly positive
/// `asymDIFFUSION` and strictly positive predictor variance.
///
/// Driver, Oud, and Voelkle (2017, p. 16; §7.2, pp. 20–21; Eq. 3,
/// p. 5; Table 2, p. 12; footnote 4; JSS PDF re-opened
/// 2026-08-23T16:21Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print continuous-time parameters and, when appropriate,
/// standardised matrices with the suffix `std`. Table 2 names `B`
/// `TIPREDEFFECT`. Footnote 4: standardisations use only the
/// relevant variance, not the total. The affecting variance is
/// predictor variance `TIPREDVAR` `v`. The affected variance is
/// within-subject `asymDIFFUSION` `-q / (2 a)`, because the process
/// dynamics are individual, or average individual, temporal
/// dynamics. Form strictly positive `asymDIFFUSION` first, then
/// strictly positive `v`, then `B · √v / √(-q / (2 a))`.
/// Unstandardised `B` is defined for a zero coefficient and for
/// zero predictor variance; standardised `TIPREDEFFECT` is not.
/// Zero `asymDIFFUSION` or zero `v` has no positive SD and fails
/// closed. The asymptotic standardisation
/// `(-B / a) · √v / √(-q / (2 a))` is the total change, not this
/// continuous coefficient. The finite-interval standardisation
/// `A^{-1}[e^{A Δt} − I] B · √v / √p` depends on the event interval
/// and is not this continuous map. Section 7.1 warns that omitting
/// trait variance confounds between- and within-person information.
/// `B · √v / √(trait + p + added)` uses the total, not
/// `asymDIFFUSION`, and is not this map when `TRAITVAR` is nonzero.
/// `TRAITVAR` is not the standardisation variance. This is not a
/// Kalman filter, not a matrix `expm`, not DSEM, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero,
/// [`PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance`]
/// when predictor variance is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, predictor variance is negative, or the product
/// overflows.
pub fn recover_standardised_continuous_time_independent_predictor_effect(
    time_independent_effect: f64,
    predictor_variance: f64,
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance,
        );
    }
    if !predictor_variance.is_finite() || predictor_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if predictor_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance,
        );
    }
    let coefficient = require_finite(time_independent_effect)?;
    let process_sd = within.sqrt();
    let predictor_sd = predictor_variance.sqrt();
    let ratio = require_finite(predictor_sd / process_sd)?;
    require_finite(coefficient * ratio)
}

/// Refuse treating unstandardised `TIPREDEFFECT` as p. 16
/// `TIPREDEFFECTstd`.
///
/// `B` is defined for a zero coefficient and for zero predictor
/// variance. Footnote 4 `TIPREDEFFECTstd` requires strictly
/// positive `asymDIFFUSION` and strictly positive `v`. Equal
/// numbers when `v = p` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect`].
pub fn refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
    unstandardised_continuous_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_continuous_effect,
        standardised_continuous_effect,
    );
    Err(PsychometricError::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
}

/// Refuse treating p. 16 `asymTIPREDEFFECTstd` as p. 16
/// `TIPREDEFFECTstd`.
///
/// `(-B / a) · √v / √(-q / (2 a))` is the standardised expected
/// total change. Footnote 4 `TIPREDEFFECTstd` is the continuous
/// coefficient `B · √v / √(-q / (2 a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect`].
pub fn refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect(
    standardised_asymptotic_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        standardised_asymptotic_effect,
        standardised_continuous_effect,
    );
    Err(PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
}

/// Refuse treating a finite-interval standardised `TIPREDEFFECT`
/// as p. 16 `TIPREDEFFECTstd`.
///
/// `A^{-1}[e^{A Δt} − I] B · √v / √p` depends on the event
/// interval. Footnote 4 `TIPREDEFFECTstd` is
/// `B · √v / √(-q / (2 a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect`].
pub fn refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect(
    standardised_discrete_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_discrete_effect, standardised_continuous_effect);
    Err(PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
}

/// Refuse treating Driver §7.1 trait-contaminated continuous
/// time-independent predictor effect as p. 16 `TIPREDEFFECTstd`.
///
/// `B · √v / √(trait + p + added)` mixes between-subject
/// `TRAITVAR` into the affected SD. Footnote 4 standardises using
/// only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect`].
pub fn refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
    trait_contaminated_continuous_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_continuous_effect,
        standardised_continuous_effect,
    );
    Err(PsychometricError::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect)
}

/// Exact scalar p. 16 `TDPREDEFFECTstd` after strictly positive
/// `asymDIFFUSION` and strictly positive predictor variance.
///
/// Driver, Oud, and Voelkle (2017, p. 16; §7.2, pp. 20–21; Eq. 3,
/// p. 5; Table 2, p. 12; footnote 4; JSS PDF re-opened
/// 2026-08-23T21:10Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// print continuous-time parameters and, when appropriate,
/// standardised matrices with the suffix `std`. Table 2 names `M`
/// `TDPREDEFFECT`. Footnote 4: standardisations use only the
/// relevant variance, not the total. The affecting variance is
/// time-dependent predictor variance `v`. The affected variance is
/// within-subject `asymDIFFUSION` `-q / (2 a)`, because the process
/// dynamics are individual, or average individual, temporal
/// dynamics. Form strictly positive `asymDIFFUSION` first, then
/// strictly positive `v`, then `m · √v / √(-q / (2 a))`.
/// Unstandardised `M` is defined for a zero coefficient and for
/// zero predictor variance; standardised `TDPREDEFFECT` is not.
/// Zero `asymDIFFUSION` or zero `v` has no positive SD and fails
/// closed. `TIPREDEFFECTstd` `B · √v / √(-q / (2 a))` is a
/// different named matrix even when `M = B` and the predictor
/// variances match. The finite-interval intercept-style
/// standardisation `A^{-1}[e^{A Δt} − I] M · √v / √p` depends on
/// the event interval and is not this continuous Dirac coefficient.
/// Section 7.1 warns that omitting trait variance confounds
/// between- and within-person information.
/// `m · √v / √(trait + p + added)` uses the total, not
/// `asymDIFFUSION`, and is not this map when `TRAITVAR` is nonzero.
/// `TRAITVAR` is not the standardisation variance. This is not a
/// Kalman filter, not a matrix `expm`, not DSEM, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_latent_variance`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`] when
/// the log-rate is not strictly negative,
/// [`PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance`]
/// when `asymDIFFUSION` is zero,
/// [`PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance`]
/// when predictor variance is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, predictor variance is negative, or the product
/// overflows.
pub fn recover_standardised_continuous_time_dependent_predictor_effect(
    time_dependent_effect: f64,
    predictor_variance: f64,
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let within = recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?;
    if within == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance,
        );
    }
    if !predictor_variance.is_finite() || predictor_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if predictor_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance,
        );
    }
    let coefficient = require_finite(time_dependent_effect)?;
    let process_sd = within.sqrt();
    let predictor_sd = predictor_variance.sqrt();
    let ratio = require_finite(predictor_sd / process_sd)?;
    require_finite(coefficient * ratio)
}

/// Refuse treating unstandardised `TDPREDEFFECT` as p. 16
/// `TDPREDEFFECTstd`.
///
/// `M` is defined for a zero coefficient and for zero predictor
/// variance. Footnote 4 `TDPREDEFFECTstd` requires strictly
/// positive `asymDIFFUSION` and strictly positive `v`. Equal
/// numbers when `v = p` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect`].
pub fn refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
    unstandardised_continuous_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        unstandardised_continuous_effect,
        standardised_continuous_effect,
    );
    Err(PsychometricError::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
}

/// Refuse treating p. 16 `TIPREDEFFECTstd` as p. 16
/// `TDPREDEFFECTstd`.
///
/// `B · √v / √(-q / (2 a))` standardises Table 2 `TIPREDEFFECT`.
/// Footnote 4 `TDPREDEFFECTstd` is `m · √v / √(-q / (2 a))`. Equal
/// numbers when `M = B` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect`].
pub fn refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect(
    standardised_continuous_time_independent_effect: f64,
    standardised_continuous_time_dependent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        standardised_continuous_time_independent_effect,
        standardised_continuous_time_dependent_effect,
    );
    Err(PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect)
}

/// Refuse treating a finite-interval standardised `TDPREDEFFECT`
/// as p. 16 `TDPREDEFFECTstd`.
///
/// `A^{-1}[e^{A Δt} − I] M · √v / √p` treats the Dirac coefficient
/// as an intercept-style integrated effect and depends on the event
/// interval. Footnote 4 `TDPREDEFFECTstd` is
/// `m · √v / √(-q / (2 a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect`].
pub fn refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
    standardised_discrete_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_discrete_effect, standardised_continuous_effect);
    Err(PsychometricError::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
}

/// Refuse treating Driver §7.1 trait-contaminated continuous
/// time-dependent predictor effect as p. 16 `TDPREDEFFECTstd`.
///
/// `m · √v / √(trait + p + added)` mixes between-subject
/// `TRAITVAR` into the affected SD. Footnote 4 standardises using
/// only `asymDIFFUSION`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect`].
pub fn refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
    trait_contaminated_continuous_effect: f64,
    standardised_continuous_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_continuous_effect,
        standardised_continuous_effect,
    );
    Err(PsychometricError::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect)
}

/// Exact scalar Table 3 / p. 16 `T0TIPREDEFFECTstd` after strictly
/// positive free `T0VAR` and strictly positive predictor variance.
///
/// Driver, Oud, and Voelkle (2017, Table 3, p. 13; p. 16; footnote
/// 4; Eq. 3, p. 5; 2017-era ctsem `summary.ctsemFit.R`; JSS PDF
/// re-opened 2026-08-23T17:20Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TIPREDEFFECT` the effect of time-independent predictors
/// on latents at `T0`. Page 16 prints standardised matrices with
/// the suffix `std` when appropriate. Footnote 4: standardisations
/// use only the relevant variance, not the total. The affecting
/// variance is predictor variance `TIPREDVAR` `v`. The affected
/// variance is free first-occasion `T0VAR` `p_0`, not within-subject
/// `asymDIFFUSION` `-q / (2 a)`, because Table 3 is the first
/// occasion, not the process dynamics. Form strictly positive
/// `p_0` first, then strictly positive `v`, then
/// `t0_b · √v / √p_0`. Unstandardised `t0_b` is defined for a zero
/// coefficient and for zero predictor variance; standardised
/// `T0TIPREDEFFECT` is not. Zero `p_0` or zero `v` has no positive
/// SD and fails closed. `T0` is an event-time occasion, so a
/// non-event clock fails closed. Free `T0VAR` does not require
/// stable `a < 0`. The continuous standardisation
/// `B · √v / √(-q / (2 a))` uses `asymDIFFUSION` and is not this
/// first-occasion map. The asymptotic standardisation
/// `(-B / a) · √v / √(-q / (2 a))` is the total change, not this
/// first-occasion coefficient. Section 7.1 warns that omitting
/// trait variance confounds between- and within-person information.
/// `t0_b · √v / √(trait + p_0 + added)` uses the total, not free
/// `T0VAR`, and is not this map when `TRAITVAR` is nonzero.
/// `TRAITVAR` is not the standardisation variance. This is not a
/// Kalman filter, not a matrix `expm`, not DSEM, and not ctsem
/// estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance`]
/// when `T0VAR` is zero,
/// [`PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance`]
/// when predictor variance is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or the product overflows.
pub fn recover_standardised_initial_time_independent_predictor_effect(
    initial_time_independent_effect: f64,
    predictor_variance: f64,
    initial_latent_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !initial_latent_variance.is_finite() || initial_latent_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_latent_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance,
        );
    }
    if !predictor_variance.is_finite() || predictor_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if predictor_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance,
        );
    }
    let coefficient = require_finite(initial_time_independent_effect)?;
    let process_sd = initial_latent_variance.sqrt();
    let predictor_sd = predictor_variance.sqrt();
    let ratio = require_finite(predictor_sd / process_sd)?;
    require_finite(coefficient * ratio)
}

/// Refuse treating unstandardised `T0TIPREDEFFECT` as Table 3 /
/// p. 16 `T0TIPREDEFFECTstd`.
///
/// `t0_b` is defined for a zero coefficient and for zero predictor
/// variance. Footnote 4 `T0TIPREDEFFECTstd` requires strictly
/// positive free `T0VAR` and strictly positive `v`. Equal numbers
/// when `v = p_0` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect`].
pub fn refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
    unstandardised_initial_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (unstandardised_initial_effect, standardised_initial_effect);
    Err(PsychometricError::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
}

/// Refuse treating p. 16 `TIPREDEFFECTstd` as Table 3 / p. 16
/// `T0TIPREDEFFECTstd`.
///
/// `B · √v / √(-q / (2 a))` standardises the continuous coefficient
/// against `asymDIFFUSION`. Footnote 4 `T0TIPREDEFFECTstd` is
/// `t0_b · √v / √p_0` against free first-occasion `T0VAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect`].
pub fn refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect(
    standardised_continuous_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_continuous_effect, standardised_initial_effect);
    Err(PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
}

/// Refuse treating p. 16 `asymTIPREDEFFECTstd` as Table 3 / p. 16
/// `T0TIPREDEFFECTstd`.
///
/// `(-B / a) · √v / √(-q / (2 a))` is the standardised expected
/// total change. Footnote 4 `T0TIPREDEFFECTstd` is the
/// first-occasion coefficient `t0_b · √v / √p_0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect`].
pub fn refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect(
    standardised_asymptotic_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_asymptotic_effect, standardised_initial_effect);
    Err(PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
}

/// Refuse treating Driver §7.1 trait-contaminated first-occasion
/// time-independent predictor effect as Table 3 / p. 16
/// `T0TIPREDEFFECTstd`.
///
/// `t0_b · √v / √(trait + p_0 + added)` mixes between-subject
/// `TRAITVAR` into the affected SD. Footnote 4 standardises using
/// only free `T0VAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect`].
pub fn refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
    trait_contaminated_initial_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_initial_effect,
        standardised_initial_effect,
    );
    Err(PsychometricError::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect)
}

/// Exact scalar Table 3 / p. 16 `T0TDPREDEFFECTstd` after strictly
/// positive free `T0VAR` and strictly positive predictor variance.
///
/// Driver, Oud, and Voelkle (2017, Table 3, p. 13; Table 2, p. 12;
/// p. 16; footnote 4; Eq. 3, p. 5; 2017-era ctsem
/// `summary.ctsemFit.R`; JSS PDF re-opened 2026-08-23T21:34Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TDPREDEFFECT` the effect of time-dependent predictors on
/// latents at `T0`. Table 2 names `M` `TDPREDEFFECT` and names
/// `T0TDPREDCOV` the first-occasion covariance, not this
/// coefficient. Page 16 prints standardised matrices with the
/// suffix `std` when appropriate. Footnote 4: standardisations use
/// only the relevant variance, not the total. The affecting
/// variance is time-dependent predictor variance `v`, not
/// `TIPREDVAR`. The affected variance is free first-occasion
/// `T0VAR` `p_0`, not within-subject `asymDIFFUSION`
/// `-q / (2 a)`, because Table 3 is the first occasion, not the
/// process dynamics. Form strictly positive `p_0` first, then
/// strictly positive `v`, then `t0_m · √v / √p_0`. Unstandardised
/// `t0_m` is defined for a zero coefficient and for zero predictor
/// variance; standardised `T0TDPREDEFFECT` is not. Zero `p_0` or
/// zero `v` has no positive SD and fails closed. `T0` is an
/// event-time occasion, so a non-event clock fails closed. Free
/// `T0VAR` does not require stable `a < 0`. The continuous
/// standardisation `m · √v / √(-q / (2 a))` uses `asymDIFFUSION`
/// and is not this first-occasion map. `T0TIPREDEFFECTstd`
/// `t0_b · √v / √p_0` is a different named matrix even when
/// `t0_m = t0_b` and the predictor variances match. Section 7.1
/// warns that omitting trait variance confounds between- and
/// within-person information. `t0_m · √v / √(trait + p_0 + added)`
/// uses the total, not free `T0VAR`, and is not this map when
/// `TRAITVAR` is nonzero. `TRAITVAR` is not the standardisation
/// variance. This is not a Kalman filter, not a matrix `expm`, not
/// DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance`]
/// when `T0VAR` is zero,
/// [`PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance`]
/// when predictor variance is zero, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or the product overflows.
pub fn recover_standardised_initial_time_dependent_predictor_effect(
    initial_time_dependent_effect: f64,
    predictor_variance: f64,
    initial_latent_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !initial_latent_variance.is_finite() || initial_latent_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_latent_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance,
        );
    }
    if !predictor_variance.is_finite() || predictor_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if predictor_variance == 0.0 {
        return Err(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance,
        );
    }
    let coefficient = require_finite(initial_time_dependent_effect)?;
    let process_sd = initial_latent_variance.sqrt();
    let predictor_sd = predictor_variance.sqrt();
    let ratio = require_finite(predictor_sd / process_sd)?;
    require_finite(coefficient * ratio)
}

/// Refuse treating unstandardised `T0TDPREDEFFECT` as Table 3 /
/// p. 16 `T0TDPREDEFFECTstd`.
///
/// `t0_m` is defined for a zero coefficient and for zero predictor
/// variance. Footnote 4 `T0TDPREDEFFECTstd` requires strictly
/// positive free `T0VAR` and strictly positive `v`. Equal numbers
/// when `v = p_0` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect`].
pub fn refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
    unstandardised_initial_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (unstandardised_initial_effect, standardised_initial_effect);
    Err(PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
}

/// Refuse treating p. 16 `TDPREDEFFECTstd` as Table 3 / p. 16
/// `T0TDPREDEFFECTstd`.
///
/// `m · √v / √(-q / (2 a))` standardises the continuous Dirac
/// coefficient against `asymDIFFUSION`. Footnote 4
/// `T0TDPREDEFFECTstd` is `t0_m · √v / √p_0` against free
/// first-occasion `T0VAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect`].
pub fn refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect(
    standardised_continuous_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (standardised_continuous_effect, standardised_initial_effect);
    Err(PsychometricError::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
}

/// Refuse treating Table 3 / p. 16 `T0TIPREDEFFECTstd` as Table 3
/// / p. 16 `T0TDPREDEFFECTstd`.
///
/// `t0_b · √v / √p_0` standardises Table 3 `T0TIPREDEFFECT`.
/// Footnote 4 `T0TDPREDEFFECTstd` is `t0_m · √v / √p_0`. Equal
/// numbers when `t0_m = t0_b` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect`].
pub fn refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect(
    standardised_initial_time_independent_effect: f64,
    standardised_initial_time_dependent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        standardised_initial_time_independent_effect,
        standardised_initial_time_dependent_effect,
    );
    Err(PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect)
}

/// Refuse treating Driver §7.1 trait-contaminated first-occasion
/// time-dependent predictor effect as Table 3 / p. 16
/// `T0TDPREDEFFECTstd`.
///
/// `t0_m · √v / √(trait + p_0 + added)` mixes between-subject
/// `TRAITVAR` into the affected SD. Footnote 4 standardises using
/// only free `T0VAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect`].
pub fn refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
    trait_contaminated_initial_effect: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        trait_contaminated_initial_effect,
        standardised_initial_effect,
    );
    Err(PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect)
}

/// Exact scalar analog of 2017-era `addedT0TIPREDVAR` after a
/// first-occasion time-dependent predictor.
///
/// Driver, Oud, and Voelkle (2017, Table 2, p. 12; Table 3, p. 13;
/// p. 16; §7.2, pp. 20–21; Eq. 3, p. 5; 2017-era ctsem
/// `summary.ctsemFit.R`; JSS PDF re-opened 2026-08-23T22:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `TDPREDEFFECT` `M` the continuous Dirac coefficient and
/// name `T0TDPREDCOV` the covariance between latents at `T0` and
/// time-dependent predictors. Table 3 names `T0TIPREDEFFECT` the
/// effect of time-independent predictors on latents at `T0`; it
/// does not name a TD first-occasion effect matrix. Page 16 prints
/// extra summary matrices when `verbose = TRUE`. The 2017-era
/// `summary.ctsemFit.R` forms `addedT0TIPREDVAR` as
/// `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` immediately
/// after `T0TIPREDEFFECTstd`. That file comments out `TDPREDVAR`
/// and does not form `addedT0TDPREDVAR`. The scalar analog of that
/// quadratic form using the stack's first-occasion TD coefficient
/// `t0_m` and Table 2 `TDPREDVAR` `v` is `t0_m² v`. Form `t0_m`
/// first, then square, then multiply by `v`. A zero coefficient or
/// zero predictor variance is exactly zero. `v < 0` fails closed.
/// `T0` is an event-time occasion, so a non-event clock fails
/// closed. Free first-occasion `t0_m` does not require stable
/// `a < 0`. `t0_b² v` is `addedT0TIPREDVAR` and is not this extra
/// even when `t0_m = t0_b`. `t0_m · √v / √p_0` is
/// `T0TDPREDEFFECTstd` and is not this variance. Table 2
/// `T0TDPREDCOV` is the first-occasion covariance, not `t0_m² v`.
/// Free `T0VAR` `p_0` is the first-occasion state, not the extra TD
/// variance. `TRAITVAR` is a zero-drift latent process, not
/// `t0_m² v`. This is not a Kalman filter, not a matrix `expm`,
/// not DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, the predictor variance is negative, or the product
/// overflows.
pub fn recover_initial_time_dependent_predictor_variance(
    initial_time_dependent_effect: f64,
    predictor_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !initial_time_dependent_effect.is_finite()
        || !predictor_variance.is_finite()
        || predictor_variance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_time_dependent_effect == 0.0 || predictor_variance == 0.0 {
        return Ok(0.0);
    }
    let squared = require_finite(initial_time_dependent_effect * initial_time_dependent_effect)?;
    require_finite(squared * predictor_variance)
}

/// Refuse treating the first-occasion TD extra variance as
/// 2017-era `addedT0TIPREDVAR`.
///
/// `t0_m² v` uses the first-occasion TD coefficient. `t0_b² v` uses
/// Table 3 `T0TIPREDEFFECT`. Equal numbers when `t0_m = t0_b` are
/// still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeIndependentVariance`].
pub fn refuse_initial_time_dependent_variance_as_initial_time_independent_variance(
    initial_time_dependent_variance: f64,
    initial_time_independent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_dependent_variance,
        initial_time_independent_variance,
    );
    Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeIndependentVariance)
}

/// Refuse treating the first-occasion TD extra variance as Table 3
/// / p. 16 `T0TDPREDEFFECTstd`.
///
/// `t0_m² v` is a variance. `t0_m · √v / √p_0` is a standardised
/// coefficient. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentVarianceIsNotStandardisedInitialTimeDependentEffect`].
pub fn refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect(
    initial_time_dependent_variance: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_variance, standardised_initial_effect);
    Err(PsychometricError::InitialTimeDependentVarianceIsNotStandardisedInitialTimeDependentEffect)
}

/// Refuse treating the first-occasion TD extra variance as Table 2
/// `T0TDPREDCOV`.
///
/// `t0_m² v` is extra first-occasion variance accounted for by a
/// first-occasion TD coefficient. Table 2 names `T0TDPREDCOV` the
/// covariance between latents at `T0` and time-dependent
/// predictors. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeDependentCovariance`].
pub fn refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance(
    initial_time_dependent_variance: f64,
    initial_time_dependent_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_dependent_variance,
        initial_time_dependent_covariance,
    );
    Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeDependentCovariance)
}

/// Refuse treating the first-occasion TD extra variance as free
/// first-occasion `T0VAR`.
///
/// `t0_m² v` is extra first-occasion variance accounted for by a
/// first-occasion TD coefficient. Free `T0VAR` `p_0` is the
/// first-occasion state.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentVarianceIsNotInitialLatentVariance`].
pub fn refuse_initial_time_dependent_variance_as_initial_latent_variance(
    initial_time_dependent_variance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_variance, initial_latent_variance);
    Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialLatentVariance)
}

/// Refuse treating the first-occasion TD extra variance as
/// `TRAITVAR`.
///
/// `t0_m² v` is extra first-occasion variance accounted for by a
/// first-occasion TD coefficient. Section 4.3 `TRAITVAR` is a
/// zero-drift latent process.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentVarianceIsNotTraitVariance`].
pub fn refuse_initial_time_dependent_variance_as_trait_variance(
    initial_time_dependent_variance: f64,
    trait_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_variance, trait_variance);
    Err(PsychometricError::InitialTimeDependentVarianceIsNotTraitVariance)
}

/// Exact scalar Eq. 5 of the analog of 2017-era `addedT0TIPREDVAR`
/// for a first-occasion time-dependent predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 2, p. 12;
/// Table 3, p. 13; p. 16; §7.2, pp. 20–21; 2017-era ctsem
/// `summary.ctsemFit.R`; JSS PDF re-opened 2026-08-23T22:26Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Equation 5 maps extra latent variance through
/// `Λ`. The 2017-era `summary.ctsemFit.R` forms the latent extra
/// `addedT0TIPREDVAR` as
/// `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` immediately
/// after `T0TIPREDEFFECTstd`. That file comments out `TDPREDVAR`
/// and does not form `addedT0TDPREDVAR`. The scalar analog of that
/// quadratic form using the stack's first-occasion TD coefficient
/// `t0_m` and Table 2 `TDPREDVAR` `v` is `t0_m² v`. Equation 5 of
/// that analog extra, with `θ = 0` and `ψ = 0`, is `λ² t0_m² v`.
/// Form the analog extra first, then `(λ extra) λ`. Do not form
/// `λ²` first: at `λ = 1e308`, `extra = 1e-308`, `λ²` overflows
/// and `λ² extra` is non-finite, but `(λ extra) λ = 1e308`. A zero
/// loading or zero extra is exactly zero. `v < 0` fails closed.
/// `T0` is an event-time occasion, so a non-event clock fails
/// closed. Free first-occasion `t0_m` does not require stable
/// `a < 0`. `t0_m² v` is the latent extra and is not this observed
/// extra. `λ² p_0 + θ` is first-occasion observed variance and is
/// not this extra. `λ² t0_b² v` is Eq. 5 of `addedT0TIPREDVAR` and
/// is not this extra even when `t0_m = t0_b`. `MANIFESTVAR` `θ` is
/// measurement error and is not this extra. `Ψ` is intercept
/// variance and is not extra TD. This is not a Kalman filter, not
/// a matrix `expm`, not DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_initial_time_dependent_predictor_variance`]
/// and [`recover_manifest_observed_variance`].
pub fn recover_initial_time_dependent_observed_variance(
    loading: f64,
    initial_time_dependent_effect: f64,
    predictor_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let extra = recover_initial_time_dependent_predictor_variance(
        initial_time_dependent_effect,
        predictor_variance,
        clock,
    )?;
    recover_manifest_observed_variance(loading, extra, 0.0)
}

/// Refuse treating Eq. 5 of the analog first-occasion TD extra as
/// the latent extra.
///
/// `λ² t0_m² v` is extra observed-indicator variance.
/// `t0_m² v` is extra latent variance. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeDependentVariance`].
pub fn refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance(
    initial_observed_predictor_variance: f64,
    initial_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        initial_predictor_variance,
    );
    Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeDependentVariance)
}

/// Refuse treating Eq. 5 of the analog first-occasion TD extra as
/// first-occasion observed variance.
///
/// `λ² t0_m² v` is extra observed TD variance. `λ² p_0 + θ` is
/// first-occasion observed-indicator variance. Those are not the
/// same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialObservedVariance`].
pub fn refuse_initial_time_dependent_observed_variance_as_initial_observed_variance(
    initial_observed_predictor_variance: f64,
    initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        initial_observed_variance,
    );
    Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialObservedVariance)
}

/// Refuse treating Eq. 5 of the analog first-occasion TD extra as
/// Eq. 5 of 2017-era `addedT0TIPREDVAR`.
///
/// `λ² t0_m² v` uses the first-occasion TD coefficient.
/// `λ² t0_b² v` uses Table 3 `T0TIPREDEFFECT`. Equal numbers when
/// `t0_m = t0_b` are still distinct named quantities.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeIndependentObservedVariance`].
pub fn refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance(
    initial_time_dependent_observed_variance: f64,
    initial_time_independent_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_dependent_observed_variance,
        initial_time_independent_observed_variance,
    );
    Err(
        PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeIndependentObservedVariance,
    )
}

/// Refuse treating Eq. 5 of the analog first-occasion TD extra as
/// `MANIFESTVAR`.
///
/// `λ² t0_m² v` is extra observed TD variance. Table 2 names
/// `MANIFESTVAR` as `Θ`, the variance of `ζ`. Those are not the
/// same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentObservedVarianceIsNotMeasurementError`].
pub fn refuse_initial_time_dependent_observed_variance_as_measurement_error(
    initial_observed_predictor_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        measurement_error_variance,
    );
    Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotMeasurementError)
}

/// Exact scalar 2017-era `addedT0TIPREDVAR` after a first-occasion
/// time-independent predictor.
///
/// Driver, Oud, and Voelkle (2017, Table 3, p. 13; p. 16; §7.2,
/// pp. 20–21; Eq. 3, p. 5; 2017-era ctsem `summary.ctsemFit.R`;
/// JSS PDF re-opened 2026-08-23T18:20Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TIPREDEFFECT` the effect of time-independent predictors
/// on latents at `T0`. Page 16 prints extra summary matrices when
/// `verbose = TRUE`. The 2017-era `summary.ctsemFit.R` forms
/// `addedT0TIPREDVAR` as
/// `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` immediately
/// after `T0TIPREDEFFECTstd`. Section 7.2 names `addedTIPREDVAR` the
/// stable between-subject variance accounted for by
/// time-independent predictors at the process asymptote,
/// `(B / a)² v`. The first-occasion analogue uses free
/// `T0TIPREDEFFECT`, not `-B / a`. The scalar map is `t0_b² v`.
/// Form `t0_b` first, then square, then multiply by `v`. A zero
/// coefficient or zero predictor variance is exactly zero. `v < 0`
/// fails closed. `T0` is an event-time occasion, so a non-event
/// clock fails closed. Free `T0TIPREDEFFECT` does not require
/// stable `a < 0`. `(B / a)² v` is `addedTIPREDVAR` and is not this
/// first-occasion map. `t0_b · √v / √p_0` is `T0TIPREDEFFECTstd`
/// and is not this variance. Free `T0VAR` `p_0` is the
/// first-occasion state, not the extra TI variance. `TRAITVAR` is a
/// zero-drift latent process, not `t0_b² v`. This is not a Kalman
/// filter, not a matrix `expm`, not DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, the predictor variance is negative, or the product
/// overflows.
pub fn recover_initial_time_independent_predictor_variance(
    initial_time_independent_effect: f64,
    predictor_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !initial_time_independent_effect.is_finite()
        || !predictor_variance.is_finite()
        || predictor_variance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_time_independent_effect == 0.0 || predictor_variance == 0.0 {
        return Ok(0.0);
    }
    let squared =
        require_finite(initial_time_independent_effect * initial_time_independent_effect)?;
    require_finite(squared * predictor_variance)
}

/// Refuse treating 2017-era `addedT0TIPREDVAR` as §7.2
/// `addedTIPREDVAR`.
///
/// `t0_b² v` uses free first-occasion `T0TIPREDEFFECT`.
/// `(B / a)² v` uses the asymptotic unit effect `-B / a` and
/// requires stable `a < 0`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance`].
pub fn refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance(
    initial_predictor_variance: f64,
    asymptotic_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_predictor_variance, asymptotic_predictor_variance);
    Err(PsychometricError::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance)
}

/// Refuse treating 2017-era `addedT0TIPREDVAR` as Table 3 / p. 16
/// `T0TIPREDEFFECTstd`.
///
/// `t0_b² v` is a variance. `t0_b · √v / √p_0` is a standardised
/// coefficient. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect`].
pub fn refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect(
    initial_predictor_variance: f64,
    standardised_initial_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_predictor_variance, standardised_initial_effect);
    Err(PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect)
}

/// Refuse treating 2017-era `addedT0TIPREDVAR` as free first-occasion
/// `T0VAR`.
///
/// `t0_b² v` is extra first-occasion variance accounted for by a
/// time-independent predictor. Free `T0VAR` `p_0` is the
/// first-occasion state.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentVarianceIsNotInitialLatentVariance`].
pub fn refuse_initial_time_independent_variance_as_initial_latent_variance(
    initial_predictor_variance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_predictor_variance, initial_latent_variance);
    Err(PsychometricError::InitialTimeIndependentVarianceIsNotInitialLatentVariance)
}

/// Refuse treating 2017-era `addedT0TIPREDVAR` as `TRAITVAR`.
///
/// `t0_b² v` is extra first-occasion variance accounted for by a
/// time-independent predictor. Section 4.3 `TRAITVAR` is a
/// zero-drift latent process.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentVarianceIsNotTraitVariance`].
pub fn refuse_initial_time_independent_variance_as_trait_variance(
    initial_predictor_variance: f64,
    trait_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_predictor_variance, trait_variance);
    Err(PsychometricError::InitialTimeIndependentVarianceIsNotTraitVariance)
}

/// Exact scalar Eq. 5 of 2017-era `addedT0TIPREDVAR`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 3, p. 13;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; 2017-era ctsem
/// `summary.ctsemFit.R`; JSS PDF re-opened 2026-08-23T19:10Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Equation 5 maps extra latent variance through
/// `Λ`. The 2017-era `summary.ctsemFit.R` forms the latent extra
/// `addedT0TIPREDVAR` as
/// `T0TIPREDEFFECT %*% TIPREDVAR %*% t(T0TIPREDEFFECT)` immediately
/// after `T0TIPREDEFFECTstd`. The scalar latent extra is `t0_b² v`.
/// Equation 5 of that extra, with `θ = 0` and `ψ = 0`, is
/// `λ² t0_b² v`. Form `addedT0TIPREDVAR` first, then
/// `(λ extra) λ`. Do not form `λ²` first: at `λ = 1e308`,
/// `extra = 1e-308`, `λ²` overflows and `λ² extra` is non-finite,
/// but `(λ extra) λ = 1e308`. A zero loading or zero extra is
/// exactly zero. `v < 0` fails closed. `T0` is an event-time
/// occasion, so a non-event clock fails closed. Free
/// `T0TIPREDEFFECT` does not require stable `a < 0`. `t0_b² v` is
/// the latent extra and is not this observed extra.
/// `λ² p_0 + θ` is first-occasion observed variance and is not this
/// extra. `λ² (B / a)² v` is Eq. 5 of `addedTIPREDVAR` and is not
/// this first-occasion observed extra. `MANIFESTVAR` `θ` is
/// measurement error and is not this extra. `Ψ` is intercept
/// variance and is not extra TI. This is not a Kalman filter, not
/// a matrix `expm`, not DSEM, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_initial_time_independent_predictor_variance`]
/// and [`recover_manifest_observed_variance`].
pub fn recover_initial_time_independent_observed_variance(
    loading: f64,
    initial_time_independent_effect: f64,
    predictor_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let extra = recover_initial_time_independent_predictor_variance(
        initial_time_independent_effect,
        predictor_variance,
        clock,
    )?;
    recover_manifest_observed_variance(loading, extra, 0.0)
}

/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as the
/// latent extra.
///
/// `λ² t0_b² v` is extra observed-indicator variance.
/// `t0_b² v` is extra latent variance. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance`].
pub fn refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance(
    initial_observed_predictor_variance: f64,
    initial_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        initial_predictor_variance,
    );
    Err(
        PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance,
    )
}

/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as
/// first-occasion observed variance.
///
/// `λ² t0_b² v` is extra observed TI variance. `λ² p_0 + θ` is
/// first-occasion observed-indicator variance. Those are not the
/// same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance`].
pub fn refuse_initial_time_independent_observed_variance_as_initial_observed_variance(
    initial_observed_predictor_variance: f64,
    initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        initial_observed_variance,
    );
    Err(PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance)
}

/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as Eq. 5 of
/// `addedTIPREDVAR`.
///
/// `λ² t0_b² v` uses free first-occasion `T0TIPREDEFFECT`.
/// `λ² (B / a)² v` uses the asymptotic unit effect `-B / a` and
/// requires stable `a < 0`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance`].
pub fn refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance(
    initial_observed_predictor_variance: f64,
    asymptotic_observed_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        asymptotic_observed_predictor_variance,
    );
    Err(
        PsychometricError::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance,
    )
}

/// Refuse treating Eq. 5 of 2017-era `addedT0TIPREDVAR` as
/// `MANIFESTVAR`.
///
/// `λ² t0_b² v` is extra observed TI variance. Table 2 names
/// `MANIFESTVAR` as `Θ`, the variance of `ζ`. Those are not the
/// same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentObservedVarianceIsNotMeasurementError`].
pub fn refuse_initial_time_independent_observed_variance_as_measurement_error(
    initial_observed_predictor_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_observed_predictor_variance,
        measurement_error_variance,
    );
    Err(PsychometricError::InitialTimeIndependentObservedVarianceIsNotMeasurementError)
}

/// Recover the exact scalar pair `(φ, a)` on event time.
///
/// # Errors
///
/// Propagates [`recover_discrete_lag_one`] and [`recover_local_log_rate`].
pub fn recover_event_time_discrete_lag_and_log_rate(
    earlier: f64,
    later: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<DiscreteLagAndLogRate, PsychometricError> {
    let discrete_lag = recover_discrete_lag_one(earlier, later)?;
    let log_rate = recover_local_log_rate(discrete_lag, event_delta, clock)?;
    Ok(DiscreteLagAndLogRate {
        discrete_lag,
        log_rate,
        event_delta,
    })
}

/// Map a discrete lag from one event interval onto another through `a`.
///
/// Voelkle et al. (2012, ZORA accepted manuscript pp. 2, 16, 33) show that
/// discrete-time autoregressive coefficients from different intervals are
/// not comparable. The licensed path is `a = ln(φ_src) / Δt_src` then
/// `φ_ref = exp(a Δt_ref)`. Equal source and reference intervals still go
/// through that map. This is not DSEM.
///
/// # Errors
///
/// Propagates [`recover_local_log_rate`] and
/// [`recover_discrete_lag_from_log_rate`].
pub fn map_discrete_lag_across_event_intervals(
    discrete_lag: f64,
    source_delta: f64,
    reference_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let log_rate = recover_local_log_rate(discrete_lag, source_delta, clock)?;
    recover_discrete_lag_from_log_rate(log_rate, reference_delta, clock)
}

/// Exact scalar discrete effect of a constant event-time predictor.
///
/// Voelkle et al. (2012, Eq. 12; ZORA accepted manuscript, Introducing
/// Intercepts, manuscript p. 20): adding a continuous-time intercept
/// `b` yields the discrete increment `A^{-1}(exp(A Δt) − I) b`. Driver,
/// Oud, and Voelkle (2017, Eq. 3) write the same term as
/// `A^{-1}[e^{A Δt} − I] ξ`. The scalar case is
/// `b*_y.x(Δt) = (a_yx / a_xx) (exp(a_xx Δt) − 1)` for `a_xx ≠ 0`.
/// The algebraically identical finite-`expm1` evaluation is
/// `a_yx (expm1(z) / a_xx)` with `z = a_xx Δt`. Dividing the increment
/// by the finite auto-effect keeps a finite Eq. 12 result when `z`
/// overflows to `-∞` (`exp(z) → 0`, so Eq. 12 → `-a_yx / a_xx`) and
/// when `a_yx Δt` overflows. When binary64 `z` underflows to `+0`, the
/// mathematical limit of Eq. 12 is `a_yx Δt`. When `a_yx = 0`, Eq. 12
/// is exactly `0` even if `expm1(z)` overflows (`0 * +∞` is `NaN`).
/// When `expm1(z)` overflows to `+∞` at a finite `z`, rewrite as
/// `sign(a_yx / a_xx) exp(ln|a_yx| + z − ln|a_xx|) − a_yx / a_xx` so a
/// finite Eq. 12 result is not lost. `z → +∞` is an unstable process
/// and fails closed unless `a_yx = 0`. The first-order product is not
/// the general discrete effect. This is not DSEM and not a matrix
/// `expm`.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when
/// either rate is non-finite, the predictor auto-effect is zero, or the
/// mapped effect is non-finite.
pub fn recover_discrete_constant_predictor_effect(
    outcome_on_predictor: f64,
    predictor_log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !outcome_on_predictor.is_finite()
        || !predictor_log_rate.is_finite()
        || predictor_log_rate == 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Voelkle Eq. 12 / Driver Eq. 3: (0 / a)(exp(a Δt) − 1) = 0.
    // Direct 0 * (expm1(z) / a) is NaN when expm1 overflows.
    if outcome_on_predictor == 0.0 {
        return Ok(0.0);
    }
    let increment_argument = predictor_log_rate * event_delta;
    if increment_argument == 0.0 {
        // Binary64 underflow of a_xx Δt. lim z→0 of Eq. 12 is a_yx Δt.
        return require_finite(outcome_on_predictor * event_delta);
    }
    let increment = increment_argument.exp_m1();
    if increment.is_finite() {
        // Divide expm1(z) by the finite a_xx, not by z. expm1(-∞)/-∞
        // is +0 and loses the equilibrium increment -a_yx/a_xx (Voelkle
        // 2012, Introducing Intercepts: the exponential vanishes as Δt
        // grows).
        return require_finite(outcome_on_predictor * (increment / predictor_log_rate));
    }
    // expm1 overflowed. z → +∞ diverges (unstable auto-effect).
    if !increment_argument.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite z, overflowed expm1. (a_yx/a_xx)(exp(z) − 1) =
    // sign(a_yx/a_xx) exp(ln|a_yx| + z − ln|a_xx|) − a_yx/a_xx.
    // The subtracted scale must itself be finite: if a_yx/a_xx overflows,
    // the rewrite term is not a binary64 number. That path is dead if
    // dominant is required first (dominant is then also infinite), so
    // refuse the scale before forming the exponential.
    let scale = outcome_on_predictor / predictor_log_rate;
    if !scale.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let log_abs_dominant =
        outcome_on_predictor.abs().ln() + increment_argument - predictor_log_rate.abs().ln();
    let dominant = require_finite(
        outcome_on_predictor.signum() * predictor_log_rate.signum() * log_abs_dominant.exp(),
    )?;
    require_finite(dominant - scale)
}

/// Refuse treating discrete lags from unequal event intervals as one coefficient.
///
/// Always fails closed. Map each lag through
/// [`map_discrete_lag_across_event_intervals`] instead.
///
/// # Errors
///
/// Always returns [`PsychometricError::UnequalIntervalPoolingForbidden`].
pub fn refuse_pooled_discrete_lag_across_unequal_intervals(
    first_delta: f64,
    second_delta: f64,
) -> Result<f64, PsychometricError> {
    let _ = (first_delta, second_delta);
    Err(PsychometricError::UnequalIntervalPoolingForbidden)
}

/// Exact scalar discrete effect of a time-varying event-time predictor.
///
/// Voelkle et al. (2012, Eq. 14; ZORA accepted manuscript, Introducing
/// Intercepts, manuscript p. 21): when the predictor can take a new value
/// at each occasion **and** the sampling interval equals the interval
/// during which that predictor is assumed constant, the discrete effect
/// is `b*_y.x(Δt) = a_yx Δt`. It does not depend on the predictor
/// auto-effect. The manuscript calls this a first-order approximation
/// that deteriorates as `Δt` grows. It is not Eq. 12. The general case
/// (sampling interval ≠ constancy interval) cites Oud and Jansen (2000),
/// which is unread, and fails closed. This is not DSEM.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when any interval is not
/// strictly positive, [`PsychometricError::UnmatchedTimeVaryingInterval`]
/// when the event, sampling, and constancy intervals are not the same
/// finite value, and [`PsychometricError::InvalidNumericInput`] when the
/// continuous effect is non-finite or the product overflows.
pub fn recover_discrete_time_varying_predictor_effect(
    outcome_on_predictor: f64,
    event_delta: f64,
    sampling_interval: f64,
    constancy_interval: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite()
        || event_delta <= 0.0
        || !sampling_interval.is_finite()
        || sampling_interval <= 0.0
        || !constancy_interval.is_finite()
        || constancy_interval <= 0.0
    {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if event_delta.to_bits() != sampling_interval.to_bits()
        || sampling_interval.to_bits() != constancy_interval.to_bits()
    {
        return Err(PsychometricError::UnmatchedTimeVaryingInterval);
    }
    if !outcome_on_predictor.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    require_finite(outcome_on_predictor * event_delta)
}

/// Refuse mapping a time-varying predictor when sampling ≠ constancy.
///
/// Always fails closed. Oud and Jansen (2000) is unread. Use
/// [`recover_discrete_time_varying_predictor_effect`] only when the
/// intervals already match, or [`recover_discrete_constant_predictor_effect`]
/// for a constant predictor (Eq. 12).
///
/// # Errors
///
/// Always returns [`PsychometricError::UnmatchedTimeVaryingInterval`].
pub fn refuse_unmatched_time_varying_predictor_interval(
    sampling_interval: f64,
    constancy_interval: f64,
) -> Result<f64, PsychometricError> {
    let _ = (sampling_interval, constancy_interval);
    Err(PsychometricError::UnmatchedTimeVaryingInterval)
}

/// Exact scalar discrete process noise on event time.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3; JSS PDF re-opened 2026-08-17T21:03Z,
/// p. 4) write the discrete process-noise covariance
/// `Q_Δt = ∫_0^{Δt} expm(A(Δt−τ)) L G G⊤ L⊤ expm(A(Δt−τ))⊤ dτ`.
/// This slice takes scalar `L = 1` (every latent subject to system noise).
/// The noiseless scalar closed form with continuous diffusion
/// `q = G G⊤ ≥ 0` is `q (exp(2 a Δt) − 1) / (2 a)` for `a ≠ 0` and
/// `q Δt` for `a = 0`. The algebraically identical finite-`expm1`
/// evaluation is `0.5 q (expm1(z) / a)` with `z = 2 (a Δt)`. Form
/// `z` as twice the already-finite product `a Δt`. Forming `2 a`
/// first overflows when `|a|` is at the binary64 extreme even if
/// `a Δt` and `Q_Δt` are finite (`a = ±1e308`, `Δt = 1e-308`).
/// When binary64 `z` underflows to `+0`, the mathematical limit is
/// `q Δt`. When `z → −∞` the exponential vanishes and the result is
/// the equilibrium variance `−q / (2 a) = −0.5 q / a` for stable
/// `a < 0`. When `expm1(z)` overflows to `+∞` at a finite `z`,
/// rewrite as `sign(q / a) exp(ln|q| + z − ln|a| − ln 2) − 0.5 q / a`.
/// An overflowing rewrite scale `0.5 q / a` is not a finite `Q_Δt`
/// (`q = 1e308`, `a = 0.1`, `Δt = 4000` → `z = 800`, `0.5 q / a = +∞`).
/// `z → +∞` is an unstable process and fails closed unless `q = 0`.
/// A zero diffusion is exactly zero even if `expm1` overflows. This
/// is not a Kalman filter, not DSEM, and not a matrix `expm`.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when
/// the diffusion is negative or non-finite, the log-rate is non-finite, or
/// the mapped variance is non-finite.
pub fn recover_discrete_process_noise(
    continuous_diffusion: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !continuous_diffusion.is_finite() || continuous_diffusion < 0.0 || !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Driver Eq. 3: the integral of a zero diffusion is zero.
    // Direct 0 * (expm1(z) / (2 a)) is NaN when expm1 overflows.
    if continuous_diffusion == 0.0 {
        return Ok(0.0);
    }
    if log_rate == 0.0 {
        return require_finite(continuous_diffusion * event_delta);
    }
    // z = 2 (a Δt), not (2 a) Δt. 2 a overflows at |a| = 1e308 even
    // when a Δt is finite (Driver Eq. 3 scalar closed form).
    let drift_interval = log_rate * event_delta;
    let increment_argument = 2.0 * drift_interval;
    if increment_argument == 0.0 {
        // Binary64 underflow of 2 a Δt. lim z→0 of Eq. 3 Q_Δt is q Δt.
        return require_finite(continuous_diffusion * event_delta);
    }
    let increment = increment_argument.exp_m1();
    if increment.is_finite() {
        // Q = q expm1(z) / (2 a) = 0.5 q (expm1(z) / a). Divide by
        // the finite a, not by 2 a: 2 a overflows when |a| = 1e308.
        // expm1(−∞) is −1, so this path also keeps −0.5 q / a.
        return require_finite(0.5 * continuous_diffusion * (increment / log_rate));
    }
    // expm1 overflowed. z → +∞ diverges (unstable auto-effect).
    // z → −∞ is already handled above because expm1(−∞) is finite.
    if !increment_argument.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite z, overflowed expm1. (q / (2 a))(exp(z) − 1) =
    // sign(q / a) exp(ln|q| + z − ln|a| − ln 2) − 0.5 q / a.
    // Driver Eq. 3 (JSS PDF re-opened 2026-08-18T03:07Z, p. 4):
    // Q_Δt is that integral. If 0.5 q / a overflows, the rewrite
    // scale is not finite and Q_Δt is not finite.
    let half_scale = 0.5 * continuous_diffusion / log_rate;
    if !half_scale.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let log_abs_dominant = continuous_diffusion.abs().ln() + increment_argument
        - log_rate.abs().ln()
        - std::f64::consts::LN_2;
    let dominant =
        require_finite(continuous_diffusion.signum() * log_rate.signum() * log_abs_dominant.exp())?;
    require_finite(dominant - half_scale)
}

/// Exact scalar lagged latent covariance on event time.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3–4, pp. 4–5; JSS PDF
/// re-opened 2026-08-18T11:20Z) write `η(t) = exp(A Δt) η(t0) + … +`
/// the stochastic integral (Eq. 3) and that the integral exhibits
/// covariance `Q_Δt` (Eq. 4). The homogeneous-process consequence is
/// `cov(η_ti, η_{t-1,i}) = A_Δt cov(η_{t-1,i})`. The scalar map is
/// `exp(a Δt) p` with prior variance `p ≥ 0`. This is not `Q_Δt`.
/// Binary64 underflow of `exp(a Δt)` to `+0` is a vanishing
/// covariance and is kept. A zero prior variance is exactly zero even
/// if the exponential overflows. When `exp(a Δt)` overflows at a
/// finite `a Δt`, rewrite as `exp(ln p + a Δt)`. An overflowing
/// rewrite fails closed. A finite `exp(a Δt)` whose product with `p`
/// overflows also fails closed. The JSS article has no numbered §2.2.
/// This is not a Kalman filter, not DSEM, and not a matrix `expm`.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is not
/// strictly positive, and [`PsychometricError::InvalidNumericInput`] when
/// the prior variance is negative or non-finite, the log-rate is
/// non-finite, or the mapped covariance is non-finite.
pub fn recover_discrete_lagged_latent_covariance(
    prior_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !prior_variance.is_finite() || prior_variance < 0.0 || !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // 0 * +∞ is NaN. Driver Eq. 3: A_Δt * 0 = 0.
    if prior_variance == 0.0 {
        return Ok(0.0);
    }
    let drift_interval = log_rate * event_delta;
    let auto_effect = drift_interval.exp();
    if auto_effect.is_finite() {
        // +0 underflow is a vanishing lagged covariance.
        return require_finite(auto_effect * prior_variance);
    }
    if !drift_interval.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite a Δt, overflowed exp. exp(a Δt) p = exp(ln p + a Δt).
    require_finite((prior_variance.ln() + drift_interval).exp())
}

/// Exact scalar discrete latent variance on event time.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3–4, pp. 4–5; JSS PDF
/// re-opened 2026-08-18T11:20Z) write `Q_Δt` as the covariance of the
/// stochastic integral (Eq. 4) after the homogeneous map
/// `η(t) = exp(A Δt) η(t0) + …` (Eq. 3). That pair is
/// `Q_Δt = cov(η_ti | η_{t-1,i})` and
/// `cov(η_ti, η_{t-1,i}) = A_Δt cov(η_{t-1,i})` when `ξ` and `z` are
/// given. The law of total variance on that pair is
/// `Var(η_ti) = A_Δt Var(η_{t-1,i}) A_Δt⊤ + Q_Δt`. The scalar map is
/// `exp(2 a Δt) p + Q_Δt`. This is not `Q_Δt` alone and not a Kalman
/// measurement update. A zero prior variance is exactly `Q_Δt`.
/// Binary64 underflow of `exp(2 a Δt)` keeps `Q_Δt`. When `exp(z)`
/// overflows at a finite `z = 2 (a Δt)`, rewrite as
/// `exp(ln p + z) + Q_Δt`. An overflowing rewrite fails closed.
/// A finite `exp(z) p` whose sum with `Q_Δt` overflows fails closed.
/// A zero diffusion skips the process-noise `z → +∞` refusal; the
/// carried term `exp(2 a Δt) p` is then still non-finite when
/// `2 (a Δt)` overflows to `+∞` (`p = 2`, `q = 0`, `a = 1e308`,
/// `Δt = 2`) and fails closed. The JSS article has no numbered §2.2.
///
/// # Errors
///
/// Propagates [`recover_discrete_process_noise`]. Returns
/// [`PsychometricError::InvalidNumericInput`] when the prior variance is
/// negative or non-finite or the mapped variance is non-finite.
pub fn recover_discrete_latent_variance(
    prior_variance: f64,
    continuous_diffusion: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let process_noise =
        recover_discrete_process_noise(continuous_diffusion, log_rate, event_delta, clock)?;
    if !prior_variance.is_finite() || prior_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if prior_variance == 0.0 {
        return Ok(process_noise);
    }
    let increment_argument = 2.0 * (log_rate * event_delta);
    if increment_argument == 0.0 {
        return require_finite(prior_variance + process_noise);
    }
    let auto_effect_square = increment_argument.exp();
    if auto_effect_square.is_finite() {
        return require_finite(auto_effect_square * prior_variance + process_noise);
    }
    if !increment_argument.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let carried = require_finite((prior_variance.ln() + increment_argument).exp())?;
    require_finite(carried + process_noise)
}

/// Exact scalar stationary within-subject variance on event time.
///
/// Driver, Oud, and Voelkle (2017, Eq. 4, p. 5; JSS PDF re-opened
/// 2026-08-19T04:10Z) write `Q_Δt` as
/// `irow(A#^{-1}[e^{A# Δt} − I] row(Q))` with `A# = A ⊗ I + I ⊗ A`.
/// The scalar Kronecker sum is `2 a`. As `Δt → ∞` with stable
/// `a < 0`, `e^{2 a Δt} → 0` and Eq. 4 becomes `-q / (2 a)`. The
/// JSS summary names that limit `asymDIFFUSION` and takes it as the
/// total within-subject variance (p. 16). Section 4.3 (pp. 9–10)
/// constrains a stationary `T0VAR` to that same model-predicted
/// variance. When `2 a` is finite, form `q / -(2 a)` so `q / a`
/// overflow does not lose a finite Lyapunov solution (`q = MAX`,
/// `a = -0.75` → `MAX / 1.5`; `CodeRabbit` on `75ecdd3`). When `2 a`
/// overflows, form `(q / a) * -0.5`. Do not form `2 a` as the only
/// path: at `a = -1e308`, `q = 1e308`, `2 a` overflows and
/// `-q / (2 a)` collapses to `+0`, but `(q / a) * -0.5 = 0.5`. Do
/// not form `0.5 q` first: at `q = from_bits(1)`, `a = -from_bits(1)`,
/// `-0.5 * q` underflows to `-0` and the quotient is `+0`, but
/// the representable Lyapunov solution is `0.5`. A zero diffusion is
/// exactly zero. `a ≥ 0` has no finite stationary variance
/// (including Brownian `a = 0`, whose variance grows as `q Δt`).
/// An overflowing Lyapunov solution fails closed. This is not a Kalman
/// filter, not DSEM, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::StationaryVarianceRequiresStableDrift`]
/// when the log-rate is not strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when the diffusion is
/// negative or non-finite, the log-rate is non-finite, or the mapped
/// variance is non-finite.
pub fn recover_stationary_latent_variance(
    continuous_diffusion: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !continuous_diffusion.is_finite() || continuous_diffusion < 0.0 || !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if log_rate >= 0.0 {
        return Err(PsychometricError::StationaryVarianceRequiresStableDrift);
    }
    // Driver Eq. 4 as Δt → ∞: (0 − 1) q / (2 a) = −q / (2 a).
    // Direct 0 * (1 / (2 a)) is not needed; a zero diffusion is zero.
    if continuous_diffusion == 0.0 {
        return Ok(0.0);
    }
    // −q / (2 a). When 2 a is finite, divide by that Kronecker sum
    // so q/a overflow does not lose a finite Lyapunov solution
    // (q = MAX, a = −0.75 → MAX/1.5; CodeRabbit on 75ecdd3).
    // When 2 a overflows (|a| = 1e308), form (q/a)*−0.5 instead.
    // Do not form 0.5 q first (min-subnormal underflow).
    let twice_rate = log_rate * 2.0;
    let stationary = if twice_rate.is_finite() {
        continuous_diffusion / -twice_rate
    } else {
        (continuous_diffusion / log_rate) * -0.5
    };
    require_finite(stationary)
}

/// Refuse treating finite-interval process noise as `asymDIFFUSION`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 4 and p. 16): `Q_Δt` at a
/// finite event interval is the covariance of the stochastic integral
/// over that interval. The asymptotic within-subject variance is the
/// `Δt → ∞` limit. Section 4.3 distinguishes that stationary
/// constraint from a predetermined `T0VAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::FiniteIntervalProcessNoiseIsNotStationary`].
pub fn refuse_finite_interval_process_noise_as_stationary_variance(
    process_noise: f64,
    event_delta: f64,
) -> Result<f64, PsychometricError> {
    let _ = (process_noise, event_delta);
    Err(PsychometricError::FiniteIntervalProcessNoiseIsNotStationary)
}

/// Exact scalar trait-plus-state latent variance.
///
/// Driver, Oud, and Voelkle (2017, §4.3, p. 9; JSS PDF re-opened
/// 2026-08-18T21:07Z) add a stable trait process with `DRIFT` and
/// `DIFFUSION` fixed to zero. The scalar sum is `trait + state`. The
/// ctsem `TRAITVAR` parameterization that adds the trait to the
/// `DIFFUSION` matrix is a software rewrite; it does not license
/// treating trait variance as process noise. This is not RI-CLPM, not
/// a Kalman filter, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when either
/// variance is negative or non-finite, or the sum overflows.
pub fn recover_trait_plus_state_latent_variance(
    trait_variance: f64,
    state_variance: f64,
) -> Result<f64, PsychometricError> {
    if !trait_variance.is_finite() || trait_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if !state_variance.is_finite() || state_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if trait_variance == 0.0 {
        return Ok(state_variance);
    }
    if state_variance == 0.0 {
        return Ok(trait_variance);
    }
    require_finite(trait_variance + state_variance)
}

/// Exact scalar trait-plus-state lagged latent covariance.
///
/// Driver, Oud, and Voelkle (2017, §4.3, p. 9): a stable trait has no
/// temporal dynamics, so `cov(trait_t, trait_{t-1}) = trait`. The
/// state lagged covariance remains `exp(a Δt) p` (Eq. 3–4). The
/// scalar sum is `trait + exp(a Δt) p`. Evolving the summed variance
/// as if it were all state is not this map. This is not RI-CLPM.
///
/// # Errors
///
/// Propagates [`recover_discrete_lagged_latent_covariance`]. Returns
/// [`PsychometricError::InvalidNumericInput`] when the trait variance
/// is negative or non-finite or the sum overflows.
pub fn recover_trait_plus_state_lagged_covariance(
    trait_variance: f64,
    state_prior_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !trait_variance.is_finite() || trait_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let state_lagged = recover_discrete_lagged_latent_covariance(
        state_prior_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    if trait_variance == 0.0 {
        return Ok(state_lagged);
    }
    require_finite(trait_variance + state_lagged)
}

/// Refuse treating Driver §4.3 trait variance as process noise.
///
/// A stable trait has `DIFFUSION` fixed to zero. The ctsem
/// `TRAITVAR` rewrite that adds the trait to `DIFFUSION` is not a
/// license to treat trait variance as `Q_Δt`.
///
/// # Errors
///
/// Always returns [`PsychometricError::TraitVarianceIsNotProcessNoise`].
pub fn refuse_trait_variance_as_process_noise(
    trait_variance: f64,
    process_noise: f64,
) -> Result<f64, PsychometricError> {
    let _ = (trait_variance, process_noise);
    Err(PsychometricError::TraitVarianceIsNotProcessNoise)
}

/// Refuse treating Driver §4.3 trait variance as `asymDIFFUSION`.
///
/// Trait variance is time-invariant between-subject variance. The
/// stationary within-subject variance is the `Δt → ∞` limit of Eq. 4.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitVarianceIsNotStationaryWithinSubject`].
pub fn refuse_trait_variance_as_stationary_within_subject(
    trait_variance: f64,
    stationary_state_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (trait_variance, stationary_state_variance);
    Err(PsychometricError::TraitVarianceIsNotStationaryWithinSubject)
}

/// Exact scalar observed-indicator variance from Driver Equation 5
/// with `MANIFESTTRAITVAR = 0`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 2, p. 12; JSS
/// PDF re-opened 2026-08-19T04:18Z) write `y_i(t) = τ_i + Λ η_i(t) +
/// ε_i(t)` with `ε ~ N(0, Θ)` and `τ_i ~ N(μ_τ, Ψ_τ)`. Equation 1
/// (p. 4) is the latent SDE, not the measurement model. Table 2 names
/// `Θ` `MANIFESTVAR` and `Ψ_τ` `MANIFESTTRAITVAR`. The p. 16 summary
/// restates those names; it is not the equation. With `Ψ_τ = 0` the
/// scalar map is `Var(y) = λ² Var(η) + θ`. Form `(λ p) λ` then add
/// `θ`. Do not form `λ²` first: at `λ = 1e308`, `p = 1e-308`, `λ²`
/// overflows and `λ² p` is non-finite, but `(λ p) λ = 1e308`. A zero
/// loading or zero latent variance is exactly `θ`. A zero
/// measurement error is exactly `λ² p`. Negative latent or
/// measurement-error variance fails closed. An overflowing product
/// or sum fails closed. This is not a Kalman filter, not ESEM
/// estimation, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the loading
/// is non-finite, either variance is negative or non-finite, or the
/// mapped variance is non-finite.
pub fn recover_manifest_observed_variance(
    loading: f64,
    latent_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    if !loading.is_finite()
        || !latent_variance.is_finite()
        || latent_variance < 0.0
        || !measurement_error_variance.is_finite()
        || measurement_error_variance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if loading == 0.0 || latent_variance == 0.0 {
        return Ok(measurement_error_variance);
    }
    let explained = require_finite((loading * latent_variance) * loading)?;
    if measurement_error_variance == 0.0 {
        return Ok(explained);
    }
    require_finite(explained + measurement_error_variance)
}

/// Refuse treating Driver Eq. 5 measurement error as `Var(y)`.
///
/// Table 2 (p. 12) names `MANIFESTVAR` as `Θ`, the variance of `ε`.
/// Equation 5 maps `Var(y) = λ² Var(η) + θ` when `Ψ_τ = 0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotObservedVariance`].
pub fn refuse_measurement_error_as_observed_variance(
    measurement_error_variance: f64,
    observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (measurement_error_variance, observed_variance);
    Err(PsychometricError::MeasurementErrorIsNotObservedVariance)
}

/// Refuse treating Driver Eq. 5 latent variance as `Var(y)`.
///
/// `Var(η)` is the latent process variance. Equation 5 maps
/// `Var(y) = λ² Var(η) + θ` when `Ψ_τ = 0`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LatentVarianceIsNotObservedVariance`].
pub fn refuse_latent_variance_as_observed_variance(
    latent_variance: f64,
    observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (latent_variance, observed_variance);
    Err(PsychometricError::LatentVarianceIsNotObservedVariance)
}

/// Exact scalar observed-indicator variance from Driver Equation 5
/// with nonzero `MANIFESTTRAITVAR`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 2, p. 12; JSS
/// PDF re-opened 2026-08-19T04:18Z) write `τ_i ~ N(μ_τ, Ψ_τ)` on the
/// indicator intercept. The scalar map is `Var(y) = λ² Var(η) + θ +
/// ψ`. Form the `Ψ_τ = 0` map first, then add `ψ`. Do not form
/// `λ²` first. A zero manifest trait is exactly `λ² p + θ`. A zero
/// loading or zero latent variance is exactly `θ + ψ`. `Ψ_τ` is not
/// `Θ`: Table 2 names `MANIFESTTRAITVAR` separately from
/// `MANIFESTVAR`. `TRAITVAR` is latent additional variance and is
/// scaled by `λ²`; `MANIFESTTRAITVAR` is not. Negative trait
/// variance fails closed. An overflowing sum fails closed. This is
/// not a Kalman filter, not ESEM estimation, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_manifest_observed_variance`]. Returns
/// [`PsychometricError::InvalidNumericInput`] when the manifest-trait
/// variance is negative or non-finite or the sum overflows.
pub fn recover_manifest_trait_plus_state_observed_variance(
    loading: f64,
    latent_variance: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
) -> Result<f64, PsychometricError> {
    if !manifest_trait_variance.is_finite() || manifest_trait_variance < 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let within =
        recover_manifest_observed_variance(loading, latent_variance, measurement_error_variance)?;
    if manifest_trait_variance == 0.0 {
        return Ok(within);
    }
    require_finite(within + manifest_trait_variance)
}

/// Refuse treating Driver Eq. 5 `MANIFESTTRAITVAR` as `MANIFESTVAR`.
///
/// Table 2 (p. 12) names `MANIFESTTRAITVAR` as `Ψ_τ`, additional
/// intercept variance on the indicators, and `MANIFESTVAR` as `Θ`,
/// the variance of `ε`. Equation 5 maps `Var(y) = λ² Var(η) + θ +
/// ψ`. `Ψ_τ` is not `Θ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ManifestTraitVarianceIsNotMeasurementError`].
pub fn refuse_manifest_trait_variance_as_measurement_error(
    manifest_trait_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (manifest_trait_variance, measurement_error_variance);
    Err(PsychometricError::ManifestTraitVarianceIsNotMeasurementError)
}

/// Exact scalar lagged observed-indicator covariance from Driver
/// Equation 5.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; JSS PDF re-opened 2026-08-19T04:18Z) write
/// `y_i(t) = τ_i + Λ η_i(t) + ε_i(t)` with independent measurement
/// error and a person-level intercept `τ_i ~ N(μ_τ, Ψ_τ)`. The
/// scalar lagged covariance is `cov(y_t, y_{t-1}) = λ² cov(η_t,
/// η_{t-1}) + ψ`. `Θ` does not enter: `ε_t` and `ε_{t-1}` are
/// independent. Form `(λ c) λ` then add `ψ`. Do not form `λ²`
/// first. A zero loading or zero latent lagged covariance is
/// exactly `ψ`. A zero manifest trait is exactly `λ² c`. Negative
/// latent lagged covariance or trait variance fails closed. An
/// overflowing product or sum fails closed. This is not a Kalman
/// filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the loading
/// is non-finite, the latent lagged covariance is negative or
/// non-finite, the manifest-trait variance is negative or
/// non-finite, or the mapped covariance is non-finite.
pub fn recover_manifest_lagged_observed_covariance(
    loading: f64,
    lagged_latent_covariance: f64,
    manifest_trait_variance: f64,
) -> Result<f64, PsychometricError> {
    if !loading.is_finite()
        || !lagged_latent_covariance.is_finite()
        || lagged_latent_covariance < 0.0
        || !manifest_trait_variance.is_finite()
        || manifest_trait_variance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if loading == 0.0 || lagged_latent_covariance == 0.0 {
        return Ok(manifest_trait_variance);
    }
    let explained = require_finite((loading * lagged_latent_covariance) * loading)?;
    if manifest_trait_variance == 0.0 {
        return Ok(explained);
    }
    require_finite(explained + manifest_trait_variance)
}

/// Refuse treating Driver Eq. 3–4 lagged latent covariance as
/// `cov(y_t, y_{t-1})`.
///
/// Equation 5 maps `cov(y_t, y_{t-1}) = λ² cov(η_t, η_{t-1}) + ψ`.
/// The latent lagged covariance is not the observed lagged
/// covariance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance`].
pub fn refuse_latent_lagged_covariance_as_observed_covariance(
    lagged_latent_covariance: f64,
    observed_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (lagged_latent_covariance, observed_lagged_covariance);
    Err(PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance)
}

/// Refuse treating Driver Eq. 5 measurement error as lagged observed
/// covariance.
///
/// `MANIFESTVAR` is `Θ`. Independent `ε_t` does not enter
/// `cov(y_t, y_{t-1})`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance`].
pub fn refuse_measurement_error_as_lagged_observed_covariance(
    measurement_error_variance: f64,
    observed_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (measurement_error_variance, observed_lagged_covariance);
    Err(PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance)
}

/// Exact scalar observed-indicator mean from Driver Equation 5.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 2, p. 12; JSS
/// PDF re-opened 2026-08-19T14:08Z) write `y_i(t) = Γ + Λ η_i(t) +
/// ζ_i(t)` with `ζ ~ N(0, Θ)` and `Γ ~ N(τ, Ψ)`. The expected
/// intercept is `τ`. Table 2 names `τ` `MANIFESTMEANS`. The scalar
/// map is `E(y) = τ + λ μ`. Form `λ μ` then add `τ`. A zero loading
/// or zero latent mean is exactly `τ`. A zero intercept is exactly
/// `λ μ`. `MANIFESTMEANS` is not `E(y)`. `E(η)` is not `E(y)`.
/// `CINT` `κ` is the latent continuous intercept from Equation 1,
/// not `τ`. `T0MEANS` is the initial latent mean, not `E(y)`. An
/// overflowing product or sum fails closed. This is not a Kalman
/// filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the loading,
/// latent mean, or intercept is non-finite, or the mapped mean is
/// non-finite.
pub fn recover_manifest_observed_mean(
    loading: f64,
    latent_mean: f64,
    manifest_mean: f64,
) -> Result<f64, PsychometricError> {
    if !loading.is_finite() || !latent_mean.is_finite() || !manifest_mean.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if loading == 0.0 || latent_mean == 0.0 {
        return Ok(manifest_mean);
    }
    let explained = require_finite(loading * latent_mean)?;
    if manifest_mean == 0.0 {
        return Ok(explained);
    }
    require_finite(explained + manifest_mean)
}

/// Refuse treating Driver Eq. 5 `MANIFESTMEANS` as `E(y)`.
///
/// Table 2 (p. 12) names `τ` the expected intercept `Γ`. Equation 5
/// maps `E(y) = τ + λ μ`.
///
/// # Errors
///
/// Always returns [`PsychometricError::ManifestMeansIsNotObservedMean`].
pub fn refuse_manifest_means_as_observed_mean(
    manifest_mean: f64,
    observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (manifest_mean, observed_mean);
    Err(PsychometricError::ManifestMeansIsNotObservedMean)
}

/// Refuse treating Driver Eq. 5 latent mean as `E(y)`.
///
/// `E(η)` is the latent process mean. Equation 5 maps `E(y) = τ + λ μ`.
/// `T0MEANS` is that latent mean at the first occasion, not `E(y)`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LatentMeanIsNotObservedMean`].
pub fn refuse_latent_mean_as_observed_mean(
    latent_mean: f64,
    observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (latent_mean, observed_mean);
    Err(PsychometricError::LatentMeanIsNotObservedMean)
}

/// Refuse treating Driver Table 2 `CINT` as `MANIFESTMEANS`.
///
/// Table 2 (p. 12) names `κ` `CINT`, the latent continuous intercept
/// from Equation 1, and `τ` `MANIFESTMEANS`, the expected `Γ` from
/// Equation 5. `κ` is not `τ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ContinuousInterceptIsNotManifestMeans`].
pub fn refuse_continuous_intercept_as_manifest_means(
    continuous_intercept: f64,
    manifest_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (continuous_intercept, manifest_mean);
    Err(PsychometricError::ContinuousInterceptIsNotManifestMeans)
}

/// Exact scalar discrete intercept increment from Driver Equation 3.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 4; Table 2, p. 12; JSS
/// PDF re-opened 2026-08-19T18:10Z) write the expected-value term
/// `A^{-1}[e^{A Δt} − I] b` after the stochastic integral is taken
/// to have mean zero. Table 2 names `κ` `CINT`. The scalar map is
/// `κ (expm1(a Δt) / a)` for `a ≠ 0`. A zero drift is the Eq. 3
/// integral with `A = 0`: `κ Δt`. That path has no matrix inverse.
/// A zero intercept is exactly zero. `CINT` is not this discrete
/// increment. The `a ≠ 0` evaluation is
/// [`recover_discrete_constant_predictor_effect`]. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when
/// `event_delta` is not strictly positive, and
/// [`PsychometricError::InvalidNumericInput`] when the intercept or
/// drift is non-finite or the mapped increment is non-finite.
pub fn recover_discrete_continuous_intercept_effect(
    continuous_intercept: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !continuous_intercept.is_finite() || !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if log_rate == 0.0 {
        if continuous_intercept == 0.0 {
            return Ok(0.0);
        }
        return require_finite(continuous_intercept * event_delta);
    }
    recover_discrete_constant_predictor_effect(continuous_intercept, log_rate, event_delta, clock)
}

/// Exact scalar discrete latent mean from Driver Equation 3.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 4; Table 2, p. 12; JSS
/// PDF re-opened 2026-08-19T18:10Z) write
/// `η(t) = exp(A Δt) η(t0) + ∫ exp(A(t−s)) (b + …) ds` plus a
/// stochastic integral of mean zero. With no time-varying covariates
/// the scalar expected-value map is
/// `μ_t = exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`. Table 2 names `μ_0`
/// at the first occasion `T0MEANS` and `κ` `CINT`. Form the CINT
/// increment first, then add the carried `T0MEANS` term. A zero
/// initial mean is exactly the increment. A zero intercept is exactly
/// `exp(a Δt) μ_0`. A zero drift carries `T0MEANS` unchanged and adds
/// `κ Δt`. As `Δt → ∞` with stable `a < 0`, `μ_t → −κ / a`. Binary64
/// underflow of `exp(a Δt)` to `+0` drops the carried `T0MEANS` and
/// keeps the equilibrium increment. `T0MEANS` is not `μ_t`. `CINT` is
/// not the discrete increment. `CINT` is not `T0MEANS`. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_continuous_intercept_effect`] and
/// returns [`PsychometricError::InvalidNumericInput`] when the initial
/// mean is non-finite, the carried exponential overflows, or the
/// mapped mean is non-finite.
pub fn recover_discrete_latent_mean(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let intercept_effect = recover_discrete_continuous_intercept_effect(
        continuous_intercept,
        log_rate,
        event_delta,
        clock,
    )?;
    if !initial_latent_mean.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_latent_mean == 0.0 {
        return Ok(intercept_effect);
    }
    let carried = if log_rate == 0.0 {
        initial_latent_mean
    } else {
        let increment_argument = log_rate * event_delta;
        if increment_argument == 0.0 {
            initial_latent_mean
        } else {
            let discrete_lag = increment_argument.exp();
            if discrete_lag == 0.0 {
                0.0
            } else if !discrete_lag.is_finite() {
                return Err(PsychometricError::InvalidNumericInput);
            } else {
                require_finite(discrete_lag * initial_latent_mean)?
            }
        }
    };
    if intercept_effect == 0.0 {
        return Ok(carried);
    }
    if carried == 0.0 {
        return Ok(intercept_effect);
    }
    require_finite(carried + intercept_effect)
}

/// Exact scalar discrete observed-indicator mean from Driver
/// Equations 3 and 5.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; Eq. 5, p. 5;
/// Table 2, p. 12; JSS PDF re-opened 2026-08-19T22:10Z) write
/// `η_i(t) = exp(A Δt) η_i(t0) + A^{-1}[exp(A Δt) − I] ξ_i + …`
/// with `ξ_i ~ N(κ, φ_ξ)` (p. 4) and a stochastic integral of
/// mean zero, then `y_i(t) = Γ_i + Λ η_i(t) + ζ_i(t)` with
/// `Γ ~ N(τ, Ψ)` and `ζ ~ N(0, Θ)`. The scalar expected-value
/// composition is `E(y_t) = τ + λ μ_t` with
/// `μ_t = exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`. Form `μ_t`
/// first, then `τ + λ μ_t`. Table 2 names `μ_0` `T0MEANS`, `κ`
/// `CINT`, and `τ` `MANIFESTMEANS`. A zero loading is exactly
/// `τ`. A zero evolved latent mean is exactly `τ`. A zero
/// intercept is exactly `λ μ_t`. The first-occasion map
/// `τ + λ μ_0` is not `E(y_t)`. `MANIFESTMEANS` is not
/// `E(y_t)`. `T0MEANS` is not `E(y_t)`. `μ_t` is not `E(y_t)`.
/// `CINT` is not `E(y_t)`. This is not a Kalman filter and not
/// ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_manifest_observed_mean`].
pub fn recover_discrete_observed_mean(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    recover_manifest_observed_mean(loading, evolved_latent_mean, manifest_mean)
}

/// Refuse treating the first-occasion observed mean as `E(y_t)`.
///
/// Equation 5 of `T0MEANS` is `τ + λ μ_0`. Equation 5 of the
/// Eq. 3 evolved mean is `τ + λ μ_t`. Those are not the same
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean`].
pub fn refuse_initial_observed_mean_as_evolved_observed_mean(
    initial_observed_mean: f64,
    evolved_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_observed_mean, evolved_observed_mean);
    Err(PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean)
}

/// Exact scalar contemporaneous impulse from Driver Equation 3.
///
/// Driver, Oud, and Voelkle (2017, Eq. 1–3, pp. 4–5; Table 2, p. 12;
/// §7.2, pp. 20–21; JSS PDF re-opened 2026-08-20T07:10Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write the time-dependent predictor as the Dirac impulse
/// `χ_i(t) = Σ_{u ∈ U_i} x_{i,u} δ(t − u)` (Eq. 2). Equation 3's
/// fourth summand is `M Σ x_{i,u} δ(t − u)`. Table 2 names `M`
/// `TDPREDEFFECT`. Section 7.2 calls this "a sudden impulse to the
/// system which then dissipates back to the process mean" and reports
/// `TDPREDEFFECT` as "the initial impact of the predictor on the
/// processes." The scalar contemporaneous jump is `m x`. It is not
/// the second-summand `CINT` map `A^{-1}[e^{A Δt} − I] κ`, not the
/// third-summand time-independent map `A^{-1}[e^{A Δt} − I] B z`,
/// and not Voelkle et al. (2012, Eq. 14) `a_{yx} Δt`. The §7.2
/// lasting level change sets `CINT` to `TDPREDEFFECT * −DRIFT`
/// (`κ = −a m x`) and is not this jump. The extra near-zero-drift
/// latent process also named in §7.2 is a third specification. A
/// zero effect or zero predictor is exactly zero. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the effect
/// or predictor is non-finite or the product overflows.
pub fn recover_time_dependent_predictor_impulse(
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
) -> Result<f64, PsychometricError> {
    if !time_dependent_effect.is_finite() || !time_dependent_predictor.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if time_dependent_effect == 0.0 || time_dependent_predictor == 0.0 {
        return Ok(0.0);
    }
    require_finite(time_dependent_effect * time_dependent_predictor)
}

/// Exact scalar level-change `CINT` from Driver Section 7.2.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 20–21; Eq. 1–3, pp. 4–5;
/// Table 2, p. 12; JSS PDF re-opened 2026-08-20T19:45Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// contrast a sudden Dirac that dissipates back to the process mean
/// with a lasting level change. To generate that lasting change,
/// `CINT` is set to `TDPREDEFFECT * −DRIFT`. The scalar setting is
/// `κ = −a m x`. Form `m x` first, then multiply by `−a`. A zero
/// effect or zero predictor is exactly zero. Stable `a < 0` is
/// required so `−κ / a = m x` is an equilibrium offset. `a ≥ 0`
/// cannot hold a new process mean. `−a m x` is not the
/// contemporaneous jump `m x`. `−a m x` is not a free `CINT`.
/// `−a m x` is not `A^{-1}[e^{A Δt} − I] B z`. The extra
/// near-zero-drift latent process also named in §7.2 is a different
/// specification and is not this `CINT` setting. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when an input
/// is non-finite or a product overflows, and
/// [`PsychometricError::LevelChangeRequiresStableDrift`] when the
/// drift is not strictly negative and the impulse is nonzero.
pub fn recover_level_change_continuous_intercept(
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    log_rate: f64,
) -> Result<f64, PsychometricError> {
    if !time_dependent_effect.is_finite()
        || !time_dependent_predictor.is_finite()
        || !log_rate.is_finite()
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if time_dependent_effect == 0.0 || time_dependent_predictor == 0.0 {
        return Ok(0.0);
    }
    if log_rate >= 0.0 {
        return Err(PsychometricError::LevelChangeRequiresStableDrift);
    }
    let impulse = require_finite(time_dependent_effect * time_dependent_predictor)?;
    require_finite(-log_rate * impulse)
}

/// Refuse treating the §7.2 level-change `CINT` as the
/// contemporaneous Dirac.
///
/// `κ = −a m x` is not the jump `m x`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LevelChangeInterceptIsNotImpulse`].
pub fn refuse_level_change_intercept_as_impulse(
    level_change_intercept: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_intercept, time_dependent_impulse);
    Err(PsychometricError::LevelChangeInterceptIsNotImpulse)
}

/// Refuse treating the §7.2 level-change `CINT` as a free `CINT`.
///
/// `κ = −a m x` is not an arbitrary continuous intercept.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept`].
pub fn refuse_level_change_intercept_as_free_continuous_intercept(
    level_change_intercept: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_intercept, continuous_intercept);
    Err(PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept)
}

/// Refuse treating the §7.2 level-change `CINT` as the Eq. 3
/// process increment.
///
/// `κ = −a m x` is not `A^{-1}[e^{A Δt} − I] B z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LevelChangeInterceptIsNotProcessIncrement`].
pub fn refuse_level_change_intercept_as_process_increment(
    level_change_intercept: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_intercept, time_independent_increment);
    Err(PsychometricError::LevelChangeInterceptIsNotProcessIncrement)
}

/// Exact scalar discrete increment of the §7.2 level-change `CINT`.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 20–21; Eq. 3, pp. 4–5;
/// Table 2, p. 12; JSS PDF re-opened 2026-08-20T19:50Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// set `CINT` to `TDPREDEFFECT * −DRIFT` so a sudden impulse holds a
/// new process mean. Equation 3 maps that intercept through
/// `A^{-1}[e^{A Δt} − I] κ`. With `κ = −a m x` the scalar increment
/// is `(e^{a Δt} − 1)/a · (−a m x) = (1 − e^{a Δt}) m x`. Form the
/// level-change `CINT` first, then the discrete intercept map.
/// Underflow of `e^{a Δt}` to `+0` keeps the equilibrium offset
/// `m x`. A zero effect or zero predictor is exactly zero. Stable
/// `a < 0` is required. `(1 − e^{a Δt}) m x` is not the
/// contemporaneous jump `m x`. `(1 − e^{a Δt}) m x` is not `κ`.
/// `(1 − e^{a Δt}) m x` is not `A^{-1}[e^{A Δt} − I] B z`. The extra
/// near-zero-drift latent process also named in §7.2 is a different
/// specification. This is not a Kalman filter and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_level_change_continuous_intercept`] and
/// [`recover_discrete_continuous_intercept_effect`].
pub fn recover_level_change_discrete_increment(
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let intercept = recover_level_change_continuous_intercept(
        time_dependent_effect,
        time_dependent_predictor,
        log_rate,
    )?;
    recover_discrete_continuous_intercept_effect(intercept, log_rate, event_delta, clock)
}

/// Refuse treating the §7.2 level-change CINT increment as the
/// contemporaneous Dirac.
///
/// `(1 − e^{a Δt}) m x` is not the jump `m x`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LevelChangeIncrementIsNotImpulse`].
pub fn refuse_level_change_increment_as_impulse(
    level_change_increment: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_increment, time_dependent_impulse);
    Err(PsychometricError::LevelChangeIncrementIsNotImpulse)
}

/// Refuse treating the §7.2 level-change CINT increment as `CINT`.
///
/// `(1 − e^{a Δt}) m x` is not `κ = −a m x`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LevelChangeIncrementIsNotIntercept`].
pub fn refuse_level_change_increment_as_intercept(
    level_change_increment: f64,
    level_change_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_increment, level_change_intercept);
    Err(PsychometricError::LevelChangeIncrementIsNotIntercept)
}

/// Refuse treating the §7.2 level-change CINT increment as the Eq. 3
/// process increment.
///
/// `(1 − e^{a Δt}) m x` is not `A^{-1}[e^{A Δt} − I] B z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LevelChangeIncrementIsNotProcessIncrement`].
pub fn refuse_level_change_increment_as_process_increment(
    level_change_increment: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (level_change_increment, time_independent_increment);
    Err(PsychometricError::LevelChangeIncrementIsNotProcessIncrement)
}

/// Exact scalar contribution of the §7.2 extra near-zero-drift process.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 22–23; Eq. 1–3, pp. 4–5;
/// Table 2, p. 12; JSS PDF re-opened 2026-08-20T23:10Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// specify a lasting level change by an extra latent process, not by
/// rewriting `CINT`. `T0MEANS`, `CINT`, `T0VAR`, `DIFFUSION`, and
/// `TRAITVAR` of that process are fixed to 0. `TDPREDEFFECT` on it is
/// fixed to 1 to identify the effect. Its `DRIFT` diagonal is very
/// close to 0 (printed example `−0.000001`; precisely 0 causes
/// computational problems). The original process is driven by the
/// `DRIFT` coupling `a_{ηξ}`. After a unit identification impulse the
/// extra state is `x e^{ε t}` and the scalar contribution to the
/// original process is `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`.
/// Form `a_{ηξ} x` first. When `ε = a` the contribution is
/// `a_{ηξ} x Δt e^{a Δt}`. A zero coupling or zero predictor is
/// exactly zero. `ε ≥ 0` cannot hold a lasting extra state and fails
/// closed. That contribution is not `κ = −a m x`, not
/// `(1 − e^{a Δt}) m x`, and not the dissipating Dirac `m x`. This
/// is not a Kalman filter, not a matrix `expm`, and not ctsem
/// estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when the interval
/// is not strictly positive,
/// [`PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift`]
/// when the extra drift is not strictly negative and the contribution
/// is nonzero, and [`PsychometricError::InvalidNumericInput`] when an
/// input is non-finite or a product, exponential, or quotient
/// overflows.
pub fn recover_level_change_extra_process_contribution(
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    original_log_rate: f64,
    extra_log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !original_from_extra_drift.is_finite()
        || !time_dependent_predictor.is_finite()
        || !original_log_rate.is_finite()
        || !extra_log_rate.is_finite()
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if original_from_extra_drift == 0.0 || time_dependent_predictor == 0.0 {
        return Ok(0.0);
    }
    if extra_log_rate >= 0.0 {
        return Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift);
    }
    let coupling = require_finite(original_from_extra_drift * time_dependent_predictor)?;
    let extra_argument = extra_log_rate * event_delta;
    let extra_lag = if extra_argument == 0.0 {
        1.0
    } else {
        extra_argument.exp()
    };
    let original_argument = original_log_rate * event_delta;
    let original_lag = if original_log_rate == 0.0 || original_argument == 0.0 {
        1.0
    } else {
        let lag = original_argument.exp();
        if !lag.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        lag
    };
    let rate_gap = extra_log_rate - original_log_rate;
    let gap_argument = rate_gap * event_delta;
    if gap_argument == 0.0 {
        return require_finite(coupling * event_delta * original_lag);
    }
    let increment = gap_argument.exp_m1();
    if increment.is_finite() {
        if original_lag == 0.0 {
            return require_finite(coupling * extra_lag / rate_gap);
        }
        return require_finite(coupling * original_lag * (increment / rate_gap));
    }
    require_finite(coupling * (extra_lag - original_lag) / rate_gap)
}

/// Refuse treating the §7.2 extra-process contribution as the
/// contemporaneous Dirac.
///
/// `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)` is not the jump `m x`.
///
/// # Errors
///
/// Always returns [`PsychometricError::LevelChangeExtraProcessIsNotImpulse`].
pub fn refuse_level_change_extra_process_as_impulse(
    extra_process_contribution: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (extra_process_contribution, time_dependent_impulse);
    Err(PsychometricError::LevelChangeExtraProcessIsNotImpulse)
}

/// Refuse treating the §7.2 extra-process contribution as the
/// level-change `CINT`.
///
/// `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)` is not `κ = −a m x`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LevelChangeExtraProcessIsNotIntercept`].
pub fn refuse_level_change_extra_process_as_intercept(
    extra_process_contribution: f64,
    level_change_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (extra_process_contribution, level_change_intercept);
    Err(PsychometricError::LevelChangeExtraProcessIsNotIntercept)
}

/// Refuse treating the §7.2 extra-process contribution as the Eq. 3
/// level-change increment.
///
/// `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)` is not
/// `(1 − e^{a Δt}) m x`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::LevelChangeExtraProcessIsNotIncrement`].
pub fn refuse_level_change_extra_process_as_increment(
    extra_process_contribution: f64,
    level_change_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (extra_process_contribution, level_change_increment);
    Err(PsychometricError::LevelChangeExtraProcessIsNotIncrement)
}

/// Exact scalar evolved latent mean plus a §7.2 extra-process contribution.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; §7.2, pp. 22–23; JSS
/// PDF re-opened 2026-08-21T06:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write the first two summands as the carried `T0MEANS` and `CINT`
/// increment. Section 7.2 then drives the original process by the
/// extra near-zero-drift latent process through the `DRIFT` coupling
/// `a_{ηξ}`. Form `μ_t` first, then add the extra-process
/// contribution. A zero contribution is exactly `μ_t`. A zero evolved
/// mean is exactly the contribution. The first-occasion map
/// `μ_0 + contribution` is not this composition when the process has
/// already evolved. The contemporaneous Dirac `μ_t + m x` is not this
/// composition. The printed specification puts `TDPREDEFFECT` on the
/// extra process, not on the original process. This is not a Kalman
/// filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_level_change_extra_process_contribution`], and returns
/// [`PsychometricError::InvalidNumericInput`] when the sum overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_latent_mean_with_extra_process(
    initial_latent_mean: f64,
    original_log_rate: f64,
    continuous_intercept: f64,
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    extra_log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        original_log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let contribution = recover_level_change_extra_process_contribution(
        original_from_extra_drift,
        time_dependent_predictor,
        original_log_rate,
        extra_log_rate,
        event_delta,
        clock,
    )?;
    if contribution == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(contribution);
    }
    require_finite(evolved_latent_mean + contribution)
}

/// Exact scalar observed mean of a §7.2 extra-process contribution.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 1–3, pp. 4–5;
/// §7.2, pp. 22–23; JSS PDF re-opened 2026-08-21T06:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The expected intercept is `τ`. Section 7.2's
/// printed extra process has `LAMBDA` 0: it is not an observed
/// indicator. Original indicators load on the original process after
/// the `DRIFT` coupling. The latent process at `t` after that
/// contribution is `μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`.
/// The scalar composition is
/// `E(y_t) = τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`.
/// Form the evolved-plus-contribution latent mean first, then
/// `τ + λ` of that mean. Table 2 names `τ` `MANIFESTMEANS`. A zero
/// loading is exactly `τ`. A zero evolved-plus-contribution latent
/// mean is exactly `τ`. A zero intercept is exactly `λ` of that
/// latent mean. The evolved observed mean `τ + λ μ_t` is not this
/// composition when the contribution is nonzero. The contemporaneous
/// map `τ + λ(μ_t + m x)` is not this composition. `MANIFESTMEANS` is
/// not `E(y_t)`. The extra-process contribution is not `E(y_t)`. The
/// evolved-plus-contribution latent mean is not `E(y_t)`. The extra
/// process itself is not an observed indicator. This is not a Kalman
/// filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean_with_extra_process`] and
/// [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_extra_process(
    loading: f64,
    initial_latent_mean: f64,
    original_log_rate: f64,
    continuous_intercept: f64,
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    extra_log_rate: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let extra_latent_mean = recover_discrete_latent_mean_with_extra_process(
        initial_latent_mean,
        original_log_rate,
        continuous_intercept,
        original_from_extra_drift,
        time_dependent_predictor,
        extra_log_rate,
        event_delta,
        clock,
    )?;
    recover_manifest_observed_mean(loading, extra_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the extra-process
/// observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the §7.2 extra-process contribution is
/// `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`. Those
/// are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean`].
pub fn refuse_evolved_observed_mean_as_extra_process_observed_mean(
    evolved_observed_mean: f64,
    extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, extra_process_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean)
}

/// Refuse treating the contemporaneous-impulse observed mean as the
/// extra-process observed mean.
///
/// Equation 5 of the contemporaneous Dirac is `τ + λ(μ_t + m x)`.
/// Equation 5 of the §7.2 extra-process contribution is
/// `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`. Those
/// are not the same map. The printed specification puts
/// `TDPREDEFFECT` on the extra process, not on the original process.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean`].
pub fn refuse_impulse_observed_mean_as_extra_process_observed_mean(
    impulse_observed_mean: f64,
    extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (impulse_observed_mean, extra_process_observed_mean);
    Err(PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean)
}

/// Refuse treating the §7.2 extra-process contribution as `E(y_t)`.
///
/// The contribution is not `τ + λ` of the evolved-plus-contribution
/// latent mean. The extra process has `LAMBDA` 0 in the printed
/// specification.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ExtraProcessContributionIsNotObservedMean`].
pub fn refuse_extra_process_contribution_as_observed_mean(
    extra_process_contribution: f64,
    extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (extra_process_contribution, extra_process_observed_mean);
    Err(PsychometricError::ExtraProcessContributionIsNotObservedMean)
}

/// Refuse treating the evolved-plus-contribution latent mean as
/// `E(y_t)`.
///
/// Equation 5 maps `E(y_t) = τ + λ` of that mean. The latent mean is
/// not the observed mean.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ExtraProcessLatentMeanIsNotObservedMean`].
pub fn refuse_extra_process_latent_mean_as_observed_mean(
    extra_process_latent_mean: f64,
    extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (extra_process_latent_mean, extra_process_observed_mean);
    Err(PsychometricError::ExtraProcessLatentMeanIsNotObservedMean)
}

/// Exact scalar §7.2 extra-process contribution of a `TDPREDEFFECT`
/// impulse strictly after `t0`.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 22–23; JSS PDF
/// re-opened 2026-08-21T06:32Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TDPREDEFFECT` when the extra process begins at `t = 0`
/// and `TDPREDEFFECT` when it begins after `t = 0`. The printed
/// extra `TDPREDEFFECT` is 1. The original process is driven through
/// the `DRIFT` coupling, not through a Dirac on the original
/// process. After an identification impulse at `u` with
/// `t0 < u < t` the scalar contribution is
/// `a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a)`. Form the
/// interior interval `t − u` first, then the extra-process
/// contribution on that interval. An impulse at `u = t0` is the
/// first-occasion extra-process map. An impulse at `u = t` has not
/// yet driven the original process. This is not a Kalman filter,
/// not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::NonPositiveInterval`] when
/// `t − u` is not strictly interior to `(0, t − t0)`, and
/// otherwise propagates
/// [`recover_level_change_extra_process_contribution`].
pub fn recover_level_change_extra_process_contribution_after(
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    original_log_rate: f64,
    extra_log_rate: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !elapsed_after_impulse.is_finite() || elapsed_after_impulse <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if elapsed_after_impulse >= event_delta {
        return Err(PsychometricError::NonPositiveInterval);
    }
    recover_level_change_extra_process_contribution(
        original_from_extra_drift,
        time_dependent_predictor,
        original_log_rate,
        extra_log_rate,
        elapsed_after_impulse,
        clock,
    )
}

/// Exact scalar evolved latent mean plus a §7.2 extra-process
/// contribution after `t0`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; §7.2, pp. 22–23;
/// JSS PDF re-opened 2026-08-21T06:32Z) evolve `T0MEANS` and `CINT`
/// over `Δt = t − t0`. `TDPREDEFFECT` on the extra process after
/// `t0` drives the original process only over `t − u` with
/// `t0 < u < t`. Form `μ_t` first, then add the after-t0
/// extra-process contribution. A zero contribution is exactly
/// `μ_t`. The first-occasion extra-process map uses `Δt` for both
/// the evolution and the extra drive and is not this composition
/// when `u ≠ t0`. The impulse-carry `μ_t + e^{a(t−u)} m x` is a
/// Dirac on the original process and is not this composition.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_level_change_extra_process_contribution_after`], and
/// returns [`PsychometricError::InvalidNumericInput`] when the sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_latent_mean_with_extra_process_after(
    initial_latent_mean: f64,
    original_log_rate: f64,
    continuous_intercept: f64,
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    extra_log_rate: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        original_log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let contribution = recover_level_change_extra_process_contribution_after(
        original_from_extra_drift,
        time_dependent_predictor,
        original_log_rate,
        extra_log_rate,
        event_delta,
        elapsed_after_impulse,
        clock,
    )?;
    if contribution == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(contribution);
    }
    require_finite(evolved_latent_mean + contribution)
}

/// Exact scalar observed mean of a §7.2 extra-process contribution
/// after `t0`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; §7.2, pp. 22–23;
/// JSS PDF re-opened 2026-08-21T06:32Z) write
/// `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The printed extra process has `LAMBDA` 0.
/// Original indicators load on the original process after the
/// `DRIFT` coupling over `t − u` with `t0 < u < t`. The scalar
/// composition is
/// `E(y_t) = τ + λ(μ_t + a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a))`.
/// Form the evolved-plus-after-contribution latent mean first, then
/// `τ + λ` of that mean. The first-occasion extra-process observed
/// mean uses `Δt` for both the evolution and the extra drive and is
/// not this composition when `u ≠ t0`. The evolved observed mean
/// `τ + λ μ_t` is not this composition. The impulse-carry map
/// `τ + λ(μ_t + e^{a(t−u)} m x)` is not this composition. The
/// extra process itself is not an observed indicator. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean_with_extra_process_after`]
/// and [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_extra_process_after(
    loading: f64,
    initial_latent_mean: f64,
    original_log_rate: f64,
    continuous_intercept: f64,
    original_from_extra_drift: f64,
    time_dependent_predictor: f64,
    extra_log_rate: f64,
    manifest_mean: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let extra_latent_mean = recover_discrete_latent_mean_with_extra_process_after(
        initial_latent_mean,
        original_log_rate,
        continuous_intercept,
        original_from_extra_drift,
        time_dependent_predictor,
        extra_log_rate,
        event_delta,
        elapsed_after_impulse,
        clock,
    )?;
    recover_manifest_observed_mean(loading, extra_latent_mean, manifest_mean)
}

/// Refuse treating the first-occasion extra-process observed mean
/// as the after-t0 extra-process observed mean.
///
/// `T0TDPREDEFFECT` on the extra process uses `Δt = t − t0`.
/// `TDPREDEFFECT` after `t0` uses `t − u` with `t0 < u < t`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean`].
pub fn refuse_extra_process_observed_mean_as_after_extra_process_observed_mean(
    extra_process_observed_mean: f64,
    after_extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        extra_process_observed_mean,
        after_extra_process_observed_mean,
    );
    Err(PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean)
}

/// Refuse treating the evolved observed mean as the after-t0
/// extra-process observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the after-t0 extra-process contribution is
/// `τ + λ(μ_t + a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a))`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean`].
pub fn refuse_evolved_observed_mean_as_after_extra_process_observed_mean(
    evolved_observed_mean: f64,
    after_extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, after_extra_process_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean)
}

/// Refuse treating the impulse-carry observed mean as the after-t0
/// extra-process observed mean.
///
/// `e^{a(t−u)} m x` is a Dirac on the original process. Extra-process
/// `TDPREDEFFECT` after `t0` drives the original process through
/// `DRIFT`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean`].
pub fn refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean(
    impulse_carry_observed_mean: f64,
    after_extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        impulse_carry_observed_mean,
        after_extra_process_observed_mean,
    );
    Err(PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean)
}

/// Refuse treating the after-t0 extra-process contribution as
/// `E(y_t)`.
///
/// The contribution is not `τ + λ` of the
/// evolved-plus-after-contribution latent mean. The extra process
/// has `LAMBDA` 0 in the printed specification.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AfterExtraProcessContributionIsNotObservedMean`].
pub fn refuse_after_extra_process_contribution_as_observed_mean(
    after_extra_process_contribution: f64,
    after_extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        after_extra_process_contribution,
        after_extra_process_observed_mean,
    );
    Err(PsychometricError::AfterExtraProcessContributionIsNotObservedMean)
}

/// Refuse treating the evolved-plus-after-contribution latent mean
/// as `E(y_t)`.
///
/// Equation 5 maps `E(y_t) = τ + λ` of that mean. The latent mean
/// is not the observed mean.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean`].
pub fn refuse_after_extra_process_latent_mean_as_observed_mean(
    after_extra_process_latent_mean: f64,
    after_extra_process_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        after_extra_process_latent_mean,
        after_extra_process_observed_mean,
    );
    Err(PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean)
}

/// Exact scalar evolved latent mean plus a contemporaneous impulse.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5) write the first two
/// summands as the carried `T0MEANS` and `CINT` increment, then add
/// the fourth-summand impulse at the observation instant. Form `μ_t`
/// first, then add `m x`. A zero impulse is exactly `μ_t`. A zero
/// evolved mean is exactly the impulse. The first-occasion map
/// `μ_0 + m x` is not this composition when the process has already
/// evolved. The level-change form is not this map.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_time_dependent_predictor_impulse`], and returns
/// [`PsychometricError::InvalidNumericInput`] when the sum overflows.
pub fn recover_discrete_latent_mean_with_impulse(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let impulse =
        recover_time_dependent_predictor_impulse(time_dependent_effect, time_dependent_predictor)?;
    if impulse == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(impulse);
    }
    require_finite(evolved_latent_mean + impulse)
}

/// Exact scalar observed mean of a contemporaneous impulse.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 1–3, pp. 4–5;
/// Table 2, p. 12; §7.2, pp. 20–21; JSS PDF re-opened 2026-08-20T09:01Z
/// from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The expected intercept is `τ`. The latent process
/// at `t` after a contemporaneous Dirac (`u = t`) is `μ_t + m x`.
/// The scalar composition is `E(y_t) = τ + λ(μ_t + m x)`. Form the
/// evolved-plus-impulse latent mean first, then `τ + λ` of that
/// mean. Table 2 names `τ` `MANIFESTMEANS`. A zero loading is
/// exactly `τ`. A zero evolved-plus-impulse latent mean is exactly
/// `τ`. A zero intercept is exactly `λ(μ_t + m x)`. The evolved
/// observed mean `τ + λ μ_t` is not this composition when the
/// impulse is nonzero. The carry map
/// `τ + λ(μ_t + e^{a(t−u)} m x)` is not this composition when
/// `u ≠ t`. `MANIFESTMEANS` is not `E(y_t)`. The
/// evolved-plus-impulse latent mean is not `E(y_t)`. The §7.2
/// level-change form is a different specification and is not this
/// map. This is not a Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean_with_impulse`] and
/// [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_impulse(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let impulse_latent_mean = recover_discrete_latent_mean_with_impulse(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        time_dependent_effect,
        time_dependent_predictor,
        event_delta,
        clock,
    )?;
    recover_manifest_observed_mean(loading, impulse_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the contemporaneous-
/// impulse observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the Eq. 3 contemporaneous impulse is `τ + λ(μ_t + m x)`.
/// Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean`].
pub fn refuse_evolved_observed_mean_as_impulse_observed_mean(
    evolved_observed_mean: f64,
    impulse_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, impulse_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean)
}

/// Refuse treating the contemporaneous-impulse observed mean as the
/// impulse-carry observed mean.
///
/// Equation 5 of the contemporaneous Dirac is `τ + λ(μ_t + m x)`.
/// Equation 5 of the Eq. 1–2 carried latent mean is
/// `τ + λ(μ_t + e^{a(t−u)} m x)`. Those are not the same map when
/// `u ≠ t`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean`].
pub fn refuse_impulse_observed_mean_as_impulse_carry_observed_mean(
    impulse_observed_mean: f64,
    impulse_carry_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (impulse_observed_mean, impulse_carry_observed_mean);
    Err(PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
}

/// Refuse treating the Eq. 3 impulse as `CINT`.
///
/// Table 2 names `M` `TDPREDEFFECT` and `κ` `CINT`. The impulse is
/// `m x`. The continuous intercept is not that jump.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseIsNotContinuousIntercept`].
pub fn refuse_time_dependent_impulse_as_continuous_intercept(
    time_dependent_impulse: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse, continuous_intercept);
    Err(PsychometricError::TimeDependentImpulseIsNotContinuousIntercept)
}

/// Refuse treating the Eq. 3 impulse as the time-independent effect.
///
/// The third summand is `A^{-1}[e^{A Δt} − I] B z`. Table 2 names
/// `B` `TIPREDEFFECT`. The fourth-summand impulse is `M x`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect`].
pub fn refuse_time_dependent_impulse_as_time_independent_effect(
    time_dependent_impulse: f64,
    time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse, time_independent_effect);
    Err(PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect)
}

/// Refuse treating the Eq. 3 impulse as Voelkle et al. (2012, Eq. 14).
///
/// Equation 14 is `a_{yx} Δt` for a piecewise-constant time-varying
/// predictor whose sampling interval equals its constancy interval.
/// The Dirac impulse is `m x`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect`].
pub fn refuse_time_dependent_impulse_as_time_varying_discrete_effect(
    time_dependent_impulse: f64,
    time_varying_discrete_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse, time_varying_discrete_effect);
    Err(PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect)
}

/// Exact scalar discrete time-independent predictor effect from
/// Driver Equation 3.
///
/// Driver, Oud, and Voelkle (2017, Eq. 1–3, pp. 4–5; Table 2, p. 12;
/// JSS PDF re-opened 2026-08-20T10:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write the latent SDE
/// `dη = (A η + b + A_{ηξ} ξ + B z) dt + G dW + M dχ`. Equation 3's
/// second summand is `A^{-1}[e^{A Δt} − I](b + A_{ηξ} ξ + B z)`.
/// Table 2 names `B` `TIPREDEFFECT`, `κ`/`b` `CINT`, and `M`
/// `TDPREDEFFECT`. The scalar map of the time-independent predictor
/// is `(e^{a Δt} − 1)/a · B z` for `a ≠ 0`. Form `B z` first, then
/// the discrete intercept map. A zero drift is the Eq. 3 integral
/// `B z Δt`. A zero effect or zero predictor is exactly zero.
/// `TIPREDEFFECT` is `B`, not that discrete increment. `B z` is not
/// `CINT`. `A^{-1}[e^{A Δt} − I] B z` is not the contemporaneous
/// impulse `M x` and is not Voelkle et al. (2012, Eq. 14) `a_{yx} Δt`.
/// This is not a Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when
/// `event_delta` is not strictly positive, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or `B z` or the mapped increment overflows.
pub fn recover_discrete_time_independent_predictor_effect(
    time_independent_effect: f64,
    time_independent_predictor: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !time_independent_effect.is_finite()
        || !time_independent_predictor.is_finite()
        || !log_rate.is_finite()
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if time_independent_effect == 0.0 || time_independent_predictor == 0.0 {
        return Ok(0.0);
    }
    let continuous = require_finite(time_independent_effect * time_independent_predictor)?;
    if log_rate == 0.0 {
        return require_finite(continuous * event_delta);
    }
    recover_discrete_constant_predictor_effect(continuous, log_rate, event_delta, clock)
}

/// Exact scalar evolved latent mean plus a time-independent predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5) write the first two
/// summands as the carried `T0MEANS`, the `CINT` increment, and the
/// `TIPREDEFFECT` increment `A^{-1}[e^{A Δt} − I] B z`. Form `μ_t`
/// first, then add that increment. A zero time-independent increment
/// is exactly `μ_t`. A zero evolved mean is exactly the increment.
/// Adding `B z` to `μ_t` is not this map. Adding `M x` is not this
/// map.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_discrete_time_independent_predictor_effect`], and
/// returns [`PsychometricError::InvalidNumericInput`] when the sum
/// overflows.
pub fn recover_discrete_latent_mean_with_time_independent_predictor(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_independent_effect: f64,
    time_independent_predictor: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let time_independent_increment = recover_discrete_time_independent_predictor_effect(
        time_independent_effect,
        time_independent_predictor,
        log_rate,
        event_delta,
        clock,
    )?;
    if time_independent_increment == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(time_independent_increment);
    }
    require_finite(evolved_latent_mean + time_independent_increment)
}

/// Refuse treating the Eq. 3 time-independent increment as `CINT`.
///
/// Table 2 names `B` `TIPREDEFFECT` and `κ` `CINT`. The discrete
/// increment is `A^{-1}[e^{A Δt} − I] B z`. The continuous intercept
/// is not that increment.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentEffectIsNotContinuousIntercept`].
pub fn refuse_time_independent_effect_as_continuous_intercept(
    time_independent_increment: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_independent_increment, continuous_intercept);
    Err(PsychometricError::TimeIndependentEffectIsNotContinuousIntercept)
}

/// Refuse treating the Eq. 3 time-independent increment as `M x`.
///
/// The fourth-summand impulse is contemporaneous. The second-summand
/// `TIPREDEFFECT` map integrates `B z` over the event interval.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse`].
pub fn refuse_time_independent_effect_as_time_dependent_impulse(
    time_independent_increment: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_independent_increment, time_dependent_impulse);
    Err(PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse)
}

/// Refuse treating the Eq. 3 time-independent increment as Voelkle
/// et al. (2012, Eq. 14).
///
/// Equation 14 is `a_{yx} Δt` for a piecewise-constant time-varying
/// predictor whose sampling interval equals its constancy interval.
/// `TIPREDEFFECT` integrates a constant `z` through the drift.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect`].
pub fn refuse_time_independent_effect_as_time_varying_discrete_effect(
    time_independent_increment: f64,
    time_varying_discrete_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_independent_increment, time_varying_discrete_effect);
    Err(PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect)
}

/// Refuse treating Driver Table 2 `TIPREDEFFECT` as the discrete
/// increment.
///
/// `B` is the continuous-time coefficient. Equation 3 maps
/// `A^{-1}[e^{A Δt} − I] B z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect`].
pub fn refuse_time_independent_coefficient_as_discrete_effect(
    time_independent_coefficient: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_independent_coefficient, time_independent_increment);
    Err(PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect)
}

/// Exact scalar §7.2 `asymTIPREDEFFECT`.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 20–21; Eq. 3, p. 5;
/// Table 2, p. 12; JSS PDF opened 2026-08-21T13:08Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `TIPREDEFFECT` the continuous-time coefficient `B`. Equation 3
/// maps a finite event interval as `A^{-1}[e^{A Δt} − I] B z`. Section
/// 7.2 then names `asymTIPREDEFFECT` the expected total change in
/// process means given an increase of 1 on the time-independent
/// predictor. For stable `a < 0` that total change is `-A^{-1} B`.
/// The scalar map is `-B z / a`. Form `B z` first, then divide by
/// `-a`. A zero coefficient or zero predictor is exactly zero.
/// `a ≥ 0` cannot hold a finite process-mean change and fails closed.
/// `-B z / a` is not the coefficient `B`, not the finite-interval
/// increment `A^{-1}[e^{A Δt} − I] B z`, not `CINT`, and not `M x`.
/// This is not a Kalman filter, not a matrix `expm`, and not ctsem
/// estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the drift is not strictly negative and the effect is nonzero,
/// and [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or `B z` or the quotient overflows.
pub fn recover_asymptotic_time_independent_predictor_effect(
    time_independent_effect: f64,
    time_independent_predictor: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !time_independent_effect.is_finite()
        || !time_independent_predictor.is_finite()
        || !log_rate.is_finite()
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if time_independent_effect == 0.0 || time_independent_predictor == 0.0 {
        return Ok(0.0);
    }
    if log_rate >= 0.0 {
        return Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift);
    }
    let continuous = require_finite(time_independent_effect * time_independent_predictor)?;
    require_finite(continuous / -log_rate)
}

/// Refuse treating §7.2 `asymTIPREDEFFECT` as `TIPREDEFFECT`.
///
/// `-B z / a` is the expected total change in process means. Table 2
/// names `B` `TIPREDEFFECT`. The coefficient is not that total change.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient`].
pub fn refuse_asymptotic_time_independent_effect_as_coefficient(
    asymptotic_effect: f64,
    time_independent_coefficient: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_effect, time_independent_coefficient);
    Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient)
}

/// Refuse treating §7.2 `asymTIPREDEFFECT` as the finite-interval
/// discrete increment.
///
/// `-B z / a` is the `Δt → ∞` limit of `A^{-1}[e^{A Δt} − I] B z`
/// under stable `a < 0`. A finite event interval is not that limit.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect`].
pub fn refuse_asymptotic_time_independent_effect_as_discrete_effect(
    asymptotic_effect: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_effect, time_independent_increment);
    Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect)
}

/// Refuse treating §7.2 `asymTIPREDEFFECT` as `CINT`.
///
/// `-B z / a` is the expected total change from a time-independent
/// predictor. Table 2 names `κ` `CINT`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept`].
pub fn refuse_asymptotic_time_independent_effect_as_continuous_intercept(
    asymptotic_effect: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_effect, continuous_intercept);
    Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept)
}

/// Refuse treating §7.2 `asymTIPREDEFFECT` as `M x`.
///
/// The fourth-summand impulse is contemporaneous. The asymptotic
/// time-independent effect is a new process mean, not a Dirac.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse`].
pub fn refuse_asymptotic_time_independent_effect_as_time_dependent_impulse(
    asymptotic_effect: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_effect, time_dependent_impulse);
    Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse)
}

/// Exact scalar §7.2 `addedTIPREDVAR`.
///
/// Driver, Oud, and Voelkle (2017, §7.2, pp. 20–21; Eq. 3, p. 5;
/// Table 2, p. 12; JSS PDF opened 2026-08-21T13:08Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `asymTIPREDEFFECT` the expected total change in process means
/// given a unit increase on a time-independent predictor. The scalar
/// map is `-B / a` for stable `a < 0`. Section 7.2 then names
/// `addedTIPREDVAR` the stable between-subject variance accounted for
/// by those predictors. For predictor variance `v ≥ 0` that variance
/// is `(-B / a)² v`. Form the unit asymptotic effect first, then
/// square, then multiply by `v`. A zero coefficient or zero predictor
/// variance is exactly zero. `v < 0` fails closed. `a ≥ 0` cannot hold
/// a finite process-mean change and fails closed. `(B / a)² v` is not
/// `TRAITVAR`, not `asymDIFFUSION`, and not the expected total change
/// `-B z / a`. This is not a Kalman filter, not a matrix `expm`, and
/// not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the drift is not strictly negative and the variance is nonzero,
/// and [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, the predictor variance is negative, or the product
/// overflows.
pub fn recover_asymptotic_time_independent_predictor_variance(
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !time_independent_effect.is_finite()
        || !predictor_variance.is_finite()
        || !log_rate.is_finite()
        || predictor_variance < 0.0
    {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if time_independent_effect == 0.0 || predictor_variance == 0.0 {
        return Ok(0.0);
    }
    let unit_effect = recover_asymptotic_time_independent_predictor_effect(
        time_independent_effect,
        1.0,
        log_rate,
        clock,
    )?;
    let squared = require_finite(unit_effect * unit_effect)?;
    require_finite(squared * predictor_variance)
}

/// Refuse treating §7.2 `addedTIPREDVAR` as `TRAITVAR`.
///
/// `(B / a)² v` is between-subject variance accounted for by a
/// time-independent predictor. Section 4.3 `TRAITVAR` is a zero-drift
/// latent process. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance`].
pub fn refuse_asymptotic_time_independent_variance_as_trait_variance(
    added_predictor_variance: f64,
    trait_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (added_predictor_variance, trait_variance);
    Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance)
}

/// Refuse treating §7.2 `addedTIPREDVAR` as `asymDIFFUSION`.
///
/// `(B / a)² v` is between-subject variance from a time-independent
/// predictor. `asymDIFFUSION` is the stationary within-subject
/// variance `-q / (2 a)`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject`].
pub fn refuse_asymptotic_time_independent_variance_as_stationary_within_subject(
    added_predictor_variance: f64,
    stationary_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (added_predictor_variance, stationary_variance);
    Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject)
}

/// Refuse treating §7.2 `addedTIPREDVAR` as `asymTIPREDEFFECT`.
///
/// `(B / a)² v` is a variance. `-B z / a` is the expected total
/// change in process means. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect`].
pub fn refuse_asymptotic_time_independent_variance_as_asymptotic_effect(
    added_predictor_variance: f64,
    asymptotic_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (added_predictor_variance, asymptotic_effect);
    Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect)
}

/// Exact scalar Eq. 5 of §7.2 `addedTIPREDVAR`.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Table 2, p. 12; §7.2,
/// pp. 20–21; 2017-era ctsem `summary.ctsemFit.R`; JSS PDF re-opened
/// 2026-08-23T19:23Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Equation 5 maps extra latent variance through
/// `Λ`. Section 7.2 names `addedTIPREDVAR` the stable between-subject
/// variance accounted for by time-independent predictors. The 2017-era
/// `summary.ctsemFit.R` forms that latent extra as
/// `asymTIPREDEFFECT %*% TIPREDVAR %*% t(asymTIPREDEFFECT)`. The
/// scalar latent extra is `(B / a)² v`. Equation 5 of that extra,
/// with `θ = 0` and `ψ = 0`, is `λ² (B / a)² v`. Form
/// `addedTIPREDVAR` first, then `(λ extra) λ`. Do not form `λ²`
/// first: at `λ = 1e308`, `extra = 1e-308`, `λ²` overflows and
/// `λ² extra` is non-finite, but `(λ extra) λ = 1e308`. A zero
/// loading or zero extra is exactly zero. `v < 0` fails closed. A
/// non-event clock fails closed. `a ≥ 0` cannot hold a finite
/// process-mean change when the extra is nonzero and fails closed.
/// `(B / a)² v` is the latent extra and is not this observed extra.
/// `λ² t0_b² v` is Eq. 5 of `addedT0TIPREDVAR` and is not this
/// asymptotic observed extra. `λ² p + θ` is stationary
/// observed-indicator variance and is not this extra. `MANIFESTVAR`
/// `θ` is measurement error and is not this extra. `Ψ` is intercept
/// variance and is not extra TI. The printed 2-latent
/// `addedTIPREDVAR` 2.838 is not this scalar map. This is not a
/// Kalman filter, not a matrix `expm`, not DSEM, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_asymptotic_time_independent_predictor_variance`]
/// and [`recover_manifest_observed_variance`].
pub fn recover_asymptotic_time_independent_observed_variance(
    loading: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let extra = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    recover_manifest_observed_variance(loading, extra, 0.0)
}

/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as the latent extra.
///
/// `λ² (B / a)² v` is extra observed-indicator variance.
/// `(B / a)² v` is extra latent variance. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance`].
pub fn refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance(
    asymptotic_observed_predictor_variance: f64,
    asymptotic_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        asymptotic_observed_predictor_variance,
        asymptotic_predictor_variance,
    );
    Err(
        PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance,
    )
}

/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as Eq. 5 of
/// `addedT0TIPREDVAR`.
///
/// `λ² (B / a)² v` uses the asymptotic unit effect `-B / a` and
/// requires stable `a < 0`. `λ² t0_b² v` uses free first-occasion
/// `T0TIPREDEFFECT`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance`].
pub fn refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance(
    asymptotic_observed_predictor_variance: f64,
    initial_observed_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        asymptotic_observed_predictor_variance,
        initial_observed_predictor_variance,
    );
    Err(
        PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance,
    )
}

/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as stationary
/// observed-indicator variance.
///
/// `λ² (B / a)² v` is extra observed TI variance. `λ² p + θ` is
/// stationary observed-indicator variance. Those are not the same
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance`].
pub fn refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance(
    asymptotic_observed_predictor_variance: f64,
    stationary_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        asymptotic_observed_predictor_variance,
        stationary_observed_variance,
    );
    Err(PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance)
}

/// Refuse treating Eq. 5 of §7.2 `addedTIPREDVAR` as `MANIFESTVAR`.
///
/// `λ² (B / a)² v` is extra observed TI variance. Table 2 names
/// `MANIFESTVAR` as `Θ`, the variance of `ζ`. Those are not the
/// same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError`].
pub fn refuse_asymptotic_time_independent_observed_variance_as_measurement_error(
    asymptotic_observed_predictor_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        asymptotic_observed_predictor_variance,
        measurement_error_variance,
    );
    Err(PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError)
}

/// Exact scalar Table 2 `asymCINT`.
///
/// Driver, Oud, and Voelkle (2017, Table 2, p. 12; Eq. 3, p. 5;
/// §4.3 / p. 16; JSS PDF opened 2026-08-21T16:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `asymCINT` the asymptotic (`Δt = ∞`) expected change in
/// processes for a 1 unit change in intercept (`CINT`). Table 2 names
/// `κ` `CINT`. Equation 3 maps a finite event interval as
/// `A^{-1}[e^{A Δt} − I] κ`. For stable `a < 0` that `Δt → ∞` limit
/// is `-A^{-1} κ`. The scalar map is `-κ / a`. A unit intercept is
/// `-1 / a`. Form `κ` first, then divide by `-a`. A zero intercept is
/// exactly zero. `a ≥ 0` cannot hold a finite process-mean change and
/// fails closed. `-κ / a` is not `κ`, not the finite-interval
/// increment `A^{-1}[e^{A Δt} − I] κ`, not `T0MEANS`, and not
/// `asymTIPREDEFFECT` `-B z / a`. Page 16 notes that a `T0MEANS`
/// stationarity constraint includes time-independent predictors; that
/// composition is not this intercept-only map. The printed 2-latent
/// `CINT` values are not this scalar map. This is not a Kalman filter,
/// not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift`]
/// when the drift is not strictly negative and the intercept is
/// nonzero, and [`PsychometricError::InvalidNumericInput`] when an
/// input is non-finite or the quotient overflows.
pub fn recover_asymptotic_continuous_intercept(
    continuous_intercept: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !continuous_intercept.is_finite() || !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if continuous_intercept == 0.0 {
        return Ok(0.0);
    }
    if log_rate >= 0.0 {
        return Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift);
    }
    require_finite(continuous_intercept / -log_rate)
}

/// Refuse treating Table 2 `asymCINT` as `CINT`.
///
/// `-κ / a` is the expected change in process means. Table 2 names
/// `κ` `CINT`. The intercept is not that total change.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept`].
pub fn refuse_asymptotic_continuous_intercept_as_continuous_intercept(
    asymptotic_intercept: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_intercept, continuous_intercept);
    Err(PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept)
}

/// Refuse treating Table 2 `asymCINT` as the finite-interval discrete
/// intercept increment.
///
/// `-κ / a` is the `Δt → ∞` limit of `A^{-1}[e^{A Δt} − I] κ` under
/// stable `a < 0`. A finite event interval is not that limit.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement`].
pub fn refuse_asymptotic_continuous_intercept_as_discrete_increment(
    asymptotic_intercept: f64,
    discrete_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_intercept, discrete_increment);
    Err(PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement)
}

/// Refuse treating Table 2 `asymCINT` as `T0MEANS`.
///
/// `-κ / a` is the intercept contribution to the stationary process
/// mean. Table 2 names `μ_0` `T0MEANS`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean`].
pub fn refuse_asymptotic_continuous_intercept_as_initial_latent_mean(
    asymptotic_intercept: f64,
    initial_latent_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_intercept, initial_latent_mean);
    Err(PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean)
}

/// Refuse treating Table 2 `asymCINT` as `asymTIPREDEFFECT`.
///
/// `-κ / a` is the intercept contribution. `-B z / a` is the
/// time-independent predictor contribution. Page 16 notes that a
/// `T0MEANS` stationarity constraint includes time-independent
/// predictors; that composition is not this intercept-only map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect`].
pub fn refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect(
    asymptotic_intercept: f64,
    asymptotic_time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_intercept, asymptotic_time_independent_effect);
    Err(PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect)
}

/// Exact scalar p. 16 stationary `T0MEANS`.
///
/// Driver, Oud, and Voelkle (2017, p. 16; Table 2, p. 12; Eq. 3, p. 5;
/// JSS PDF opened 2026-08-21T16:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain `T0MEANS` to the model-implied values using
/// `T0MEANSbase` / `T0MEANSfree` when the first observation is
/// determined by the process in the same way as later observations.
/// Those constraints include extra effects due to time-independent
/// predictors (`asymTIPREDEFFECT`). Table 2 names `κ` `CINT` and
/// names `asymCINT` the `Δt → ∞` intercept contribution `-κ / a`.
/// For stable `a < 0` the scalar composition is
/// `-κ / a + −B z / a`. Form the intercept contribution first, then
/// include the TI extra effect, then add. A zero intercept and a zero
/// TI contribution is exactly zero. `a ≥ 0` cannot hold a finite
/// process-mean change when either contribution is nonzero and fails
/// closed. That constrained first-occasion mean is not free
/// `T0MEANS`, not `asymCINT` alone, not `asymTIPREDEFFECT` alone, and
/// not the finite-interval discrete latent mean
/// `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`. The printed 2-latent
/// `T0MEANS` 2.823 is not this scalar map. This is not a Kalman
/// filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift`]
/// or [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the drift is not strictly negative and the corresponding
/// contribution is nonzero, and [`PsychometricError::InvalidNumericInput`]
/// when an input is non-finite or a quotient or sum overflows.
pub fn recover_stationary_initial_latent_mean(
    continuous_intercept: f64,
    time_independent_effect: f64,
    time_independent_predictor: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let intercept = recover_asymptotic_continuous_intercept(continuous_intercept, log_rate, clock)?;
    let tipred = recover_asymptotic_time_independent_predictor_effect(
        time_independent_effect,
        time_independent_predictor,
        log_rate,
        clock,
    )?;
    require_finite(intercept + tipred)
}

/// Refuse treating p. 16 stationary `T0MEANS` as free `T0MEANS`.
///
/// `-κ / a + −B z / a` is the constrained first-occasion mean. Table 2
/// names the free first-occasion latent mean `T0MEANS`. Those are not
/// the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean`].
pub fn refuse_stationary_initial_latent_mean_as_initial_latent_mean(
    stationary_mean: f64,
    initial_latent_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_mean, initial_latent_mean);
    Err(PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean)
}

/// Refuse treating p. 16 stationary `T0MEANS` as `asymCINT`.
///
/// The constraint includes time-independent predictors. `-κ / a` is
/// the intercept contribution and is not that composition when
/// `B z ≠ 0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept`].
pub fn refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept(
    stationary_mean: f64,
    asymptotic_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_mean, asymptotic_intercept);
    Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept)
}

/// Refuse treating p. 16 stationary `T0MEANS` as `asymTIPREDEFFECT`.
///
/// The constraint includes the intercept contribution. `-B z / a` is
/// the TI extra effect and is not that composition when `κ ≠ 0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect`].
pub fn refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect(
    stationary_mean: f64,
    asymptotic_time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_mean, asymptotic_time_independent_effect);
    Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect)
}

/// Refuse treating p. 16 stationary `T0MEANS` as a finite-interval
/// discrete latent mean.
///
/// `-κ / a + −B z / a` is the `Δt → ∞` constrained first-occasion
/// mean. `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ` is a finite event
/// interval and is not that limit.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean`].
pub fn refuse_stationary_initial_latent_mean_as_discrete_mean(
    stationary_mean: f64,
    discrete_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_mean, discrete_mean);
    Err(PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean)
}

/// Exact scalar observed mean of §4.3 stationary `T0MEANS`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Table 2, p. 12; Eq. 3, p. 5; JSS PDF re-opened 2026-08-21T20:07Z
/// from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain the first-occasion mean to the model-predicted mean
/// when `stationary` includes `"T0MEANS"`. Equation 5 writes
/// `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The constrained latent mean is
/// `-κ / a + −B z / a`. The scalar composition is
/// `E(y_0) = τ + λ(−κ / a + −B z / a)`. Form the stationary latent
/// mean first, then `τ + λ` of that mean. A zero loading is exactly
/// `τ`. A zero intercept and a zero TI contribution is exactly `τ`.
/// `τ + λ μ_0` for free `T0MEANS` is not this composition.
/// `τ + λ(−κ / a)` is not this composition when `B z ≠ 0`.
/// `τ + λ μ_t` is not this composition. `MANIFESTMEANS` is not
/// `E(y_0)`. The constrained latent mean is not `E(y_0)`. This is
/// not a Kalman filter, not a matrix `expm`, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_initial_latent_mean`] and
/// [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_initial_observed_mean(
    loading: f64,
    continuous_intercept: f64,
    time_independent_effect: f64,
    time_independent_predictor: f64,
    log_rate: f64,
    manifest_mean: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let stationary_latent_mean = recover_stationary_initial_latent_mean(
        continuous_intercept,
        time_independent_effect,
        time_independent_predictor,
        log_rate,
        clock,
    )?;
    recover_manifest_observed_mean(loading, stationary_latent_mean, manifest_mean)
}

/// Refuse treating §4.3 stationary `T0MEANS` as `E(y_0)`.
///
/// `−κ / a + −B z / a` is the constrained latent mean. Equation 5
/// maps `E(y_0) = τ + λ` of that mean.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentMeanIsNotObservedMean`].
pub fn refuse_stationary_initial_latent_mean_as_observed_mean(
    stationary_latent_mean: f64,
    stationary_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_latent_mean, stationary_observed_mean);
    Err(PsychometricError::StationaryInitialLatentMeanIsNotObservedMean)
}

/// Refuse treating `MANIFESTMEANS` as Eq. 5 of §4.3 stationary
/// `T0MEANS`.
///
/// Table 2 names `τ` `MANIFESTMEANS`. `τ + λ(−κ / a + −B z / a)` is
/// not `τ` when the loading and constrained mean are nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans`].
pub fn refuse_stationary_initial_observed_mean_as_manifest_means(
    stationary_observed_mean: f64,
    manifest_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_observed_mean, manifest_mean);
    Err(PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans)
}

/// Refuse treating evolved `τ + λ μ_t` as Eq. 5 of §4.3 stationary
/// `T0MEANS`.
///
/// A finite-interval evolved observed mean is not the constrained
/// first-occasion observed mean.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean`].
pub fn refuse_evolved_observed_mean_as_stationary_initial_observed_mean(
    evolved_observed_mean: f64,
    stationary_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, stationary_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean)
}

/// Refuse treating `τ + λ(−κ / a)` as Eq. 5 of §4.3 stationary
/// `T0MEANS`.
///
/// The constraint includes time-independent predictors.
/// `τ + λ(−κ / a)` is not that composition when `B z ≠ 0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean`].
pub fn refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean(
    asymptotic_intercept_observed_mean: f64,
    stationary_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (asymptotic_intercept_observed_mean, stationary_observed_mean);
    Err(
        PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean,
    )
}

/// Refuse treating `τ + λ μ_0` as Eq. 5 of §4.3 stationary
/// `T0MEANS`.
///
/// Free first-occasion `T0MEANS` is not the constrained
/// first-occasion mean.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean`].
pub fn refuse_initial_observed_mean_as_stationary_initial_observed_mean(
    initial_observed_mean: f64,
    stationary_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_observed_mean, stationary_observed_mean);
    Err(PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean)
}

/// Exact scalar §4.3 / p. 16 stationary `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; p. 16; Table 2,
/// p. 12; §7.2, pp. 20–21; Eq. 4, p. 5; JSS PDF re-opened
/// 2026-08-22T03:07Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain `T0VAR` to the model-predicted variance when
/// `stationary` includes `"T0VAR"`. Section 4.3 writes that the
/// first-occasion variances are constrained according to the model
/// predicted variances across all time points. Page 16 names
/// `asymDIFFUSION` the total within-subject variance as `Δt → ∞`.
/// For stable `a < 0` that scalar is `-q / (2 a)`. Section 4.3
/// (p. 9) adds a stable trait process with `DRIFT` and `DIFFUSION`
/// fixed to zero (`TRAITVAR`). Section 7.2 names `addedTIPREDVAR`
/// the stable between-subject variance accounted for by
/// time-independent predictors; the scalar map is `(B / a)² v`.
/// The constrained first-occasion variance is
/// `trait + −q / (2 a) + (B / a)² v`. Form the within-subject
/// contribution first, then include the trait, then include the TI
/// extra variance, then add. A zero trait, a zero diffusion, and a
/// zero TI contribution is exactly zero. A zero diffusion and a
/// zero TI contribution is exactly the trait. `a ≥ 0` cannot hold a
/// finite process variance when the diffusion or the TI
/// contribution is nonzero and fails closed. Trait-only variance
/// does not require a stable drift. That constrained
/// first-occasion variance is not free `T0VAR`, not
/// `asymDIFFUSION` alone, not `TRAITVAR` alone, not
/// `addedTIPREDVAR` alone, and not the finite-interval discrete
/// latent variance `exp(2 a Δt) p + Q_Δt`. The printed 2-latent
/// `addedTIPREDVAR` 2.838 is not this scalar map. This is not a
/// Kalman filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`]
/// when the diffusion is nonzero and the drift is not strictly
/// negative,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
pub fn recover_stationary_initial_latent_variance(
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let state = if continuous_diffusion == 0.0 {
        0.0
    } else {
        recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?
    };
    let trait_plus_state = recover_trait_plus_state_latent_variance(trait_variance, state)?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_state + added)
}

/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as free `T0VAR`.
///
/// `trait + −q / (2 a) + (B / a)² v` is the constrained
/// first-occasion variance. Table 2 names the free first-occasion
/// latent variance `T0VAR`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance`].
pub fn refuse_stationary_initial_latent_variance_as_initial_latent_variance(
    stationary_variance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_variance, initial_latent_variance);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance)
}

/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as
/// `asymDIFFUSION`.
///
/// The constraint includes trait variance and time-independent
/// predictor variance. `-q / (2 a)` is the within-subject
/// contribution and is not that composition when `TRAITVAR` or
/// `addedTIPREDVAR` is nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject`].
pub fn refuse_stationary_initial_latent_variance_as_stationary_within_subject(
    stationary_t0_variance: f64,
    asymptotic_within_subject: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_t0_variance, asymptotic_within_subject);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject)
}

/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as `TRAITVAR`.
///
/// The constraint includes the within-subject process variance and
/// time-independent predictor variance. `TRAITVAR` is not that
/// composition when those contributions are nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance`].
pub fn refuse_stationary_initial_latent_variance_as_trait_variance(
    stationary_t0_variance: f64,
    trait_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_t0_variance, trait_variance);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance)
}

/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as
/// `addedTIPREDVAR`.
///
/// The constraint includes trait variance and `asymDIFFUSION`.
/// `(B / a)² v` is the TI extra variance and is not that
/// composition when those contributions are nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance`].
pub fn refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance(
    stationary_t0_variance: f64,
    added_predictor_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_t0_variance, added_predictor_variance);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance)
}

/// Refuse treating §4.3 / p. 16 stationary `T0VAR` as a
/// finite-interval discrete latent variance.
///
/// `trait + −q / (2 a) + (B / a)² v` is the `Δt → ∞` constrained
/// first-occasion variance. `exp(2 a Δt) p + Q_Δt` is a finite
/// event interval and is not that limit.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance`].
pub fn refuse_stationary_initial_latent_variance_as_discrete_variance(
    stationary_t0_variance: f64,
    discrete_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_t0_variance, discrete_variance);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance)
}

/// Exact scalar Eq. 5 of §4.3 / p. 16 stationary `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-22T03:20Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain `T0VAR` to the model-predicted variance when
/// `stationary` includes `"T0VAR"`. Equation 5 writes
/// `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The constrained latent variance is
/// `trait + −q / (2 a) + (B / a)² v`. The scalar composition is
/// `Var(y_0) = λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`. Form
/// the stationary latent variance first, then `λ² p + θ + ψ`. A
/// zero loading is exactly `θ + ψ`. A zero trait, a zero diffusion,
/// and a zero TI contribution is exactly `θ + ψ`. `λ² p_0` for
/// free `T0VAR` is not this composition. `λ²(−q / (2 a)) + θ` is
/// not this composition when `TRAITVAR` or `addedTIPREDVAR` is
/// nonzero. Evolving the constrained variance as if it were all
/// state is not this composition when the trait or TI contribution
/// is nonzero. `MANIFESTVAR` is not `Var(y_0)`. The constrained
/// latent variance is not `Var(y_0)`. `TRAITVAR` is latent and is
/// scaled by `λ²`; `MANIFESTTRAITVAR` is not. This is not a Kalman
/// filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_initial_latent_variance`] and
/// [`recover_manifest_trait_plus_state_observed_variance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_initial_observed_variance(
    loading: f64,
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let stationary_latent_variance = recover_stationary_initial_latent_variance(
        trait_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    recover_manifest_trait_plus_state_observed_variance(
        loading,
        stationary_latent_variance,
        measurement_error_variance,
        manifest_trait_variance,
    )
}

/// Refuse treating §4.3 stationary `T0VAR` as `Var(y_0)`.
///
/// `trait + −q / (2 a) + (B / a)² v` is the constrained latent
/// variance. Equation 5 maps `Var(y_0) = λ²` of that variance plus
/// `θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance`].
pub fn refuse_stationary_initial_latent_variance_as_observed_variance(
    stationary_latent_variance: f64,
    stationary_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_latent_variance, stationary_observed_variance);
    Err(PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of §4.3 stationary
/// `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`.
/// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` is not `θ` when
/// the loading and constrained variance are nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError`].
pub fn refuse_stationary_initial_observed_variance_as_measurement_error(
    stationary_observed_variance: f64,
    measurement_error_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (stationary_observed_variance, measurement_error_variance);
    Err(PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError)
}

/// Refuse treating evolved `λ² Var(η_t) + θ` as Eq. 5 of §4.3
/// stationary `T0VAR`.
///
/// Evolving the constrained first-occasion variance as if it were
/// all state is not `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`
/// when the trait or TI contribution is nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance`].
pub fn refuse_evolved_observed_variance_as_stationary_initial_observed_variance(
    evolved_observed_variance: f64,
    stationary_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_variance, stationary_observed_variance);
    Err(PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance)
}

/// Refuse treating Eq. 5 of `asymDIFFUSION` as Eq. 5 of §4.3
/// stationary `T0VAR`.
///
/// `λ²(−q / (2 a)) + θ` is the within-subject observed contribution
/// and is not that composition when `TRAITVAR` or `addedTIPREDVAR`
/// is nonzero.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance`].
pub fn refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance(
    within_subject_observed_variance: f64,
    stationary_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        within_subject_observed_variance,
        stationary_observed_variance,
    );
    Err(
        PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance,
    )
}

/// Refuse treating Eq. 5 of free `T0VAR` as Eq. 5 of §4.3
/// stationary `T0VAR`.
///
/// `λ² p_0 + θ` is the free first-occasion observed variance.
/// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` is not that map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance`].
pub fn refuse_initial_observed_variance_as_stationary_initial_observed_variance(
    free_initial_observed_variance: f64,
    stationary_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (free_initial_observed_variance, stationary_observed_variance);
    Err(PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance)
}

/// Exact scalar lagged covariance of §4.3 / p. 16 stationary
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-22T19:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain `T0VAR` to the model-predicted variance when
/// `stationary` includes `"T0VAR"`. Equation 3 writes
/// `η(t) = exp(A Δt) η(t0) + …`. Equation 4 writes
/// `cov(η_t, η_{t-1}) = A_Δt cov(η_{t-1})`. Page 16 names
/// `asymDIFFUSION` the total within-subject variance `-q / (2 a)`.
/// Section 4.3 (p. 9) adds a stable trait process with `DRIFT` and
/// `DIFFUSION` fixed to zero (`TRAITVAR`). Section 7.2 names
/// `addedTIPREDVAR` the stable between-subject variance accounted
/// for by time-independent predictors; the scalar map is
/// `(B / a)² v`. Trait variance and that TI extra variance are
/// time-invariant between-subject; they do not decay with
/// `e^{a Δt}`. The lagged covariance of the constrained process is
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v`. Form the lagged
/// within-subject covariance first, then include the trait, then
/// include the TI extra variance, then add. A zero trait, a zero
/// diffusion, and a zero TI contribution is exactly zero. A zero
/// diffusion and a zero TI contribution is exactly the trait.
/// As `Δt → ∞` with stable `a < 0` the state term vanishes and the
/// lagged covariance is `trait + (B / a)² v`. As `Δt → 0+` the
/// lagged covariance approaches contemporaneous `T0VAR`. Those
/// limits are not this finite-lag map. Evolving the constrained
/// total as if it were all state is not this map.
/// `trait + e^{a Δt} p` is not this map when `addedTIPREDVAR` is
/// nonzero. Contemporaneous `T0VAR` is not this map. `a ≥ 0` cannot
/// hold a finite process variance when the diffusion or the TI
/// contribution is nonzero and fails closed. Trait-only covariance
/// does not require a stable drift. The interval must be event time
/// and strictly positive. This is not a Kalman filter, not a matrix
/// `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_initial_latent_variance`] path
/// refusals and [`recover_trait_plus_state_lagged_covariance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`]
/// when the diffusion is nonzero and the drift is not strictly
/// negative,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_lagged_latent_covariance(
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let state = if continuous_diffusion == 0.0 {
        0.0
    } else {
        recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?
    };
    let trait_plus_state = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        state,
        log_rate,
        event_delta,
        clock,
    )?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_state + added)
}

/// Refuse treating lagged §4.3 stationary `T0VAR` as contemporaneous
/// stationary `T0VAR`.
///
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` is the lagged
/// covariance at a strictly positive event interval.
/// `trait + −q / (2 a) + (B / a)² v` is the first-occasion
/// variance. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance`].
pub fn refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance(
    lagged_covariance: f64,
    contemporaneous_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (lagged_covariance, contemporaneous_variance);
    Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance)
}

/// Refuse treating lagged §4.3 stationary `T0VAR` as decayed total
/// stationary variance.
///
/// Evolving `trait + −q / (2 a) + (B / a)² v` as if it were all
/// state yields `e^{a Δt}` of that total. Trait variance and
/// `addedTIPREDVAR` do not decay.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance`].
pub fn refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance(
    lagged_covariance: f64,
    decayed_total: f64,
) -> Result<f64, PsychometricError> {
    let _ = (lagged_covariance, decayed_total);
    Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance)
}

/// Refuse treating §4.3 trait-plus-state lagged covariance as lagged
/// stationary `T0VAR`.
///
/// `trait + e^{a Δt} p` omits `addedTIPREDVAR`. The constrained
/// lagged covariance includes that TI extra variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance`].
pub fn refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance(
    trait_plus_state_lagged: f64,
    stationary_lagged: f64,
) -> Result<f64, PsychometricError> {
    let _ = (trait_plus_state_lagged, stationary_lagged);
    Err(PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance)
}

/// Exact scalar Eq. 5 of lagged §4.3 / p. 16 stationary `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-22T19:13Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Independent measurement error does not enter
/// `cov(y_t, y_{t-1})`. The lagged latent covariance is
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v`. The scalar
/// composition is
/// `cov(y_t, y_{t-1}) = λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ`.
/// Form the lagged latent covariance first, then `λ² c + ψ`. A zero
/// loading is exactly `ψ`. A zero trait, a zero diffusion, and a
/// zero TI contribution is exactly `ψ`. `MANIFESTVAR` `θ` is not
/// this composition. Contemporaneous `Var(y_0)` includes `θ` and is
/// not this composition. The lagged latent covariance is not this
/// observed covariance. Evolving the constrained total as if it
/// were all state is not this composition when the trait or TI
/// contribution is nonzero. `TRAITVAR` is latent and is scaled by
/// `λ²`; `MANIFESTTRAITVAR` is not. This is not a Kalman filter,
/// not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_lagged_latent_covariance`] and
/// [`recover_manifest_lagged_observed_covariance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_lagged_observed_covariance(
    loading: f64,
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let lagged_latent = recover_stationary_lagged_latent_covariance(
        trait_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    recover_manifest_lagged_observed_covariance(loading, lagged_latent, manifest_trait_variance)
}

/// Refuse treating lagged §4.3 stationary `T0VAR` as lagged observed
/// covariance.
///
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` is the lagged latent
/// covariance. Equation 5 maps `cov(y_t, y_{t-1}) = λ²` of that
/// covariance plus `ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance`].
pub fn refuse_stationary_lagged_latent_covariance_as_observed_covariance(
    lagged_latent_covariance: f64,
    lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (lagged_latent_covariance, lagged_observed_covariance);
    Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of lagged §4.3 stationary
/// `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. Independent `ε_t` does not
/// enter `cov(y_t, y_{t-1})`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance`].
pub fn refuse_measurement_error_as_stationary_lagged_observed_covariance(
    measurement_error_variance: f64,
    lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (measurement_error_variance, lagged_observed_covariance);
    Err(PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance)
}

/// Refuse treating Eq. 5 of contemporaneous §4.3 stationary `T0VAR`
/// as lagged stationary observed covariance.
///
/// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` is
/// contemporaneous and includes `θ`.
/// `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ` is not that
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance`].
pub fn refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance(
    contemporaneous_observed_variance: f64,
    lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        contemporaneous_observed_variance,
        lagged_observed_covariance,
    );
    Err(PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance)
}

/// Exact scalar later-occasion variance of §4.3 / p. 16 stationary
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-22T23:05Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// constrain the first-occasion variance according to the
/// model-predicted variances across all time points when `stationary`
/// includes `"T0VAR"`. Equation 3 writes `η(t) = exp(A Δt) η(t0) + … +`
/// the stochastic integral. Equation 4 writes that the integral
/// exhibits covariance `Q_Δt`. The law of total variance on the
/// within-subject state is `e^{2 a Δt}(−q / (2 a)) + Q_Δt`. Trait
/// variance and `addedTIPREDVAR` are time-invariant between-subject;
/// they do not enter that process-noise integral. The later-occasion
/// composition is
/// `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v`. Form the
/// evolved within-subject variance first, then include the trait,
/// then include the TI extra variance, then add. A zero trait, a
/// zero diffusion, and a zero TI contribution is exactly zero. A
/// zero diffusion and a zero TI contribution is exactly the trait.
/// Under stationarity that composition equals contemporaneous
/// `T0VAR`. Evolving the constrained total as if it were all state
/// (`e^{2 a Δt} p + Q_Δt`) is not this map. The lagged covariance
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` omits `Q_Δt` and is
/// not this map. `Q_Δt` is not this map. `a ≥ 0` cannot hold a
/// finite process variance when the diffusion or the TI contribution
/// is nonzero and fails closed. Trait-only variance does not require
/// a stable drift. The interval must be event time and strictly
/// positive. This is not a Kalman filter, not a matrix `expm`, and
/// not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_initial_latent_variance`] path
/// refusals and [`recover_discrete_latent_variance`]. Returns
/// [`PsychometricError::EventTimeRequired`] for any non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::StationaryVarianceRequiresStableDrift`]
/// when the diffusion is nonzero and the drift is not strictly
/// negative,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_later_latent_variance(
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let state = if continuous_diffusion == 0.0 {
        0.0
    } else {
        recover_stationary_latent_variance(continuous_diffusion, log_rate, clock)?
    };
    let evolved_state = recover_discrete_latent_variance(
        state,
        continuous_diffusion,
        log_rate,
        event_delta,
        clock,
    )?;
    let trait_plus_evolved =
        recover_trait_plus_state_latent_variance(trait_variance, evolved_state)?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_evolved + added)
}

/// Refuse treating later-occasion §4.3 stationary `T0VAR` as lagged
/// stationary covariance.
///
/// `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v` is the
/// unconditional variance at a later event occasion. The lagged
/// covariance omits `Q_Δt` and uses `e^{a Δt}` of the state.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance`].
pub fn refuse_stationary_later_latent_variance_as_lagged_covariance(
    later_variance: f64,
    lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (later_variance, lagged_covariance);
    Err(PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance)
}

/// Refuse treating later-occasion §4.3 stationary `T0VAR` as the free
/// discrete evolution of the constrained total.
///
/// Evolving `trait + −q / (2 a) + (B / a)² v` as if it were all
/// state yields `e^{2 a Δt}` of that total plus `Q_Δt`. Trait
/// variance and `addedTIPREDVAR` do not enter `Q_Δt`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance`].
pub fn refuse_stationary_later_latent_variance_as_discrete_variance(
    later_variance: f64,
    free_discrete_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (later_variance, free_discrete_variance);
    Err(PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance)
}

/// Refuse treating later-occasion §4.3 stationary `T0VAR` as
/// finite-interval process noise.
///
/// `Q_Δt` is the covariance of the stochastic integral. The
/// later-occasion composition includes the trait, the evolved state,
/// and `addedTIPREDVAR`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise`].
pub fn refuse_stationary_later_latent_variance_as_process_noise(
    later_variance: f64,
    process_noise: f64,
) -> Result<f64, PsychometricError> {
    let _ = (later_variance, process_noise);
    Err(PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise)
}

/// Exact scalar Eq. 5 of later-occasion §4.3 / p. 16 stationary
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-22T23:05Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The later-occasion latent variance is
/// `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v`. The scalar
/// composition is
/// `Var(y_t) = λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`.
/// Form the later-occasion latent variance first, then `λ² p + θ + ψ`.
/// A zero loading is exactly `θ + ψ`. A zero trait, a zero diffusion,
/// and a zero TI contribution is exactly `θ + ψ`. Under stationarity
/// that composition equals contemporaneous `Var(y_0)`. The lagged
/// observed covariance omits `Q_Δt` and `θ`. `MANIFESTVAR` `θ` is
/// not this composition. The later-occasion latent variance is not
/// this observed variance. `TRAITVAR` is latent and is scaled by
/// `λ²`; `MANIFESTTRAITVAR` is not. This is not a Kalman filter, not
/// a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_stationary_later_latent_variance`] and
/// [`recover_manifest_trait_plus_state_observed_variance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_stationary_later_observed_variance(
    loading: f64,
    trait_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let later_latent = recover_stationary_later_latent_variance(
        trait_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    recover_manifest_trait_plus_state_observed_variance(
        loading,
        later_latent,
        measurement_error_variance,
        manifest_trait_variance,
    )
}

/// Refuse treating later-occasion §4.3 stationary `T0VAR` as
/// later-occasion observed variance.
///
/// `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v` is the
/// later-occasion latent variance. Equation 5 maps `Var(y_t) = λ²`
/// of that variance plus `θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance`].
pub fn refuse_stationary_later_latent_variance_as_observed_variance(
    later_latent_variance: f64,
    later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (later_latent_variance, later_observed_variance);
    Err(PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-occasion §4.3
/// stationary `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. `θ` is not
/// `λ²(trait + e^{2 a Δt} p + Q_Δt + (B / a)² v) + θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance`].
pub fn refuse_measurement_error_as_stationary_later_observed_variance(
    measurement_error_variance: f64,
    later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (measurement_error_variance, later_observed_variance);
    Err(PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance)
}

/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as
/// later-occasion stationary observed variance.
///
/// `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ` omits `Q_Δt`
/// and `θ`.
/// `λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`
/// is not that map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance`].
pub fn refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance(
    lagged_observed_covariance: f64,
    later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (lagged_observed_covariance, later_observed_variance);
    Err(PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance)
}

/// Exact scalar later-occasion variance of §4.3 predetermined
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T05:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// treat the first time point as predetermined when no assumptions
/// are made about the process prior to the initial time point. Free
/// `T0VAR` `p_0` is then estimated. The process gradually
/// transitions from the variances of the initial parameters toward
/// those of the parameters when the model is stationary. Equation 3
/// writes `η(t) = exp(A Δt) η(t0) + … +` the stochastic integral.
/// Equation 4 writes that the integral exhibits covariance `Q_Δt`.
/// The law of total variance on the within-subject state is
/// `e^{2 a Δt} p_0 + Q_Δt`. Trait variance and `addedTIPREDVAR` are
/// time-invariant between-subject; they do not enter that
/// process-noise integral. The later-occasion composition is
/// `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v`. Form the evolved
/// free first-occasion variance first, then include the trait, then
/// include the TI extra variance, then add. A zero trait, a zero
/// initial variance, a zero diffusion, and a zero TI contribution is
/// exactly zero. A zero diffusion, a zero initial variance, and a
/// zero TI contribution is exactly the trait. As `Δt → ∞` with
/// stable `a < 0` the carried `p_0` vanishes and `Q_Δt` approaches
/// `−q / (2 a)`, so the composition approaches contemporaneous
/// stationary `T0VAR`. As `Δt → 0+` the composition approaches
/// `trait + p_0 + (B / a)² v`. Setting `p_0 = −q / (2 a)` recovers
/// the stationary later-occasion map. Evolving
/// `trait + p_0 + (B / a)² v` as if it were all state
/// (`e^{2 a Δt}` of that total plus `Q_Δt`) is not this map. Free
/// `T0VAR` `p_0` is not this map. Stationary later-occasion
/// variance uses `−q / (2 a)` in place of `p_0` and is not this map
/// when `p_0` is free. `a ≥ 0` cannot hold a finite TI extra
/// variance when that contribution is nonzero and fails closed.
/// Nonzero diffusion with `a ≥ 0` is a growing process and is kept.
/// Trait-only variance does not require a stable drift. The
/// interval must be event time and strictly positive. This is not a
/// Kalman filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_variance`],
/// [`recover_trait_plus_state_latent_variance`], and
/// [`recover_asymptotic_time_independent_predictor_variance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_latent_variance(
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let evolved_state = recover_discrete_latent_variance(
        initial_latent_variance,
        continuous_diffusion,
        log_rate,
        event_delta,
        clock,
    )?;
    let trait_plus_evolved =
        recover_trait_plus_state_latent_variance(trait_variance, evolved_state)?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_evolved + added)
}

/// Refuse treating predetermined later-occasion variance as later-
/// occasion stationary `T0VAR`.
///
/// `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v` uses free `T0VAR`.
/// `trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v` uses the
/// stationary within-subject variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance`].
pub fn refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance(
    predetermined_later_variance: f64,
    stationary_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_later_variance, stationary_later_variance);
    Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance)
}

/// Refuse treating predetermined later-occasion variance as the free
/// discrete evolution of the total.
///
/// Evolving `trait + p_0 + (B / a)² v` as if it were all state
/// yields `e^{2 a Δt}` of that total plus `Q_Δt`. Trait variance
/// and `addedTIPREDVAR` do not enter `Q_Δt`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLatentVarianceIsNotDiscreteVariance`].
pub fn refuse_predetermined_later_latent_variance_as_discrete_variance(
    predetermined_later_variance: f64,
    free_discrete_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_later_variance, free_discrete_variance);
    Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotDiscreteVariance)
}

/// Refuse treating predetermined later-occasion variance as free
/// first-occasion `T0VAR`.
///
/// `p_0` is the predetermined first-occasion state variance.
/// `e^{2 a Δt} p_0 + Q_Δt` is not `p_0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance`].
pub fn refuse_predetermined_later_latent_variance_as_initial_latent_variance(
    predetermined_later_variance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_later_variance, initial_latent_variance);
    Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance)
}

/// Exact scalar Eq. 5 of later-occasion §4.3 predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-23T05:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The predetermined later-occasion latent variance
/// is `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v`. The scalar
/// composition is
/// `Var(y_t) = λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`.
/// Form the predetermined later-occasion latent variance first,
/// then `λ² p + θ + ψ`. A zero loading is exactly `θ + ψ`. A zero
/// trait, a zero initial variance, a zero diffusion, and a zero TI
/// contribution is exactly `θ + ψ`. Setting `p_0 = −q / (2 a)`
/// recovers the stationary later-occasion observed variance. The
/// stationary later-occasion observed variance is not this
/// composition when `p_0` is free. `MANIFESTVAR` `θ` is not this
/// composition. The predetermined later-occasion latent variance is
/// not this observed variance. `TRAITVAR` is latent and is scaled
/// by `λ²`; `MANIFESTTRAITVAR` is not. This is not a Kalman filter,
/// not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_predetermined_later_latent_variance`] and
/// [`recover_manifest_trait_plus_state_observed_variance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_observed_variance(
    loading: f64,
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let later_latent = recover_predetermined_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    recover_manifest_trait_plus_state_observed_variance(
        loading,
        later_latent,
        measurement_error_variance,
        manifest_trait_variance,
    )
}

/// Refuse treating predetermined later-occasion variance as
/// predetermined later-occasion observed variance.
///
/// `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v` is the
/// predetermined later-occasion latent variance. Equation 5 maps
/// `Var(y_t) = λ²` of that variance plus `θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLatentVarianceIsNotObservedVariance`].
pub fn refuse_predetermined_later_latent_variance_as_observed_variance(
    predetermined_later_latent_variance: f64,
    predetermined_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_latent_variance,
        predetermined_later_observed_variance,
    );
    Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotObservedVariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined later-
/// occasion `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. `θ` is not
/// `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotPredeterminedLaterObservedVariance`].
pub fn refuse_measurement_error_as_predetermined_later_observed_variance(
    measurement_error_variance: f64,
    predetermined_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        measurement_error_variance,
        predetermined_later_observed_variance,
    );
    Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterObservedVariance)
}

/// Refuse treating Eq. 5 of later-occasion §4.3 stationary `T0VAR`
/// as predetermined later-occasion observed variance.
///
/// `λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ`
/// uses the stationary within-subject variance.
/// `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ` is not
/// that map when `p_0` is free.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance`].
pub fn refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance(
    stationary_later_observed_variance: f64,
    predetermined_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        stationary_later_observed_variance,
        predetermined_later_observed_variance,
    );
    Err(PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance)
}

/// Exact scalar lagged covariance of §4.3 predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T09:04Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// treat the first time point as predetermined when no assumptions
/// are made about the process prior to the initial time point. Free
/// `T0VAR` `p_0` is then estimated. Equation 3 writes
/// `η(t) = exp(A Δt) η(t0) + …`. Equation 4 writes
/// `cov(η_t, η_{t-1}) = A_Δt cov(η_{t-1})`. The lagged within-subject
/// covariance of free `T0VAR` is `e^{a Δt} p_0`. Trait variance and
/// `addedTIPREDVAR` are time-invariant between-subject; they do not
/// decay with `e^{a Δt}`. The lagged composition is
/// `trait + e^{a Δt} p_0 + (B / a)² v`. Form the lagged free
/// first-occasion covariance first, then include the trait, then
/// include the TI extra variance, then add. A zero trait, a zero
/// initial variance, and a zero TI contribution is exactly zero. A
/// zero initial variance and a zero TI contribution is exactly the
/// trait. As `Δt → ∞` with stable `a < 0` the state term vanishes.
/// As `Δt → 0+` the composition approaches
/// `trait + p_0 + (B / a)² v`. Setting `p_0 = −q / (2 a)` recovers
/// the stationary lagged map. Evolving `trait + p_0 + (B / a)² v`
/// as if it were all state (`e^{a Δt}` of that total) is not this
/// map. Free `T0VAR` `p_0` is not this map. Stationary lagged
/// covariance uses `−q / (2 a)` in place of `p_0` and is not this
/// map when `p_0` is free. The later-occasion map
/// `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v` includes `Q_Δt`
/// and is not this map. `a ≥ 0` cannot hold a finite TI extra
/// variance when that contribution is nonzero and fails closed.
/// A zero-diffusion carry with `a ≥ 0` is `e^{a Δt} p_0` and is
/// kept. Trait-only variance does not require a stable drift. The
/// interval must be event time and strictly positive. This is not a
/// Kalman filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_trait_plus_state_lagged_covariance`] and
/// [`recover_asymptotic_time_independent_predictor_variance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `event_delta` is
/// not strictly positive,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
pub fn recover_predetermined_lagged_latent_covariance(
    trait_variance: f64,
    initial_latent_variance: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let trait_plus_state = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        initial_latent_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_state + added)
}

/// Refuse treating predetermined lagged covariance as lagged
/// stationary `T0VAR`.
///
/// `trait + e^{a Δt} p_0 + (B / a)² v` uses free `T0VAR`.
/// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` uses the
/// stationary within-subject variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance`].
pub fn refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance(
    predetermined_lagged_covariance: f64,
    stationary_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_lagged_covariance,
        stationary_lagged_covariance,
    );
    Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance)
}

/// Refuse treating predetermined lagged covariance as predetermined
/// later-occasion variance.
///
/// `e^{a Δt} p_0` omits `Q_Δt`. Later-occasion variance is
/// `e^{2 a Δt} p_0 + Q_Δt` of the within-subject state.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance`].
pub fn refuse_predetermined_lagged_latent_covariance_as_later_latent_variance(
    predetermined_lagged_covariance: f64,
    predetermined_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_lagged_covariance,
        predetermined_later_variance,
    );
    Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance)
}

/// Refuse treating predetermined lagged covariance as the decayed
/// total.
///
/// Evolving `trait + p_0 + (B / a)² v` as if it were all state
/// yields `e^{a Δt}` of that total. Trait variance and
/// `addedTIPREDVAR` do not decay.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal`].
pub fn refuse_predetermined_lagged_latent_covariance_as_decayed_total(
    predetermined_lagged_covariance: f64,
    decayed_total: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_lagged_covariance, decayed_total);
    Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal)
}

/// Refuse treating predetermined lagged covariance as free
/// first-occasion `T0VAR`.
///
/// `p_0` is the predetermined first-occasion state variance.
/// `e^{a Δt} p_0` is not `p_0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance`].
pub fn refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance(
    predetermined_lagged_covariance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_lagged_covariance, initial_latent_variance);
    Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance)
}

/// Exact scalar Eq. 5 of lagged §4.3 predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-23T09:04Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Independent `ε_t` does not enter
/// `cov(y_t, y_{t-1})`. The predetermined lagged latent covariance
/// is `trait + e^{a Δt} p_0 + (B / a)² v`. The scalar composition
/// is `cov(y_t, y_{t-1}) = λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ`.
/// Form the predetermined lagged latent covariance first, then
/// `λ² c + ψ`. A zero loading is exactly `ψ`. A zero trait, a zero
/// initial variance, and a zero TI contribution is exactly `ψ`.
/// Setting `p_0 = −q / (2 a)` recovers the stationary lagged
/// observed covariance. The stationary lagged observed covariance
/// is not this composition when `p_0` is free. `MANIFESTVAR` `θ`
/// is not this composition. The predetermined lagged latent
/// covariance is not this observed covariance. Predetermined later
/// observed variance includes `Q_Δt` and `θ` and is not this
/// composition. `TRAITVAR` is latent and is scaled by `λ²`;
/// `MANIFESTTRAITVAR` is not. This is not a Kalman filter, not a
/// matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_predetermined_lagged_latent_covariance`] and
/// [`recover_manifest_lagged_observed_covariance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_lagged_observed_covariance(
    loading: f64,
    trait_variance: f64,
    initial_latent_variance: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    event_delta: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let lagged_latent = recover_predetermined_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        time_independent_effect,
        predictor_variance,
        log_rate,
        event_delta,
        clock,
    )?;
    recover_manifest_lagged_observed_covariance(loading, lagged_latent, manifest_trait_variance)
}

/// Refuse treating predetermined lagged covariance as predetermined
/// lagged observed covariance.
///
/// `trait + e^{a Δt} p_0 + (B / a)² v` is the predetermined lagged
/// latent covariance. Equation 5 maps `cov(y_t, y_{t-1}) = λ²` of
/// that covariance plus `ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance`].
pub fn refuse_predetermined_lagged_latent_covariance_as_observed_covariance(
    predetermined_lagged_latent_covariance: f64,
    predetermined_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_lagged_latent_covariance,
        predetermined_lagged_observed_covariance,
    );
    Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined lagged
/// `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. Independent `ε_t` does not
/// enter `cov(y_t, y_{t-1})`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance`].
pub fn refuse_measurement_error_as_predetermined_lagged_observed_covariance(
    measurement_error_variance: f64,
    predetermined_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        measurement_error_variance,
        predetermined_lagged_observed_covariance,
    );
    Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance)
}

/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as
/// predetermined lagged observed covariance.
///
/// `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`
/// includes `Q_Δt` and `θ`.
/// `λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ` omits both.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance`].
pub fn refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance(
    predetermined_later_observed_variance: f64,
    predetermined_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_observed_variance,
        predetermined_lagged_observed_covariance,
    );
    Err(
        PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance,
    )
}

/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as
/// predetermined lagged observed covariance.
///
/// `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ` uses the
/// stationary within-subject variance.
/// `λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ` is not that map
/// when `p_0` is free.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance`].
pub fn refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance(
    stationary_lagged_observed_covariance: f64,
    predetermined_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        stationary_lagged_observed_covariance,
        predetermined_lagged_observed_covariance,
    );
    Err(
        PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance,
    )
}

/// Exact scalar first-occasion variance of §4.3 predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T10:03Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// treat the first time point as predetermined when no assumptions
/// are made about the process prior to the initial time point. Free
/// `T0VAR` `p_0` is then estimated. Between-subject `TRAITVAR` and
/// `addedTIPREDVAR` are inherently stationary (p. 10). The
/// first-occasion composition is `trait + p_0 + (B / a)² v`. Form
/// the free first-occasion state variance first, then include the
/// trait, then include the TI extra variance, then add. A zero
/// trait, a zero initial variance, and a zero TI contribution is
/// exactly zero. A zero initial variance and a zero TI contribution
/// is exactly the trait. Setting `p_0 = −q / (2 a)` recovers the
/// stationary first-occasion map. Stationary first-occasion variance
/// uses `−q / (2 a)` in place of `p_0` and is not this map when
/// `p_0` is free. Free `T0VAR` `p_0` is not this map. The lagged
/// map `trait + e^{a Δt} p_0 + (B / a)² v` decays the state and is
/// not this map. The later-occasion map
/// `trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v` includes `Q_Δt`
/// and is not this map. As `Δt → 0+` those maps approach this
/// composition. `a ≥ 0` cannot hold a finite TI extra variance when
/// that contribution is nonzero and fails closed. Trait-only
/// variance does not require a stable drift. This is not a Kalman
/// filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_trait_plus_state_latent_variance`] and
/// [`recover_asymptotic_time_independent_predictor_variance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
pub fn recover_predetermined_initial_latent_variance(
    trait_variance: f64,
    initial_latent_variance: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let trait_plus_state =
        recover_trait_plus_state_latent_variance(trait_variance, initial_latent_variance)?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_state + added)
}

/// Refuse treating predetermined first-occasion variance as
/// stationary first-occasion `T0VAR`.
///
/// `trait + p_0 + (B / a)² v` uses free `T0VAR`.
/// `trait + −q / (2 a) + (B / a)² v` uses the stationary
/// within-subject variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance`].
pub fn refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance(
    predetermined_initial_variance: f64,
    stationary_initial_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_initial_variance, stationary_initial_variance);
    Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance)
}

/// Refuse treating predetermined first-occasion variance as free
/// first-occasion `T0VAR`.
///
/// `p_0` is the predetermined first-occasion state variance.
/// `trait + p_0 + (B / a)² v` is not `p_0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance`].
pub fn refuse_predetermined_initial_latent_variance_as_initial_latent_variance(
    predetermined_initial_variance: f64,
    initial_latent_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_initial_variance, initial_latent_variance);
    Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance)
}

/// Refuse treating predetermined first-occasion variance as
/// predetermined lagged covariance.
///
/// `e^{a Δt} p_0` decays the state. First-occasion variance is
/// contemporaneous `p_0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance`].
pub fn refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance(
    predetermined_initial_variance: f64,
    predetermined_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_initial_variance,
        predetermined_lagged_covariance,
    );
    Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance)
}

/// Refuse treating predetermined first-occasion variance as
/// predetermined later-occasion variance.
///
/// Later-occasion variance is `e^{2 a Δt} p_0 + Q_Δt` of the
/// within-subject state. First-occasion variance omits both.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance`].
pub fn refuse_predetermined_initial_latent_variance_as_later_latent_variance(
    predetermined_initial_variance: f64,
    predetermined_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_initial_variance, predetermined_later_variance);
    Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance)
}

/// Exact scalar Eq. 5 of first-occasion §4.3 predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T10:03Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The predetermined first-occasion latent variance
/// is `trait + p_0 + (B / a)² v`. The scalar composition is
/// `Var(y_0) = λ²(trait + p_0 + (B / a)² v) + θ + ψ`. Form the
/// predetermined first-occasion latent variance first, then
/// `λ² p + θ + ψ`. A zero loading is exactly `θ + ψ`. A zero trait,
/// a zero initial variance, and a zero TI contribution is exactly
/// `θ + ψ`. Setting `p_0 = −q / (2 a)` recovers the stationary
/// first-occasion observed variance. The stationary first-occasion
/// observed variance is not this composition when `p_0` is free.
/// `MANIFESTVAR` `θ` is not this composition. The predetermined
/// first-occasion latent variance is not this observed variance.
/// Predetermined later observed variance includes `Q_Δt` and is not
/// this composition. `TRAITVAR` is latent and is scaled by `λ²`;
/// `MANIFESTTRAITVAR` is not. This is not a Kalman filter, not a
/// matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_predetermined_initial_latent_variance`] and
/// [`recover_manifest_trait_plus_state_observed_variance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_initial_observed_variance(
    loading: f64,
    trait_variance: f64,
    initial_latent_variance: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let initial_latent = recover_predetermined_initial_latent_variance(
        trait_variance,
        initial_latent_variance,
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    recover_manifest_trait_plus_state_observed_variance(
        loading,
        initial_latent,
        measurement_error_variance,
        manifest_trait_variance,
    )
}

/// Refuse treating predetermined first-occasion variance as
/// predetermined first-occasion observed variance.
///
/// `trait + p_0 + (B / a)² v` is the predetermined first-occasion
/// latent variance. Equation 5 maps `Var(y_0) = λ²` of that
/// variance plus `θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedInitialLatentVarianceIsNotObservedVariance`].
pub fn refuse_predetermined_initial_latent_variance_as_observed_variance(
    predetermined_initial_latent_variance: f64,
    predetermined_initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_initial_latent_variance,
        predetermined_initial_observed_variance,
    );
    Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotObservedVariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of predetermined
/// first-occasion `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. `θ` is not
/// `λ²(trait + p_0 + (B / a)² v) + θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotPredeterminedInitialObservedVariance`].
pub fn refuse_measurement_error_as_predetermined_initial_observed_variance(
    measurement_error_variance: f64,
    predetermined_initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        measurement_error_variance,
        predetermined_initial_observed_variance,
    );
    Err(PsychometricError::MeasurementErrorIsNotPredeterminedInitialObservedVariance)
}

/// Refuse treating Eq. 5 of §4.3 stationary `T0VAR` as predetermined
/// first-occasion observed variance.
///
/// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` uses the
/// stationary within-subject variance.
/// `λ²(trait + p_0 + (B / a)² v) + θ + ψ` is not that map when
/// `p_0` is free.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance`].
pub fn refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance(
    stationary_initial_observed_variance: f64,
    predetermined_initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        stationary_initial_observed_variance,
        predetermined_initial_observed_variance,
    );
    Err(
        PsychometricError::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance,
    )
}

/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as
/// predetermined first-occasion observed variance.
///
/// `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`
/// includes `Q_Δt`. `λ²(trait + p_0 + (B / a)² v) + θ + ψ` omits it.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance`].
pub fn refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance(
    predetermined_later_observed_variance: f64,
    predetermined_initial_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_observed_variance,
        predetermined_initial_observed_variance,
    );
    Err(
        PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance,
    )
}

/// Exact scalar later-start lagged covariance of §4.3 predetermined
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T10:27Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// treat the first time point as predetermined when no assumptions
/// are made about the process prior to the initial time point. Free
/// `T0VAR` `p_0` is then estimated. Section 4.3 notes that the process
/// gradually transitions from the initial variances toward the
/// stationary variances, and that the initial time point need not
/// reflect the first measurement occasion (`startoffset`). Equation 3
/// writes `η(t) = exp(A Δt) η(t0) + … +` the stochastic integral.
/// Equation 4 writes `cov(η_t, η_{t-1}) = A_Δt cov(η_{t-1})`. After a
/// later start `u` the within-subject state variance is
/// `e^{2 a u} p_0 + Q_u`. Lagging that later start over `s` is
/// `e^{a s}(e^{2 a u} p_0 + Q_u)`. Trait variance and
/// `addedTIPREDVAR` are time-invariant between-subject; they do not
/// decay with `e^{a s}`. The lagged composition is
/// `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v`. Form the
/// later-start within-subject variance first, then lag that state,
/// then include the trait, then include the TI extra variance, then
/// add. A zero trait, a zero initial variance, a zero diffusion, and
/// a zero TI contribution is exactly zero. A zero initial variance, a
/// zero diffusion, and a zero TI contribution is exactly the trait.
/// Setting `p_0 = −q / (2 a)` recovers the stationary lagged map.
/// Stationary lagged covariance uses `−q / (2 a)` in place of `p_0`
/// and is not this map when `p_0` is free. First-occasion lagged
/// covariance `trait + e^{a s} p_0 + (B / a)² v` omits `e^{a s} Q_u`
/// and is not this map when `u > 0`. Later-occasion variance includes
/// `Q_u` without lagging that later state and is not this map.
/// Evolving the later total as if it were all state (`e^{a s}` of
/// `trait + e^{2 a u} p_0 + Q_u + (B / a)² v`) is not this map. As
/// `u → 0+` the composition approaches first-occasion lagged
/// covariance. As `s → 0+` the composition approaches later-occasion
/// variance at `u`. As `s → ∞` with stable `a < 0` the state term
/// vanishes. `a ≥ 0` cannot hold a finite TI extra variance when that
/// contribution is nonzero and fails closed. Nonzero diffusion with
/// `a ≥ 0` is a growing process and is kept. Both intervals must be
/// event time and strictly positive. This is not a Kalman filter, not
/// a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_variance`],
/// [`recover_trait_plus_state_lagged_covariance`], and
/// [`recover_asymptotic_time_independent_predictor_variance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `start_delta` or
/// `lag_delta` is not strictly positive,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_lagged_latent_covariance(
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    start_delta: f64,
    lag_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let later_state = recover_discrete_latent_variance(
        initial_latent_variance,
        continuous_diffusion,
        log_rate,
        start_delta,
        clock,
    )?;
    let trait_plus_lagged = recover_trait_plus_state_lagged_covariance(
        trait_variance,
        later_state,
        log_rate,
        lag_delta,
        clock,
    )?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_lagged + added)
}

/// Refuse treating later-start lagged covariance as first-occasion
/// lagged covariance of predetermined `T0VAR`.
///
/// `e^{a s}(e^{2 a u} p_0 + Q_u)` includes `e^{a s} Q_u`.
/// `e^{a s} p_0` omits that process-noise carry.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance`].
pub fn refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance(
    predetermined_later_lagged_covariance: f64,
    predetermined_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_lagged_covariance,
        predetermined_lagged_covariance,
    );
    Err(
        PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance,
    )
}

/// Refuse treating later-start lagged covariance as later-occasion
/// variance of predetermined `T0VAR`.
///
/// Later-occasion variance is `e^{2 a u} p_0 + Q_u` of the
/// within-subject state. Later-start lag is `e^{a s}` of that state.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance`].
pub fn refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance(
    predetermined_later_lagged_covariance: f64,
    predetermined_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_lagged_covariance,
        predetermined_later_variance,
    );
    Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance)
}

/// Refuse treating later-start lagged covariance as lagged
/// stationary `T0VAR`.
///
/// `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v` uses free
/// `T0VAR`. `trait + e^{a s}(−q / (2 a)) + (B / a)² v` uses the
/// stationary within-subject variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance`].
pub fn refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance(
    predetermined_later_lagged_covariance: f64,
    stationary_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_lagged_covariance,
        stationary_lagged_covariance,
    );
    Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance)
}

/// Refuse treating later-start lagged covariance as the decayed
/// later total.
///
/// Evolving `trait + e^{2 a u} p_0 + Q_u + (B / a)² v` as if it
/// were all state yields `e^{a s}` of that total. Trait variance
/// and `addedTIPREDVAR` do not decay.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal`].
pub fn refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total(
    predetermined_later_lagged_covariance: f64,
    decayed_later_total: f64,
) -> Result<f64, PsychometricError> {
    let _ = (predetermined_later_lagged_covariance, decayed_later_total);
    Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal)
}

/// Exact scalar Eq. 5 of later-start lagged §4.3 predetermined
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-23T10:27Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Independent `ε_t` does not enter
/// `cov(y_t, y_{t-s})`. The later-start lagged latent covariance is
/// `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v`. The scalar
/// composition is
/// `cov(y_{t0+u+s}, y_{t0+u}) = λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`.
/// Form the later-start lagged latent covariance first, then
/// `λ² c + ψ`. A zero loading is exactly `ψ`. A zero trait, a zero
/// initial variance, a zero diffusion, and a zero TI contribution is
/// exactly `ψ`. Setting `p_0 = −q / (2 a)` recovers the stationary
/// lagged observed covariance. The stationary lagged observed
/// covariance is not this composition when `p_0` is free.
/// `MANIFESTVAR` `θ` is not this composition. The later-start lagged
/// latent covariance is not this observed covariance. First-occasion
/// lagged observed covariance omits `e^{a s} Q_u` and is not this
/// composition when `u > 0`. Predetermined later observed variance
/// includes `Q_u` and `θ` and is not this composition. `TRAITVAR` is
/// latent and is scaled by `λ²`; `MANIFESTTRAITVAR` is not. This is
/// not a Kalman filter, not a matrix `expm`, and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates [`recover_predetermined_later_lagged_latent_covariance`]
/// and [`recover_manifest_lagged_observed_covariance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_lagged_observed_covariance(
    loading: f64,
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    start_delta: f64,
    lag_delta: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let lagged_latent = recover_predetermined_later_lagged_latent_covariance(
        trait_variance,
        initial_latent_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        clock,
    )?;
    recover_manifest_lagged_observed_covariance(loading, lagged_latent, manifest_trait_variance)
}

/// Refuse treating later-start lagged covariance as later-start
/// lagged observed covariance.
///
/// `trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v` is the
/// later-start lagged latent covariance. Equation 5 maps
/// `cov(y_{t0+u+s}, y_{t0+u}) = λ²` of that covariance plus `ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance`].
pub fn refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance(
    predetermined_later_lagged_latent_covariance: f64,
    predetermined_later_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_lagged_latent_covariance,
        predetermined_later_lagged_observed_covariance,
    );
    Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-start lagged
/// predetermined `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. Independent `ε_t` does not
/// enter `cov(y_t, y_{t-s})`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance`].
pub fn refuse_measurement_error_as_predetermined_later_lagged_observed_covariance(
    measurement_error_variance: f64,
    predetermined_later_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        measurement_error_variance,
        predetermined_later_lagged_observed_covariance,
    );
    Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance)
}

/// Refuse treating Eq. 5 of first-occasion lagged predetermined
/// `T0VAR` as later-start lagged observed covariance.
///
/// `λ²(trait + e^{a s} p_0 + (B / a)² v) + ψ` omits `e^{a s} Q_u`.
/// `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`
/// includes it.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance`].
pub fn refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
    predetermined_lagged_observed_covariance: f64,
    predetermined_later_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_lagged_observed_covariance,
        predetermined_later_lagged_observed_covariance,
    );
    Err(
        PsychometricError::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    )
}

/// Refuse treating Eq. 5 of lagged §4.3 stationary `T0VAR` as
/// later-start lagged observed covariance of predetermined `T0VAR`.
///
/// `λ²(trait + e^{a s}(−q / (2 a)) + (B / a)² v) + ψ` uses the
/// stationary within-subject variance.
/// `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ` is
/// not that map when `p_0` is free.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance`].
pub fn refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
    stationary_lagged_observed_covariance: f64,
    predetermined_later_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        stationary_lagged_observed_covariance,
        predetermined_later_lagged_observed_covariance,
    );
    Err(
        PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    )
}

/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as
/// later-start lagged observed covariance.
///
/// `λ²(trait + e^{2 a u} p_0 + Q_u + (B / a)² v) + θ + ψ` includes
/// `Q_u` and `θ`.
/// `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ`
/// lags the later state and omits `θ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance`].
pub fn refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance(
    predetermined_later_observed_variance: f64,
    predetermined_later_lagged_observed_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_observed_variance,
        predetermined_later_lagged_observed_covariance,
    );
    Err(
        PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    )
}

/// Exact scalar later-start later-occasion variance of §4.3
/// predetermined `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5;
/// Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF re-opened
/// 2026-08-23T11:05Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// treat the first time point as predetermined when no assumptions
/// are made about the process prior to the initial time point. Free
/// `T0VAR` `p_0` is then estimated. Section 4.3 notes that the process
/// gradually transitions from the initial variances toward the
/// stationary variances, and that the initial time point need not
/// reflect the first measurement occasion (`startoffset`). Equation 3
/// writes `η(t) = exp(A Δt) η(t0) + … +` the stochastic integral.
/// Equation 4 writes that the integral exhibits covariance `Q_Δt`.
/// After a later start `u` the within-subject state variance is
/// `e^{2 a u} p_0 + Q_u`. Evolving that later start over `s` is
/// `e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s`. Chapman–Kolmogorov
/// writes `Q_{u+s} = e^{2 a s} Q_u + Q_s`, so the later-occasion map
/// over `u + s` is the same composition. Trait variance and
/// `addedTIPREDVAR` are time-invariant between-subject; they do not
/// enter `Q_s`. The later-start later-occasion composition is
/// `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v`. Form
/// the later-start within-subject variance first, then evolve that
/// state, then include the trait, then include the TI extra variance,
/// then add. A zero trait, a zero initial variance, a zero diffusion,
/// and a zero TI contribution is exactly zero. A zero initial
/// variance, a zero diffusion, and a zero TI contribution is exactly
/// the trait. Setting `p_0 = −q / (2 a)` recovers the stationary
/// later-occasion map. Stationary later-occasion variance uses
/// `−q / (2 a)` in place of `p_0` and is not this map when `p_0` is
/// free. Later-occasion variance at `u` omits `Q_s` and is not this
/// map when `s > 0`. Later-start lagged covariance is `e^{a s}` of
/// the later state and omits `Q_s`; it is not this map. Evolving the
/// later total as if it were all state (`e^{2 a s}` of
/// `trait + e^{2 a u} p_0 + Q_u + (B / a)² v` plus `Q_s`) is not this
/// map. Later-occasion variance over the lag interval alone ignores
/// `startoffset` and omits `e^{2 a s} Q_u`; it is not this map when
/// `u > 0`. As `u → 0+` the composition approaches later-occasion
/// variance over `s`. As `s → 0+` the composition approaches
/// later-occasion variance at `u`. As `s → ∞` with stable `a < 0`
/// the carried later state vanishes and `Q_s` approaches
/// `−q / (2 a)`, so the composition approaches contemporaneous
/// stationary `T0VAR`. `a ≥ 0` cannot hold a finite TI extra variance
/// when that contribution is nonzero and fails closed. Nonzero
/// diffusion with `a ≥ 0` is a growing process and is kept. Both
/// intervals must be event time and strictly positive. This is not a
/// Kalman filter, not a matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_variance`],
/// [`recover_trait_plus_state_latent_variance`], and
/// [`recover_asymptotic_time_independent_predictor_variance`].
/// Returns [`PsychometricError::EventTimeRequired`] for any
/// non-event clock,
/// [`PsychometricError::NonPositiveInterval`] when `start_delta` or
/// `lag_delta` is not strictly positive,
/// [`PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift`]
/// when the TI contribution is nonzero and the drift is not
/// strictly negative, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite, a variance is negative, or a product or sum
/// overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_start_later_latent_variance(
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    start_delta: f64,
    lag_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    let later_state = recover_discrete_latent_variance(
        initial_latent_variance,
        continuous_diffusion,
        log_rate,
        start_delta,
        clock,
    )?;
    let evolved_state = recover_discrete_latent_variance(
        later_state,
        continuous_diffusion,
        log_rate,
        lag_delta,
        clock,
    )?;
    let trait_plus_evolved =
        recover_trait_plus_state_latent_variance(trait_variance, evolved_state)?;
    let added = recover_asymptotic_time_independent_predictor_variance(
        time_independent_effect,
        predictor_variance,
        log_rate,
        clock,
    )?;
    require_finite(trait_plus_evolved + added)
}

/// Refuse treating later-start later-occasion variance as later-
/// occasion variance at the later start.
///
/// Later-occasion variance at `u` is `e^{2 a u} p_0 + Q_u` of the
/// within-subject state. Later-start later-occasion variance adds
/// `Q_s`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance(
    predetermined_later_start_later_variance: f64,
    predetermined_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_variance,
        predetermined_later_variance,
    );
    Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance)
}

/// Refuse treating later-start later-occasion variance as later-start
/// lagged covariance of predetermined `T0VAR`.
///
/// Later-start lag is `e^{a s}` of the later state and omits `Q_s`.
/// Later-start later-occasion variance is
/// `e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance(
    predetermined_later_start_later_variance: f64,
    predetermined_later_lagged_covariance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_variance,
        predetermined_later_lagged_covariance,
    );
    Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance)
}

/// Refuse treating later-start later-occasion variance as later-
/// occasion stationary `T0VAR`.
///
/// `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v` uses
/// free `T0VAR`. `trait + e^{2 a s}(−q / (2 a)) + Q_s + (B / a)² v`
/// uses the stationary within-subject variance.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance(
    predetermined_later_start_later_variance: f64,
    stationary_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_variance,
        stationary_later_variance,
    );
    Err(
        PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance,
    )
}

/// Refuse treating later-start later-occasion variance as the evolved
/// later total.
///
/// Evolving `trait + e^{2 a u} p_0 + Q_u + (B / a)² v` as if it
/// were all state yields `e^{2 a s}` of that total plus `Q_s`. Trait
/// variance and `addedTIPREDVAR` do not enter `Q_s`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total(
    predetermined_later_start_later_variance: f64,
    decayed_later_total: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_variance,
        decayed_later_total,
    );
    Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal)
}

/// Refuse treating later-start later-occasion variance as later-
/// occasion variance over the lag interval alone.
///
/// Ignoring `startoffset` yields `e^{2 a s} p_0 + Q_s` and omits
/// `e^{2 a s} Q_u`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance(
    predetermined_later_start_later_variance: f64,
    lag_interval_later_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_variance,
        lag_interval_later_variance,
    );
    Err(
        PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance,
    )
}

/// Exact scalar Eq. 5 of later-start later-occasion §4.3 predetermined
/// `T0VAR`.
///
/// Driver, Oud, and Voelkle (2017, §4.3, pp. 9–10; Eq. 5, p. 5;
/// Eq. 3–4, pp. 4–5; Table 2, p. 12; p. 16; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-23T11:05Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The later-start later-occasion latent variance is
/// `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v`. The
/// scalar composition is
/// `Var(y_{t0+u+s}) = λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`.
/// Form the later-start later-occasion latent variance first, then
/// `λ² p + θ + ψ`. A zero loading is exactly `θ + ψ`. A zero trait,
/// a zero initial variance, a zero diffusion, and a zero TI
/// contribution is exactly `θ + ψ`. Setting `p_0 = −q / (2 a)`
/// recovers the stationary later-occasion observed variance. The
/// stationary later-occasion observed variance is not this
/// composition when `p_0` is free. `MANIFESTVAR` `θ` is not this
/// composition. The later-start later-occasion latent variance is
/// not this observed variance. Predetermined later observed variance
/// at `u` omits `Q_s` and is not this composition when `s > 0`.
/// Later-start lagged observed covariance omits `Q_s` and `θ` and is
/// not this composition. `TRAITVAR` is latent and is scaled by `λ²`;
/// `MANIFESTTRAITVAR` is not. This is not a Kalman filter, not a
/// matrix `expm`, and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_predetermined_later_start_later_latent_variance`]
/// and [`recover_manifest_trait_plus_state_observed_variance`].
#[allow(clippy::too_many_arguments)]
pub fn recover_predetermined_later_start_later_observed_variance(
    loading: f64,
    trait_variance: f64,
    initial_latent_variance: f64,
    continuous_diffusion: f64,
    time_independent_effect: f64,
    predictor_variance: f64,
    log_rate: f64,
    start_delta: f64,
    lag_delta: f64,
    measurement_error_variance: f64,
    manifest_trait_variance: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let later_latent = recover_predetermined_later_start_later_latent_variance(
        trait_variance,
        initial_latent_variance,
        continuous_diffusion,
        time_independent_effect,
        predictor_variance,
        log_rate,
        start_delta,
        lag_delta,
        clock,
    )?;
    recover_manifest_trait_plus_state_observed_variance(
        loading,
        later_latent,
        measurement_error_variance,
        manifest_trait_variance,
    )
}

/// Refuse treating later-start later-occasion variance as later-start
/// later-occasion observed variance.
///
/// `trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v` is
/// the later-start later-occasion latent variance. Equation 5 maps
/// `Var(y_{t0+u+s}) = λ²` of that variance plus `θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance`].
pub fn refuse_predetermined_later_start_later_latent_variance_as_observed_variance(
    predetermined_later_start_later_latent_variance: f64,
    predetermined_later_start_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_start_later_latent_variance,
        predetermined_later_start_later_observed_variance,
    );
    Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance)
}

/// Refuse treating `MANIFESTVAR` as Eq. 5 of later-start later-
/// occasion predetermined `T0VAR`.
///
/// Table 2 names `θ` `MANIFESTVAR`. `θ` is not
/// `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance`].
pub fn refuse_measurement_error_as_predetermined_later_start_later_observed_variance(
    measurement_error_variance: f64,
    predetermined_later_start_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        measurement_error_variance,
        predetermined_later_start_later_observed_variance,
    );
    Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance)
}

/// Refuse treating Eq. 5 of predetermined later-occasion `T0VAR` as
/// later-start later-occasion observed variance.
///
/// `λ²(trait + e^{2 a u} p_0 + Q_u + (B / a)² v) + θ + ψ` omits
/// `Q_s`. `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`
/// includes it.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance`].
pub fn refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance(
    predetermined_later_observed_variance: f64,
    predetermined_later_start_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_observed_variance,
        predetermined_later_start_later_observed_variance,
    );
    Err(
        PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    )
}

/// Refuse treating Eq. 5 of later-start lagged predetermined `T0VAR`
/// as later-start later-occasion observed variance.
///
/// `λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ` omits
/// `Q_s` and `θ`.
/// `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`
/// includes both.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance`].
pub fn refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance(
    predetermined_later_lagged_observed_covariance: f64,
    predetermined_later_start_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        predetermined_later_lagged_observed_covariance,
        predetermined_later_start_later_observed_variance,
    );
    Err(
        PsychometricError::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    )
}

/// Refuse treating Eq. 5 of later-occasion §4.3 stationary `T0VAR` as
/// later-start later-occasion observed variance of predetermined
/// `T0VAR`.
///
/// `λ²(trait + e^{2 a s}(−q / (2 a)) + Q_s + (B / a)² v) + θ + ψ`
/// uses the stationary within-subject variance.
/// `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`
/// is not that map when `p_0` is free.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance`].
pub fn refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance(
    stationary_later_observed_variance: f64,
    predetermined_later_start_later_observed_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        stationary_later_observed_variance,
        predetermined_later_start_later_observed_variance,
    );
    Err(
        PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    )
}

/// Exact scalar observed mean of a time-independent predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 3, p. 5; Table 2,
/// p. 12; JSS PDF re-opened 2026-08-20T12:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Equation 3 (p. 5) writes the time-independent
/// predictor as the printed addend `A^{-1}[e^{A(t−t0)} − I] B z_i`
/// after the `T0MEANS` carry and the `CINT` increment. Table 2 names
/// `B` `TIPREDEFFECT`. The expected intercept is `τ`. The latent
/// process at `t` after that increment is
/// `μ_t + A^{-1}[e^{A Δt} − I] B z`. The scalar composition is
/// `E(y_t) = τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Form the
/// evolved-plus-increment latent mean first, then `τ + λ` of that
/// mean. A zero loading is exactly `τ`. A zero evolved-plus-increment
/// latent mean is exactly `τ`. A zero intercept is exactly
/// `λ(μ_t + increment)`. The evolved observed mean `τ + λ μ_t` is
/// not this composition when the increment is nonzero. The
/// contemporaneous map `τ + λ(μ_t + m x)` is not this composition.
/// The carry map `τ + λ(μ_t + e^{a(t−u)} m x)` is not this
/// composition when `u ≠ t`. `MANIFESTMEANS` is not `E(y_t)`. The
/// evolved-plus-increment latent mean is not `E(y_t)`. `TIPREDEFFECT`
/// is `B`, not that observed mean. This is not a Kalman filter and
/// not ctsem estimation.
///
/// # Errors
///
/// Propagates
/// [`recover_discrete_latent_mean_with_time_independent_predictor`]
/// and [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_time_independent_predictor(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_independent_effect: f64,
    time_independent_predictor: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let composed_latent_mean = recover_discrete_latent_mean_with_time_independent_predictor(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        time_independent_effect,
        time_independent_predictor,
        event_delta,
        clock,
    )?;
    recover_manifest_observed_mean(loading, composed_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the time-independent-
/// predictor observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the Eq. 3 time-independent predictor is
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Those are not the same
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean`].
pub fn refuse_evolved_observed_mean_as_time_independent_observed_mean(
    evolved_observed_mean: f64,
    time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, time_independent_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean)
}

/// Refuse treating the contemporaneous-impulse observed mean as the
/// time-independent-predictor observed mean.
///
/// Equation 5 of the contemporaneous Dirac is `τ + λ(μ_t + m x)`.
/// Equation 5 of the Eq. 3 time-independent predictor is
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Those are not the same
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean`].
pub fn refuse_impulse_observed_mean_as_time_independent_observed_mean(
    impulse_observed_mean: f64,
    time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (impulse_observed_mean, time_independent_observed_mean);
    Err(PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean)
}

/// Refuse treating the impulse-carry observed mean as the
/// time-independent-predictor observed mean.
///
/// Equation 5 of the Eq. 1–2 carried latent mean is
/// `τ + λ(μ_t + e^{a(t−u)} m x)`. Equation 5 of the Eq. 3
/// time-independent predictor is
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Those are not the same
/// map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean`].
pub fn refuse_impulse_carry_observed_mean_as_time_independent_observed_mean(
    impulse_carry_observed_mean: f64,
    time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (impulse_carry_observed_mean, time_independent_observed_mean);
    Err(PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean)
}

/// Exact scalar first-occasion time-independent predictor shift.
///
/// Driver, Oud, and Voelkle (2017, Table 3, p. 13; Eq. 3 first
/// summand, p. 5; JSS PDF opened 2026-08-20T15:14Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TIPREDEFFECT` the effect of time-independent predictors on
/// latents at `T0`. Table 2 / Table 3 name `TIPREDEFFECT` `B`, which
/// enters Equation 3 as the printed addend
/// `A^{-1}[e^{A(t−t0)} − I] B z`. Those are not the same matrix. The
/// scalar first-occasion shift is `t0_b z`. It is not `B`, not
/// `A^{-1}[e^{A Δt} − I] B z`, not `κ`, and not `M x`. A zero effect
/// or zero predictor is exactly zero. This is not a Kalman filter and
/// not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the effect
/// or predictor is non-finite or the product overflows.
pub fn recover_initial_time_independent_predictor_effect(
    initial_time_independent_effect: f64,
    time_independent_predictor: f64,
) -> Result<f64, PsychometricError> {
    if !initial_time_independent_effect.is_finite() || !time_independent_predictor.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_time_independent_effect == 0.0 || time_independent_predictor == 0.0 {
        return Ok(0.0);
    }
    require_finite(initial_time_independent_effect * time_independent_predictor)
}

/// Exact scalar carried first-occasion time-independent predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; Table 3, p. 13; JSS
/// PDF opened 2026-08-20T15:14Z) write the first summand as
/// `e^{A(t−t0)} η_i(t0)`. A Table 3 `T0TIPREDEFFECT` shift that is
/// already in `η(t0)` therefore appears at `t` as `e^{A Δt} t0_b z`.
/// Form `t0_b z` first, then `e^{a Δt} t0_b z`. A zero drift is
/// `t0_b z` with no dissipation of the first-occasion shift. Binary64
/// underflow of `e^{a Δt}` to `+0` is a vanishing carry of that
/// shift and is kept. This carry is not the first-occasion shift, not
/// `A^{-1}[e^{A Δt} − I] B z` (`TIPREDEFFECT`), not `CINT`, and not
/// `M x`. When `exp` overflows at a finite `a Δt`, rewrite as
/// `sign(t0_b z) exp(ln|t0_b z| + a Δt)`. An overflowing rewrite
/// fails closed. This is not a Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when
/// `event_delta` is not strictly positive, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or the mapped carry overflows.
pub fn recover_initial_time_independent_predictor_carry(
    initial_time_independent_effect: f64,
    time_independent_predictor: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let initial_shift = recover_initial_time_independent_predictor_effect(
        initial_time_independent_effect,
        time_independent_predictor,
    )?;
    if initial_shift == 0.0 {
        return Ok(0.0);
    }
    let drift_interval = log_rate * event_delta;
    let auto_effect = drift_interval.exp();
    if auto_effect.is_finite() {
        // +0 underflow is a vanishing carry of the T0 shift.
        return require_finite(auto_effect * initial_shift);
    }
    if !drift_interval.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite a Δt, overflowed exp.
    // e^{a Δt} t0_b z = sign(t0_b z) exp(ln|t0_b z| + a Δt).
    require_finite(initial_shift.signum() * (initial_shift.abs().ln() + drift_interval).exp())
}

/// Exact scalar evolved latent mean plus a first-occasion TI predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; Table 3, p. 13) write
/// the first summand as the carried `T0MEANS`, which includes any
/// `T0TIPREDEFFECT` shift already in `η(t0)`. Form `μ_t` first, then
/// add `e^{a Δt} t0_b z`. A zero carry is exactly `μ_t`. A zero
/// evolved mean is exactly the carry. Adding `t0_b z` without the
/// exponential is not this composition when `a Δt ≠ 0`. Adding
/// `A^{-1}[e^{A Δt} − I] B z` is not this composition.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_initial_time_independent_predictor_carry`], and returns
/// [`PsychometricError::InvalidNumericInput`] when the sum overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_latent_mean_with_initial_time_independent_predictor(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    initial_time_independent_effect: f64,
    time_independent_predictor: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let initial_carry = recover_initial_time_independent_predictor_carry(
        initial_time_independent_effect,
        time_independent_predictor,
        log_rate,
        event_delta,
        clock,
    )?;
    if initial_carry == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(initial_carry);
    }
    require_finite(evolved_latent_mean + initial_carry)
}

/// Refuse treating the Table 3 first-occasion shift as the Eq. 3
/// process increment.
///
/// `T0TIPREDEFFECT` shifts `η(t0)`. `TIPREDEFFECT` `B` enters the
/// SDE and maps as `A^{-1}[e^{A Δt} − I] B z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement`].
pub fn refuse_initial_time_independent_effect_as_process_increment(
    initial_time_independent_effect: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_independent_effect, time_independent_increment);
    Err(PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement)
}

/// Refuse treating the Eq. 3 carry of `T0TIPREDEFFECT` as the
/// first-occasion shift.
///
/// `e^{A Δt} t0_b z` is the first summand's contribution at `t`.
/// `t0_b z` is the shift at `T0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect`].
pub fn refuse_initial_time_independent_carry_as_initial_effect(
    initial_time_independent_carry: f64,
    initial_time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_independent_carry,
        initial_time_independent_effect,
    );
    Err(PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect)
}

/// Refuse treating the Table 3 first-occasion shift as `CINT`.
///
/// `t0_b z` is an initial-mean shift. `κ` is the continuous intercept.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept`].
pub fn refuse_initial_time_independent_effect_as_continuous_intercept(
    initial_time_independent_effect: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_independent_effect, continuous_intercept);
    Err(PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept)
}

/// Refuse treating the Table 3 first-occasion shift as `M x`.
///
/// The product `t0_b z` is algebraically a product, as is `M x`.
/// Table 3 names `T0TIPREDEFFECT` for `T0`. Table 2 names `M`
/// `TDPREDEFFECT` for the Dirac impulse.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse`].
pub fn refuse_initial_time_independent_effect_as_time_dependent_impulse(
    initial_time_independent_effect: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_independent_effect, time_dependent_impulse);
    Err(PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse)
}

/// Refuse treating Driver Table 3 `T0TIPREDEFFECT` as the
/// first-occasion shift.
///
/// `T0TIPREDEFFECT` is the coefficient. The shift is `t0_b z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect`].
pub fn refuse_initial_time_independent_coefficient_as_initial_effect(
    initial_time_independent_coefficient: f64,
    initial_time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_independent_coefficient,
        initial_time_independent_effect,
    );
    Err(PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect)
}

/// Exact scalar observed mean of a first-occasion time-independent
/// predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 3 first summand,
/// p. 5; Table 3, p. 13; JSS PDF re-opened 2026-08-20T15:28Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Table 3 names `T0TIPREDEFFECT` the effect of
/// time-independent predictors on latents at `T0`. Equation 3's
/// first summand carries that shift as `e^{A Δt} t0_b z`. The
/// expected intercept is `τ`. The latent process at `t` after that
/// carry is `μ_t + e^{a Δt} t0_b z`. The scalar composition is
/// `E(y_t) = τ + λ(μ_t + e^{a Δt} t0_b z)`. Form the
/// evolved-plus-carry latent mean first, then `τ + λ` of that mean.
/// A zero loading is exactly `τ`. A zero evolved-plus-carry latent
/// mean is exactly `τ`. A zero intercept is exactly
/// `λ(μ_t + e^{a Δt} t0_b z)`. The evolved observed mean
/// `τ + λ μ_t` is not this composition when the carry is nonzero.
/// The process-increment map
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not this composition.
/// The contemporaneous map `τ + λ(μ_t + m x)` is not this
/// composition. The impulse-carry map
/// `τ + λ(μ_t + e^{a(t−u)} m x)` is not this composition when
/// `u ≠ t0`. `MANIFESTMEANS` is not `E(y_t)`. The
/// evolved-plus-carry latent mean is not `E(y_t)`.
/// `T0TIPREDEFFECT` is the coefficient, not that observed mean.
/// This is not a Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates
/// [`recover_discrete_latent_mean_with_initial_time_independent_predictor`]
/// and [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_initial_time_independent_predictor(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    initial_time_independent_effect: f64,
    time_independent_predictor: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let composed_latent_mean =
        recover_discrete_latent_mean_with_initial_time_independent_predictor(
            initial_latent_mean,
            log_rate,
            continuous_intercept,
            initial_time_independent_effect,
            time_independent_predictor,
            event_delta,
            clock,
        )?;
    recover_manifest_observed_mean(loading, composed_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the first-occasion
/// time-independent-predictor observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the Table 3 first-occasion TI predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_b z)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean`].
pub fn refuse_evolved_observed_mean_as_initial_time_independent_observed_mean(
    evolved_observed_mean: f64,
    initial_time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        evolved_observed_mean,
        initial_time_independent_observed_mean,
    );
    Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean)
}

/// Refuse treating the process-increment observed mean as the
/// first-occasion time-independent-predictor observed mean.
///
/// Equation 5 of the Eq. 3 `TIPREDEFFECT` increment is
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Equation 5 of the
/// Table 3 first-occasion TI predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_b z)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean`].
pub fn refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean(
    time_independent_observed_mean: f64,
    initial_time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        time_independent_observed_mean,
        initial_time_independent_observed_mean,
    );
    Err(PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean)
}

/// Refuse treating the contemporaneous-impulse observed mean as the
/// first-occasion time-independent-predictor observed mean.
///
/// Equation 5 of the contemporaneous Dirac is `τ + λ(μ_t + m x)`.
/// Equation 5 of the Table 3 first-occasion TI predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_b z)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean`].
pub fn refuse_impulse_observed_mean_as_initial_time_independent_observed_mean(
    impulse_observed_mean: f64,
    initial_time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        impulse_observed_mean,
        initial_time_independent_observed_mean,
    );
    Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean)
}

/// Refuse treating the impulse-carry observed mean as the
/// first-occasion time-independent-predictor observed mean.
///
/// Equation 5 of the Eq. 1–2 carried latent mean is
/// `τ + λ(μ_t + e^{a(t−u)} m x)`. Equation 5 of the Table 3
/// first-occasion TI predictor is `τ + λ(μ_t + e^{a Δt} t0_b z)`.
/// Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean`].
pub fn refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean(
    impulse_carry_observed_mean: f64,
    initial_time_independent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        impulse_carry_observed_mean,
        initial_time_independent_observed_mean,
    );
    Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean)
}

/// Exact scalar first-occasion time-dependent predictor shift.
///
/// Driver, Oud, and Voelkle (2017, Table 3, p. 13; Eq. 3 first
/// summand, p. 5; JSS PDF re-opened 2026-08-20T19:10Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// name `T0TDPREDEFFECT` the effect of time-dependent predictors on
/// latents at `T0`. Table 2 / Table 3 name `TDPREDEFFECT` `M`, which
/// enters Equation 3 as the printed fourth-summand Dirac `M x` at
/// `u = t`. Those are not the same matrix. The scalar first-occasion
/// shift is `t0_m x0`. It is not `M`, not `M x`, not
/// `e^{A(t−u)} M x` for `t0 < u < t`, not `t0_b z`, not
/// `A^{-1}[e^{A Δt} − I] B z`, and not `κ`. An impulse at `u ≤ t0`
/// that used `M` is already in `η(t0)` as `TDPREDEFFECT`, not as
/// `T0TDPREDEFFECT`. A zero effect or zero predictor is exactly
/// zero. This is not a Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::InvalidNumericInput`] when the effect
/// or predictor is non-finite or the product overflows.
pub fn recover_initial_time_dependent_predictor_effect(
    initial_time_dependent_effect: f64,
    time_dependent_predictor: f64,
) -> Result<f64, PsychometricError> {
    if !initial_time_dependent_effect.is_finite() || !time_dependent_predictor.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    if initial_time_dependent_effect == 0.0 || time_dependent_predictor == 0.0 {
        return Ok(0.0);
    }
    require_finite(initial_time_dependent_effect * time_dependent_predictor)
}

/// Exact scalar carried first-occasion time-dependent predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; Table 3, p. 13; JSS
/// PDF re-opened 2026-08-20T19:10Z) write the first summand as
/// `e^{A(t−t0)} η_i(t0)`. A Table 3 `T0TDPREDEFFECT` shift that is
/// already in `η(t0)` therefore appears at `t` as `e^{A Δt} t0_m x0`.
/// Form `t0_m x0` first, then `e^{a Δt} t0_m x0`. A zero drift is
/// `t0_m x0` with no dissipation of the first-occasion shift.
/// Binary64 underflow of `e^{a Δt}` to `+0` is a vanishing carry of
/// that shift and is kept. This carry is not the first-occasion
/// shift, not `M x`, not `e^{A(t−u)} M x` for `t0 < u < t`, not
/// `t0_b z`, not `A^{-1}[e^{A Δt} − I] B z`, and not `CINT`. When
/// `exp` overflows at a finite `a Δt`, rewrite as
/// `sign(t0_m x0) exp(ln|t0_m x0| + a Δt)`. An overflowing rewrite
/// fails closed. This is not a Kalman filter and not ctsem
/// estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when
/// `event_delta` is not strictly positive, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or the mapped carry overflows.
pub fn recover_initial_time_dependent_predictor_carry(
    initial_time_dependent_effect: f64,
    time_dependent_predictor: f64,
    log_rate: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let initial_shift = recover_initial_time_dependent_predictor_effect(
        initial_time_dependent_effect,
        time_dependent_predictor,
    )?;
    if initial_shift == 0.0 {
        return Ok(0.0);
    }
    let drift_interval = log_rate * event_delta;
    let auto_effect = drift_interval.exp();
    if auto_effect.is_finite() {
        // +0 underflow is a vanishing carry of the T0 TD shift.
        return require_finite(auto_effect * initial_shift);
    }
    if !drift_interval.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite a Δt, overflowed exp.
    // e^{a Δt} t0_m x0 = sign(t0_m x0) exp(ln|t0_m x0| + a Δt).
    require_finite(initial_shift.signum() * (initial_shift.abs().ln() + drift_interval).exp())
}

/// Exact scalar evolved latent mean plus a first-occasion TD predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3, p. 5; Table 3, p. 13) write
/// the first summand as the carried `T0MEANS`, which includes any
/// `T0TDPREDEFFECT` shift already in `η(t0)`. Form `μ_t` first, then
/// add `e^{a Δt} t0_m x0`. A zero carry is exactly `μ_t`. A zero
/// evolved mean is exactly the carry. Adding `t0_m x0` without the
/// exponential is not this composition when `a Δt ≠ 0`. Adding
/// `M x` or `e^{A(t−u)} M x` is not this composition.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_initial_time_dependent_predictor_carry`], and returns
/// [`PsychometricError::InvalidNumericInput`] when the sum overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_latent_mean_with_initial_time_dependent_predictor(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    initial_time_dependent_effect: f64,
    time_dependent_predictor: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let initial_carry = recover_initial_time_dependent_predictor_carry(
        initial_time_dependent_effect,
        time_dependent_predictor,
        log_rate,
        event_delta,
        clock,
    )?;
    if initial_carry == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(initial_carry);
    }
    require_finite(evolved_latent_mean + initial_carry)
}

/// Refuse treating the Table 3 first-occasion TD shift as `M x`.
///
/// `T0TDPREDEFFECT` shifts `η(t0)`. `TDPREDEFFECT` `M` enters the
/// SDE as the contemporaneous Dirac `M x` at `u = t`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse`].
pub fn refuse_initial_time_dependent_effect_as_contemporaneous_impulse(
    initial_time_dependent_effect: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_effect, time_dependent_impulse);
    Err(PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse)
}

/// Refuse treating the Eq. 3 carry of `T0TDPREDEFFECT` as the
/// first-occasion shift.
///
/// `e^{A Δt} t0_m x0` is the first summand's contribution at `t`.
/// `t0_m x0` is the shift at `T0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentCarryIsNotInitialEffect`].
pub fn refuse_initial_time_dependent_carry_as_initial_effect(
    initial_time_dependent_carry: f64,
    initial_time_dependent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_carry, initial_time_dependent_effect);
    Err(PsychometricError::InitialTimeDependentCarryIsNotInitialEffect)
}

/// Refuse treating the Table 3 first-occasion TD shift as `CINT`.
///
/// `t0_m x0` is an initial-mean shift. `κ` is the continuous intercept.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept`].
pub fn refuse_initial_time_dependent_effect_as_continuous_intercept(
    initial_time_dependent_effect: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_effect, continuous_intercept);
    Err(PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept)
}

/// Refuse treating the Table 3 first-occasion TD shift as the Eq. 3
/// process increment.
///
/// `t0_m x0` shifts `η(t0)`. `TIPREDEFFECT` `B` maps as
/// `A^{-1}[e^{A Δt} − I] B z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement`].
pub fn refuse_initial_time_dependent_effect_as_process_increment(
    initial_time_dependent_effect: f64,
    time_independent_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_effect, time_independent_increment);
    Err(PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement)
}

/// Refuse treating the Table 3 first-occasion TD shift as the Table 3
/// first-occasion TI shift.
///
/// `T0TDPREDEFFECT` and `T0TIPREDEFFECT` are different Table 3
/// matrices. `t0_m x0` is not `t0_b z`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect`].
pub fn refuse_initial_time_dependent_effect_as_initial_time_independent_effect(
    initial_time_dependent_effect: f64,
    initial_time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_dependent_effect,
        initial_time_independent_effect,
    );
    Err(PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect)
}

/// Refuse treating Driver Table 3 `T0TDPREDEFFECT` as the
/// first-occasion shift.
///
/// `T0TDPREDEFFECT` is the coefficient. The shift is `t0_m x0`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect`].
pub fn refuse_initial_time_dependent_coefficient_as_initial_effect(
    initial_time_dependent_coefficient: f64,
    initial_time_dependent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_dependent_coefficient,
        initial_time_dependent_effect,
    );
    Err(PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect)
}

/// Refuse treating the Eq. 3 carry of `T0TDPREDEFFECT` as the
/// within-interval impulse carry.
///
/// `e^{A Δt} t0_m x0` carries a Table 3 first-occasion TD shift.
/// `e^{A(t−u)} M x` for `t0 < u < t` carries a Table 2 Dirac that
/// occurred inside the interval.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry`].
pub fn refuse_initial_time_dependent_carry_as_impulse_carry(
    initial_time_dependent_carry: f64,
    impulse_carry: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_time_dependent_carry, impulse_carry);
    Err(PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry)
}

/// Exact scalar observed mean of a first-occasion time-dependent
/// predictor.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 3 first summand,
/// p. 5; Table 3, p. 13; JSS PDF re-opened 2026-08-20T19:20Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. Table 3 names `T0TDPREDEFFECT` the effect of
/// time-dependent predictors on latents at `T0`. Equation 3's first
/// summand carries that shift as `e^{A Δt} t0_m x0`. The expected
/// intercept is `τ`. The latent process at `t` after that carry is
/// `μ_t + e^{a Δt} t0_m x0`. The scalar composition is
/// `E(y_t) = τ + λ(μ_t + e^{a Δt} t0_m x0)`. Form the
/// evolved-plus-carry latent mean first, then `τ + λ` of that mean.
/// A zero loading is exactly `τ`. A zero evolved-plus-carry latent
/// mean is exactly `τ`. A zero intercept is exactly
/// `λ(μ_t + e^{a Δt} t0_m x0)`. The evolved observed mean
/// `τ + λ μ_t` is not this composition when the carry is nonzero.
/// The process-increment map
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not this composition.
/// The contemporaneous map `τ + λ(μ_t + m x)` is not this
/// composition. The impulse-carry map
/// `τ + λ(μ_t + e^{a(t−u)} m x)` is not this composition when
/// `u ≠ t0`. The first-occasion TI map
/// `τ + λ(μ_t + e^{a Δt} t0_b z)` is not this composition.
/// `MANIFESTMEANS` is not `E(y_t)`. The evolved-plus-carry latent
/// mean is not `E(y_t)`. `T0TDPREDEFFECT` is the coefficient, not
/// that observed mean. This is not a Kalman filter and not ctsem
/// estimation.
///
/// # Errors
///
/// Propagates
/// [`recover_discrete_latent_mean_with_initial_time_dependent_predictor`]
/// and [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_initial_time_dependent_predictor(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    initial_time_dependent_effect: f64,
    time_dependent_predictor: f64,
    manifest_mean: f64,
    event_delta: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let composed_latent_mean = recover_discrete_latent_mean_with_initial_time_dependent_predictor(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        initial_time_dependent_effect,
        time_dependent_predictor,
        event_delta,
        clock,
    )?;
    recover_manifest_observed_mean(loading, composed_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the first-occasion
/// time-dependent-predictor observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the Table 3 first-occasion TD predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_m x0)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean`].
pub fn refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean(
    evolved_observed_mean: f64,
    initial_time_dependent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, initial_time_dependent_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean)
}

/// Refuse treating the process-increment observed mean as the
/// first-occasion time-dependent-predictor observed mean.
///
/// Equation 5 of the Eq. 3 `TIPREDEFFECT` increment is
/// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`. Equation 5 of the
/// Table 3 first-occasion TD predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_m x0)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean`].
pub fn refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
    time_independent_observed_mean: f64,
    initial_time_dependent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        time_independent_observed_mean,
        initial_time_dependent_observed_mean,
    );
    Err(PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean)
}

/// Refuse treating the contemporaneous-impulse observed mean as the
/// first-occasion time-dependent-predictor observed mean.
///
/// Equation 5 of the contemporaneous Dirac is `τ + λ(μ_t + m x)`.
/// Equation 5 of the Table 3 first-occasion TD predictor is
/// `τ + λ(μ_t + e^{a Δt} t0_m x0)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean`].
pub fn refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean(
    impulse_observed_mean: f64,
    initial_time_dependent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (impulse_observed_mean, initial_time_dependent_observed_mean);
    Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean)
}

/// Refuse treating the impulse-carry observed mean as the
/// first-occasion time-dependent-predictor observed mean.
///
/// Equation 5 of the Eq. 1–2 carried latent mean is
/// `τ + λ(μ_t + e^{a(t−u)} m x)`. Equation 5 of the Table 3
/// first-occasion TD predictor is `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
/// Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean`].
pub fn refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean(
    impulse_carry_observed_mean: f64,
    initial_time_dependent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        impulse_carry_observed_mean,
        initial_time_dependent_observed_mean,
    );
    Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean)
}

/// Refuse treating the first-occasion TI observed mean as the
/// first-occasion TD observed mean.
///
/// Equation 5 of Table 3 `T0TIPREDEFFECT` is
/// `τ + λ(μ_t + e^{a Δt} t0_b z)`. Equation 5 of Table 3
/// `T0TDPREDEFFECT` is `τ + λ(μ_t + e^{a Δt} t0_m x0)`. Those are
/// not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean`].
pub fn refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
    initial_time_independent_observed_mean: f64,
    initial_time_dependent_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (
        initial_time_independent_observed_mean,
        initial_time_dependent_observed_mean,
    );
    Err(PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean)
}

/// Exact scalar within-interval time-dependent impulse carry from
/// Driver Equations 1–2.
///
/// Driver, Oud, and Voelkle (2017, Eq. 1–3, pp. 4–5; Table 2, p. 12;
/// §7.2, pp. 20–21; JSS PDF re-opened 2026-08-20T10:33Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `dη = (A η + ξ + B z + M χ(t)) dt + G dW` with
/// `χ_i(t) = Σ_{u ∈ U_i} x_{i,u} δ(t − u)`. The Green-function
/// integral of that Dirac on `(t0, t)` is `e^{A(t−u)} M x`. The
/// printed Eq. 3 fourth summand is the contemporaneous jump `M x`
/// at `u = t`. This map is the strictly within-interval case
/// `t0 < u < t`: form `m x` first, then `e^{a(t−u)} m x`. A zero
/// drift is `m x` with no dissipation. Binary64 underflow of
/// `e^{a(t−u)}` to `+0` is vanishing dissipation back to the process
/// mean (§7.2) and is kept. A zero effect or zero predictor is
/// exactly zero even if the exponential overflows. When `e^{a(t−u)}`
/// overflows at a finite `a(t−u)`, rewrite as
/// `sign(m x) exp(ln|m x| + a(t−u))`. An impulse at `u = t` is the
/// contemporaneous map. An impulse at `u ≤ t0` is already in `η(t0)`.
/// The §7.2 level-change form is a different specification and is
/// not this map. This is not a Kalman filter and not ctsem
/// estimation.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for any non-event
/// clock, [`PsychometricError::NonPositiveInterval`] when
/// `event_delta` or `elapsed_after_impulse` is not strictly positive
/// or the impulse is not strictly inside `(t0, t)`, and
/// [`PsychometricError::InvalidNumericInput`] when an input is
/// non-finite or `m x` or the carried product overflows.
pub fn recover_time_dependent_predictor_impulse_carry(
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    log_rate: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if !event_delta.is_finite() || event_delta <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !elapsed_after_impulse.is_finite() || elapsed_after_impulse <= 0.0 {
        return Err(PsychometricError::NonPositiveInterval);
    }
    // I_{t0 < u < t}: t−u strictly less than t−t0, so u−t0 > 0.
    if elapsed_after_impulse >= event_delta {
        return Err(PsychometricError::NonPositiveInterval);
    }
    if !log_rate.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let impulse =
        recover_time_dependent_predictor_impulse(time_dependent_effect, time_dependent_predictor)?;
    if impulse == 0.0 {
        return Ok(0.0);
    }
    let drift_interval = log_rate * elapsed_after_impulse;
    let auto_effect = drift_interval.exp();
    if auto_effect.is_finite() {
        // +0 underflow is vanishing dissipation (§7.2).
        return require_finite(auto_effect * impulse);
    }
    if !drift_interval.is_finite() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    // Finite a(t−u), overflowed exp.
    // e^{a(t−u)} m x = sign(m x) exp(ln|m x| + a(t−u)).
    require_finite(impulse.signum() * (impulse.abs().ln() + drift_interval).exp())
}

/// Exact scalar evolved latent mean plus a within-interval impulse carry.
///
/// Driver, Oud, and Voelkle (2017, Eq. 1–3, p. 5; §7.2) write the
/// first two summands as the carried `T0MEANS` and `CINT` increment,
/// then add a Dirac impulse that occurred strictly inside `(t0, t)`
/// after it has dissipated by `e^{A(t−u)}`. Form `μ_t` first, then
/// add `e^{a(t−u)} m x`. A zero carry is exactly `μ_t`. A zero
/// evolved mean is exactly the carry. Adding the contemporaneous
/// `m x` is not this composition when `u ≠ t`. The level-change
/// form is not this map.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean`] and
/// [`recover_time_dependent_predictor_impulse_carry`], and returns
/// [`PsychometricError::InvalidNumericInput`] when the sum overflows.
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_latent_mean_with_impulse_carry(
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let evolved_latent_mean = recover_discrete_latent_mean(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        event_delta,
        clock,
    )?;
    let impulse_carry = recover_time_dependent_predictor_impulse_carry(
        time_dependent_effect,
        time_dependent_predictor,
        log_rate,
        event_delta,
        elapsed_after_impulse,
        clock,
    )?;
    if impulse_carry == 0.0 {
        return Ok(evolved_latent_mean);
    }
    if evolved_latent_mean == 0.0 {
        return Ok(impulse_carry);
    }
    require_finite(evolved_latent_mean + impulse_carry)
}

/// Exact scalar observed mean of a within-interval impulse carry.
///
/// Driver, Oud, and Voelkle (2017, Eq. 5, p. 5; Eq. 1–2, pp. 4–5;
/// Eq. 3 exponential map; Table 2, p. 12; §7.2, pp. 20–21; JSS PDF
/// re-opened 2026-08-20T05:12Z from
/// <https://www.jstatsoft.org/index.php/jss/article/download/v077i05/1104>)
/// write `y_i(t) = Γ + Λ η_i(t) + ζ_i(t)` with `ζ ~ N(0, Θ)` and
/// `Γ ~ N(τ, Ψ)`. The expected intercept is `τ`. The latent process
/// at `t` after a Dirac that occurred strictly inside `(t0, t)` is
/// `μ_t + e^{a(t−u)} m x`. The scalar composition is
/// `E(y_t) = τ + λ(μ_t + e^{a(t−u)} m x)`. Form the carried latent
/// mean first, then `τ + λ` of that mean. Table 2 names `τ`
/// `MANIFESTMEANS`. A zero loading is exactly `τ`. A zero
/// evolved-plus-carry latent mean is exactly `τ`. A zero intercept
/// is exactly `λ(μ_t + carry)`. The evolved observed mean
/// `τ + λ μ_t` is not this composition when the carry is nonzero.
/// The contemporaneous map `τ + λ(μ_t + m x)` is not this
/// composition when `u ≠ t`. `MANIFESTMEANS` is not `E(y_t)`. The
/// carried latent mean is not `E(y_t)`. The §7.2 level-change form
/// is a different specification and is not this map. This is not a
/// Kalman filter and not ctsem estimation.
///
/// # Errors
///
/// Propagates [`recover_discrete_latent_mean_with_impulse_carry`] and
/// [`recover_manifest_observed_mean`].
#[allow(clippy::too_many_arguments)]
pub fn recover_discrete_observed_mean_with_impulse_carry(
    loading: f64,
    initial_latent_mean: f64,
    log_rate: f64,
    continuous_intercept: f64,
    time_dependent_effect: f64,
    time_dependent_predictor: f64,
    manifest_mean: f64,
    event_delta: f64,
    elapsed_after_impulse: f64,
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    let carried_latent_mean = recover_discrete_latent_mean_with_impulse_carry(
        initial_latent_mean,
        log_rate,
        continuous_intercept,
        time_dependent_effect,
        time_dependent_predictor,
        event_delta,
        elapsed_after_impulse,
        clock,
    )?;
    recover_manifest_observed_mean(loading, carried_latent_mean, manifest_mean)
}

/// Refuse treating the evolved observed mean as the impulse-carry
/// observed mean.
///
/// Equation 5 of the Eq. 3 evolved mean is `τ + λ μ_t`. Equation 5
/// of the Eq. 1–2 carried latent mean is
/// `τ + λ(μ_t + e^{a(t−u)} m x)`. Those are not the same map.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean`].
pub fn refuse_evolved_observed_mean_as_impulse_carry_observed_mean(
    evolved_observed_mean: f64,
    impulse_carry_observed_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (evolved_observed_mean, impulse_carry_observed_mean);
    Err(PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean)
}

/// Refuse treating the Eq. 1–2 impulse carry as the contemporaneous Dirac.
///
/// The printed Eq. 3 fourth summand is `M x` at `u = t`. The
/// within-interval carry is `e^{A(t−u)} M x` for `t0 < u < t`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse`].
pub fn refuse_time_dependent_impulse_carry_as_contemporaneous_impulse(
    time_dependent_impulse_carry: f64,
    time_dependent_impulse: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse_carry, time_dependent_impulse);
    Err(PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse)
}

/// Refuse treating the Eq. 1–2 impulse carry as `CINT`.
///
/// Table 2 names `M` `TDPREDEFFECT` and `κ` `CINT`. The dissipated
/// impulse is not the continuous intercept.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept`].
pub fn refuse_time_dependent_impulse_carry_as_continuous_intercept(
    time_dependent_impulse_carry: f64,
    continuous_intercept: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse_carry, continuous_intercept);
    Err(PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept)
}

/// Refuse treating the Eq. 1–2 impulse carry as `TIPREDEFFECT`.
///
/// The second-summand map integrates a constant `B z` over the event
/// interval. The within-interval TDPRED carry dissipates a Dirac.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect`].
pub fn refuse_time_dependent_impulse_carry_as_time_independent_effect(
    time_dependent_impulse_carry: f64,
    time_independent_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse_carry, time_independent_effect);
    Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect)
}

/// Refuse treating the Eq. 1–2 impulse carry as Voelkle et al.
/// (2012, Eq. 14).
///
/// Equation 14 is `a_{yx} Δt` for a piecewise-constant time-varying
/// predictor. The Dirac carry is `e^{A(t−u)} M x`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect`].
pub fn refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect(
    time_dependent_impulse_carry: f64,
    time_varying_discrete_effect: f64,
) -> Result<f64, PsychometricError> {
    let _ = (time_dependent_impulse_carry, time_varying_discrete_effect);
    Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect)
}

/// Refuse treating Driver Table 2 `T0MEANS` as the evolved latent mean.
///
/// Equation 3 maps `μ_t = exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`.
/// `T0MEANS` is `μ_0`, not `μ_t`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::InitialLatentMeanIsNotEvolvedMean`].
pub fn refuse_initial_latent_mean_as_evolved_mean(
    initial_latent_mean: f64,
    evolved_latent_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (initial_latent_mean, evolved_latent_mean);
    Err(PsychometricError::InitialLatentMeanIsNotEvolvedMean)
}

/// Refuse treating Driver Table 2 `CINT` as the discrete mean increment.
///
/// `κ` is the continuous intercept. Equation 3 maps it through
/// `A^{-1}[e^{A Δt} − I]`. `κ` is not that increment.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement`].
pub fn refuse_continuous_intercept_as_discrete_mean_increment(
    continuous_intercept: f64,
    discrete_mean_increment: f64,
) -> Result<f64, PsychometricError> {
    let _ = (continuous_intercept, discrete_mean_increment);
    Err(PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement)
}

/// Refuse treating Driver Table 2 `CINT` as `T0MEANS`.
///
/// Table 2 (p. 12) names `κ` `CINT` and the first-occasion latent
/// mean `T0MEANS`. `κ` is not `E(η_{i1})`.
///
/// # Errors
///
/// Always returns
/// [`PsychometricError::ContinuousInterceptIsNotInitialLatentMean`].
pub fn refuse_continuous_intercept_as_initial_latent_mean(
    continuous_intercept: f64,
    initial_latent_mean: f64,
) -> Result<f64, PsychometricError> {
    let _ = (continuous_intercept, initial_latent_mean);
    Err(PsychometricError::ContinuousInterceptIsNotInitialLatentMean)
}

/// Refuse treating Driver Eq. 3 process noise as the unconditional variance.
///
/// Driver, Oud, and Voelkle (2017, Eq. 3–4, pp. 4–5):
/// `Q_Δt = cov(η_ti | η_{t-1,i})` for the homogeneous process. That
/// residual variance is not `Var(η_ti)` when the previous state is
/// random. The JSS article has no numbered §2.2.
///
/// # Errors
///
/// Always returns [`PsychometricError::ProcessNoiseIsConditionalVariance`].
pub fn refuse_process_noise_as_unconditional_variance(
    process_noise: f64,
    prior_variance: f64,
) -> Result<f64, PsychometricError> {
    let _ = (process_noise, prior_variance);
    Err(PsychometricError::ProcessNoiseIsConditionalVariance)
}

/// Refuse the difference quotient as a continuous-time rate.
///
/// Voelkle et al. (2012) discourage `(x(t+Δt) − x(t)) / Δt` as the drift.
///
/// # Errors
///
/// Always returns [`PsychometricError::DifferenceQuotientForbidden`].
pub fn refuse_difference_quotient_as_local_rate(
    earlier: f64,
    later: f64,
    delta: f64,
) -> Result<f64, PsychometricError> {
    let _ = (earlier, later, delta);
    Err(PsychometricError::DifferenceQuotientForbidden)
}

/// Mean local log-rate across consecutive event-time pairs.
///
/// Occasions are sorted by event time. Each pair uses the exact scalar map.
/// Equal or inverted times fail closed.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for a non-event clock,
/// [`PsychometricError::InvalidNumericInput`] for fewer than two occasions or
/// non-finite values, and [`PsychometricError::NonPositiveInterval`] when
/// consecutive times are not strictly increasing.
pub fn recover_event_series_mean_log_rate(
    occasions: &[EventOccasion],
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if occasions.len() < 2 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut ordered = occasions.to_vec();
    ordered.sort_by(|left, right| {
        left.event_time
            .partial_cmp(&right.event_time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut rates = Vec::new();
    for window in ordered.windows(2) {
        let earlier = window[0];
        let later = window[1];
        if !earlier.event_time.is_finite()
            || !later.event_time.is_finite()
            || !earlier.score.is_finite()
            || !later.score.is_finite()
        {
            return Err(PsychometricError::InvalidNumericInput);
        }
        let delta = later.event_time - earlier.event_time;
        let recovered =
            recover_event_time_discrete_lag_and_log_rate(earlier.score, later.score, delta, clock)?;
        rates.push(recovered.log_rate);
    }
    let count = rates.len() as f64;
    require_finite(rates.iter().sum::<f64>() / count)
}

/// Local log-rate of cluster-mean-centered residuals on event time.
///
/// Stable between-cluster means are removed first (CWC). Consecutive
/// within-cluster residuals then use the exact scalar map. This is not DSEM.
///
/// Curran and Bauer (2011, pp. 607–608) show that subtracting the observed
/// person-specific mean from a raw autoregressive series does **not** isolate
/// the lagged within-person effect. This helper therefore does not claim to
/// recover the raw-process drift `a` from CWC of a raw AR path. For that
/// estimand, supply already-centered lagged residuals to
/// [`recover_irregular_centered_residual_log_rate`].
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for a non-event clock,
/// [`PsychometricError::InvalidNumericInput`] for empty, singleton, or
/// non-finite rows, [`PsychometricError::InsufficientClusters`] when fewer
/// than two clusters appear, and interval/lag errors from the scalar map.
pub fn recover_within_residual_event_time_log_rate(
    rows: &[ClusteredEventScore],
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if rows.len() < 2 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut groups: BTreeMap<u64, Vec<ClusteredEventScore>> = BTreeMap::new();
    for &row in rows {
        if !row.event_time.is_finite() || !row.score.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        groups.entry(row.cluster_key).or_default().push(row);
    }
    if groups.len() < 2 {
        return Err(PsychometricError::InsufficientClusters);
    }
    let mut pairs = Vec::new();
    for occasions in groups.values_mut() {
        if occasions.len() < 2 {
            continue;
        }
        let count = occasions.len() as f64;
        let mean = occasions.iter().map(|row| row.score).sum::<f64>() / count;
        occasions.sort_by(|left, right| {
            left.event_time
                .partial_cmp(&right.event_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for window in occasions.windows(2) {
            let earlier_resid = window[0].score - mean;
            let later_resid = window[1].score - mean;
            let delta = window[1].event_time - window[0].event_time;
            if !delta.is_finite() || delta <= 0.0 {
                return Err(PsychometricError::NonPositiveInterval);
            }
            if !(earlier_resid.is_finite() & later_resid.is_finite()) {
                return Err(PsychometricError::InvalidNumericInput);
            }
            pairs.push((earlier_resid, later_resid, delta));
        }
    }
    fit_scalar_log_rate(&pairs)
}

/// Mean exact scalar log-rate on already-centered residuals with irregular intervals.
///
/// Each pair is `a = ln(later / earlier) / Δt` (Voelkle et al., 2012, Eq. 7).
/// The function does **not** center again. Curran and Bauer (2011, pp. 607–608)
/// reject person-mean subtraction on a raw autoregressive series as the
/// lagged within-person residual. Intervals may be irregular. This is not DSEM.
///
/// # Errors
///
/// Returns [`PsychometricError::EventTimeRequired`] for a non-event clock,
/// [`PsychometricError::InvalidNumericInput`] for an empty series or a
/// non-finite / non-positive residual ratio, and
/// [`PsychometricError::NonPositiveInterval`] when any interval is not
/// strictly positive.
pub fn recover_irregular_centered_residual_log_rate(
    pairs: &[LaggedWithinResidual],
    clock: LagClock,
) -> Result<f64, PsychometricError> {
    if !clock.admits_structural_lag() {
        return Err(PsychometricError::EventTimeRequired);
    }
    if pairs.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut sum = 0.0_f64;
    for pair in pairs {
        if !pair.earlier_residual.is_finite()
            || !pair.later_residual.is_finite()
            || !pair.event_delta.is_finite()
        {
            return Err(PsychometricError::InvalidNumericInput);
        }
        let recovered = recover_event_time_discrete_lag_and_log_rate(
            pair.earlier_residual,
            pair.later_residual,
            pair.event_delta,
            clock,
        )?;
        sum += recovered.log_rate;
    }
    let count = pairs.len() as f64;
    require_finite(sum / count)
}

/// Least-squares scalar log-rate for already-formed residual pairs.
///
/// Pair-wise logs initialize Newton. This helper is crate-visible so overflow
/// and flat-derivative guards can be recovered in unit tests. It is not a
/// public DSEM estimator.
pub(crate) fn fit_scalar_log_rate(pairs: &[(f64, f64, f64)]) -> Result<f64, PsychometricError> {
    if pairs.is_empty() {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut start_sum = 0.0_f64;
    let mut start_count = 0.0_f64;
    for &(earlier, later, delta) in pairs {
        if earlier != 0.0 {
            let discrete_lag = later / earlier;
            if discrete_lag.is_finite() && discrete_lag > 0.0 {
                start_sum += discrete_lag.ln() / delta;
                start_count += 1.0;
            }
        }
    }
    if start_count <= 0.0 {
        return Err(PsychometricError::InvalidNumericInput);
    }
    let mut log_rate = start_sum / start_count;
    for _ in 0..16 {
        let mut score = 0.0_f64;
        let mut derivative = 0.0_f64;
        for &(earlier, later, delta) in pairs {
            let mapped = (log_rate * delta).exp();
            if !mapped.is_finite() || mapped <= 0.0 {
                return Err(PsychometricError::InvalidNumericInput);
            }
            let weight = delta * earlier;
            score += weight * mapped * later - delta * mapped * mapped * earlier * earlier;
            derivative += delta * weight * mapped * later
                - 2.0 * delta * delta * mapped * mapped * earlier * earlier;
        }
        if !score.is_finite() || !derivative.is_finite() {
            return Err(PsychometricError::InvalidNumericInput);
        }
        if derivative.abs() <= 1e-18 {
            break;
        }
        let next = log_rate - score / derivative;
        if (next - log_rate).abs() < 1e-14 {
            log_rate = next;
            break;
        }
        log_rate = next;
    }
    require_finite(log_rate)
}

#[cfg(test)]
mod tests {
    use super::{
        ClusteredEventScore, EventOccasion, LagClock, LaggedWithinResidual, fit_scalar_log_rate,
        map_discrete_lag_across_event_intervals, recover_asymptotic_continuous_intercept,
        recover_asymptotic_time_independent_observed_variance,
        recover_asymptotic_time_independent_predictor_effect,
        recover_asymptotic_time_independent_predictor_variance,
        recover_discrete_constant_predictor_effect, recover_discrete_continuous_intercept_effect,
        recover_discrete_lag_from_log_rate, recover_discrete_lag_one,
        recover_discrete_lagged_latent_covariance, recover_discrete_latent_mean,
        recover_discrete_latent_mean_with_extra_process,
        recover_discrete_latent_mean_with_extra_process_after,
        recover_discrete_latent_mean_with_impulse, recover_discrete_latent_mean_with_impulse_carry,
        recover_discrete_latent_mean_with_initial_time_dependent_predictor,
        recover_discrete_latent_mean_with_initial_time_independent_predictor,
        recover_discrete_latent_mean_with_time_independent_predictor,
        recover_discrete_latent_variance, recover_discrete_observed_mean,
        recover_discrete_observed_mean_with_extra_process,
        recover_discrete_observed_mean_with_extra_process_after,
        recover_discrete_observed_mean_with_impulse,
        recover_discrete_observed_mean_with_impulse_carry,
        recover_discrete_observed_mean_with_initial_time_dependent_predictor,
        recover_discrete_observed_mean_with_initial_time_independent_predictor,
        recover_discrete_observed_mean_with_time_independent_predictor,
        recover_discrete_process_noise, recover_discrete_time_independent_predictor_effect,
        recover_discrete_time_varying_predictor_effect, recover_event_series_mean_log_rate,
        recover_event_time_discrete_lag_and_log_rate,
        recover_initial_time_dependent_observed_variance,
        recover_initial_time_dependent_predictor_carry,
        recover_initial_time_dependent_predictor_effect,
        recover_initial_time_dependent_predictor_variance,
        recover_initial_time_independent_observed_variance,
        recover_initial_time_independent_predictor_carry,
        recover_initial_time_independent_predictor_effect,
        recover_initial_time_independent_predictor_variance,
        recover_irregular_centered_residual_log_rate, recover_level_change_continuous_intercept,
        recover_level_change_discrete_increment, recover_level_change_extra_process_contribution,
        recover_level_change_extra_process_contribution_after, recover_local_log_rate,
        recover_manifest_lagged_observed_covariance, recover_manifest_observed_mean,
        recover_manifest_observed_variance, recover_manifest_trait_plus_state_observed_variance,
        recover_predetermined_initial_latent_variance,
        recover_predetermined_initial_observed_variance,
        recover_predetermined_lagged_latent_covariance,
        recover_predetermined_lagged_observed_covariance,
        recover_predetermined_later_lagged_latent_covariance,
        recover_predetermined_later_lagged_observed_covariance,
        recover_predetermined_later_latent_variance, recover_predetermined_later_observed_variance,
        recover_predetermined_later_start_later_latent_variance,
        recover_predetermined_later_start_later_observed_variance,
        recover_standardised_asymptotic_time_independent_predictor_effect,
        recover_standardised_continuous_diffusion, recover_standardised_continuous_drift,
        recover_standardised_continuous_time_dependent_predictor_effect,
        recover_standardised_continuous_time_independent_predictor_effect,
        recover_standardised_discrete_diffusion, recover_standardised_discrete_drift,
        recover_standardised_initial_time_dependent_predictor_effect,
        recover_standardised_initial_time_independent_predictor_effect,
        recover_stationary_initial_latent_mean, recover_stationary_initial_latent_variance,
        recover_stationary_initial_observed_mean, recover_stationary_initial_observed_variance,
        recover_stationary_lagged_latent_covariance, recover_stationary_lagged_observed_covariance,
        recover_stationary_latent_variance, recover_stationary_later_latent_variance,
        recover_stationary_later_observed_variance, recover_time_dependent_predictor_impulse,
        recover_time_dependent_predictor_impulse_carry, recover_trait_plus_state_lagged_covariance,
        recover_trait_plus_state_latent_variance, recover_within_residual_event_time_log_rate,
        refuse_after_extra_process_contribution_as_observed_mean,
        refuse_after_extra_process_latent_mean_as_observed_mean,
        refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect,
        refuse_asymptotic_continuous_intercept_as_continuous_intercept,
        refuse_asymptotic_continuous_intercept_as_discrete_increment,
        refuse_asymptotic_continuous_intercept_as_initial_latent_mean,
        refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean,
        refuse_asymptotic_time_independent_effect_as_coefficient,
        refuse_asymptotic_time_independent_effect_as_continuous_intercept,
        refuse_asymptotic_time_independent_effect_as_discrete_effect,
        refuse_asymptotic_time_independent_effect_as_time_dependent_impulse,
        refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance,
        refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance,
        refuse_asymptotic_time_independent_observed_variance_as_measurement_error,
        refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance,
        refuse_asymptotic_time_independent_variance_as_asymptotic_effect,
        refuse_asymptotic_time_independent_variance_as_stationary_within_subject,
        refuse_asymptotic_time_independent_variance_as_trait_variance,
        refuse_continuous_intercept_as_discrete_mean_increment,
        refuse_continuous_intercept_as_initial_latent_mean,
        refuse_continuous_intercept_as_manifest_means, refuse_difference_quotient_as_local_rate,
        refuse_evolved_observed_mean_as_after_extra_process_observed_mean,
        refuse_evolved_observed_mean_as_extra_process_observed_mean,
        refuse_evolved_observed_mean_as_impulse_carry_observed_mean,
        refuse_evolved_observed_mean_as_impulse_observed_mean,
        refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean,
        refuse_evolved_observed_mean_as_initial_time_independent_observed_mean,
        refuse_evolved_observed_mean_as_stationary_initial_observed_mean,
        refuse_evolved_observed_mean_as_time_independent_observed_mean,
        refuse_evolved_observed_variance_as_stationary_initial_observed_variance,
        refuse_extra_process_contribution_as_observed_mean,
        refuse_extra_process_latent_mean_as_observed_mean,
        refuse_extra_process_observed_mean_as_after_extra_process_observed_mean,
        refuse_finite_interval_process_noise_as_stationary_variance,
        refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean,
        refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean,
        refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean,
        refuse_impulse_carry_observed_mean_as_time_independent_observed_mean,
        refuse_impulse_observed_mean_as_extra_process_observed_mean,
        refuse_impulse_observed_mean_as_impulse_carry_observed_mean,
        refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean,
        refuse_impulse_observed_mean_as_initial_time_independent_observed_mean,
        refuse_impulse_observed_mean_as_time_independent_observed_mean,
        refuse_initial_latent_mean_as_evolved_mean,
        refuse_initial_observed_mean_as_evolved_observed_mean,
        refuse_initial_observed_mean_as_stationary_initial_observed_mean,
        refuse_initial_observed_variance_as_stationary_initial_observed_variance,
        refuse_initial_time_dependent_carry_as_impulse_carry,
        refuse_initial_time_dependent_carry_as_initial_effect,
        refuse_initial_time_dependent_coefficient_as_initial_effect,
        refuse_initial_time_dependent_effect_as_contemporaneous_impulse,
        refuse_initial_time_dependent_effect_as_continuous_intercept,
        refuse_initial_time_dependent_effect_as_initial_time_independent_effect,
        refuse_initial_time_dependent_effect_as_process_increment,
        refuse_initial_time_dependent_observed_variance_as_initial_observed_variance,
        refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance,
        refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance,
        refuse_initial_time_dependent_observed_variance_as_measurement_error,
        refuse_initial_time_dependent_variance_as_initial_latent_variance,
        refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance,
        refuse_initial_time_dependent_variance_as_initial_time_independent_variance,
        refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect,
        refuse_initial_time_dependent_variance_as_trait_variance,
        refuse_initial_time_independent_carry_as_initial_effect,
        refuse_initial_time_independent_coefficient_as_initial_effect,
        refuse_initial_time_independent_effect_as_continuous_intercept,
        refuse_initial_time_independent_effect_as_process_increment,
        refuse_initial_time_independent_effect_as_time_dependent_impulse,
        refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean,
        refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance,
        refuse_initial_time_independent_observed_variance_as_initial_observed_variance,
        refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance,
        refuse_initial_time_independent_observed_variance_as_measurement_error,
        refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance,
        refuse_initial_time_independent_variance_as_initial_latent_variance,
        refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect,
        refuse_initial_time_independent_variance_as_trait_variance,
        refuse_latent_lagged_covariance_as_observed_covariance,
        refuse_latent_mean_as_observed_mean, refuse_latent_variance_as_observed_variance,
        refuse_level_change_extra_process_as_impulse,
        refuse_level_change_extra_process_as_increment,
        refuse_level_change_extra_process_as_intercept, refuse_level_change_increment_as_impulse,
        refuse_level_change_increment_as_intercept,
        refuse_level_change_increment_as_process_increment,
        refuse_level_change_intercept_as_free_continuous_intercept,
        refuse_level_change_intercept_as_impulse,
        refuse_level_change_intercept_as_process_increment, refuse_manifest_means_as_observed_mean,
        refuse_manifest_trait_variance_as_measurement_error,
        refuse_measurement_error_as_lagged_observed_covariance,
        refuse_measurement_error_as_observed_variance,
        refuse_measurement_error_as_predetermined_initial_observed_variance,
        refuse_measurement_error_as_predetermined_lagged_observed_covariance,
        refuse_measurement_error_as_predetermined_later_lagged_observed_covariance,
        refuse_measurement_error_as_predetermined_later_observed_variance,
        refuse_measurement_error_as_predetermined_later_start_later_observed_variance,
        refuse_measurement_error_as_stationary_lagged_observed_covariance,
        refuse_measurement_error_as_stationary_later_observed_variance,
        refuse_pooled_discrete_lag_across_unequal_intervals,
        refuse_predetermined_initial_latent_variance_as_initial_latent_variance,
        refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance,
        refuse_predetermined_initial_latent_variance_as_later_latent_variance,
        refuse_predetermined_initial_latent_variance_as_observed_variance,
        refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance,
        refuse_predetermined_lagged_latent_covariance_as_decayed_total,
        refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance,
        refuse_predetermined_lagged_latent_covariance_as_later_latent_variance,
        refuse_predetermined_lagged_latent_covariance_as_observed_covariance,
        refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance,
        refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance,
        refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total,
        refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance,
        refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance,
        refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance,
        refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance,
        refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance,
        refuse_predetermined_later_latent_variance_as_discrete_variance,
        refuse_predetermined_later_latent_variance_as_initial_latent_variance,
        refuse_predetermined_later_latent_variance_as_observed_variance,
        refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance,
        refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance,
        refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance,
        refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance,
        refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance,
        refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total,
        refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance,
        refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance,
        refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance,
        refuse_predetermined_later_start_later_latent_variance_as_observed_variance,
        refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance,
        refuse_process_noise_as_unconditional_variance,
        refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect,
        refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect,
        refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion,
        refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect,
        refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect,
        refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect,
        refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion,
        refuse_standardised_discrete_drift_as_standardised_continuous_drift,
        refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
        refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
        refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect,
        refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect,
        refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept,
        refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect,
        refuse_stationary_initial_latent_mean_as_discrete_mean,
        refuse_stationary_initial_latent_mean_as_initial_latent_mean,
        refuse_stationary_initial_latent_mean_as_observed_mean,
        refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance,
        refuse_stationary_initial_latent_variance_as_discrete_variance,
        refuse_stationary_initial_latent_variance_as_initial_latent_variance,
        refuse_stationary_initial_latent_variance_as_observed_variance,
        refuse_stationary_initial_latent_variance_as_stationary_within_subject,
        refuse_stationary_initial_latent_variance_as_trait_variance,
        refuse_stationary_initial_observed_mean_as_manifest_means,
        refuse_stationary_initial_observed_variance_as_measurement_error,
        refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance,
        refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance,
        refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance,
        refuse_stationary_lagged_latent_covariance_as_observed_covariance,
        refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance,
        refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance,
        refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance,
        refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance,
        refuse_stationary_later_latent_variance_as_discrete_variance,
        refuse_stationary_later_latent_variance_as_lagged_covariance,
        refuse_stationary_later_latent_variance_as_observed_variance,
        refuse_stationary_later_latent_variance_as_process_noise,
        refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance,
        refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance,
        refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance,
        refuse_time_dependent_impulse_as_continuous_intercept,
        refuse_time_dependent_impulse_as_time_independent_effect,
        refuse_time_dependent_impulse_as_time_varying_discrete_effect,
        refuse_time_dependent_impulse_carry_as_contemporaneous_impulse,
        refuse_time_dependent_impulse_carry_as_continuous_intercept,
        refuse_time_dependent_impulse_carry_as_time_independent_effect,
        refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect,
        refuse_time_independent_coefficient_as_discrete_effect,
        refuse_time_independent_effect_as_continuous_intercept,
        refuse_time_independent_effect_as_time_dependent_impulse,
        refuse_time_independent_effect_as_time_varying_discrete_effect,
        refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean,
        refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean,
        refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
        refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion,
        refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift,
        refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
        refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect,
        refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect,
        refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect,
        refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion,
        refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift,
        refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance,
        refuse_trait_variance_as_process_noise, refuse_trait_variance_as_standardisation_variance,
        refuse_trait_variance_as_stationary_within_subject,
        refuse_unmatched_time_varying_predictor_interval,
        refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect,
        refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion,
        refuse_unstandardised_continuous_drift_as_standardised_continuous_drift,
        refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect,
        refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect,
        refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion,
        refuse_unstandardised_discrete_drift_as_standardised_discrete_drift,
        refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect,
        refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect,
    };
    use crate::error::PsychometricError;

    #[test]
    fn exact_scalar_map_inverts_exponential_drift() {
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let earlier = 1.5_f64;
        let later = earlier * (drift * delta).exp();
        let recovered = recover_event_time_discrete_lag_and_log_rate(
            earlier,
            later,
            delta,
            LagClock::EventTime,
        )
        .expect("exact");
        assert!((recovered.log_rate - drift).abs() < 1e-12);
        assert!((recovered.discrete_lag - (drift * delta).exp()).abs() < 1e-12);
        assert!((recovered.event_delta - delta).abs() < 1e-15);
    }

    #[test]
    fn forward_map_inverts_log_rate_and_remaps_unequal_intervals() {
        let drift = -0.4_f64;
        let source_delta = 1.0_f64;
        let reference_delta = 2.0_f64;
        let source_lag =
            recover_discrete_lag_from_log_rate(drift, source_delta, LagClock::EventTime)
                .expect("forward");
        assert!((source_lag - (drift * source_delta).exp()).abs() < 1e-12);
        let same = map_discrete_lag_across_event_intervals(
            source_lag,
            source_delta,
            source_delta,
            LagClock::EventTime,
        )
        .expect("same interval");
        assert!((same - source_lag).abs() < 1e-12);
        let remapped = map_discrete_lag_across_event_intervals(
            source_lag,
            source_delta,
            reference_delta,
            LagClock::EventTime,
        )
        .expect("remap");
        assert!((remapped - (drift * reference_delta).exp()).abs() < 1e-12);
        // Voelkle manuscript p. 2, 33: φ(1) ≠ φ(2) even for one process.
        assert!((source_lag - remapped).abs() > 1e-9);
        assert_eq!(
            refuse_pooled_discrete_lag_across_unequal_intervals(source_delta, reference_delta),
            Err(PsychometricError::UnequalIntervalPoolingForbidden)
        );
        assert_eq!(
            refuse_pooled_discrete_lag_across_unequal_intervals(source_delta, source_delta),
            Err(PsychometricError::UnequalIntervalPoolingForbidden)
        );
    }

    #[test]
    fn forward_map_and_interval_remap_fail_closed() {
        assert_eq!(
            recover_discrete_lag_from_log_rate(-0.2, 1.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(-0.2, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(-0.2, -1.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(-0.2, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(f64::NAN, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(800.0, 10.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(-800.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_from_log_rate(-1.0, 800.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let source_lag =
            recover_discrete_lag_from_log_rate(-0.7, 1.0, LagClock::EventTime).expect("source φ");
        assert!(source_lag > 0.0);
        assert_eq!(
            map_discrete_lag_across_event_intervals(source_lag, 1.0, 2000.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            map_discrete_lag_across_event_intervals(0.5, 1.0, 2.0, LagClock::AssertionTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            map_discrete_lag_across_event_intervals(0.5, 0.0, 2.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            map_discrete_lag_across_event_intervals(0.5, 1.0, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            map_discrete_lag_across_event_intervals(-0.2, 1.0, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn constant_predictor_discrete_effect_recovers_equation_twelve() {
        let outcome_on_predictor = 0.2_f64;
        let predictor_log_rate = -0.5_f64;
        let delta = 2.0_f64;
        let recovered = recover_discrete_constant_predictor_effect(
            outcome_on_predictor,
            predictor_log_rate,
            delta,
            LagClock::EventTime,
        )
        .expect("eq 12");
        let expected =
            (outcome_on_predictor / predictor_log_rate) * (predictor_log_rate * delta).exp_m1();
        assert!((recovered - expected).abs() < 1e-15);
        let first_order = outcome_on_predictor * delta;
        assert!((recovered - first_order).abs() > 1e-3);
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                predictor_log_rate,
                delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                predictor_log_rate,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                predictor_log_rate,
                -1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                predictor_log_rate,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                f64::NAN,
                predictor_log_rate,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(
                outcome_on_predictor,
                f64::NAN,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(1e300, 1e-300, 1e300, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let underflowed_argument =
            recover_discrete_constant_predictor_effect(1e308, 1e-308, 1e-308, LagClock::EventTime)
                .expect("eq 12 limit");
        assert!((underflowed_argument - 1.0).abs() < 1e-15);
        let tiny_nonzero =
            recover_discrete_constant_predictor_effect(1e308, 1e-154, 1e-154, LagClock::EventTime)
                .expect("eq 12 scaled");
        assert!(tiny_nonzero.is_finite());
        assert!((tiny_nonzero - 1e154).abs() / 1e154 < 1e-12);
        // a_yx Δt overflows; Eq. 12 remains finite (Voelkle 2012, Eq. 12).
        let product_overflow =
            recover_discrete_constant_predictor_effect(1e308, -100.0, 10.0, LagClock::EventTime)
                .expect("eq 12 finite after a_yx Δt overflow");
        let product_overflow_expected = (1e308 / -100.0) * (-100.0_f64 * 10.0).exp_m1();
        assert!((product_overflow - product_overflow_expected).abs() / 1e306 < 1e-12);
        assert!(product_overflow.is_finite());
        assert!(!(1e308_f64 * 10.0).is_finite());
    }

    #[test]
    fn constant_predictor_negative_overflow_recovers_equilibrium_increment() {
        // z → -∞: expm1(z)/z * Δt is +0; Eq. 12 → -a_yx/a_xx (Voelkle
        // 2012, Introducing Intercepts equilibrium increment).
        let increment_argument = -1e308_f64 * 2.0;
        assert!(increment_argument.is_infinite());
        assert!(increment_argument.is_sign_negative());
        let lost_scale = increment_argument.exp_m1() / increment_argument * 2.0;
        assert_eq!(lost_scale.to_bits(), 0.0_f64.to_bits());
        let negative_overflow =
            recover_discrete_constant_predictor_effect(1.0, -1e308, 2.0, LagClock::EventTime)
                .expect("eq 12 equilibrium increment");
        let negative_overflow_expected = -(1.0 / -1e308);
        assert!((negative_overflow - negative_overflow_expected).abs() / 1e-308 < 1e-12);
        assert!(negative_overflow > 0.0);
        assert!(negative_overflow.is_finite());
    }

    #[test]
    fn constant_predictor_expm1_overflow_recovers_finite_equation_twelve() {
        // expm1(800) is +∞; (1e-308/800)(exp(800)−1) is finite.
        assert!(!800.0_f64.exp_m1().is_finite());
        assert!(!(1e-308_f64 * (800.0_f64.exp_m1() / 800.0)).is_finite());
        let recovered =
            recover_discrete_constant_predictor_effect(1e-308, 800.0, 1.0, LagClock::EventTime)
                .expect("eq 12 log-space");
        let expected = (1e-308_f64.ln() + 800.0 - 800.0_f64.ln()).exp() - 1e-308 / 800.0;
        assert!((recovered - expected).abs() / expected < 1e-12);
        assert!(recovered.is_finite());
        assert!(recovered > 0.0);
        let negative =
            recover_discrete_constant_predictor_effect(-1e-308, 800.0, 1.0, LagClock::EventTime)
                .expect("eq 12 signed log-space");
        assert!((negative + expected).abs() / expected < 1e-12);
        assert_eq!(
            recover_discrete_constant_predictor_effect(0.0, 800.0, 1.0, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(0.0, 1e308, 2.0, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(1.0, 800.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_constant_predictor_effect(1.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        // a_yx/a_xx overflows; the Eq. 12 rewrite term is not a binary64 number.
        assert!(!800.0_f64.exp_m1().is_finite());
        assert!(!(1e308_f64 / 1e-10).is_finite());
        assert_eq!(
            recover_discrete_constant_predictor_effect(1e308, 1e-10, 8e12, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn time_varying_predictor_discrete_effect_recovers_equation_fourteen() {
        let outcome_on_predictor = 0.2_f64;
        let delta = 2.0_f64;
        let recovered = recover_discrete_time_varying_predictor_effect(
            outcome_on_predictor,
            delta,
            delta,
            delta,
            LagClock::EventTime,
        )
        .expect("eq 14");
        assert!((recovered - outcome_on_predictor * delta).abs() < 1e-15);
        let constant = recover_discrete_constant_predictor_effect(
            outcome_on_predictor,
            -0.5,
            delta,
            LagClock::EventTime,
        )
        .expect("eq 12");
        // Voelkle 2012, p. 21: Eq. 14 is not Eq. 12.
        assert!((recovered - constant).abs() > 1e-3);
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                0.0,
                delta,
                delta,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn time_varying_predictor_unmatched_and_invalid_inputs_fail_closed() {
        let outcome_on_predictor = 0.2_f64;
        let delta = 2.0_f64;
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                delta,
                delta,
                delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                0.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                -1.0,
                1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                1.0,
                f64::NAN,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                1.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                1.0,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::UnmatchedTimeVaryingInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                outcome_on_predictor,
                2.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::UnmatchedTimeVaryingInterval)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                f64::NAN,
                delta,
                delta,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_time_varying_predictor_effect(
                1e308,
                10.0,
                10.0,
                10.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            refuse_unmatched_time_varying_predictor_interval(1.0, 2.0),
            Err(PsychometricError::UnmatchedTimeVaryingInterval)
        );
        assert_eq!(
            refuse_unmatched_time_varying_predictor_interval(1.0, 1.0),
            Err(PsychometricError::UnmatchedTimeVaryingInterval)
        );
    }

    #[test]
    fn discrete_process_noise_recovers_driver_equation_three() {
        let diffusion = 0.4_f64;
        let drift = -0.5_f64;
        let delta = 1.0_f64;
        let recovered =
            recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime)
                .expect("q_dt");
        let expected = diffusion * ((2.0 * drift * delta).exp() - 1.0) / (2.0 * drift);
        assert!((recovered - expected).abs() < 1e-15);
        // a = 0 is the integral of a constant diffusion: q Δt.
        assert_eq!(
            recover_discrete_process_noise(diffusion, 0.0, 2.5, LagClock::EventTime),
            Ok(diffusion * 2.5)
        );
        // Binary64 underflow of 2 a Δt recovers the same limit.
        let underflowed = recover_discrete_process_noise(1.0, 1e-308, 1e-308, LagClock::EventTime)
            .expect("z underflow");
        assert!((underflowed - 1e-308).abs() < 1e-320);
        // z → −∞ keeps the equilibrium variance −q / (2 a).
        let equilibrium =
            recover_discrete_process_noise(0.4, -1e300, 2.0, LagClock::EventTime).expect("eq var");
        assert!((equilibrium - (0.4 / (2.0 * 1e300))).abs() < 1e-315);
        // Finite z, overflowed expm1: log-space rewrite stays finite.
        let overflowed = recover_discrete_process_noise(1e-308, 400.0, 1.0, LagClock::EventTime)
            .expect("expm1 overflow");
        let rewrite_scale = 1e-308 / 800.0;
        let rewrite_log = (1e-308_f64).ln() + 800.0 - 800.0_f64.ln();
        let rewrite = rewrite_log.exp() - rewrite_scale;
        assert!((overflowed - rewrite).abs() / rewrite.abs() < 1e-12);
        assert_eq!(
            recover_discrete_process_noise(0.0, 800.0, 1.0, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            recover_discrete_process_noise(0.0, 1e308, 2.0, LagClock::EventTime),
            Ok(0.0)
        );
        // Forming 2 a first overflows; z = 2 (a Δt) stays finite.
        let twice_rate_overflow =
            recover_discrete_process_noise(1.0, 1e308, 1e-308, LagClock::EventTime)
                .expect("2a overflow");
        let expected_twice_rate = 0.5 * 2.0_f64.exp_m1() / 1e308;
        assert!((twice_rate_overflow - expected_twice_rate).abs() / expected_twice_rate < 1e-12);
        // 2 a overflows to −∞; expm1(−∞) = −1 keeps −0.5 q / a.
        let overflowed_equilibrium =
            recover_discrete_process_noise(1e308, -1e308, 2.0, LagClock::EventTime)
                .expect("2a eq var");
        assert!((overflowed_equilibrium - 0.5).abs() < 1e-15);
    }

    #[test]
    fn discrete_process_noise_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_process_noise(0.4, -0.5, 1.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_process_noise(0.4, -0.5, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_process_noise(0.4, -0.5, -1.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_process_noise(0.4, -0.5, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_process_noise(-0.1, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_process_noise(f64::NAN, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_process_noise(0.4, f64::NAN, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_process_noise(1.0, 800.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_process_noise(1.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        // Finite z, overflowed expm1, overflowing 0.5 q / a.
        // q (e^{2 a Δt} − 1) / (2 a) is then non-finite (Driver Eq. 3).
        assert_eq!(
            recover_discrete_process_noise(1e308, 0.1, 4000.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn lagged_covariance_and_latent_variance_follow_driver_equations_three_and_four() {
        let prior = 2.0_f64;
        let diffusion = 0.4_f64;
        let drift = -0.5_f64;
        let delta = 1.0_f64;
        let lagged =
            recover_discrete_lagged_latent_covariance(prior, drift, delta, LagClock::EventTime)
                .expect("lagged cov");
        let expected_lagged = (drift * delta).exp() * prior;
        assert!((lagged - expected_lagged).abs() < 1e-15);
        let process_noise =
            recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime)
                .expect("q_dt");
        let latent =
            recover_discrete_latent_variance(prior, diffusion, drift, delta, LagClock::EventTime)
                .expect("var");
        let expected_var = (2.0 * drift * delta).exp() * prior + process_noise;
        assert!((latent - expected_var).abs() < 1e-15);
        assert!((latent - process_noise).abs() > 1e-3);
        assert_eq!(
            refuse_process_noise_as_unconditional_variance(process_noise, prior),
            Err(PsychometricError::ProcessNoiseIsConditionalVariance)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(0.0, 800.0, 1.0, LagClock::EventTime),
            Ok(0.0)
        );
        let underflowed_lagged =
            recover_discrete_lagged_latent_covariance(2.0, -1e308, 2.0, LagClock::EventTime)
                .expect("underflow lagged");
        assert_eq!(underflowed_lagged.to_bits(), 0.0_f64.to_bits());
        let rewritten =
            recover_discrete_lagged_latent_covariance(1e-308, 800.0, 1.0, LagClock::EventTime)
                .expect("rewrite lagged");
        let expected_rewrite = (1e-308_f64.ln() + 800.0).exp();
        assert!((rewritten - expected_rewrite).abs() / expected_rewrite < 1e-12);
        let zero_prior =
            recover_discrete_latent_variance(0.0, diffusion, drift, delta, LagClock::EventTime)
                .expect("zero prior");
        assert!((zero_prior - process_noise).abs() < 1e-15);
        let drifted_zero =
            recover_discrete_latent_variance(2.0, diffusion, 0.0, 2.5, LagClock::EventTime)
                .expect("a=0");
        assert!((drifted_zero - (2.0 + diffusion * 2.5)).abs() < 1e-15);
        let underflowed_var =
            recover_discrete_latent_variance(2.0, 1.0, 1e-308, 1e-308, LagClock::EventTime)
                .expect("z underflow");
        assert!((underflowed_var - (2.0 + 1.0 * 1e-308)).abs() < 1e-15);
        let vanished =
            recover_discrete_latent_variance(2.0, 1e308, -1e308, 2.0, LagClock::EventTime)
                .expect("phi_sq underflow");
        assert!((vanished - 0.5).abs() < 1e-15);
        let rewritten_var =
            recover_discrete_latent_variance(1e-308, 1e-308, 400.0, 1.0, LagClock::EventTime)
                .expect("rewrite var");
        assert!(rewritten_var.is_finite());
        assert!(rewritten_var > 0.0);
    }

    #[test]
    fn lagged_covariance_and_latent_variance_overflow_paths_fail_closed() {
        assert_eq!(
            recover_discrete_lagged_latent_covariance(1e308, 800.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(1e308, 1e-308, 400.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(2.0, 1.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        // Zero diffusion is exactly Q_Δt = 0 (Driver Eq. 3). That skip
        // does not license exp(2 a Δt) p when 2 (a Δt) overflows to +∞.
        assert_eq!(
            recover_discrete_latent_variance(2.0, 0.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(1e308, 700.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(1e308, 1e-308, 350.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(1e308, 1e308, 0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let carried = (-90.622_f64).exp();
        let diffusion_sum = (-83.938_f64).exp();
        assert_eq!(
            recover_discrete_latent_variance(
                carried,
                diffusion_sum,
                400.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(-0.1, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(f64::NAN, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, f64::NAN, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, -0.5, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, -0.5, -1.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, -0.5, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_lagged_latent_covariance(2.0, -0.5, 1.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_variance(-0.1, 0.4, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(f64::NAN, 0.4, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_variance(2.0, 0.4, -0.5, 1.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
    }

    #[test]
    fn stationary_variance_recovers_driver_equation_four_asymptote() {
        let diffusion = 0.4_f64;
        let drift = -0.5_f64;
        let recovered = recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime)
            .expect("asym");
        let expected = (diffusion / drift) * -0.5;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - 0.4).abs() < 1e-15);
        // Starting from p_∞, Var(η_t) is invariant across finite Δt.
        for delta in [0.5_f64, 1.0, 2.0, 10.0] {
            let evolved = recover_discrete_latent_variance(
                recovered,
                diffusion,
                drift,
                delta,
                LagClock::EventTime,
            )
            .expect("invariant");
            assert!(
                (evolved - recovered).abs() < 1e-12,
                "stationary variance must be invariant at Δt={delta}"
            );
        }
        let finite_noise =
            recover_discrete_process_noise(diffusion, drift, 1.0, LagClock::EventTime)
                .expect("finite q_dt");
        assert!((finite_noise - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_finite_interval_process_noise_as_stationary_variance(finite_noise, 1.0),
            Err(PsychometricError::FiniteIntervalProcessNoiseIsNotStationary)
        );
        assert_eq!(
            refuse_finite_interval_process_noise_as_stationary_variance(recovered, 1.0),
            Err(PsychometricError::FiniteIntervalProcessNoiseIsNotStationary)
        );
        assert_eq!(
            recover_stationary_latent_variance(0.0, drift, LagClock::EventTime),
            Ok(0.0)
        );
        // Do not form 2 a first: 2*(-1e308) overflows; (q/a)*-0.5 is 0.5.
        let twice_rate_overflow =
            recover_stationary_latent_variance(1e308, -1e308, LagClock::EventTime)
                .expect("2a overflow");
        assert!((twice_rate_overflow - 0.5).abs() < 1e-15);
        assert!(!(2.0 * -1e308_f64).is_finite());
        let lost = -1e308_f64 / (2.0 * -1e308_f64);
        assert!(lost.abs() < 1e-15);
        // Do not form 0.5 q first: 0.5 * from_bits(1) underflows.
        let min_subnormal = f64::from_bits(1);
        assert!((0.5 * min_subnormal).abs() < 1e-300);
        assert!((-0.5 * min_subnormal / -min_subnormal).abs() < 1e-300);
        let subnormal_ratio =
            recover_stationary_latent_variance(min_subnormal, -min_subnormal, LagClock::EventTime)
                .expect("subnormal ratio");
        assert!((subnormal_ratio - 0.5).abs() < 1e-15);
        assert!(((min_subnormal / -min_subnormal) * -0.5 - 0.5).abs() < 1e-15);
        // Do not form q/a first: MAX/-0.75 overflows; MAX/(2*0.75) is finite.
        assert!(!(f64::MAX / -0.75_f64).is_finite());
        assert!(!((f64::MAX / -0.75_f64) * -0.5).is_finite());
        let twice = -0.75_f64 * 2.0;
        assert!(twice.is_finite());
        let expected_max = f64::MAX / -twice;
        assert!(expected_max.is_finite());
        assert_eq!(expected_max.to_bits(), (f64::MAX / 1.5).to_bits());
        let quotient_overflow =
            recover_stationary_latent_variance(f64::MAX, -0.75, LagClock::EventTime)
                .expect("q/a overflow");
        assert_eq!(quotient_overflow.to_bits(), expected_max.to_bits());
    }

    #[test]
    fn stationary_variance_unstable_and_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_latent_variance(0.4, -0.5, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_latent_variance(0.4, 0.0, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_latent_variance(0.4, 0.5, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_latent_variance(0.0, 0.0, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_latent_variance(-0.1, -0.5, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_latent_variance(f64::NAN, -0.5, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_latent_variance(0.4, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        // The Lyapunov solution overflows when |q| >> |a|.
        assert!(!((1e308_f64 / -1e-10_f64) * -0.5).is_finite());
        assert!(!(1e308_f64 / (2.0 * 1e-10_f64)).is_finite());
        assert_eq!(
            recover_stationary_latent_variance(1e308, -1e-10, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn trait_plus_state_recovers_driver_section_four_point_three() {
        let trait_variance = 1.5_f64;
        let diffusion = 0.4_f64;
        let drift = -0.5_f64;
        let delta = 1.0_f64;
        let state = recover_stationary_latent_variance(diffusion, drift, LagClock::EventTime)
            .expect("state");
        let total = recover_trait_plus_state_latent_variance(trait_variance, state).expect("sum");
        assert!((total - (trait_variance + state)).abs() < 1e-15);
        let lagged = recover_trait_plus_state_lagged_covariance(
            trait_variance,
            state,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("lagged");
        let state_lagged =
            recover_discrete_lagged_latent_covariance(state, drift, delta, LagClock::EventTime)
                .expect("state lagged");
        assert!((lagged - (trait_variance + state_lagged)).abs() < 1e-15);
        // Evolving the summed variance as if it were all state is not
        // the trait-plus-state map (Driver §4.3; Hamaker et al., 2015).
        let evolved_as_state =
            recover_discrete_latent_variance(total, diffusion, drift, delta, LagClock::EventTime)
                .expect("wrong");
        let evolved_state =
            recover_discrete_latent_variance(state, diffusion, drift, delta, LagClock::EventTime)
                .expect("state evolved");
        let evolved_right =
            recover_trait_plus_state_latent_variance(trait_variance, evolved_state).expect("right");
        assert!((evolved_right - total).abs() < 1e-12);
        assert!((evolved_as_state - evolved_right).abs() > 1e-3);
        assert_eq!(
            recover_trait_plus_state_latent_variance(0.0, state),
            Ok(state)
        );
        assert_eq!(
            recover_trait_plus_state_latent_variance(trait_variance, 0.0),
            Ok(trait_variance)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(
                0.0,
                state,
                drift,
                delta,
                LagClock::EventTime
            ),
            Ok(state_lagged)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(
                trait_variance,
                0.0,
                drift,
                delta,
                LagClock::EventTime
            ),
            Ok(trait_variance)
        );
        let process_noise =
            recover_discrete_process_noise(diffusion, drift, delta, LagClock::EventTime)
                .expect("q_dt");
        assert_eq!(
            refuse_trait_variance_as_process_noise(trait_variance, process_noise),
            Err(PsychometricError::TraitVarianceIsNotProcessNoise)
        );
        assert_eq!(
            refuse_trait_variance_as_stationary_within_subject(trait_variance, state),
            Err(PsychometricError::TraitVarianceIsNotStationaryWithinSubject)
        );
    }

    #[test]
    fn trait_plus_state_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_trait_plus_state_latent_variance(-0.1, 0.4),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_latent_variance(0.4, -0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_latent_variance(f64::NAN, 0.4),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_latent_variance(0.4, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_latent_variance(1e308, 1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(-0.1, 0.4, -0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(
                f64::NAN,
                0.4,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(0.4, 0.4, -0.5, 1.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_trait_plus_state_lagged_covariance(1e308, 1e308, 0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn non_event_clocks_and_difference_quotient_fail_closed() {
        for clock in [
            LagClock::SystemTime,
            LagClock::AssertionTime,
            LagClock::DocumentTime,
            LagClock::AvailabilityTime,
            LagClock::KnowledgeCutoff,
        ] {
            assert_eq!(
                recover_local_log_rate(0.5, 1.0, clock),
                Err(PsychometricError::EventTimeRequired)
            );
            assert!(!clock.admits_structural_lag());
            assert!(!clock.as_str().is_empty());
        }
        assert!(LagClock::EventTime.admits_structural_lag());
        assert_eq!(LagClock::EventTime.as_str(), "event_time");
        assert_eq!(
            refuse_difference_quotient_as_local_rate(1.0, 0.5, 1.0),
            Err(PsychometricError::DifferenceQuotientForbidden)
        );
    }

    #[test]
    fn invalid_lag_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_lag_one(0.0, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_one(f64::NAN, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_lag_one(1.0, f64::INFINITY),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(0.5, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.5, -1.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.5, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_local_log_rate(0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(-0.2, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_local_log_rate(f64::NAN, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn series_mean_log_rate_recovers_and_refuses() {
        let drift = -0.25_f64;
        let occasions = [
            EventOccasion {
                event_time: 0.0,
                score: 2.0,
            },
            EventOccasion {
                event_time: 1.0,
                score: 2.0 * drift.exp(),
            },
            EventOccasion {
                event_time: 3.0,
                score: 2.0 * (drift * 3.0).exp(),
            },
        ];
        let series =
            recover_event_series_mean_log_rate(&occasions, LagClock::EventTime).expect("series");
        assert!((series - drift).abs() < 1e-12);
        assert_eq!(
            recover_event_series_mean_log_rate(&occasions, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(&[], LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[EventOccasion {
                    event_time: 0.0,
                    score: 1.0,
                }],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[
                    EventOccasion {
                        event_time: f64::NAN,
                        score: 1.0,
                    },
                    EventOccasion {
                        event_time: 1.0,
                        score: 0.5,
                    },
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[occasion(0.0, 1.0), occasion(f64::NAN, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[occasion(0.0, f64::NAN), occasion(1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[occasion(0.0, 1.0), occasion(1.0, f64::NAN)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[
                    EventOccasion {
                        event_time: 0.0,
                        score: 1.0,
                    },
                    EventOccasion {
                        event_time: 0.0,
                        score: 0.5,
                    },
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    fn clustered(cluster_key: u64, event_time: f64, score: f64) -> ClusteredEventScore {
        ClusteredEventScore {
            cluster_key,
            event_time,
            score,
        }
    }

    fn occasion(event_time: f64, score: f64) -> EventOccasion {
        EventOccasion { event_time, score }
    }

    fn decaying_clustered_scores(drift: f64) -> [ClusteredEventScore; 12] {
        [
            clustered(1, 0.0, 10.0 + 1.0),
            clustered(1, 1.0, 10.0 + drift.exp()),
            clustered(1, 2.0, 10.0 + (drift * 2.0).exp()),
            clustered(1, 3.0, 10.0 + (drift * 3.0).exp()),
            clustered(1, 4.0, 10.0 + (drift * 4.0).exp()),
            clustered(1, 5.0, 10.0 + (drift * 5.0).exp()),
            clustered(2, 0.0, -6.0 + 1.2),
            clustered(2, 1.0, -6.0 + 1.2 * drift.exp()),
            clustered(2, 2.0, -6.0 + 1.2 * (drift * 2.0).exp()),
            clustered(2, 3.0, -6.0 + 1.2 * (drift * 3.0).exp()),
            clustered(2, 4.0, -6.0 + 1.2 * (drift * 4.0).exp()),
            clustered(2, 5.0, -6.0 + 1.2 * (drift * 5.0).exp()),
        ]
    }

    #[test]
    fn within_residual_paths_recover_and_refuse() {
        let drift = -0.25_f64;
        let clustered = decaying_clustered_scores(drift);
        let within = recover_within_residual_event_time_log_rate(&clustered, LagClock::EventTime)
            .expect("cwc lag");
        let within_error = (within - drift).abs();
        assert!(within_error.is_finite());
    }

    #[test]
    fn within_residual_invalid_rows_fail_closed() {
        let rows = decaying_clustered_scores(-0.25);
        assert_eq!(
            recover_within_residual_event_time_log_rate(&rows, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(&[], LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, 0.0, 1.0), clustered(1, 1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InsufficientClusters)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, f64::NAN, 1.0), clustered(2, 1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_event_series_mean_log_rate(
                &[occasion(0.0, 1.0), occasion(1.0, f64::NAN)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, 1.0),
                    clustered(1, 1.0, 0.5),
                    clustered(2, 0.0, 2.0),
                    clustered(2, 1.0, 1.0),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, 0.0, 1.0), clustered(2, 1.0, f64::INFINITY)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, 1.0),
                    clustered(1, 0.0, 1.2),
                    clustered(2, 0.0, 2.0),
                    clustered(2, 1.0, 1.5),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    fn lagged(
        earlier_residual: f64,
        later_residual: f64,
        event_delta: f64,
    ) -> LaggedWithinResidual {
        LaggedWithinResidual {
            earlier_residual,
            later_residual,
            event_delta,
        }
    }

    #[test]
    fn irregular_centered_residuals_recover_exact_drift() {
        let drift = -0.4_f64;
        let pairs = [
            lagged(1.2, 1.2 * (drift * 0.5).exp(), 0.5),
            lagged(0.8, 0.8 * (drift * 1.75).exp(), 1.75),
            lagged(-1.1, -1.1 * (drift * 2.25).exp(), 2.25),
        ];
        let recovered = recover_irregular_centered_residual_log_rate(&pairs, LagClock::EventTime)
            .expect("irregular");
        assert!((recovered - drift).abs() < 1e-12);
    }

    #[test]
    fn irregular_centered_residuals_fail_closed() {
        let ok = lagged(1.0, 0.8, 1.0);
        assert_eq!(
            recover_irregular_centered_residual_log_rate(&[ok], LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(&[], LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(f64::NAN, 0.8, 1.0)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(1.0, f64::INFINITY, 1.0)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(1.0, 0.8, f64::NAN)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(0.0, 0.8, 1.0)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(1.0, -0.8, 1.0)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(1.0, 0.8, 0.0)],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_irregular_centered_residual_log_rate(
                &[lagged(1.0, 0.8, -0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    #[test]
    fn singleton_cluster_is_skipped_and_all_singletons_fail_closed() {
        let drift = -0.2_f64;
        let mixed = [
            clustered(1, 0.0, 10.0 + 1.0),
            clustered(1, 1.0, 10.0 + drift.exp()),
            clustered(1, 2.0, 10.0 + (drift * 2.0).exp()),
            clustered(1, 3.0, 10.0 + (drift * 3.0).exp()),
            clustered(2, 0.0, 4.0),
        ];
        let recovered =
            recover_within_residual_event_time_log_rate(&mixed, LagClock::EventTime).expect("skip");
        assert!(recovered.is_finite());
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[clustered(1, 0.0, 1.0), clustered(2, 1.0, 0.5)],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn overflowing_cwc_residuals_fail_closed() {
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, f64::MAX),
                    clustered(1, 1.0, f64::MAX),
                    clustered(2, 0.0, 1.0),
                    clustered(2, 1.0, 0.5),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn newton_overflow_and_flat_derivative_fail_closed() {
        assert_eq!(
            fit_scalar_log_rate(&[(1e-300, 1.0, 1e-8), (1.0, 1.0, 1.0)]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            fit_scalar_log_rate(&[(1e200, 1e200, 1.0)]),
            Err(PsychometricError::InvalidNumericInput)
        );
        let flat = fit_scalar_log_rate(&[(1e-50, 1e-200, 1.0)]).expect("flat");
        assert!(flat.is_finite());
        assert_eq!(
            fit_scalar_log_rate(&[(0.0, 1.0, 1.0), (1.0, -1.0, 1.0)]),
            Err(PsychometricError::InvalidNumericInput)
        );
        let skipped_start =
            fit_scalar_log_rate(&[(1e-320, 1.0, 1.0), (1.0, 0.5, 1.0)]).expect("skip inf ratio");
        assert!(skipped_start.is_finite());
        assert_eq!(
            fit_scalar_log_rate(&[(1e154, 1e154, 1.0)]),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            fit_scalar_log_rate(&[(1.0, 1e-300, 1.0), (1.0, 1e-300, 2.0)]),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn one_sided_residual_overflow_and_nonfinite_interval_fail_closed() {
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, 0.0, -f64::MAX),
                    clustered(1, 1.0, -f64::MAX),
                    clustered(1, 2.0, -f64::MAX),
                    clustered(1, 3.0, f64::MAX),
                    clustered(2, 0.0, 1.0),
                    clustered(2, 1.0, 0.8),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_within_residual_event_time_log_rate(
                &[
                    clustered(1, f64::MAX, 1.0),
                    clustered(1, -f64::MAX, 0.5),
                    clustered(2, 0.0, 1.0),
                    clustered(2, 1.0, 0.5),
                ],
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    #[test]
    fn manifest_observed_variance_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let latent = 0.4_f64;
        let measurement_error = 0.1_f64;
        let recovered =
            recover_manifest_observed_variance(loading, latent, measurement_error).expect("eq5");
        let expected = (loading * latent) * loading + measurement_error;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - 1.7).abs() < 1e-15);
        assert!((measurement_error - recovered).abs() > 1e-3);
        assert!((latent - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_measurement_error_as_observed_variance(measurement_error, recovered),
            Err(PsychometricError::MeasurementErrorIsNotObservedVariance)
        );
        assert_eq!(
            refuse_latent_variance_as_observed_variance(latent, recovered),
            Err(PsychometricError::LatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            recover_manifest_observed_variance(0.0, latent, measurement_error),
            Ok(measurement_error)
        );
        assert_eq!(
            recover_manifest_observed_variance(loading, 0.0, measurement_error),
            Ok(measurement_error)
        );
        assert_eq!(
            recover_manifest_observed_variance(loading, latent, 0.0),
            Ok(1.6)
        );
        // Do not form λ² first: (1e308)² overflows; (λ p) λ is 1e308.
        let scaled = recover_manifest_observed_variance(1e308, 1e-308, 0.0).expect("scale");
        assert!((scaled - 1e308).abs() / 1e308 < 1e-15);
        assert!(!(1e308_f64 * 1e308_f64).is_finite());
    }

    #[test]
    fn manifest_trait_plus_state_observed_variance_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let latent = 0.4_f64;
        let measurement_error = 0.1_f64;
        let manifest_trait = 0.5_f64;
        let recovered = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("eq5-trait");
        let expected = (loading * latent) * loading + measurement_error + manifest_trait;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - 2.2).abs() < 1e-15);
        let without_trait =
            recover_manifest_observed_variance(loading, latent, measurement_error).expect("psi0");
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(
                loading,
                latent,
                measurement_error,
                0.0
            ),
            Ok(without_trait)
        );
        assert!((without_trait - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_manifest_trait_variance_as_measurement_error(manifest_trait, measurement_error),
            Err(PsychometricError::ManifestTraitVarianceIsNotMeasurementError)
        );
        // Zero loading: Var(y) = θ + ψ, not ψ stuffed as Θ.
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(
                0.0,
                latent,
                measurement_error,
                manifest_trait
            ),
            Ok(measurement_error + manifest_trait)
        );
        // TRAITVAR is latent and scaled by λ²; MANIFESTTRAITVAR is not.
        let latent_trait_as_state =
            recover_manifest_observed_variance(loading, latent + manifest_trait, measurement_error)
                .expect("traitvar");
        assert!((latent_trait_as_state - recovered).abs() > 1e-3);
        // Do not form λ² first, then add ψ.
        let scaled = recover_manifest_trait_plus_state_observed_variance(1e308, 1e-308, 0.0, 1.0)
            .expect("scale-psi");
        assert!((scaled - 1e308).abs() / 1e308 < 1e-15);
    }

    #[test]
    fn manifest_trait_plus_state_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(2.0, 0.4, 0.1, -0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(2.0, 0.4, 0.1, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(1e308, 1.0, 0.0, 0.3),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_trait_plus_state_observed_variance(1e308, 1e-308, 1e308, 1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn manifest_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_manifest_observed_variance(f64::NAN, 0.4, 0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(2.0, -0.1, 0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(2.0, 0.4, -0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(2.0, f64::NAN, 0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(2.0, 0.4, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(1e308, 1.0, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_variance(1e308, 1.0, 1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn manifest_lagged_observed_covariance_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let lagged = 0.4_f64;
        let manifest_trait = 0.5_f64;
        let recovered =
            recover_manifest_lagged_observed_covariance(loading, lagged, manifest_trait)
                .expect("eq5-lag");
        let expected = (loading * lagged) * loading + manifest_trait;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - 2.1).abs() < 1e-15);
        assert_eq!(
            recover_manifest_lagged_observed_covariance(loading, lagged, 0.0),
            Ok(1.6)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(0.0, lagged, manifest_trait),
            Ok(manifest_trait)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(loading, 0.0, manifest_trait),
            Ok(manifest_trait)
        );
        assert_eq!(
            refuse_latent_lagged_covariance_as_observed_covariance(lagged, recovered),
            Err(PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance)
        );
        assert_eq!(
            refuse_measurement_error_as_lagged_observed_covariance(0.1, recovered),
            Err(PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance)
        );
        let scaled =
            recover_manifest_lagged_observed_covariance(1e308, 1e-308, 0.0).expect("scale");
        assert!((scaled - 1e308).abs() / 1e308 < 1e-15);
        assert!(!(1e308_f64 * 1e308_f64).is_finite());
    }

    #[test]
    fn manifest_lagged_observed_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_manifest_lagged_observed_covariance(f64::NAN, 0.4, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(2.0, -0.1, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(2.0, 0.4, -0.1),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(1e308, 1.0, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(1e308, 1e-308, 1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn manifest_observed_mean_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let latent_mean = 0.4_f64;
        let manifest_mean = 0.5_f64;
        let recovered =
            recover_manifest_observed_mean(loading, latent_mean, manifest_mean).expect("eq5-mean");
        let expected = loading * latent_mean + manifest_mean;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - 1.3).abs() < 1e-15);
        assert_eq!(
            recover_manifest_observed_mean(loading, latent_mean, 0.0),
            Ok(0.8)
        );
        assert_eq!(
            recover_manifest_observed_mean(0.0, latent_mean, manifest_mean),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_manifest_observed_mean(loading, 0.0, manifest_mean),
            Ok(manifest_mean)
        );
        assert_eq!(recover_manifest_observed_mean(-2.0, 0.5, 1.0), Ok(0.0));
        assert_eq!(
            refuse_manifest_means_as_observed_mean(manifest_mean, recovered),
            Err(PsychometricError::ManifestMeansIsNotObservedMean)
        );
        assert_eq!(
            refuse_latent_mean_as_observed_mean(latent_mean, recovered),
            Err(PsychometricError::LatentMeanIsNotObservedMean)
        );
        assert_eq!(
            refuse_continuous_intercept_as_manifest_means(0.3, manifest_mean),
            Err(PsychometricError::ContinuousInterceptIsNotManifestMeans)
        );
        let scaled = recover_manifest_observed_mean(1e308, 1e-308, 0.0).expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded = recover_manifest_observed_mean(1e308, 1.0, 0.0).expect("lambda-mu");
        assert!((finite_loaded - 1e308).abs() / 1e308 < 1e-15);
        assert!(!(1e308_f64 * 1e308_f64).is_finite());
    }

    #[test]
    fn manifest_observed_mean_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_manifest_observed_mean(f64::NAN, 0.4, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_mean(2.0, f64::NAN, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_mean(2.0, 0.4, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_mean(1e308, 2.0, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_observed_mean(1.0, 1e308, 1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(recover_manifest_observed_mean(0.0, 1e308, 0.5), Ok(0.5));
        assert_eq!(recover_manifest_observed_mean(1e308, 0.0, 0.5), Ok(0.5));
    }

    #[test]
    fn discrete_latent_mean_recovers_driver_equation_three() {
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let recovered =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("eq3-mean");
        let expected =
            (drift * delta).exp() * initial + intercept * ((drift * delta).exp_m1() / drift);
        assert!((recovered - expected).abs() < 1e-15);
        let increment = recover_discrete_continuous_intercept_effect(
            intercept,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("cint");
        assert!((increment - intercept * ((drift * delta).exp_m1() / drift)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean(0.0, drift, intercept, delta, LagClock::EventTime),
            Ok(increment)
        );
        assert_eq!(
            recover_discrete_latent_mean(initial, drift, 0.0, delta, LagClock::EventTime),
            Ok((drift * delta).exp() * initial)
        );
        assert_eq!(
            recover_discrete_latent_mean(initial, 0.0, intercept, delta, LagClock::EventTime),
            Ok(initial + intercept * delta)
        );
        assert_eq!(
            recover_discrete_continuous_intercept_effect(
                intercept,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Ok(intercept * delta)
        );
        assert_eq!(
            recover_discrete_continuous_intercept_effect(0.0, 0.0, delta, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            refuse_initial_latent_mean_as_evolved_mean(initial, recovered),
            Err(PsychometricError::InitialLatentMeanIsNotEvolvedMean)
        );
        assert_eq!(
            refuse_continuous_intercept_as_discrete_mean_increment(intercept, increment),
            Err(PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement)
        );
        assert_eq!(
            refuse_continuous_intercept_as_initial_latent_mean(intercept, initial),
            Err(PsychometricError::ContinuousInterceptIsNotInitialLatentMean)
        );
        let equilibrium =
            recover_discrete_latent_mean(initial, -1e308, 1.0, 2.0, LagClock::EventTime)
                .expect("eq3-equilibrium");
        let equilibrium_expected = -(1.0 / -1e308);
        assert!((equilibrium - equilibrium_expected).abs() / 1e-308 < 1e-12);
        assert_eq!(
            recover_discrete_latent_mean(1e308, 1.0, 0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(0.0, 1e308, 0.0, 2.0, LagClock::EventTime),
            Ok(0.0)
        );
        // CINT = 0 so the increment path stays finite; exp(a Δt) then
        // overflows and the carried T0MEANS term fails closed.
        assert_eq!(
            recover_discrete_latent_mean(1.0, 710.0, 0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert!(!(710.0_f64.exp()).is_finite());
    }

    #[test]
    fn discrete_latent_mean_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_latent_mean(f64::NAN, -0.5, 0.3, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1.0, f64::NAN, 0.3, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1.0, -0.5, f64::NAN, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1.0, -0.5, 0.3, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_latent_mean(1.0, -0.5, 0.3, 2.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_continuous_intercept_effect(1.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1.0, 1e308, 1.0, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1e308, 0.0, 1e308, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean(1e308, 0.0, 1e308, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let underflow_argument = 1e-308_f64 * 1e-308_f64;
        assert_eq!(underflow_argument.to_bits(), 0.0_f64.to_bits());
        let underflow = recover_discrete_latent_mean(2.0, 1e-308, 4.0, 1e-308, LagClock::EventTime)
            .expect("a-delta-underflow");
        assert!((underflow - 2.0).abs() < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_recovers_driver_equations_three_and_five() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        let expected = manifest_mean + loading * evolved;
        assert!((recovered - expected).abs() < 1e-15);
        let first_occasion =
            recover_manifest_observed_mean(loading, initial, manifest_mean).expect("t0");
        assert!((first_occasion - recovered).abs() > 1e-3);
        assert_eq!(
            recover_discrete_observed_mean(
                0.0,
                initial,
                drift,
                intercept,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_discrete_observed_mean(
                loading,
                initial,
                drift,
                intercept,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Ok(loading * evolved)
        );
        let zero_evolved = recover_discrete_observed_mean(
            loading,
            0.0,
            0.0,
            0.0,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-mu");
        assert!((zero_evolved - manifest_mean).abs() < 1e-15);
        let integrator = recover_discrete_observed_mean(
            loading,
            initial,
            0.0,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("a0");
        assert!(
            (integrator - (manifest_mean + loading * (initial + intercept * delta))).abs() < 1e-15
        );
        let equilibrium = recover_discrete_observed_mean(
            loading,
            initial,
            -1e308,
            1.0,
            manifest_mean,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-equilibrium");
        let equilibrium_latent = -(1.0 / -1e308);
        assert!((equilibrium - (manifest_mean + loading * equilibrium_latent)).abs() < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_refuses_first_occasion_and_overflow() {
        let loading = 2.0_f64;
        let recovered =
            recover_discrete_observed_mean(loading, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("eq3-eq5-mean");
        let evolved =
            recover_discrete_latent_mean(1.0, -0.5, 0.3, 2.0, LagClock::EventTime).expect("mu-t");
        let first_occasion = recover_manifest_observed_mean(loading, 1.0, 0.5).expect("t0");
        assert_eq!(
            refuse_initial_observed_mean_as_evolved_observed_mean(first_occasion, recovered),
            Err(PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean)
        );
        assert_eq!(
            refuse_latent_mean_as_observed_mean(evolved, recovered),
            Err(PsychometricError::LatentMeanIsNotObservedMean)
        );
        assert_eq!(
            refuse_manifest_means_as_observed_mean(0.5, recovered),
            Err(PsychometricError::ManifestMeansIsNotObservedMean)
        );
        let scaled =
            recover_discrete_observed_mean(1e308, 1e-308, 0.0, 0.0, 0.0, 1.0, LagClock::EventTime)
                .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded =
            recover_discrete_observed_mean(1e308, 1.0, 0.0, 0.0, 0.0, 1.0, LagClock::EventTime)
                .expect("lambda-mu");
        assert!((finite_loaded - 1e308).abs() / 1e308 < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_observed_mean(f64::NAN, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean(2.0, 1.0, -0.5, 0.3, 0.5, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean(2.0, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_observed_mean(1e308, 2.0, 0.0, 0.0, 0.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean(1.0, 1.0, 710.0, 0.0, 0.5, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn time_dependent_impulse_recovers_driver_equation_three_fourth_summand() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        assert!((impulse - 1.2).abs() < 1e-15);
        assert_eq!(
            recover_time_dependent_predictor_impulse(0.0, predictor),
            Ok(0.0)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse(effect, 0.0),
            Ok(0.0)
        );
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let composed = recover_discrete_latent_mean_with_impulse(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-impulse");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        assert!((composed - (evolved + impulse)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean_with_impulse(
                initial,
                drift,
                intercept,
                0.0,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
        let intercept_effect =
            recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
                .expect("cint");
        assert!((impulse - intercept_effect).abs() > 1e-3);
        let equation_fourteen = recover_discrete_time_varying_predictor_effect(
            effect,
            delta,
            delta,
            delta,
            LagClock::EventTime,
        )
        .expect("eq14");
        assert!((impulse - equation_fourteen).abs() > 1e-3);
    }

    #[test]
    fn time_dependent_impulse_refuses_cint_tipred_and_equation_fourteen() {
        let effect = 0.4_f64;
        let predictor = 2.0_f64;
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let intercept_effect =
            recover_discrete_continuous_intercept_effect(effect, -0.5, 2.0, LagClock::EventTime)
                .expect("cint");
        let equation_fourteen = recover_discrete_time_varying_predictor_effect(
            effect,
            2.0,
            2.0,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq14");
        assert_eq!(
            refuse_time_dependent_impulse_as_continuous_intercept(impulse, effect),
            Err(PsychometricError::TimeDependentImpulseIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_time_dependent_impulse_as_time_independent_effect(impulse, intercept_effect),
            Err(PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect)
        );
        assert_eq!(
            refuse_time_dependent_impulse_as_time_varying_discrete_effect(
                impulse,
                equation_fourteen
            ),
            Err(PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect)
        );
    }

    #[test]
    fn time_dependent_impulse_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_time_dependent_predictor_impulse(f64::NAN, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse(1.0, f64::INFINITY),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse(1e308, 2.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_impulse(
                1.0,
                -0.5,
                0.3,
                0.4,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_impulse(
                1.0,
                -0.5,
                0.3,
                0.4,
                2.0,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_impulse(
                1e308,
                0.0,
                0.0,
                1e308,
                1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn level_change_continuous_intercept_recovers_driver_section_seven_point_two() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let intercept = recover_level_change_continuous_intercept(effect, predictor, drift)
            .expect("level-change");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
        assert!((intercept - 0.6).abs() < 1e-15);
        assert!((impulse - 1.2).abs() < 1e-15);
        let equilibrium = intercept / (-drift);
        assert!((equilibrium - impulse).abs() < 1e-15);
        assert_eq!(
            recover_level_change_continuous_intercept(0.0, predictor, drift),
            Ok(0.0)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(effect, 0.0, drift),
            Ok(0.0)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(effect, predictor, 0.0),
            Err(PsychometricError::LevelChangeRequiresStableDrift)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(effect, predictor, 0.5),
            Err(PsychometricError::LevelChangeRequiresStableDrift)
        );
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            2.0,
            LagClock::EventTime,
        )
        .expect("tipred");
        assert_eq!(
            refuse_level_change_intercept_as_impulse(intercept, impulse),
            Err(PsychometricError::LevelChangeInterceptIsNotImpulse)
        );
        assert_eq!(
            refuse_level_change_intercept_as_free_continuous_intercept(intercept, 0.3),
            Err(PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept)
        );
        assert_eq!(
            refuse_level_change_intercept_as_process_increment(intercept, increment),
            Err(PsychometricError::LevelChangeInterceptIsNotProcessIncrement)
        );
    }

    #[test]
    fn level_change_continuous_intercept_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_level_change_continuous_intercept(f64::NAN, 1.0, -0.5),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(1.0, f64::INFINITY, -0.5),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(0.4, 3.0, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(1e308, 2.0, -0.5),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_continuous_intercept(1.0, 2.0, -1e308),
            Err(PsychometricError::InvalidNumericInput)
        );
        let scaled = recover_level_change_continuous_intercept(1e-308, 1.0, -1.0).expect("scale");
        assert!((scaled - 1e-308).abs() < 1e-320);
        let rewritten =
            recover_level_change_continuous_intercept(1e-308, 1.0, -1e308).expect("rewrite");
        assert!((rewritten - 1.0).abs() < 1e-12);
    }

    #[test]
    fn level_change_discrete_increment_recovers_driver_equation_three_of_section_seven_point_two() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let increment = recover_level_change_discrete_increment(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("level-change-increment");
        let intercept = recover_level_change_continuous_intercept(effect, predictor, drift)
            .expect("level-change");
        let via_cint = recover_discrete_continuous_intercept_effect(
            intercept,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("cint-increment");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
        let expected = (1.0 - (drift * delta).exp()) * impulse;
        assert!((increment - expected).abs() < 1e-15);
        assert!((increment - via_cint).abs() < 1e-15);
        assert!((increment - impulse).abs() > 1e-3);
        assert!((increment - intercept).abs() > 1e-3);
        let tipred = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        assert_eq!(
            refuse_level_change_increment_as_impulse(increment, impulse),
            Err(PsychometricError::LevelChangeIncrementIsNotImpulse)
        );
        assert_eq!(
            refuse_level_change_increment_as_intercept(increment, intercept),
            Err(PsychometricError::LevelChangeIncrementIsNotIntercept)
        );
        assert_eq!(
            refuse_level_change_increment_as_process_increment(increment, tipred),
            Err(PsychometricError::LevelChangeIncrementIsNotProcessIncrement)
        );
        let equilibrated = recover_level_change_discrete_increment(
            effect,
            predictor,
            -800.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("underflow");
        assert!((equilibrated - impulse).abs() < 1e-15);
        assert_eq!(
            recover_level_change_discrete_increment(
                0.0,
                predictor,
                drift,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn level_change_discrete_increment_invalid_inputs_fail_closed() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        assert_eq!(
            recover_level_change_discrete_increment(
                effect,
                predictor,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::LevelChangeRequiresStableDrift)
        );
        assert_eq!(
            recover_level_change_discrete_increment(
                effect,
                predictor,
                0.5,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::LevelChangeRequiresStableDrift)
        );
        assert_eq!(
            recover_level_change_discrete_increment(
                effect,
                predictor,
                drift,
                delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_level_change_discrete_increment(
                effect,
                predictor,
                drift,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_discrete_increment(1e308, 2.0, drift, delta, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_discrete_increment(
                f64::NAN,
                predictor,
                drift,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_discrete_increment(
                0.0,
                predictor,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn extra_process_contribution_recovers_driver_section_seven_point_two() {
        let coupling = 0.569_907_f64;
        let predictor = 1.0_f64;
        let original = -0.1393_f64;
        let extra = -0.000_001_f64;
        let delta = 1.0_f64;
        let recovered = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            original,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("extra-process");
        let expected = coupling * predictor * ((extra * delta).exp() - (original * delta).exp())
            / (extra - original);
        assert!((recovered - expected).abs() < 1e-15);
        let equal_rate = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            extra,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("equal-rate");
        let equal_expected = coupling * predictor * delta * (extra * delta).exp();
        assert!((equal_rate - equal_expected).abs() < 1e-15);
        let brownian = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            0.0,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("brownian-original");
        let brownian_expected = coupling * predictor * (extra * delta).exp_m1() / extra;
        assert!((brownian - brownian_expected).abs() < 1e-15);
        assert_eq!(
            recover_level_change_extra_process_contribution(
                0.0,
                predictor,
                original,
                extra,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                0.0,
                original,
                extra,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                0.0,
                original,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn extra_process_contribution_is_not_cint_rewrite_or_impulse() {
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.05_f64;
        let delta = 2.0_f64;
        let recovered = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            original,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("extra-process");
        let intercept = recover_level_change_continuous_intercept(coupling, predictor, original)
            .expect("level-change");
        let increment = recover_level_change_discrete_increment(
            coupling,
            predictor,
            original,
            delta,
            LagClock::EventTime,
        )
        .expect("level-change-increment");
        let impulse =
            recover_time_dependent_predictor_impulse(coupling, predictor).expect("impulse");
        assert!((recovered - intercept).abs() > 1e-3);
        assert!((recovered - increment).abs() > 1e-3);
        assert!((recovered - impulse).abs() > 1e-3);
        assert_eq!(
            refuse_level_change_extra_process_as_impulse(recovered, impulse),
            Err(PsychometricError::LevelChangeExtraProcessIsNotImpulse)
        );
        assert_eq!(
            refuse_level_change_extra_process_as_intercept(recovered, intercept),
            Err(PsychometricError::LevelChangeExtraProcessIsNotIntercept)
        );
        assert_eq!(
            refuse_level_change_extra_process_as_increment(recovered, increment),
            Err(PsychometricError::LevelChangeExtraProcessIsNotIncrement)
        );
    }

    #[test]
    fn extra_process_contribution_invalid_inputs_fail_closed() {
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.000_001_f64;
        let delta = 2.0_f64;
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                predictor,
                original,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                predictor,
                original,
                0.5,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                predictor,
                original,
                extra,
                delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                predictor,
                original,
                extra,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                f64::NAN,
                predictor,
                original,
                extra,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                1e308,
                2.0,
                original,
                extra,
                delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(
                coupling,
                predictor,
                710.0,
                extra,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn nonfinite_short_circuit_operands_of_fail_closed_guards_execute() {
        let event = LagClock::EventTime;
        assert_eq!(
            recover_manifest_lagged_observed_covariance(2.0, f64::NAN, 0.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_manifest_lagged_observed_covariance(2.0, 0.4, f64::NAN),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_continuous_intercept_effect(0.3, -0.5, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(0.4, 3.0, -0.5, -1e-6, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(0.4, f64::NAN, -0.5, -1e-6, 2.0, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(0.4, 3.0, f64::NAN, -1e-6, 2.0, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution(0.4, 3.0, -0.5, f64::NAN, 2.0, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                0.4,
                3.0,
                -0.5,
                -0.05,
                2.0,
                f64::NAN,
                event
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(0.2, 1.0, -0.5, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(0.2, f64::NAN, -0.5, 2.0, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(0.2, f64::NAN, -0.5, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(0.2, 1.0, f64::NAN, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(f64::NAN, 1.0, -0.5, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(0.2, f64::NAN, -0.5, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(0.2, 1.0, f64::NAN, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(0.3, f64::NAN, event),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(0.4, 3.0, -0.5, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(0.4, 3.0, -0.5, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(0.4, 3.0, -0.5, f64::NAN, 1.0, event),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(0.4, 3.0, -0.5, 2.0, f64::NAN, event),
            Err(PsychometricError::NonPositiveInterval)
        );
    }

    #[test]
    fn extra_process_contribution_underflow_and_overflow_paths() {
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.000_001_f64;
        let vanished = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            -800.0,
            extra,
            1.0,
            LagClock::EventTime,
        )
        .expect("underflow");
        let vanished_expected = coupling * predictor * (extra * 1.0).exp() / (extra - -800.0);
        assert!((vanished - vanished_expected).abs() < 1e-15);
        let extra_underflow = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            original,
            -f64::from_bits(1),
            0.5,
            LagClock::EventTime,
        )
        .expect("extra-argument-underflow");
        assert!(extra_underflow.is_finite());
        let original_underflow = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            -2e-160_f64,
            -1e-160_f64,
            1e-200_f64,
            LagClock::EventTime,
        )
        .expect("gap-argument-underflow");
        let original_underflow_expected = coupling * predictor * 1e-200_f64;
        assert!((original_underflow - original_underflow_expected).abs() <= 1e-200_f64);
        let vanished_finite_increment = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            -800.0,
            -92.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("original-lag-underflow-finite-increment");
        let vanished_finite_expected = coupling * predictor * (-92.0_f64).exp() / (-92.0 - -800.0);
        assert!((vanished_finite_increment - vanished_finite_expected).abs() < 1e-15);
        let overflow_fallback = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            -0.8,
            extra,
            900.0,
            LagClock::EventTime,
        )
        .expect("expm1-overflow-fallback");
        let overflow_expected =
            coupling * predictor * ((extra * 900.0).exp() - (-0.8_f64 * 900.0).exp())
                / (extra - -0.8);
        assert!((overflow_fallback - overflow_expected).abs() < 1e-12);
    }

    #[test]
    fn extra_process_observed_mean_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.05_f64;
        let delta = 2.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_extra_process(
            loading,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-extra-process-mean");
        let composed = recover_discrete_latent_mean_with_extra_process(
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("extra-latent");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            original,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        assert!((impulse_observed - recovered).abs() > 1e-3);
        let contribution = recover_level_change_extra_process_contribution(
            coupling,
            predictor,
            original,
            extra,
            delta,
            LagClock::EventTime,
        )
        .expect("extra-process");
        assert_eq!(
            refuse_evolved_observed_mean_as_extra_process_observed_mean(
                evolved_observed,
                recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean)
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_extra_process_observed_mean(
                impulse_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean)
        );
        assert_eq!(
            refuse_extra_process_contribution_as_observed_mean(contribution, recovered),
            Err(PsychometricError::ExtraProcessContributionIsNotObservedMean)
        );
        assert_eq!(
            refuse_extra_process_latent_mean_as_observed_mean(composed, recovered),
            Err(PsychometricError::ExtraProcessLatentMeanIsNotObservedMean)
        );
    }

    #[test]
    fn extra_process_observed_mean_zero_loading_is_manifest_mean_and_refuses_clock() {
        let loading = 2.0_f64;
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.05_f64;
        let delta = 2.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        assert_eq!(
            recover_discrete_observed_mean_with_extra_process(
                0.0,
                initial,
                original,
                intercept,
                coupling,
                predictor,
                extra,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_extra_process(
                loading,
                initial,
                original,
                intercept,
                coupling,
                predictor,
                extra,
                manifest_mean,
                delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn after_extra_process_observed_mean_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.05_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_extra_process_after(
            loading,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            manifest_mean,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("eq5-after-extra-process-mean");
        let composed = recover_discrete_latent_mean_with_extra_process_after(
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("after-extra-latent");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let first_occasion = recover_discrete_observed_mean_with_extra_process(
            loading,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            extra,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-t0-extra-process-mean");
        assert!((first_occasion - recovered).abs() > 1e-3);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            original,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        let carry_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            original,
            intercept,
            coupling,
            predictor,
            manifest_mean,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-carry-mean");
        assert!((carry_observed - recovered).abs() > 1e-3);
        let contribution = recover_level_change_extra_process_contribution_after(
            coupling,
            predictor,
            original,
            extra,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("after-extra-process");
        assert_eq!(
            refuse_extra_process_observed_mean_as_after_extra_process_observed_mean(
                first_occasion,
                recovered
            ),
            Err(PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean)
        );
        assert_eq!(
            refuse_evolved_observed_mean_as_after_extra_process_observed_mean(
                evolved_observed,
                recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean)
        );
        assert_eq!(
            refuse_impulse_carry_observed_mean_as_after_extra_process_observed_mean(
                carry_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean)
        );
        assert_eq!(
            refuse_after_extra_process_contribution_as_observed_mean(contribution, recovered),
            Err(PsychometricError::AfterExtraProcessContributionIsNotObservedMean)
        );
        assert_eq!(
            refuse_after_extra_process_latent_mean_as_observed_mean(composed, recovered),
            Err(PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn after_extra_process_contribution_refuses_non_interior_interval() {
        let coupling = 0.4_f64;
        let predictor = 3.0_f64;
        let original = -0.5_f64;
        let extra = -0.05_f64;
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                coupling,
                predictor,
                original,
                extra,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                coupling,
                predictor,
                original,
                extra,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_extra_process_after(
                2.0,
                1.0,
                original,
                0.3,
                coupling,
                predictor,
                extra,
                0.5,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_extra_process_after(
                0.0,
                1.0,
                original,
                0.3,
                coupling,
                predictor,
                extra,
                0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.5)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                coupling,
                predictor,
                original,
                extra,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                coupling,
                predictor,
                original,
                extra,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_level_change_extra_process_contribution_after(
                coupling,
                predictor,
                original,
                extra,
                f64::NAN,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_extra_process_after(
                1.0,
                original,
                0.3,
                coupling,
                predictor,
                extra,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        let evolved = recover_discrete_latent_mean(1.0, original, 0.3, 2.0, LagClock::EventTime)
            .expect("mu-t");
        assert_eq!(
            recover_discrete_latent_mean_with_extra_process_after(
                1.0,
                original,
                0.3,
                0.0,
                predictor,
                extra,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
    }

    #[test]
    fn asymptotic_time_independent_effect_recovers_driver_section_seven_point_two() {
        // Driver et al. (2017, §7.2, p. 21) print LeisureTime
        // TIPREDEFFECT = −0.225 and asymTIPREDEFFECT = −1.673 for a
        // unit increase. Reconstruct a = −B / asym.
        let effect = -0.225_f64;
        let predictor = 1.0_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -effect / printed_asym;
        let recovered = recover_asymptotic_time_independent_predictor_effect(
            effect,
            predictor,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        let expected = -(effect * predictor) / log_rate;
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - printed_asym).abs() < 1e-12);
        let happiness = recover_asymptotic_time_independent_predictor_effect(
            0.549,
            1.0,
            -0.549 / 0.219,
            LagClock::EventTime,
        )
        .expect("happiness-asym");
        assert!((happiness - 0.219).abs() < 1e-12);
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                0.0,
                predictor,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                effect,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn asymptotic_time_independent_effect_is_not_coefficient_discrete_cint_or_impulse() {
        let effect = -0.225_f64;
        let predictor = 2.0_f64;
        let log_rate = -0.134_488_942_f64;
        let recovered = recover_asymptotic_time_independent_predictor_effect(
            effect,
            predictor,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        let discrete = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("discreteTIPREDEFFECT");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("impulse");
        assert!((recovered - effect).abs() > 1e-3);
        assert!((recovered - discrete).abs() > 1e-3);
        assert!((recovered - impulse).abs() > 1e-3);
        assert_eq!(
            refuse_asymptotic_time_independent_effect_as_coefficient(recovered, effect),
            Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient)
        );
        assert_eq!(
            refuse_asymptotic_time_independent_effect_as_discrete_effect(recovered, discrete),
            Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect)
        );
        assert_eq!(
            refuse_asymptotic_time_independent_effect_as_continuous_intercept(recovered, 0.3),
            Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_asymptotic_time_independent_effect_as_time_dependent_impulse(recovered, impulse),
            Err(PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse)
        );
    }

    #[test]
    fn asymptotic_time_independent_effect_invalid_inputs_fail_closed() {
        let effect = -0.225_f64;
        let predictor = 1.0_f64;
        let log_rate = -0.134_488_942_f64;
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                effect,
                predictor,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                effect,
                predictor,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                effect,
                predictor,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                f64::NAN,
                predictor,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                1e308,
                2.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_effect(
                1e308,
                1.0,
                -1e-308,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn asymptotic_time_independent_variance_recovers_driver_section_seven_point_two() {
        // Driver et al. (2017, §7.2, p. 21) print LeisureTime
        // asymTIPREDEFFECT = −1.673. addedTIPREDVAR is the variance of
        // that mean shift. Reconstruct a from B and the printed total
        // change; the printed 2.838 is the 2-latent TRAITVAR model, not
        // this scalar map.
        let effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -effect / printed_asym;
        let predictor_variance = 1.0_f64;
        let recovered = recover_asymptotic_time_independent_predictor_variance(
            effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let expected = printed_asym * printed_asym * predictor_variance;
        assert!((recovered - expected).abs() < 1e-12);
        let doubled = recover_asymptotic_time_independent_predictor_variance(
            effect,
            2.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("doubled-v");
        assert!((doubled - 2.0 * expected).abs() < 1e-12);
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                0.0,
                predictor_variance,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                effect,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn asymptotic_time_independent_variance_is_not_trait_stationary_or_mean_effect() {
        let effect = -0.225_f64;
        let log_rate = -0.134_488_942_f64;
        let predictor_variance = 2.0_f64;
        let recovered = recover_asymptotic_time_independent_predictor_variance(
            effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let mean_effect = recover_asymptotic_time_independent_predictor_effect(
            effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        let stationary = recover_stationary_latent_variance(0.4, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let trait_plus = recover_trait_plus_state_latent_variance(0.8, 0.3).expect("trait");
        assert!((recovered - mean_effect).abs() > 1e-3);
        assert!((recovered - stationary).abs() > 1e-3);
        assert!((recovered - trait_plus).abs() > 1e-3);
        assert_eq!(
            refuse_asymptotic_time_independent_variance_as_trait_variance(recovered, trait_plus),
            Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance)
        );
        assert_eq!(
            refuse_asymptotic_time_independent_variance_as_stationary_within_subject(
                recovered, stationary
            ),
            Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject)
        );
        assert_eq!(
            refuse_asymptotic_time_independent_variance_as_asymptotic_effect(
                recovered,
                mean_effect
            ),
            Err(PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect)
        );
    }

    #[test]
    fn asymptotic_time_independent_variance_invalid_inputs_fail_closed() {
        let effect = -0.225_f64;
        let log_rate = -0.134_488_942_f64;
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                effect,
                1.0,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                effect,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                effect,
                -1.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                1e308,
                1.0,
                -1e-308,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                1e200,
                1.0,
                -1e-200,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn asymptotic_continuous_intercept_recovers_driver_table_two() {
        // Driver et al. (2017, Table 2, p. 12; Eq. 3, p. 5; p. 16)
        // name asymCINT the Δt → ∞ intercept contribution −κ / a.
        // Reconstruct a from the printed LeisureTime TIPREDEFFECT
        // −0.225 / asymTIPREDEFFECT −1.673. The printed 2-latent CINT
        // values are not this scalar map.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let intercept = 0.3_f64;
        let recovered =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let expected = intercept / -log_rate;
        assert!((recovered - expected).abs() < 1e-12);
        let unit = recover_asymptotic_continuous_intercept(1.0, log_rate, LagClock::EventTime)
            .expect("unit-asymCINT");
        assert!((unit - 1.0 / -log_rate).abs() < 1e-12);
        let synthetic = recover_asymptotic_continuous_intercept(0.3, -0.5, LagClock::EventTime)
            .expect("synthetic");
        assert!((synthetic - 0.6).abs() < 1e-15);
        let large_delta = recover_discrete_continuous_intercept_effect(
            intercept,
            log_rate,
            1e8,
            LagClock::EventTime,
        )
        .expect("large-delta");
        assert!((recovered - large_delta).abs() < 1e-9);
        assert_eq!(
            recover_asymptotic_continuous_intercept(0.0, 0.0, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(0.0, 0.5, LagClock::EventTime),
            Ok(0.0)
        );
    }

    #[test]
    fn asymptotic_continuous_intercept_is_not_cint_increment_t0_or_tipred() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        let recovered =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let discrete = recover_discrete_continuous_intercept_effect(
            intercept,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("dtCINT");
        let tipred = recover_asymptotic_time_independent_predictor_effect(
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        assert!((recovered - intercept).abs() > 1e-3);
        assert!((recovered - discrete).abs() > 1e-3);
        assert!((recovered - 2.823).abs() > 1e-3);
        assert!((recovered - tipred).abs() > 1e-3);
        assert_eq!(
            refuse_asymptotic_continuous_intercept_as_continuous_intercept(recovered, intercept),
            Err(PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_asymptotic_continuous_intercept_as_discrete_increment(recovered, discrete),
            Err(PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement)
        );
        assert_eq!(
            refuse_asymptotic_continuous_intercept_as_initial_latent_mean(recovered, 2.823),
            Err(PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean)
        );
        assert_eq!(
            refuse_asymptotic_continuous_intercept_as_asymptotic_time_independent_effect(
                recovered, tipred
            ),
            Err(
                PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect
            )
        );
    }

    #[test]
    fn asymptotic_continuous_intercept_invalid_inputs_fail_closed() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        assert_eq!(
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(intercept, 0.0, LagClock::EventTime),
            Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(intercept, 0.5, LagClock::EventTime),
            Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(f64::NAN, log_rate, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_continuous_intercept(1e308, -1e-308, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn stationary_initial_latent_mean_recovers_driver_page_sixteen() {
        // Driver et al. (2017, p. 16; Table 2, p. 12; Eq. 3)
        // constrain T0MEANS to model-implied values that include
        // extra effects due to time-independent predictors
        // (asymTIPREDEFFECT). Reconstruct a from printed LeisureTime
        // TIPREDEFFECT −0.225 / asymTIPREDEFFECT −1.673. The printed
        // 2-latent T0MEANS 2.823 is not this scalar map.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let intercept = 0.3_f64;
        let recovered = recover_stationary_initial_latent_mean(
            intercept,
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0MEANS");
        let intercept_only =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let tipred = recover_asymptotic_time_independent_predictor_effect(
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        assert!((recovered - (intercept_only + tipred)).abs() < 1e-12);
        let synthetic =
            recover_stationary_initial_latent_mean(0.3, 0.2, 1.0, -0.5, LagClock::EventTime)
                .expect("synthetic");
        assert!((synthetic - 1.0).abs() < 1e-15);
        let intercept_only_path = recover_stationary_initial_latent_mean(
            intercept,
            0.0,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("intercept-only");
        assert!((intercept_only_path - intercept_only).abs() < 1e-15);
        let tipred_only = recover_stationary_initial_latent_mean(
            0.0,
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("ti-only");
        assert!((tipred_only - tipred).abs() < 1e-15);
        assert_eq!(
            recover_stationary_initial_latent_mean(0.0, 0.0, 1.0, 0.0, LagClock::EventTime),
            Ok(0.0)
        );
        assert_eq!(
            recover_stationary_initial_latent_mean(0.0, 0.0, 1.0, 0.5, LagClock::EventTime),
            Ok(0.0)
        );
    }

    #[test]
    fn stationary_initial_latent_mean_is_not_t0_cint_tipred_or_discrete() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        let recovered = recover_stationary_initial_latent_mean(
            intercept,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0MEANS");
        let intercept_only =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let tipred = recover_asymptotic_time_independent_predictor_effect(
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        let discrete =
            recover_discrete_latent_mean(2.823, log_rate, intercept, 1.0, LagClock::EventTime)
                .expect("μ_t");
        assert!((recovered - 2.823).abs() > 1e-3);
        assert!((recovered - intercept_only).abs() > 1e-3);
        assert!((recovered - tipred).abs() > 1e-3);
        assert!((recovered - discrete).abs() > 1e-3);
        assert_eq!(
            refuse_stationary_initial_latent_mean_as_initial_latent_mean(recovered, 2.823),
            Err(PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean)
        );
        assert_eq!(
            refuse_stationary_initial_latent_mean_as_asymptotic_continuous_intercept(
                recovered,
                intercept_only
            ),
            Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept)
        );
        assert_eq!(
            refuse_stationary_initial_latent_mean_as_asymptotic_time_independent_effect(
                recovered, tipred
            ),
            Err(PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect)
        );
        assert_eq!(
            refuse_stationary_initial_latent_mean_as_discrete_mean(recovered, discrete),
            Err(PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean)
        );
    }

    #[test]
    fn stationary_initial_latent_mean_invalid_inputs_fail_closed() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        assert_eq!(
            recover_stationary_initial_latent_mean(
                intercept,
                -0.225,
                1.0,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_initial_latent_mean(intercept, 0.0, 1.0, 0.0, LagClock::EventTime),
            Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_latent_mean(0.0, -0.225, 1.0, 0.5, LagClock::EventTime),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_latent_mean(
                f64::NAN,
                -0.225,
                1.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_initial_latent_mean(1e308, 1e308, 1.0, -1e-308, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn stationary_initial_observed_mean_recovers_driver_equation_five_of_section_four_point_three()
    {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 5, p. 5)
        // constrain first-occasion means to the model-predicted
        // mean. Equation 5 maps E(y_0) = τ + λ of that mean.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let intercept = 0.3_f64;
        let loading = 2.0_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_stationary_initial_observed_mean(
            loading,
            intercept,
            printed_effect,
            1.0,
            log_rate,
            manifest_mean,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0MEANS");
        let latent = recover_stationary_initial_latent_mean(
            intercept,
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0MEANS");
        let expected =
            recover_manifest_observed_mean(loading, latent, manifest_mean).expect("τ+λμ");
        assert!((recovered - expected).abs() < 1e-12);
        let intercept_only =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let intercept_only_observed =
            recover_manifest_observed_mean(loading, intercept_only, manifest_mean)
                .expect("τ+λ(−κ/a)");
        assert!((recovered - intercept_only_observed).abs() > 1e-3);
        let free_initial_observed =
            recover_manifest_observed_mean(loading, 2.823, manifest_mean).expect("τ+λμ_0");
        assert!((recovered - free_initial_observed).abs() > 1e-3);
        let evolved_from_free = recover_discrete_observed_mean(
            loading,
            2.823,
            log_rate,
            intercept,
            manifest_mean,
            1.0,
            LagClock::EventTime,
        )
        .expect("τ+λμ_t");
        assert!((recovered - evolved_from_free).abs() > 1e-3);
        assert!((recovered - manifest_mean).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_stationary_initial_observed_mean(
                0.0,
                intercept,
                printed_effect,
                1.0,
                log_rate,
                manifest_mean,
                LagClock::EventTime,
            ),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_stationary_initial_observed_mean(
                loading,
                0.0,
                0.0,
                1.0,
                0.0,
                manifest_mean,
                LagClock::EventTime,
            ),
            Ok(manifest_mean)
        );
        let evolved_from_stationary =
            recover_discrete_observed_mean_with_time_independent_predictor(
                loading,
                latent,
                log_rate,
                intercept,
                printed_effect,
                1.0,
                manifest_mean,
                2.0,
                LagClock::EventTime,
            )
            .expect("invariance");
        assert!((evolved_from_stationary - recovered).abs() < 1e-12);
        let evolved_latent = recover_discrete_latent_mean_with_time_independent_predictor(
            latent,
            log_rate,
            intercept,
            printed_effect,
            1.0,
            2.0,
            LagClock::EventTime,
        )
        .expect("stationary invariance");
        assert!((evolved_latent - latent).abs() < 1e-12);
    }

    #[test]
    fn stationary_initial_observed_mean_is_not_manifest_latent_evolved_or_free() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_stationary_initial_observed_mean(
            loading,
            intercept,
            -0.225,
            1.0,
            log_rate,
            manifest_mean,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0MEANS");
        let latent = recover_stationary_initial_latent_mean(
            intercept,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0MEANS");
        let intercept_only =
            recover_asymptotic_continuous_intercept(intercept, log_rate, LagClock::EventTime)
                .expect("asymCINT");
        let intercept_only_observed =
            recover_manifest_observed_mean(loading, intercept_only, manifest_mean)
                .expect("τ+λ(−κ/a)");
        let free_initial_observed =
            recover_manifest_observed_mean(loading, 2.823, manifest_mean).expect("τ+λμ_0");
        let evolved = recover_discrete_observed_mean(
            loading,
            2.823,
            log_rate,
            intercept,
            manifest_mean,
            1.0,
            LagClock::EventTime,
        )
        .expect("τ+λμ_t");
        assert_eq!(
            refuse_stationary_initial_latent_mean_as_observed_mean(latent, recovered),
            Err(PsychometricError::StationaryInitialLatentMeanIsNotObservedMean)
        );
        assert_eq!(
            refuse_stationary_initial_observed_mean_as_manifest_means(recovered, manifest_mean),
            Err(PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans)
        );
        assert_eq!(
            refuse_evolved_observed_mean_as_stationary_initial_observed_mean(evolved, recovered),
            Err(PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean)
        );
        assert_eq!(
            refuse_asymptotic_continuous_intercept_observed_mean_as_stationary_initial_observed_mean(
                intercept_only_observed,
                recovered
            ),
            Err(
                PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean
            )
        );
        assert_eq!(
            refuse_initial_observed_mean_as_stationary_initial_observed_mean(
                free_initial_observed,
                recovered
            ),
            Err(PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean)
        );
    }

    #[test]
    fn stationary_initial_observed_mean_invalid_inputs_fail_closed() {
        let intercept = 0.3_f64;
        let log_rate = -0.134_488_942_f64;
        assert_eq!(
            recover_stationary_initial_observed_mean(
                2.0,
                intercept,
                -0.225,
                1.0,
                log_rate,
                0.5,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_initial_observed_mean(
                2.0,
                intercept,
                0.0,
                1.0,
                0.0,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_observed_mean(
                2.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_observed_mean(
                f64::NAN,
                intercept,
                -0.225,
                1.0,
                log_rate,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_initial_observed_mean(
                2.0,
                1e308,
                1e308,
                1.0,
                -1e-308,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn stationary_initial_latent_variance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3, pp. 9–10; p. 16) constrain T0VAR
        // to model-predicted variances. The scalar composition is
        // trait + −q / (2 a) + (B / a)² v. Reconstruct a from printed
        // LeisureTime TIPREDEFFECT −0.225 / asymTIPREDEFFECT −1.673.
        // The printed 2-latent addedTIPREDVAR 2.838 is not this
        // scalar map.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let trait_plus_state =
            recover_trait_plus_state_latent_variance(trait_variance, state).expect("trait+state");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_plus_state + added)).abs() < 1e-12);
        let state_only = recover_stationary_initial_latent_variance(
            0.0,
            diffusion,
            0.0,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("state-only");
        assert!((state_only - state).abs() < 1e-15);
        let trait_only = recover_stationary_initial_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        )
        .expect("trait-only");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let added_only = recover_stationary_initial_latent_variance(
            0.0,
            0.0,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("ti-only");
        assert!((added_only - added).abs() < 1e-15);
        assert_eq!(
            recover_stationary_initial_latent_variance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.5,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn stationary_initial_latent_variance_is_not_t0_state_trait_tipred_or_discrete() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let recovered = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let added = recover_asymptotic_time_independent_predictor_variance(
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let discrete = recover_discrete_latent_variance(
            recovered,
            diffusion,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("Var(η_t)");
        assert!((recovered - 2.0).abs() > 1e-3);
        assert!((recovered - state).abs() > 1e-3);
        assert!((recovered - trait_variance).abs() > 1e-3);
        assert!((recovered - added).abs() > 1e-3);
        assert!((recovered - discrete).abs() > 1e-3);
        assert!((recovered - 2.838).abs() > 1e-3);
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_initial_latent_variance(recovered, 2.0),
            Err(PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance)
        );
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_stationary_within_subject(
                recovered, state
            ),
            Err(PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject)
        );
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_trait_variance(recovered, trait_variance),
            Err(PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance)
        );
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_asymptotic_time_independent_variance(
                recovered, added
            ),
            Err(
                PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance
            )
        );
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_discrete_variance(recovered, discrete),
            Err(PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance)
        );
    }

    #[test]
    fn stationary_initial_latent_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_initial_latent_variance(
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                f64::NAN,
                0.4,
                0.0,
                0.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_initial_latent_variance(
                f64::MAX,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn stationary_initial_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
     {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 5, p. 5)
        // constrain first-occasion variances to the model-predicted
        // variance. Equation 5 maps Var(y_0) = λ² of that variance
        // plus θ + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let recovered = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0VAR");
        let latent = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let expected = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("λ²p+θ+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let state_only_observed =
            recover_manifest_observed_variance(loading, state, measurement_error)
                .expect("λ²(−q/2a)+θ");
        assert!((recovered - state_only_observed).abs() > 1e-3);
        let free_initial_observed =
            recover_manifest_observed_variance(loading, 2.0, measurement_error).expect("λ²p_0+θ");
        assert!((recovered - free_initial_observed).abs() > 1e-3);
        let discrete =
            recover_discrete_latent_variance(latent, diffusion, log_rate, 1.0, LagClock::EventTime)
                .expect("Var(η_t)");
        let evolved = recover_manifest_observed_variance(loading, discrete, measurement_error)
            .expect("λ²Var(η_t)+θ");
        assert!((recovered - evolved).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_stationary_initial_observed_variance(
                0.0,
                trait_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                measurement_error,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(measurement_error + manifest_trait)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                loading,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                measurement_error,
                0.0,
                LagClock::EventTime,
            ),
            Ok(measurement_error)
        );
        let zero_manifest_trait = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            0.0,
            LagClock::EventTime,
        )
        .expect("ψ=0");
        let expected_zero_psi =
            recover_manifest_observed_variance(loading, latent, measurement_error).expect("λ²p+θ");
        assert!((zero_manifest_trait - expected_zero_psi).abs() < 1e-12);
    }

    #[test]
    fn stationary_initial_observed_variance_is_not_manifest_latent_evolved_or_free() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let recovered = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0VAR");
        let latent = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let state_only_observed =
            recover_manifest_observed_variance(loading, state, measurement_error)
                .expect("λ²(−q/2a)+θ");
        let free_initial_observed =
            recover_manifest_observed_variance(loading, 2.0, measurement_error).expect("λ²p_0+θ");
        let discrete =
            recover_discrete_latent_variance(latent, diffusion, log_rate, 1.0, LagClock::EventTime)
                .expect("Var(η_t)");
        let evolved = recover_manifest_observed_variance(loading, discrete, measurement_error)
            .expect("λ²Var(η_t)+θ");
        assert_eq!(
            refuse_stationary_initial_latent_variance_as_observed_variance(latent, recovered),
            Err(PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            refuse_stationary_initial_observed_variance_as_measurement_error(
                recovered,
                measurement_error
            ),
            Err(PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError)
        );
        assert_eq!(
            refuse_evolved_observed_variance_as_stationary_initial_observed_variance(
                evolved, recovered
            ),
            Err(PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance)
        );
        assert_eq!(
            refuse_stationary_within_subject_observed_variance_as_stationary_initial_observed_variance(
                state_only_observed,
                recovered
            ),
            Err(
                PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance
            )
        );
        assert_eq!(
            refuse_initial_observed_variance_as_stationary_initial_observed_variance(
                free_initial_observed,
                recovered
            ),
            Err(PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance)
        );
    }

    #[test]
    fn stationary_initial_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_initial_observed_variance(
                2.0,
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.5,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                2.0,
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                2.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.6)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                f64::NAN,
                1.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_initial_observed_variance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn stationary_lagged_latent_covariance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5; p. 16)
        // constrain T0VAR. The lagged covariance of that stationary
        // process is trait + e^{a Δt}(−q / (2 a)) + (B / a)² v.
        // Trait and addedTIPREDVAR do not decay.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let trait_plus_state = recover_trait_plus_state_lagged_covariance(
            trait_variance,
            state,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait+state lagged");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_plus_state + added)).abs() < 1e-12);
        let contemporaneous = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        assert!((recovered - contemporaneous).abs() > 1e-3);
        let decayed = recover_discrete_lagged_latent_covariance(
            contemporaneous,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt} p_stat");
        assert!((recovered - decayed).abs() > 1e-3);
        let state_only = recover_stationary_lagged_latent_covariance(
            0.0,
            diffusion,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("state-only lagged");
        let lagged_state = recover_discrete_lagged_latent_covariance(
            state,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt} asymDIFFUSION");
        assert!((state_only - lagged_state).abs() < 1e-15);
        let trait_only = recover_stationary_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait-only lagged");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let added_only = recover_stationary_lagged_latent_covariance(
            0.0,
            0.0,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("ti-only lagged");
        assert!((added_only - added).abs() < 1e-15);
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                event_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let far = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e8,
            LagClock::EventTime,
        )
        .expect("Δt→∞");
        assert!((far - (trait_variance + added)).abs() < 1e-12);
        let near = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+");
        assert!((near - contemporaneous).abs() < 1e-9);
    }

    #[test]
    fn stationary_lagged_latent_covariance_is_not_contemporaneous_decayed_or_trait_state() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let contemporaneous = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let decayed = recover_discrete_lagged_latent_covariance(
            contemporaneous,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt} p_stat");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let trait_plus_state = recover_trait_plus_state_lagged_covariance(
            trait_variance,
            state,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait+state lagged");
        assert!((recovered - contemporaneous).abs() > 1e-3);
        assert!((recovered - decayed).abs() > 1e-3);
        assert!((recovered - trait_plus_state).abs() > 1e-3);
        assert_eq!(
            refuse_stationary_lagged_latent_covariance_as_stationary_initial_latent_variance(
                recovered,
                contemporaneous
            ),
            Err(
                PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance
            )
        );
        assert_eq!(
            refuse_stationary_lagged_latent_covariance_as_decayed_stationary_variance(
                recovered, decayed
            ),
            Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance)
        );
        assert_eq!(
            refuse_trait_plus_state_lagged_covariance_as_stationary_lagged_latent_covariance(
                trait_plus_state,
                recovered
            ),
            Err(
                PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance
            )
        );
    }

    #[test]
    fn stationary_lagged_latent_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                f64::NAN,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_lagged_latent_covariance(
                f64::MAX,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn stationary_lagged_observed_covariance_recovers_driver_equation_five_of_section_four_point_three()
     {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 5, p. 5)
        // lagged observed covariance of stationary T0VAR is
        // λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ.
        // Θ does not enter.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        let latent = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
            .expect("λ²c+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let contemporaneous = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0VAR");
        assert!((recovered - contemporaneous).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                0.0,
                trait_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                event_delta,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(manifest_trait)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                loading,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                event_delta,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        let zero_manifest_trait = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            0.0,
            LagClock::EventTime,
        )
        .expect("ψ=0");
        let expected_zero_psi =
            recover_manifest_lagged_observed_covariance(loading, latent, 0.0).expect("λ²c");
        assert!((zero_manifest_trait - expected_zero_psi).abs() < 1e-12);
    }

    #[test]
    fn stationary_lagged_observed_covariance_is_not_manifest_latent_or_contemporaneous() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        let latent = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let contemporaneous = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0VAR");
        assert_eq!(
            refuse_stationary_lagged_latent_covariance_as_observed_covariance(latent, recovered),
            Err(PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance)
        );
        assert_eq!(
            refuse_measurement_error_as_stationary_lagged_observed_covariance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance)
        );
        assert_eq!(
            refuse_stationary_initial_observed_variance_as_stationary_lagged_observed_covariance(
                contemporaneous,
                recovered
            ),
            Err(
                PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance
            )
        );
    }

    #[test]
    fn stationary_lagged_observed_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.1)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                f64::NAN,
                1.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_lagged_observed_covariance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn stationary_later_latent_variance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5; p. 16)
        // constrain T0VAR across all time points. The later-occasion
        // variance is trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v.
        // Under stationarity that equals contemporaneous T0VAR.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let evolved_state = recover_discrete_latent_variance(
            state,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt}p+Q_Δt");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_variance + evolved_state + added)).abs() < 1e-12);
        let contemporaneous = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        assert!((recovered - contemporaneous).abs() < 1e-12);
        let lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        assert!((recovered - lagged).abs() > 1e-3);
        let free_discrete = recover_discrete_latent_variance(
            contemporaneous,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt} p_stat + Q_Δt");
        assert!((recovered - free_discrete).abs() > 1e-3);
        let process_noise =
            recover_discrete_process_noise(diffusion, log_rate, event_delta, LagClock::EventTime)
                .expect("Q_Δt");
        assert!((recovered - process_noise).abs() > 1e-3);
        let state_only = recover_stationary_later_latent_variance(
            0.0,
            diffusion,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("state-only later");
        assert!((state_only - evolved_state).abs() < 1e-15);
        assert!((state_only - state).abs() < 1e-12);
        let trait_only = recover_stationary_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait-only later");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let added_only = recover_stationary_later_latent_variance(
            0.0,
            0.0,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("ti-only later");
        assert!((added_only - added).abs() < 1e-15);
        assert_eq!(
            recover_stationary_later_latent_variance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                event_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let far = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e8,
            LagClock::EventTime,
        )
        .expect("Δt→∞");
        assert!((far - contemporaneous).abs() < 1e-12);
        let near = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+");
        assert!((near - contemporaneous).abs() < 1e-9);
    }

    #[test]
    fn stationary_later_latent_variance_is_not_lagged_discrete_or_process_noise() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        let lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let contemporaneous = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let free_discrete = recover_discrete_latent_variance(
            contemporaneous,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt} p_stat + Q_Δt");
        let process_noise =
            recover_discrete_process_noise(diffusion, log_rate, event_delta, LagClock::EventTime)
                .expect("Q_Δt");
        assert!((recovered - lagged).abs() > 1e-3);
        assert!((recovered - free_discrete).abs() > 1e-3);
        assert!((recovered - process_noise).abs() > 1e-3);
        assert_eq!(
            refuse_stationary_later_latent_variance_as_lagged_covariance(recovered, lagged),
            Err(PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance)
        );
        assert_eq!(
            refuse_stationary_later_latent_variance_as_discrete_variance(recovered, free_discrete),
            Err(PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance)
        );
        assert_eq!(
            refuse_stationary_later_latent_variance_as_process_noise(recovered, process_noise),
            Err(PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise)
        );
    }

    #[test]
    fn stationary_later_latent_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_later_latent_variance(
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                f64::NAN,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_later_latent_variance(
                f64::MAX,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn stationary_later_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
     {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 5, p. 5)
        // later-occasion observed variance of stationary T0VAR is
        // λ²(trait + e^{2 a Δt}(−q / (2 a)) + Q_Δt + (B / a)² v) + θ + ψ.
        // Under stationarity that equals contemporaneous Var(y_0).
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-stationary-T0VAR");
        let latent = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        let expected = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("λ²p+θ+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let contemporaneous = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-T0VAR");
        assert!((recovered - contemporaneous).abs() < 1e-12);
        let lagged = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        assert!((recovered - lagged).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_stationary_later_observed_variance(
                0.0,
                trait_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                event_delta,
                measurement_error,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(measurement_error + manifest_trait)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                loading,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                event_delta,
                0.0,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        let zero_manifest_trait = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.0,
            LagClock::EventTime,
        )
        .expect("ψ=0");
        let expected_zero_psi = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            0.0,
        )
        .expect("λ²p+θ");
        assert!((zero_manifest_trait - expected_zero_psi).abs() < 1e-12);
    }

    #[test]
    fn stationary_later_observed_variance_is_not_manifest_latent_or_lagged() {
        let trait_variance = 1.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-later-stationary-T0VAR");
        let latent = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        let lagged = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        assert_eq!(
            refuse_stationary_later_latent_variance_as_observed_variance(latent, recovered),
            Err(PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            refuse_measurement_error_as_stationary_later_observed_variance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance)
        );
        assert_eq!(
            refuse_stationary_lagged_observed_covariance_as_stationary_later_observed_variance(
                lagged, recovered
            ),
            Err(
                PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance
            )
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn stationary_later_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                0.5,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                1.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                0.0,
                0.4,
                0.0,
                1.0,
                0.0,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.6)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                f64::NAN,
                1.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_stationary_later_observed_variance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_latent_variance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 3–4, pp. 4–5)
        // treat the first time point as predetermined. Free T0VAR p_0
        // then transitions toward stationarity:
        // trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        let evolved_state = recover_discrete_latent_variance(
            initial_latent_variance,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt}p_0+Q_Δt");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_variance + evolved_state + added)).abs() < 1e-12);
        let stationary_later = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        assert!((recovered - stationary_later).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_later_latent_variance(
            trait_variance,
            state,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary_later).abs() < 1e-12);
        let contemporaneous = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary T0VAR");
        let first_occasion_total = trait_variance + initial_latent_variance + added;
        let free_discrete = recover_discrete_latent_variance(
            first_occasion_total,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt}(trait+p_0+added)+Q_Δt");
        assert!((recovered - free_discrete).abs() > 1e-3);
        assert!((recovered - initial_latent_variance).abs() > 1e-3);
        let state_only = recover_predetermined_later_latent_variance(
            0.0,
            initial_latent_variance,
            diffusion,
            0.0,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("state-only predetermined later");
        assert!((state_only - evolved_state).abs() < 1e-15);
        let trait_only = recover_predetermined_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait-only predetermined later");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let added_only = recover_predetermined_later_latent_variance(
            0.0,
            0.0,
            0.0,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("ti-only predetermined later");
        assert!((added_only - added).abs() < 1e-15);
        assert_eq!(
            recover_predetermined_later_latent_variance(
                0.0,
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                event_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let far = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e8,
            LagClock::EventTime,
        )
        .expect("Δt→∞");
        assert!((far - contemporaneous).abs() < 1e-12);
        let near = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+");
        assert!((near - first_occasion_total).abs() < 1e-9);
        let growing = recover_predetermined_later_latent_variance(
            0.0,
            initial_latent_variance,
            diffusion,
            0.0,
            0.0,
            0.5,
            event_delta,
            LagClock::EventTime,
        )
        .expect("growing process");
        assert!(growing > initial_latent_variance);
    }

    #[test]
    fn predetermined_later_latent_variance_is_not_stationary_discrete_or_initial() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        let stationary_later = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary later T0VAR");
        let added = recover_asymptotic_time_independent_predictor_variance(
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let free_discrete = recover_discrete_latent_variance(
            trait_variance + initial_latent_variance + added,
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{2aΔt}(trait+p_0+added)+Q_Δt");
        assert!((recovered - stationary_later).abs() > 1e-3);
        assert!((recovered - free_discrete).abs() > 1e-3);
        assert!((recovered - initial_latent_variance).abs() > 1e-3);
        assert_eq!(
            refuse_predetermined_later_latent_variance_as_stationary_later_latent_variance(
                recovered,
                stationary_later
            ),
            Err(
                PsychometricError::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_latent_variance_as_discrete_variance(
                recovered,
                free_discrete
            ),
            Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotDiscreteVariance)
        );
        assert_eq!(
            refuse_predetermined_later_latent_variance_as_initial_latent_variance(
                recovered,
                initial_latent_variance
            ),
            Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_latent_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_latent_variance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_latent_variance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_latent_variance(
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_later_latent_variance(
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let growing = recover_predetermined_later_latent_variance(
            0.0,
            2.0,
            0.4,
            0.0,
            0.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("Brownian a=0");
        assert!((growing - 2.4).abs() < 1e-12);
        assert_eq!(
            recover_predetermined_later_latent_variance(
                f64::NAN,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_later_latent_variance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_later_latent_variance(
                f64::MAX,
                0.0,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_observed_variance_recovers_driver_equation_five_of_section_four_point_three()
     {
        // Driver et al. (2017, §4.3, pp. 9–10; Eq. 5, p. 5)
        // later-occasion observed variance of predetermined T0VAR is
        // λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-predetermined-T0VAR");
        let latent = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        let expected = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("λ²p+θ+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let stationary_later = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-stationary-T0VAR");
        assert!((recovered - stationary_later).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            state,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary_later).abs() < 1e-12);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_later_observed_variance(
                0.0,
                trait_variance,
                initial_latent_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                event_delta,
                measurement_error,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(measurement_error + manifest_trait)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                loading,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                event_delta,
                0.0,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        let zero_manifest_trait = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.0,
            LagClock::EventTime,
        )
        .expect("ψ=0");
        let expected_zero_psi = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            0.0,
        )
        .expect("λ²p+θ");
        assert!((zero_manifest_trait - expected_zero_psi).abs() < 1e-12);
    }

    #[test]
    fn predetermined_later_observed_variance_is_not_manifest_latent_or_stationary() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-later-predetermined-T0VAR");
        let latent = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        let stationary_later = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-later-stationary-T0VAR");
        assert_eq!(
            refuse_predetermined_later_latent_variance_as_observed_variance(latent, recovered),
            Err(PsychometricError::PredeterminedLaterLatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            refuse_measurement_error_as_predetermined_later_observed_variance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterObservedVariance)
        );
        assert_eq!(
            refuse_stationary_later_observed_variance_as_predetermined_later_observed_variance(
                stationary_later,
                recovered
            ),
            Err(
                PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance
            )
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_observed_variance(
                2.0,
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                1.0,
                0.5,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                2.0,
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.6)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                f64::NAN,
                1.0,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_later_observed_variance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_lagged_latent_covariance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3; Eq. 3–4): cov(η_t, η_{t0}) =
        // trait + e^{a Δt} p_0 + (B / a)² v.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        let lagged_state = recover_discrete_lagged_latent_covariance(
            initial_latent_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt}p_0");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_variance + lagged_state + added)).abs() < 1e-12);
        let stationary_lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        assert!((recovered - stationary_lagged).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            state,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary_lagged).abs() < 1e-12);
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        assert!((recovered - later).abs() > 1e-3);
        let first_occasion_total = trait_variance + initial_latent_variance + added;
        let decayed_total = recover_discrete_lagged_latent_covariance(
            first_occasion_total,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt}(trait+p_0+added)");
        assert!((recovered - decayed_total).abs() > 1e-3);
        assert!((recovered - initial_latent_variance).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                event_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let trait_only = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait-only predetermined lagged");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let far = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            1e8,
            LagClock::EventTime,
        )
        .expect("Δt→∞");
        assert!((far - (trait_variance + added)).abs() < 1e-12);
        let near = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+");
        assert!((near - first_occasion_total).abs() < 1e-9);
        let growing = recover_predetermined_lagged_latent_covariance(
            0.0,
            initial_latent_variance,
            0.0,
            0.0,
            0.5,
            event_delta,
            LagClock::EventTime,
        )
        .expect("growing carry");
        assert!(growing > initial_latent_variance);
    }

    #[test]
    fn predetermined_lagged_latent_covariance_is_not_stationary_later_or_decayed() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        let stationary_lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged T0VAR");
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        let added = recover_asymptotic_time_independent_predictor_variance(
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let decayed_total = recover_discrete_lagged_latent_covariance(
            trait_variance + initial_latent_variance + added,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("e^{aΔt}(trait+p_0+added)");
        assert_eq!(
            refuse_predetermined_lagged_latent_covariance_as_stationary_lagged_covariance(
                recovered,
                stationary_lagged
            ),
            Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance)
        );
        assert_eq!(
            refuse_predetermined_lagged_latent_covariance_as_later_latent_variance(
                recovered, later
            ),
            Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance)
        );
        assert_eq!(
            refuse_predetermined_lagged_latent_covariance_as_decayed_total(
                recovered,
                decayed_total
            ),
            Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal)
        );
        assert_eq!(
            refuse_predetermined_lagged_latent_covariance_as_initial_latent_variance(
                recovered,
                initial_latent_variance
            ),
            Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance)
        );
    }

    #[test]
    fn predetermined_lagged_latent_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let brownian = recover_predetermined_lagged_latent_covariance(
            0.0,
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("Brownian a=0");
        assert!((brownian - 2.0).abs() < 1e-12);
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                f64::NAN,
                2.0,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_lagged_latent_covariance(
                f64::MAX,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_lagged_observed_covariance_recovers_driver_equation_five() {
        // Driver et al. (2017, Eq. 5 of lagged predetermined T0VAR):
        // λ²(trait + e^{a Δt} p_0 + (B / a)² v) + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_lagged_observed_covariance(
            loading,
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-predetermined-T0VAR");
        let latent = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
            .expect("λ²c+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let stationary_lagged = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        assert!((recovered - stationary_lagged).abs() > 1e-3);
        let later = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-predetermined-T0VAR");
        assert!((recovered - later).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                0.0,
                trait_variance,
                initial_latent_variance,
                printed_effect,
                1.0,
                log_rate,
                event_delta,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(manifest_trait)
        );
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                loading,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                event_delta,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
    }

    #[test]
    fn predetermined_lagged_observed_covariance_is_not_manifest_later_or_stationary() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_lagged_observed_covariance(
            loading,
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-predetermined-T0VAR");
        let latent = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        let later = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-later-predetermined-T0VAR");
        let stationary_lagged = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            0.1,
            LagClock::EventTime,
        )
        .expect("eq5-lagged-stationary-T0VAR");
        assert_eq!(
            refuse_predetermined_lagged_latent_covariance_as_observed_covariance(latent, recovered),
            Err(PsychometricError::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance)
        );
        assert_eq!(
            refuse_measurement_error_as_predetermined_lagged_observed_covariance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance)
        );
        assert_eq!(
            refuse_predetermined_later_observed_variance_as_predetermined_lagged_observed_covariance(
                later,
                recovered
            ),
            Err(
                PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance
            )
        );
        assert_eq!(
            refuse_stationary_lagged_observed_covariance_as_predetermined_lagged_observed_covariance(
                stationary_lagged,
                recovered
            ),
            Err(
                PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance
            )
        );
    }

    #[test]
    fn predetermined_lagged_observed_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                2.0,
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                1.0,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                2.0,
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                0.0,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                1.0,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.1)
        );
        let brownian = recover_predetermined_lagged_observed_covariance(
            1.0,
            0.0,
            2.0,
            0.0,
            1.0,
            0.0,
            1.0,
            0.0,
            LagClock::EventTime,
        )
        .expect("Brownian a=0");
        assert!((brownian - 2.0).abs() < 1e-12);
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                f64::NAN,
                1.0,
                2.0,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_lagged_observed_covariance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn discrete_observed_mean_with_impulse_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let composed = recover_discrete_latent_mean_with_impulse(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("mx");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert!((carried_observed - recovered).abs() > 1e-3);
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                0.0,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                loading,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                0.0,
                delta,
                LagClock::EventTime
            ),
            Ok(loading * composed)
        );
    }

    #[test]
    fn discrete_observed_mean_with_impulse_is_not_evolved_or_zero_impulse() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let zero_impulse = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            0.0,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-impulse");
        assert!((zero_impulse - evolved_observed).abs() < 1e-15);
        assert!((recovered - evolved_observed).abs() > 1e-3);
    }

    #[test]
    fn discrete_observed_mean_with_impulse_refuses_evolved_mean_and_overflow() {
        let loading = 2.0_f64;
        let recovered = recover_discrete_observed_mean_with_impulse(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let composed = recover_discrete_latent_mean_with_impulse(
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            2.0,
            LagClock::EventTime,
        )
        .expect("mx");
        let evolved_observed =
            recover_discrete_observed_mean(loading, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("eq3-eq5-mean");
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert_eq!(
            refuse_evolved_observed_mean_as_impulse_observed_mean(evolved_observed, recovered),
            Err(PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean)
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_impulse_carry_observed_mean(
                recovered,
                carried_observed
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
        );
        assert_eq!(
            refuse_latent_mean_as_observed_mean(composed, recovered),
            Err(PsychometricError::LatentMeanIsNotObservedMean)
        );
        assert_eq!(
            refuse_manifest_means_as_observed_mean(0.5, recovered),
            Err(PsychometricError::ManifestMeansIsNotObservedMean)
        );
        let scaled = recover_discrete_observed_mean_with_impulse(
            1e308,
            1e-308,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded = recover_discrete_observed_mean_with_impulse(
            1e308,
            1.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("lambda-mu");
        assert!((finite_loaded - 1e308).abs() / 1e308 < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_with_impulse_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                f64::NAN,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                1e308,
                2.0,
                0.0,
                0.0,
                0.0,
                3.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                1.0,
                1.0,
                710.0,
                0.0,
                0.0,
                3.0,
                0.5,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                1e308,
                0.0,
                0.0,
                0.0,
                1e308,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
    }

    #[test]
    fn time_independent_predictor_recovers_driver_equation_three_second_summand() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        let expected =
            recover_discrete_constant_predictor_effect(1.2, drift, delta, LagClock::EventTime)
                .expect("bz-map");
        assert!((increment - expected).abs() < 1e-15);
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                0.0,
                predictor,
                drift,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                effect,
                0.0,
                drift,
                delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let zero_drift = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            0.0,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-drift");
        assert!((zero_drift - 2.4).abs() < 1e-15);
        let intercept_effect =
            recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
                .expect("cint");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let equation_fourteen = recover_discrete_time_varying_predictor_effect(
            effect,
            delta,
            delta,
            delta,
            LagClock::EventTime,
        )
        .expect("eq14");
        assert!((increment - intercept_effect).abs() > 1e-3);
        assert!((increment - impulse).abs() > 1e-3);
        assert!((increment - equation_fourteen).abs() > 1e-3);
        assert!((increment - effect).abs() > 1e-3);
    }

    #[test]
    fn time_independent_predictor_composes_evolved_mean_and_keeps_scale() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let composed = recover_discrete_latent_mean_with_time_independent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-tipred");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        assert!((composed - (evolved + increment)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean_with_time_independent_predictor(
                initial,
                drift,
                intercept,
                0.0,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_time_independent_predictor(
                0.0,
                drift,
                0.0,
                effect,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(increment)
        );
        let scaled = recover_discrete_time_independent_predictor_effect(
            1e308,
            1e-308,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
    }

    #[test]
    fn time_independent_predictor_refuses_cint_impulse_equation_fourteen_and_coefficient() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            -0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("tipred");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let equation_fourteen = recover_discrete_time_varying_predictor_effect(
            effect,
            2.0,
            2.0,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq14");
        assert_eq!(
            refuse_time_independent_effect_as_continuous_intercept(increment, effect),
            Err(PsychometricError::TimeIndependentEffectIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_time_independent_effect_as_time_dependent_impulse(increment, impulse),
            Err(PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse)
        );
        assert_eq!(
            refuse_time_independent_effect_as_time_varying_discrete_effect(
                increment,
                equation_fourteen
            ),
            Err(PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect)
        );
        assert_eq!(
            refuse_time_independent_coefficient_as_discrete_effect(effect, increment),
            Err(PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect)
        );
    }

    #[test]
    fn time_independent_predictor_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                f64::NAN,
                1.0,
                -0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                1.0,
                f64::INFINITY,
                -0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                1e308,
                2.0,
                -0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                1e308,
                1.0,
                0.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                0.4,
                3.0,
                -0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_time_independent_predictor_effect(
                0.4,
                3.0,
                -0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_time_independent_predictor(
                1e308,
                0.0,
                0.0,
                1.0,
                1e308,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        // Latent mean is finite; Bz overflows. That `?` is not the sum overflow.
        assert_eq!(
            recover_discrete_latent_mean_with_time_independent_predictor(
                1.0,
                -0.5,
                0.3,
                1e308,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn discrete_observed_mean_with_time_independent_predictor_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-tipred-mean");
        let composed = recover_discrete_latent_mean_with_time_independent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-tipred");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        assert!((impulse_observed - recovered).abs() > 1e-3);
        assert!((carried_observed - recovered).abs() > 1e-3);
        assert_eq!(
            recover_discrete_observed_mean_with_time_independent_predictor(
                0.0,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
    }

    #[test]
    fn discrete_observed_mean_with_time_independent_predictor_is_not_evolved_or_zero_increment() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-tipred-mean");
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let zero_increment = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            0.0,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-increment");
        assert!((zero_increment - evolved_observed).abs() < 1e-15);
        assert!((recovered - evolved_observed).abs() > 1e-3);
    }

    #[test]
    fn discrete_observed_mean_with_time_independent_predictor_refuses_evolved_mean_and_overflow() {
        let loading = 2.0_f64;
        let recovered = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-tipred-mean");
        let evolved_observed =
            recover_discrete_observed_mean(loading, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("eq3-eq5-mean");
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert_eq!(
            refuse_evolved_observed_mean_as_time_independent_observed_mean(
                evolved_observed,
                recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean)
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_time_independent_observed_mean(
                impulse_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean)
        );
        assert_eq!(
            refuse_impulse_carry_observed_mean_as_time_independent_observed_mean(
                carried_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean)
        );
    }

    #[test]
    fn discrete_observed_mean_with_time_independent_predictor_invalid_inputs_fail_closed() {
        let scaled = recover_discrete_observed_mean_with_time_independent_predictor(
            1e308,
            1e-308,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded = recover_discrete_observed_mean_with_time_independent_predictor(
            1e308,
            0.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("lambda-mu0");
        assert!((finite_loaded - 0.0).abs() < 1e-15);
        assert_eq!(
            recover_discrete_observed_mean_with_time_independent_predictor(
                1e308,
                2.0,
                0.0,
                0.0,
                0.0,
                3.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_time_independent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_time_independent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_time_independent_predictor(
                1e308,
                0.0,
                0.0,
                0.0,
                1e308,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn time_dependent_impulse_carry_recovers_driver_equation_one_two_dissipation() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let carry = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            drift,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("tdpred-carry");
        let expected = (-0.5_f64).exp() * 1.2;
        assert!((carry - expected).abs() < 1e-15);
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.0,
                predictor,
                drift,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                effect,
                0.0,
                drift,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let zero_drift = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            0.0,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("zero-drift");
        assert!((zero_drift - 1.2).abs() < 1e-15);
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        assert!((carry - impulse).abs() > 1e-3);
        let vanished = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            -800.0,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("vanish");
        assert_eq!(vanished.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn time_dependent_impulse_carry_composes_evolved_mean_and_keeps_scale() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let carry = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            drift,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("tdpred-carry");
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let composed = recover_discrete_latent_mean_with_impulse_carry(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("eq3-carry");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        assert!((composed - (evolved + carry)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean_with_impulse_carry(
                initial,
                drift,
                intercept,
                0.0,
                predictor,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_impulse_carry(
                0.0,
                drift,
                0.0,
                effect,
                predictor,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(carry)
        );
        let scaled = recover_time_dependent_predictor_impulse_carry(
            1e308,
            1e-308,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let rewritten = recover_time_dependent_predictor_impulse_carry(
            1e-308,
            1.0,
            710.0,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("rewrite");
        let expected_rewrite = (1e-308_f64.ln() + 710.0).exp();
        assert!((rewritten - expected_rewrite).abs() <= expected_rewrite * 1e-12);
    }

    #[test]
    fn discrete_observed_mean_with_impulse_carry_recovers_driver_equation_five() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        let carried = recover_discrete_latent_mean_with_impulse_carry(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("carried");
        let expected = manifest_mean + loading * carried;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                0.0,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                loading,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                0.0,
                delta,
                elapsed,
                LagClock::EventTime
            ),
            Ok(loading * carried)
        );
    }

    #[test]
    fn discrete_observed_mean_with_impulse_carry_is_not_contemporaneous_or_zero_carry() {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        let contemporaneous = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-mx");
        assert!((contemporaneous - recovered).abs() > 1e-3);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let zero_carry = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            0.0,
            predictor,
            manifest_mean,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("zero-carry");
        assert!((zero_carry - evolved_observed).abs() < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_with_impulse_carry_refuses_evolved_mean_and_overflow() {
        let loading = 2.0_f64;
        let recovered = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        let carried = recover_discrete_latent_mean_with_impulse_carry(
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("carried");
        let evolved_observed =
            recover_discrete_observed_mean(loading, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("eq3-eq5-mean");
        assert_eq!(
            refuse_evolved_observed_mean_as_impulse_carry_observed_mean(
                evolved_observed,
                recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean)
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_impulse_carry_observed_mean(
                recover_discrete_observed_mean_with_impulse(
                    loading,
                    1.0,
                    -0.5,
                    0.3,
                    0.4,
                    3.0,
                    0.5,
                    2.0,
                    LagClock::EventTime,
                )
                .expect("eq5-mx"),
                recovered
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean)
        );
        assert_eq!(
            refuse_latent_mean_as_observed_mean(carried, recovered),
            Err(PsychometricError::LatentMeanIsNotObservedMean)
        );
        assert_eq!(
            refuse_manifest_means_as_observed_mean(0.5, recovered),
            Err(PsychometricError::ManifestMeansIsNotObservedMean)
        );
        let scaled = recover_discrete_observed_mean_with_impulse_carry(
            1e308,
            1e-308,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded = recover_discrete_observed_mean_with_impulse_carry(
            1e308,
            1.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("lambda-mu");
        assert!((finite_loaded - 1e308).abs() / 1e308 < 1e-15);
    }

    #[test]
    fn discrete_observed_mean_with_impulse_carry_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                f64::NAN,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                1e308,
                2.0,
                0.0,
                0.0,
                0.0,
                3.0,
                0.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                1.0,
                1.0,
                710.0,
                0.0,
                0.0,
                3.0,
                0.5,
                1.0,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                1e308,
                0.0,
                0.0,
                0.0,
                1e308,
                1.0,
                0.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn discrete_observed_mean_with_impulse_carry_interval_and_clock_fail_closed() {
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_impulse_carry(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
    }

    #[test]
    fn time_dependent_impulse_carry_refuses_contemporaneous_cint_tipred_and_equation_fourteen() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let elapsed = 1.0_f64;
        let carry = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            drift,
            delta,
            elapsed,
            LagClock::EventTime,
        )
        .expect("tdpred-carry");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let intercept_effect =
            recover_discrete_continuous_intercept_effect(effect, drift, delta, LagClock::EventTime)
                .expect("cint");
        let time_independent = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        let equation_fourteen = recover_discrete_time_varying_predictor_effect(
            effect,
            delta,
            delta,
            delta,
            LagClock::EventTime,
        )
        .expect("eq14");
        assert!((carry - impulse).abs() > 1e-3);
        assert!((carry - intercept_effect).abs() > 1e-3);
        assert!((carry - time_independent).abs() > 1e-3);
        assert!((carry - equation_fourteen).abs() > 1e-3);
        assert_eq!(
            refuse_time_dependent_impulse_carry_as_contemporaneous_impulse(carry, impulse),
            Err(PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse)
        );
        assert_eq!(
            refuse_time_dependent_impulse_carry_as_continuous_intercept(carry, effect),
            Err(PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_time_dependent_impulse_carry_as_time_independent_effect(carry, time_independent),
            Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect)
        );
        assert_eq!(
            refuse_time_dependent_impulse_carry_as_time_varying_discrete_effect(
                carry,
                equation_fourteen
            ),
            Err(PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect)
        );
    }

    #[test]
    fn time_dependent_impulse_carry_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                f64::NAN,
                1.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                f64::INFINITY,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                1e308,
                2.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                1.2,
                1.0,
                800.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        // Finite log-rate whose product with elapsed overflows. exp(±∞)
        // is not finite, then the non-finite drift interval fails closed.
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                1e308,
                3.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.0,
                3.0,
                800.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                -1e-308,
                1.0,
                710.0,
                2.0,
                1.0,
                LagClock::EventTime
            )
            .map(f64::signum),
            Ok(-1.0)
        );
    }

    #[test]
    fn time_dependent_impulse_carry_interval_and_clock_fail_closed() {
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                -0.5,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                -0.5,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                -0.5,
                2.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_time_dependent_predictor_impulse_carry(
                0.4,
                3.0,
                -0.5,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_impulse_carry(
                1e308,
                0.0,
                0.0,
                1e308,
                1.0,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn initial_time_independent_predictor_recovers_table_three_t0_shift_and_carry() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let shift = recover_initial_time_independent_predictor_effect(effect, predictor)
            .expect("t0-tipred");
        assert!((shift - 1.2).abs() < 1e-15);
        assert_eq!(
            recover_initial_time_independent_predictor_effect(0.0, predictor),
            Ok(0.0)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_effect(effect, 0.0),
            Ok(0.0)
        );
        let carry = recover_initial_time_independent_predictor_carry(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("t0-carry");
        let expected = 1.2 * (drift * delta).exp();
        assert!((carry - expected).abs() < 1e-15);
        let zero_drift = recover_initial_time_independent_predictor_carry(
            effect,
            predictor,
            0.0,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-drift");
        assert!((zero_drift - 1.2).abs() < 1e-15);
        let vanished = recover_initial_time_independent_predictor_carry(
            effect,
            predictor,
            -800.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("underflow");
        assert_eq!(vanished.to_bits(), 0.0_f64.to_bits());
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        assert!((carry - shift).abs() > 1e-3);
        assert!((carry - increment).abs() > 1e-3);
        assert!((shift - increment).abs() > 1e-3);
        assert!((shift - effect).abs() > 1e-3);
        // Algebraically a product, like M x, but Table 3 names a different matrix.
        assert!((shift - impulse).abs() < 1e-15);
    }

    #[test]
    fn initial_time_independent_predictor_composes_evolved_mean_and_keeps_scale() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let carry = recover_initial_time_independent_predictor_carry(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("t0-carry");
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let composed = recover_discrete_latent_mean_with_initial_time_independent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-t0tipred");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        assert!((composed - (evolved + carry)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_independent_predictor(
                initial,
                drift,
                intercept,
                0.0,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_independent_predictor(
                0.0,
                drift,
                0.0,
                effect,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(carry)
        );
        let scaled = recover_initial_time_independent_predictor_carry(
            1e308,
            1e-308,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let rewritten = recover_initial_time_independent_predictor_carry(
            2.0,
            0.5,
            710.0,
            1.0,
            LagClock::EventTime,
        );
        assert_eq!(rewritten, Err(PsychometricError::InvalidNumericInput));
        let finite_rewrite = recover_initial_time_independent_predictor_carry(
            1e-308,
            1.0,
            700.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("log-rewrite");
        let expected_rewrite = (1e-308_f64.ln() + 700.0).exp();
        assert!((finite_rewrite - expected_rewrite).abs() / expected_rewrite < 1e-12);
    }

    #[test]
    fn initial_time_independent_predictor_refuses_process_increment_cint_impulse_and_coefficient() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let shift = recover_initial_time_independent_predictor_effect(effect, predictor)
            .expect("t0-tipred");
        let carry = recover_initial_time_independent_predictor_carry(
            effect,
            predictor,
            -0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("t0-carry");
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            -0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("tipred");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        assert_eq!(
            refuse_initial_time_independent_effect_as_process_increment(shift, increment),
            Err(PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement)
        );
        assert_eq!(
            refuse_initial_time_independent_carry_as_initial_effect(carry, shift),
            Err(PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect)
        );
        assert_eq!(
            refuse_initial_time_independent_effect_as_continuous_intercept(shift, 0.4),
            Err(PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_initial_time_independent_effect_as_time_dependent_impulse(shift, impulse),
            Err(PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse)
        );
        assert_eq!(
            refuse_initial_time_independent_coefficient_as_initial_effect(effect, shift),
            Err(PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect)
        );
    }

    #[test]
    fn initial_time_independent_predictor_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_initial_time_independent_predictor_effect(f64::NAN, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_effect(1.0, f64::INFINITY),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_effect(1e308, 2.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                0.4,
                3.0,
                f64::NAN,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                0.4,
                3.0,
                -0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                0.4,
                3.0,
                -0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_independent_predictor(
                1e308,
                0.0,
                0.0,
                1e308,
                1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                1.0,
                1.0,
                f64::INFINITY,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                f64::NAN,
                1.0,
                -0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_carry(
                1.0,
                1.0,
                1e308,
                10.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_independent_predictor(
                1.0,
                -0.5,
                0.3,
                f64::NAN,
                1.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_independent_predictor_recovers_driver_equation_five()
     {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-t0tipred-mean");
        let composed = recover_discrete_latent_mean_with_initial_time_independent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-t0tipred");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let process_observed = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-tipred-mean");
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert!((evolved_observed - recovered).abs() > 1e-3);
        assert!((process_observed - recovered).abs() > 1e-3);
        assert!((impulse_observed - recovered).abs() > 1e-3);
        assert!((carried_observed - recovered).abs() > 1e-3);
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                0.0,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_independent_predictor_is_not_evolved_or_zero_carry()
    {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-t0tipred-mean");
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let zero_carry = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            0.0,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-carry");
        assert!((zero_carry - evolved_observed).abs() < 1e-15);
        assert!((recovered - evolved_observed).abs() > 1e-3);
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_independent_predictor_refuses_evolved_mean_and_overflow()
     {
        let loading = 2.0_f64;
        let recovered = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-t0tipred-mean");
        let evolved_observed =
            recover_discrete_observed_mean(loading, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("eq3-eq5-mean");
        let process_observed = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-tipred-mean");
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-impulse-mean");
        let carried_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry-mean");
        assert_eq!(
            refuse_evolved_observed_mean_as_initial_time_independent_observed_mean(
                evolved_observed,
                recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean)
        );
        assert_eq!(
            refuse_time_independent_observed_mean_as_initial_time_independent_observed_mean(
                process_observed,
                recovered
            ),
            Err(
                PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean
            )
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_initial_time_independent_observed_mean(
                impulse_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean)
        );
        assert_eq!(
            refuse_impulse_carry_observed_mean_as_initial_time_independent_observed_mean(
                carried_observed,
                recovered
            ),
            Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean)
        );
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_independent_predictor_invalid_inputs_fail_closed() {
        let scaled = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            1e308,
            1e-308,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let finite_loaded = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            1e308,
            0.0,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("lambda-mu0");
        assert!((finite_loaded - 0.0).abs() < 1e-15);
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                1e308,
                2.0,
                0.0,
                0.0,
                0.0,
                3.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                1e308,
                0.0,
                0.0,
                0.0,
                1e308,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn initial_time_dependent_predictor_recovers_table_three_t0_shift_and_carry() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let shift =
            recover_initial_time_dependent_predictor_effect(effect, predictor).expect("t0-tdpred");
        assert!((shift - 1.2).abs() < 1e-15);
        assert_eq!(
            recover_initial_time_dependent_predictor_effect(0.0, predictor),
            Ok(0.0)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_effect(effect, 0.0),
            Ok(0.0)
        );
        let carry = recover_initial_time_dependent_predictor_carry(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("t0-td-carry");
        let expected = 1.2 * (drift * delta).exp();
        assert!((carry - expected).abs() < 1e-15);
        let zero_drift = recover_initial_time_dependent_predictor_carry(
            effect,
            predictor,
            0.0,
            delta,
            LagClock::EventTime,
        )
        .expect("zero-drift");
        assert!((zero_drift - 1.2).abs() < 1e-15);
        let vanished = recover_initial_time_dependent_predictor_carry(
            effect,
            predictor,
            -800.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("underflow");
        assert_eq!(vanished.to_bits(), 0.0_f64.to_bits());
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("tipred");
        let tipred_shift = recover_initial_time_independent_predictor_effect(effect, predictor)
            .expect("t0-tipred");
        let impulse_carry = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            drift,
            delta,
            1.0,
            LagClock::EventTime,
        )
        .expect("td-carry");
        assert!((carry - shift).abs() > 1e-3);
        assert!((carry - increment).abs() > 1e-3);
        assert!((shift - increment).abs() > 1e-3);
        assert!((shift - effect).abs() > 1e-3);
        assert!((carry - impulse_carry).abs() > 1e-3);
        // Algebraically a product, like M x and t0_b z, but Table 3 names a different matrix.
        assert!((shift - impulse).abs() < 1e-15);
        assert!((shift - tipred_shift).abs() < 1e-15);
    }

    #[test]
    fn initial_time_dependent_predictor_composes_evolved_mean_and_keeps_scale() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let carry = recover_initial_time_dependent_predictor_carry(
            effect,
            predictor,
            drift,
            delta,
            LagClock::EventTime,
        )
        .expect("t0-td-carry");
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let composed = recover_discrete_latent_mean_with_initial_time_dependent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-t0tdpred");
        let evolved =
            recover_discrete_latent_mean(initial, drift, intercept, delta, LagClock::EventTime)
                .expect("mu-t");
        assert!((composed - (evolved + carry)).abs() < 1e-15);
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_dependent_predictor(
                initial,
                drift,
                intercept,
                0.0,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(evolved)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_dependent_predictor(
                0.0,
                drift,
                0.0,
                effect,
                predictor,
                delta,
                LagClock::EventTime
            ),
            Ok(carry)
        );
        let scaled = recover_initial_time_dependent_predictor_carry(
            1e308,
            1e-308,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        let rewritten = recover_initial_time_dependent_predictor_carry(
            2.0,
            0.5,
            710.0,
            1.0,
            LagClock::EventTime,
        );
        assert_eq!(rewritten, Err(PsychometricError::InvalidNumericInput));
        let finite_rewrite = recover_initial_time_dependent_predictor_carry(
            1e-308,
            1.0,
            700.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("log-rewrite");
        let expected_rewrite = (1e-308_f64.ln() + 700.0).exp();
        assert!((finite_rewrite - expected_rewrite).abs() / expected_rewrite < 1e-12);
    }

    #[test]
    fn initial_time_dependent_predictor_refuses_impulse_cint_process_and_coefficient() {
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let shift =
            recover_initial_time_dependent_predictor_effect(effect, predictor).expect("t0-tdpred");
        let carry = recover_initial_time_dependent_predictor_carry(
            effect,
            predictor,
            -0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("t0-td-carry");
        let increment = recover_discrete_time_independent_predictor_effect(
            effect,
            predictor,
            -0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("tipred");
        let impulse = recover_time_dependent_predictor_impulse(effect, predictor).expect("tdpred");
        let tipred_shift = recover_initial_time_independent_predictor_effect(effect, predictor)
            .expect("t0-tipred");
        let impulse_carry = recover_time_dependent_predictor_impulse_carry(
            effect,
            predictor,
            -0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("td-carry");
        assert_eq!(
            refuse_initial_time_dependent_effect_as_contemporaneous_impulse(shift, impulse),
            Err(PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse)
        );
        assert_eq!(
            refuse_initial_time_dependent_carry_as_initial_effect(carry, shift),
            Err(PsychometricError::InitialTimeDependentCarryIsNotInitialEffect)
        );
        assert_eq!(
            refuse_initial_time_dependent_effect_as_continuous_intercept(shift, 0.4),
            Err(PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept)
        );
        assert_eq!(
            refuse_initial_time_dependent_effect_as_process_increment(shift, increment),
            Err(PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement)
        );
        assert_eq!(
            refuse_initial_time_dependent_effect_as_initial_time_independent_effect(
                shift,
                tipred_shift
            ),
            Err(PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect)
        );
        assert_eq!(
            refuse_initial_time_dependent_coefficient_as_initial_effect(effect, shift),
            Err(PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect)
        );
        assert_eq!(
            refuse_initial_time_dependent_carry_as_impulse_carry(carry, impulse_carry),
            Err(PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry)
        );
    }

    #[test]
    fn initial_time_dependent_predictor_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_initial_time_dependent_predictor_effect(f64::NAN, 1.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_effect(1.0, f64::INFINITY),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_effect(1e308, 2.0),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                0.4,
                3.0,
                f64::NAN,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                0.4,
                3.0,
                -0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                0.4,
                3.0,
                -0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_dependent_predictor(
                1e308,
                0.0,
                0.0,
                1e308,
                1.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                1.0,
                1.0,
                f64::INFINITY,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                f64::NAN,
                1.0,
                -0.5,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_carry(
                1.0,
                1.0,
                1e308,
                10.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_latent_mean_with_initial_time_dependent_predictor(
                1.0,
                -0.5,
                0.3,
                f64::NAN,
                1.0,
                2.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn discrete_observed_mean_with_initial_time_dependent_predictor_recovers_driver_equation_five()
    {
        let loading = 2.0_f64;
        let drift = -0.5_f64;
        let delta = 2.0_f64;
        let effect = 0.4_f64;
        let predictor = 3.0_f64;
        let initial = 1.0_f64;
        let intercept = 0.3_f64;
        let manifest_mean = 0.5_f64;
        let recovered = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-t0tdpred-mean");
        let composed = recover_discrete_latent_mean_with_initial_time_dependent_predictor(
            initial,
            drift,
            intercept,
            effect,
            predictor,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-t0tdpred");
        let expected = manifest_mean + loading * composed;
        assert!((recovered - expected).abs() < 1e-15);
        let evolved_observed = recover_discrete_observed_mean(
            loading,
            initial,
            drift,
            intercept,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq3-eq5-mean");
        let process_observed = recover_discrete_observed_mean_with_time_independent_predictor(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-tipred");
        let impulse_observed = recover_discrete_observed_mean_with_impulse(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            LagClock::EventTime,
        )
        .expect("eq5-impulse");
        let carry_observed = recover_discrete_observed_mean_with_impulse_carry(
            loading,
            initial,
            drift,
            intercept,
            effect,
            predictor,
            manifest_mean,
            delta,
            1.0,
            LagClock::EventTime,
        )
        .expect("eq5-carry");
        let tipred_observed =
            recover_discrete_observed_mean_with_initial_time_independent_predictor(
                loading,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                LagClock::EventTime,
            )
            .expect("eq5-t0tipred");
        assert!((recovered - evolved_observed).abs() > 1e-3);
        assert!((recovered - process_observed).abs() > 1e-3);
        assert!((recovered - impulse_observed).abs() > 1e-3);
        assert!((recovered - carry_observed).abs() > 1e-3);
        // Same numbers as T0TIPRED yield the same product, but Table 3 names a different matrix.
        assert!((recovered - tipred_observed).abs() < 1e-15);
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_dependent_predictor(
                0.0,
                initial,
                drift,
                intercept,
                effect,
                predictor,
                manifest_mean,
                delta,
                LagClock::EventTime
            ),
            Ok(manifest_mean)
        );
        assert!((recovered - composed).abs() > 1e-3);
        assert!((recovered - manifest_mean).abs() > 1e-3);
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_dependent_predictor_refuses_evolved_mean_and_overflow()
     {
        let recovered = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("eq5-t0tdpred");
        let evolved =
            recover_discrete_observed_mean(2.0, 1.0, -0.5, 0.3, 0.5, 2.0, LagClock::EventTime)
                .expect("evolved");
        let process = recover_discrete_observed_mean_with_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("tipred");
        let impulse = recover_discrete_observed_mean_with_impulse(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("impulse");
        let carry = recover_discrete_observed_mean_with_impulse_carry(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("carry");
        let tipred = recover_discrete_observed_mean_with_initial_time_independent_predictor(
            2.0,
            1.0,
            -0.5,
            0.3,
            0.4,
            3.0,
            0.5,
            2.0,
            LagClock::EventTime,
        )
        .expect("t0tipred");
        assert_eq!(
            refuse_evolved_observed_mean_as_initial_time_dependent_observed_mean(
                evolved, recovered
            ),
            Err(PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean)
        );
        assert_eq!(
            refuse_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
                process, recovered
            ),
            Err(
                PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean
            )
        );
        assert_eq!(
            refuse_impulse_observed_mean_as_initial_time_dependent_observed_mean(
                impulse, recovered
            ),
            Err(PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean)
        );
        assert_eq!(
            refuse_impulse_carry_observed_mean_as_initial_time_dependent_observed_mean(
                carry, recovered
            ),
            Err(PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean)
        );
        assert_eq!(
            refuse_initial_time_independent_observed_mean_as_initial_time_dependent_observed_mean(
                tipred, recovered
            ),
            Err(PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean)
        );
    }

    #[test]
    fn discrete_observed_mean_with_initial_time_dependent_predictor_invalid_inputs_fail_closed() {
        let scaled = recover_discrete_observed_mean_with_initial_time_dependent_predictor(
            1e308,
            1e-308,
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!((scaled - 1.0).abs() < 1e-15);
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_dependent_predictor(
                1e308,
                2.0,
                0.0,
                0.0,
                0.0,
                3.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_dependent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                2.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_dependent_predictor(
                2.0,
                1.0,
                -0.5,
                0.3,
                0.4,
                3.0,
                0.5,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_discrete_observed_mean_with_initial_time_dependent_predictor(
                1e308,
                0.0,
                0.0,
                0.0,
                1e308,
                1.0,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_initial_latent_variance_recovers_driver_section_four_point_three() {
        // Driver et al. (2017, §4.3): Var(η_0) =
        // trait + p_0 + (B / a)² v.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_initial_latent_variance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("predetermined initial T0VAR");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((recovered - (trait_variance + initial_latent_variance + added)).abs() < 1e-12);
        let stationary = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary initial T0VAR");
        assert!((recovered - stationary).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_initial_latent_variance(
            trait_variance,
            state,
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary).abs() < 1e-12);
        let lagged = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        assert!((recovered - lagged).abs() > 1e-3);
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        assert!((recovered - later).abs() > 1e-3);
        assert!((recovered - initial_latent_variance).abs() > 1e-3);
        let near_lagged = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+ lagged");
        assert!((near_lagged - recovered).abs() < 1e-9);
        let near_later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            LagClock::EventTime,
        )
        .expect("Δt→0+ later");
        assert!((near_later - recovered).abs() < 1e-9);
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let trait_only = recover_predetermined_initial_latent_variance(
            trait_variance,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        )
        .expect("trait-only predetermined initial");
        assert!((trait_only - trait_variance).abs() < 1e-15);
        let unstable_trait = recover_predetermined_initial_latent_variance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            0.5,
            LagClock::EventTime,
        )
        .expect("trait-only a≥0");
        assert!((unstable_trait - trait_variance).abs() < 1e-15);
    }

    #[test]
    fn predetermined_initial_latent_variance_is_not_stationary_lagged_or_later() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_initial_latent_variance(
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("predetermined initial T0VAR");
        let stationary = recover_stationary_initial_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("stationary initial T0VAR");
        let lagged = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined lagged T0VAR");
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later T0VAR");
        assert_eq!(
            refuse_predetermined_initial_latent_variance_as_stationary_initial_latent_variance(
                recovered, stationary
            ),
            Err(
                PsychometricError::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance
            )
        );
        assert_eq!(
            refuse_predetermined_initial_latent_variance_as_initial_latent_variance(
                recovered,
                initial_latent_variance
            ),
            Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance)
        );
        assert_eq!(
            refuse_predetermined_initial_latent_variance_as_lagged_latent_covariance(
                recovered, lagged
            ),
            Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance)
        );
        assert_eq!(
            refuse_predetermined_initial_latent_variance_as_later_latent_variance(recovered, later),
            Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance)
        );
    }

    #[test]
    fn predetermined_initial_latent_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let brownian = recover_predetermined_initial_latent_variance(
            0.0,
            2.0,
            0.0,
            0.0,
            0.0,
            LagClock::EventTime,
        )
        .expect("Brownian a=0");
        assert!((brownian - 2.0).abs() < 1e-12);
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                f64::NAN,
                2.0,
                0.0,
                0.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_initial_latent_variance(
                f64::MAX,
                0.0,
                1.0,
                f64::MAX,
                -1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_initial_observed_variance_recovers_driver_equation_five() {
        // Driver et al. (2017, Eq. 5 of predetermined T0VAR):
        // λ²(trait + p_0 + (B / a)² v) + θ + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_predetermined_initial_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-initial-predetermined-T0VAR");
        let latent = recover_predetermined_initial_latent_variance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("predetermined initial T0VAR");
        let expected = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("λ²p+θ+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let stationary = recover_stationary_initial_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-initial-stationary-T0VAR");
        assert!((recovered - stationary).abs() > 1e-3);
        let later = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            event_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-predetermined-T0VAR");
        assert!((recovered - later).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                0.0,
                trait_variance,
                initial_latent_variance,
                printed_effect,
                1.0,
                log_rate,
                measurement_error,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(measurement_error + manifest_trait)
        );
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                loading,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        assert_eq!(
            refuse_predetermined_initial_latent_variance_as_observed_variance(latent, recovered),
            Err(PsychometricError::PredeterminedInitialLatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            refuse_measurement_error_as_predetermined_initial_observed_variance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotPredeterminedInitialObservedVariance)
        );
        assert_eq!(
            refuse_stationary_initial_observed_variance_as_predetermined_initial_observed_variance(
                stationary, recovered
            ),
            Err(
                PsychometricError::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_observed_variance_as_predetermined_initial_observed_variance(
                later, recovered
            ),
            Err(
                PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance
            )
        );
    }

    #[test]
    fn predetermined_initial_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                2.0,
                1.0,
                2.0,
                -0.225,
                1.0,
                -0.13,
                0.5,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                2.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        let brownian = recover_predetermined_initial_observed_variance(
            1.0,
            0.0,
            2.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            LagClock::EventTime,
        )
        .expect("Brownian a=0");
        assert!((brownian - 2.0).abs() < 1e-12);
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.6)
        );
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                2.0,
                f64::NAN,
                2.0,
                0.0,
                0.0,
                -0.5,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_initial_observed_variance(
                2.0,
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                -0.5,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_lagged_latent_covariance_recovers_driver_equation_four_after_startoffset()
     {
        // Driver et al. (2017, §4.3 startoffset; Eq. 4):
        // cov = trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later-start lagged T0VAR");
        let later_state = recover_discrete_latent_variance(
            initial_latent_variance,
            diffusion,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later state");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let expected = recover_trait_plus_state_lagged_covariance(
            trait_variance,
            later_state,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("trait + e^{as} later-state")
            + added;
        assert!((recovered - expected).abs() < 1e-12);
        let first_lagged = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            printed_effect,
            predictor_variance,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("first-occasion lagged");
        assert!((recovered - first_lagged).abs() > 1e-3);
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later variance");
        assert!((recovered - later).abs() > 1e-3);
        let stationary_lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged");
        assert!((recovered - stationary_lagged).abs() > 1e-3);
        let decayed_later = later * lag_delta.mul_add(log_rate, 0.0).exp();
        assert!((recovered - decayed_later).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            state,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary_lagged).abs() < 1e-12);
        let near_first = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("u→0+");
        assert!((near_first - first_lagged).abs() < 1e-9);
        let near_later = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            1e-12,
            LagClock::EventTime,
        )
        .expect("s→0+");
        assert!((near_later - later).abs() < 1e-9);
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                start_delta,
                lag_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let trait_only = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("trait-only later-start lagged");
        assert!((trait_only - trait_variance).abs() < 1e-15);
    }

    #[test]
    fn predetermined_later_lagged_latent_covariance_is_not_first_later_or_stationary() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later-start lagged T0VAR");
        let first_lagged = recover_predetermined_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            -0.225,
            1.0,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("first-occasion lagged");
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later variance");
        let stationary_lagged = recover_stationary_lagged_latent_covariance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("stationary lagged");
        let decayed_later = later * lag_delta.mul_add(log_rate, 0.0).exp();
        assert_eq!(
            refuse_predetermined_later_lagged_latent_covariance_as_predetermined_lagged_covariance(
                recovered, first_lagged
            ),
            Err(
                PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_lagged_latent_covariance_as_later_latent_variance(
                recovered, later
            ),
            Err(
                PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_lagged_latent_covariance_as_stationary_lagged_covariance(
                recovered,
                stationary_lagged
            ),
            Err(
                PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_lagged_latent_covariance_as_decayed_later_total(
                recovered,
                decayed_later
            ),
            Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal)
        );
    }

    #[test]
    fn predetermined_later_lagged_latent_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        let growing = recover_predetermined_later_lagged_latent_covariance(
            0.0,
            2.0,
            0.4,
            0.0,
            0.0,
            0.5,
            1.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("growing a>0");
        assert!(growing.is_finite() && growing > 2.0);
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                f64::NAN,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_later_lagged_latent_covariance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_lagged_observed_covariance_recovers_driver_equation_five() {
        // Driver et al. (2017, Eq. 5 of later-start lagged T0VAR):
        // λ²(trait + e^{a s}(e^{2 a u} p_0 + Q_u) + (B / a)² v) + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_lagged_observed_covariance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-start-lagged-predetermined-T0VAR");
        let latent = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later-start lagged T0VAR");
        let expected = recover_manifest_lagged_observed_covariance(loading, latent, manifest_trait)
            .expect("λ²c+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let first_lagged = recover_predetermined_lagged_observed_covariance(
            loading,
            trait_variance,
            initial_latent_variance,
            printed_effect,
            1.0,
            log_rate,
            lag_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-first-lagged");
        assert!((recovered - first_lagged).abs() > 1e-3);
        let later = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later");
        assert!((recovered - later).abs() > 1e-3);
        let stationary = recover_stationary_lagged_observed_covariance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            lag_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-lagged");
        assert!((recovered - stationary).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                0.0,
                trait_variance,
                initial_latent_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                start_delta,
                lag_delta,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(manifest_trait)
        );
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                loading,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                start_delta,
                lag_delta,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        assert_eq!(
            refuse_predetermined_later_lagged_latent_covariance_as_observed_covariance(
                latent, recovered
            ),
            Err(PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance)
        );
        assert_eq!(
            refuse_measurement_error_as_predetermined_later_lagged_observed_covariance(
                measurement_error,
                recovered
            ),
            Err(PsychometricError::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance)
        );
        assert_eq!(
            refuse_predetermined_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
                first_lagged, recovered
            ),
            Err(
                PsychometricError::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
            )
        );
        assert_eq!(
            refuse_stationary_lagged_observed_covariance_as_predetermined_later_lagged_observed_covariance(
                stationary, recovered
            ),
            Err(
                PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_observed_variance_as_predetermined_later_lagged_observed_covariance(
                later, recovered
            ),
            Err(
                PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance
            )
        );
    }

    #[test]
    fn predetermined_later_lagged_observed_covariance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                2.0,
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                1.0,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                2.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                2.0,
                1.0,
                2.0,
                0.4,
                0.0,
                1.0,
                -0.5,
                0.0,
                1.0,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                2.0,
                1.0,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.1)
        );
        assert_eq!(
            recover_predetermined_later_lagged_observed_covariance(
                2.0,
                f64::NAN,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_start_later_latent_variance_recovers_driver_equation_four_after_startoffset()
     {
        // Driver et al. (2017, §4.3 startoffset; Eq. 3–4 Chapman–Kolmogorov):
        // Var = trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let predictor_variance = 1.0_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later-start later T0VAR");
        let later_state = recover_discrete_latent_variance(
            initial_latent_variance,
            diffusion,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later state");
        let evolved_state = recover_discrete_latent_variance(
            later_state,
            diffusion,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("evolved later state");
        let added = recover_asymptotic_time_independent_predictor_variance(
            printed_effect,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let expected = recover_trait_plus_state_latent_variance(trait_variance, evolved_state)
            .expect("trait + evolved later-state")
            + added;
        assert!((recovered - expected).abs() < 1e-12);
        let later_full = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta + lag_delta,
            LagClock::EventTime,
        )
        .expect("later over u+s");
        assert!((recovered - later_full).abs() < 1e-12);
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later variance at u");
        assert!((recovered - later).abs() > 1e-3);
        let later_lagged = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later-start lagged");
        assert!((recovered - later_lagged).abs() > 1e-3);
        let stationary_later = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("stationary later");
        assert!((recovered - stationary_later).abs() > 1e-3);
        let decayed_later = recover_discrete_latent_variance(
            later,
            diffusion,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("evolved later total");
        assert!((recovered - decayed_later).abs() > 1e-3);
        let lag_interval = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later over s only");
        assert!((recovered - lag_interval).abs() > 1e-3);
        let state = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        let from_stationary_start = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            state,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("p_0=−q/(2a)");
        assert!((from_stationary_start - stationary_later).abs() < 1e-12);
        let near_later = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            start_delta,
            1e-12,
            LagClock::EventTime,
        )
        .expect("s→0+");
        assert!((near_later - later).abs() < 1e-9);
        let later_over_s = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later over s");
        let near_first = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            predictor_variance,
            log_rate,
            1e-12,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("u→0+");
        assert!((near_first - later_over_s).abs() < 1e-9);
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                0.0,
                0.0,
                0.0,
                0.0,
                predictor_variance,
                0.0,
                start_delta,
                lag_delta,
                LagClock::EventTime
            ),
            Ok(0.0)
        );
        let trait_only = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            0.0,
            0.0,
            0.0,
            predictor_variance,
            0.0,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("trait-only later-start later");
        assert!((trait_only - trait_variance).abs() < 1e-15);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_start_later_latent_variance_is_not_later_lagged_or_stationary() {
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let log_rate = -0.134_488_942_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("predetermined later-start later T0VAR");
        let later = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            start_delta,
            LagClock::EventTime,
        )
        .expect("later variance");
        let later_lagged = recover_predetermined_later_lagged_latent_covariance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later-start lagged");
        let stationary_later = recover_stationary_later_latent_variance(
            trait_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("stationary later");
        let decayed_later = recover_discrete_latent_variance(
            later,
            diffusion,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("evolved later total");
        let lag_interval = recover_predetermined_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            -0.225,
            1.0,
            log_rate,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later over s only");
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_later_latent_variance(
                recovered, later
            ),
            Err(
                PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_later_lagged_covariance(
                recovered,
                later_lagged
            ),
            Err(
                PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_stationary_later_latent_variance(
                recovered,
                stationary_later
            ),
            Err(
                PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_decayed_later_total(
                recovered,
                decayed_later
            ),
            Err(
                PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal
            )
        );
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_lag_interval_later_latent_variance(
                recovered,
                lag_interval
            ),
            Err(
                PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance
            )
        );
    }

    #[test]
    fn predetermined_later_start_later_latent_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                1.0,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                0.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        let growing = recover_predetermined_later_start_later_latent_variance(
            0.0,
            2.0,
            0.4,
            0.0,
            0.0,
            0.5,
            1.0,
            1.0,
            LagClock::EventTime,
        )
        .expect("growing a>0");
        assert!(growing.is_finite() && growing > 2.0);
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                f64::NAN,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_predetermined_later_start_later_latent_variance(
                f64::MAX,
                f64::MAX,
                0.0,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn predetermined_later_start_later_observed_variance_recovers_driver_equation_five() {
        // Driver et al. (2017, Eq. 5 of later-start later-occasion T0VAR):
        // λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ.
        let printed_effect = -0.225_f64;
        let printed_asym = -1.673_f64;
        let log_rate = -printed_effect / printed_asym;
        let trait_variance = 1.0_f64;
        let initial_latent_variance = 2.0_f64;
        let diffusion = 0.4_f64;
        let loading = 2.0_f64;
        let measurement_error = 0.5_f64;
        let manifest_trait = 0.1_f64;
        let start_delta = 2.0_f64;
        let lag_delta = 1.0_f64;
        let recovered = recover_predetermined_later_start_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-start-later-predetermined-T0VAR");
        let latent = recover_predetermined_later_start_later_latent_variance(
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            LagClock::EventTime,
        )
        .expect("later-start later T0VAR");
        let expected = recover_manifest_trait_plus_state_observed_variance(
            loading,
            latent,
            measurement_error,
            manifest_trait,
        )
        .expect("λ²p+θ+ψ");
        assert!((recovered - expected).abs() < 1e-12);
        let later = recover_predetermined_later_observed_variance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later");
        assert!((recovered - later).abs() > 1e-3);
        let later_lagged = recover_predetermined_later_lagged_observed_covariance(
            loading,
            trait_variance,
            initial_latent_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            start_delta,
            lag_delta,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-later-start-lagged");
        assert!((recovered - later_lagged).abs() > 1e-3);
        let stationary = recover_stationary_later_observed_variance(
            loading,
            trait_variance,
            diffusion,
            printed_effect,
            1.0,
            log_rate,
            lag_delta,
            measurement_error,
            manifest_trait,
            LagClock::EventTime,
        )
        .expect("eq5-stationary-later");
        assert!((recovered - stationary).abs() > 1e-3);
        assert!((recovered - measurement_error).abs() > 1e-3);
        assert!((recovered - latent).abs() > 1e-3);
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                0.0,
                trait_variance,
                initial_latent_variance,
                diffusion,
                printed_effect,
                1.0,
                log_rate,
                start_delta,
                lag_delta,
                measurement_error,
                manifest_trait,
                LagClock::EventTime,
            ),
            Ok(measurement_error + manifest_trait)
        );
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                loading,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                start_delta,
                lag_delta,
                0.0,
                0.0,
                LagClock::EventTime,
            ),
            Ok(0.0)
        );
        assert_eq!(
            refuse_predetermined_later_start_later_latent_variance_as_observed_variance(
                latent, recovered
            ),
            Err(PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance)
        );
        assert_eq!(
            refuse_measurement_error_as_predetermined_later_start_later_observed_variance(
                measurement_error,
                recovered
            ),
            Err(
                PsychometricError::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_observed_variance_as_predetermined_later_start_later_observed_variance(
                later, recovered
            ),
            Err(
                PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
            )
        );
        assert_eq!(
            refuse_predetermined_later_lagged_observed_covariance_as_predetermined_later_start_later_observed_variance(
                later_lagged, recovered
            ),
            Err(
                PsychometricError::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance
            )
        );
        assert_eq!(
            refuse_stationary_later_observed_variance_as_predetermined_later_start_later_observed_variance(
                stationary, recovered
            ),
            Err(
                PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
            )
        );
    }

    #[test]
    fn predetermined_later_start_later_observed_variance_invalid_inputs_fail_closed() {
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                2.0,
                1.0,
                2.0,
                0.4,
                -0.225,
                1.0,
                -0.13,
                2.0,
                1.0,
                0.5,
                0.1,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                -0.225,
                1.0,
                0.5,
                2.0,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                2.0,
                1.0,
                2.0,
                0.4,
                0.0,
                1.0,
                -0.5,
                0.0,
                1.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                2.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                2.0,
                1.0,
                0.5,
                0.1,
                LagClock::EventTime
            ),
            Ok(0.6)
        );
        assert_eq!(
            recover_predetermined_later_start_later_observed_variance(
                2.0,
                f64::NAN,
                2.0,
                0.4,
                0.0,
                0.0,
                -0.5,
                2.0,
                1.0,
                0.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_discrete_drift_recovers_driver_page_sixteen_after_positive_asymdiffusion() {
        // Driver et al. (2017, p. 16 discreteDRIFTstd; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a), then
        // φ = exp(a Δt). Scalar SD ratio is 1.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_standardised_discrete_drift(
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("discreteDRIFTstd");
        let unstandardised =
            recover_discrete_lag_from_log_rate(log_rate, event_delta, LagClock::EventTime)
                .expect("discreteDRIFT");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        assert!((recovered - unstandardised).abs() < 1e-15);
        assert!((recovered - (log_rate * event_delta).exp()).abs() < 1e-15);
        let two_and_a_half =
            recover_standardised_discrete_drift(diffusion, log_rate, 2.5, LagClock::EventTime)
                .expect("discreteDRIFTstd Δt=2.5");
        assert!((two_and_a_half - (log_rate * 2.5).exp()).abs() < 1e-15);
        assert!((recovered - two_and_a_half).abs() > 1e-9);
        let trait_variance = 1.0_f64;
        let lagged = recover_trait_plus_state_lagged_covariance(
            trait_variance,
            within,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("trait+state lag");
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = lagged / total;
        assert!((contaminated - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_unstandardised_discrete_drift_as_standardised_discrete_drift(
                unstandardised,
                recovered
            ),
            Err(PsychometricError::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift)
        );
        assert_eq!(
            refuse_trait_plus_state_autocorrelation_as_standardised_discrete_drift(
                contaminated,
                recovered
            ),
            Err(PsychometricError::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift)
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_discrete_drift_fails_closed_when_unstandardised_is_defined() {
        let log_rate = -0.5_f64;
        let event_delta = 1.0_f64;
        let unstandardised_zero_q =
            recover_discrete_lag_from_log_rate(log_rate, event_delta, LagClock::EventTime)
                .expect("e^{aΔt} at q=0");
        assert!((unstandardised_zero_q - log_rate.exp()).abs() < 1e-15);
        assert_eq!(
            recover_standardised_discrete_drift(0.0, log_rate, event_delta, LagClock::EventTime),
            Err(PsychometricError::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance)
        );
        let growing = recover_discrete_lag_from_log_rate(0.5, event_delta, LagClock::EventTime)
            .expect("growing a>0");
        assert!(growing.is_finite() && growing > 1.0);
        assert_eq!(
            recover_standardised_discrete_drift(0.4, 0.5, event_delta, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        let unit =
            recover_discrete_lag_from_log_rate(0.0, event_delta, LagClock::EventTime).expect("a=0");
        assert!((unit - 1.0).abs() < 1e-15);
        assert_eq!(
            recover_standardised_discrete_drift(0.4, 0.0, event_delta, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_discrete_drift(0.4, log_rate, event_delta, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_discrete_drift(0.4, log_rate, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_standardised_discrete_drift(0.4, log_rate, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_standardised_discrete_drift(-0.1, log_rate, event_delta, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_discrete_drift(0.4, f64::NAN, event_delta, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_discrete_drift(0.4, -800.0, 1.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_discrete_diffusion_recovers_driver_page_sixteen_after_positive_asymdiffusion() {
        // Driver et al. (2017, p. 16 discreteDIFFUSIONstd; Eq. 4; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a), then
        // Q_Δt / p. Scalar stationary ratio is 1 − exp(2 a Δt).
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let event_delta = 1.0_f64;
        let recovered = recover_standardised_discrete_diffusion(
            diffusion,
            log_rate,
            event_delta,
            LagClock::EventTime,
        )
        .expect("discreteDIFFUSIONstd");
        let process_noise =
            recover_discrete_process_noise(diffusion, log_rate, event_delta, LagClock::EventTime)
                .expect("discreteDIFFUSION");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        assert!((recovered - (process_noise / within)).abs() < 1e-15);
        let one_minus_phi_sq = 1.0 - (2.0 * log_rate * event_delta).exp();
        assert!((recovered - one_minus_phi_sq).abs() < 1e-15);
        let two_and_a_half =
            recover_standardised_discrete_diffusion(diffusion, log_rate, 2.5, LagClock::EventTime)
                .expect("discreteDIFFUSIONstd Δt=2.5");
        assert!((two_and_a_half - (1.0 - (2.0 * log_rate * 2.5).exp())).abs() < 1e-15);
        assert!((recovered - two_and_a_half).abs() > 1e-9);
        let continuous_std = diffusion / within;
        assert!((continuous_std - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = process_noise / total;
        assert!((contaminated - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_unstandardised_discrete_diffusion_as_standardised_discrete_diffusion(
                process_noise,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion
            )
        );
        assert_eq!(
            refuse_standardised_continuous_diffusion_as_standardised_discrete_diffusion(
                continuous_std,
                recovered
            ),
            Err(
                PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion
            )
        );
        assert_eq!(
            refuse_trait_contaminated_process_noise_as_standardised_discrete_diffusion(
                contaminated,
                recovered
            ),
            Err(PsychometricError::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion)
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_discrete_diffusion_fails_closed_when_unstandardised_is_defined() {
        let log_rate = -0.5_f64;
        let event_delta = 1.0_f64;
        let unstandardised_zero_q =
            recover_discrete_process_noise(0.0, log_rate, event_delta, LagClock::EventTime)
                .expect("Q_Δt at q=0");
        assert!((unstandardised_zero_q - 0.0).abs() < 1e-15);
        assert_eq!(
            recover_standardised_discrete_diffusion(0.0, log_rate, event_delta, LagClock::EventTime),
            Err(
                PsychometricError::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance
            )
        );
        let growing = recover_discrete_process_noise(0.4, 0.5, event_delta, LagClock::EventTime)
            .expect("growing a>0");
        assert!(growing.is_finite() && growing > 0.0);
        assert_eq!(
            recover_standardised_discrete_diffusion(0.4, 0.5, event_delta, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        let unit = recover_discrete_process_noise(0.4, 0.0, event_delta, LagClock::EventTime)
            .expect("a=0");
        assert!((unit - 0.4).abs() < 1e-15);
        assert_eq!(
            recover_standardised_discrete_diffusion(0.4, 0.0, event_delta, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_discrete_diffusion(
                0.4,
                log_rate,
                event_delta,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_discrete_diffusion(0.4, log_rate, 0.0, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_standardised_discrete_diffusion(0.4, log_rate, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::NonPositiveInterval)
        );
        assert_eq!(
            recover_standardised_discrete_diffusion(
                -0.1,
                log_rate,
                event_delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_discrete_diffusion(
                0.4,
                f64::NAN,
                event_delta,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_continuous_diffusion_recovers_driver_page_sixteen_after_positive_asymdiffusion()
    {
        // Driver et al. (2017, p. 16 DIFFUSIONstd; Eq. 4; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a), then
        // q / p. Scalar stationary ratio is −2 a.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let recovered =
            recover_standardised_continuous_diffusion(diffusion, log_rate, LagClock::EventTime)
                .expect("DIFFUSIONstd");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        assert!((recovered - (diffusion / within)).abs() < 1e-15);
        assert!((recovered - (-2.0 * log_rate)).abs() < 1e-15);
        let larger_q =
            recover_standardised_continuous_diffusion(2.0, log_rate, LagClock::EventTime)
                .expect("DIFFUSIONstd q=2");
        assert!((larger_q - recovered).abs() < 1e-15);
        let discrete =
            recover_standardised_discrete_diffusion(diffusion, log_rate, 1.0, LagClock::EventTime)
                .expect("discreteDIFFUSIONstd");
        assert!((discrete - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = diffusion / total;
        assert!((contaminated - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_unstandardised_continuous_diffusion_as_standardised_continuous_diffusion(
                diffusion,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion
            )
        );
        assert_eq!(
            refuse_standardised_discrete_diffusion_as_standardised_continuous_diffusion(
                discrete,
                recovered
            ),
            Err(
                PsychometricError::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion
            )
        );
        assert_eq!(
            refuse_trait_contaminated_continuous_diffusion_as_standardised_continuous_diffusion(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_continuous_diffusion_fails_closed_when_unstandardised_is_defined() {
        let log_rate = -0.5_f64;
        assert_eq!(
            recover_standardised_continuous_diffusion(0.0, log_rate, LagClock::EventTime),
            Err(
                PsychometricError::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(0.4, 0.5, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(0.4, 0.0, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(0.4, log_rate, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(-0.1, log_rate, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(0.4, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_diffusion(1e308, -1e308, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_continuous_drift_recovers_driver_page_sixteen_after_positive_asymdiffusion() {
        // Driver et al. (2017, p. 16 DRIFTstd; Eq. 1; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a). Scalar
        // stationary SD ratio is 1, so DRIFTstd equals a numerically.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let recovered =
            recover_standardised_continuous_drift(diffusion, log_rate, LagClock::EventTime)
                .expect("DRIFTstd");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        assert!((recovered - log_rate).abs() < 1e-15);
        let larger_q = recover_standardised_continuous_drift(2.0, log_rate, LagClock::EventTime)
            .expect("DRIFTstd q=2");
        assert!((larger_q - recovered).abs() < 1e-15);
        let discrete =
            recover_standardised_discrete_drift(diffusion, log_rate, 1.0, LagClock::EventTime)
                .expect("discreteDRIFTstd");
        assert!((discrete - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = log_rate * (within / total);
        assert!((contaminated - recovered).abs() > 1e-3);
        assert_eq!(
            refuse_unstandardised_continuous_drift_as_standardised_continuous_drift(
                log_rate, recovered
            ),
            Err(PsychometricError::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift)
        );
        assert_eq!(
            refuse_standardised_discrete_drift_as_standardised_continuous_drift(
                discrete, recovered
            ),
            Err(PsychometricError::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift)
        );
        assert_eq!(
            refuse_trait_contaminated_continuous_drift_as_standardised_continuous_drift(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_continuous_drift_fails_closed_when_unstandardised_is_defined() {
        let log_rate = -0.5_f64;
        assert_eq!(
            recover_standardised_continuous_drift(0.0, log_rate, LagClock::EventTime),
            Err(
                PsychometricError::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_drift(0.4, 0.5, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_drift(0.4, 0.0, LagClock::EventTime),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_drift(0.4, log_rate, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_continuous_drift(-0.1, log_rate, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_drift(0.4, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_asymptotic_time_independent_effect_recovers_driver_page_sixteen_after_positive_variances()
     {
        // Driver et al. (2017, p. 16 asymTIPREDEFFECTstd; §7.2; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a) and
        // strictly positive v, then (−B / a) · √v / √p.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_standardised_asymptotic_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECTstd");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        let unit = recover_asymptotic_time_independent_predictor_effect(
            coefficient,
            1.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECT");
        let expected = unit * predictor_variance.sqrt() / within.sqrt();
        assert!((recovered - expected).abs() < 1e-15);
        let larger_q = recover_standardised_asymptotic_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            2.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECTstd q=2");
        assert!((larger_q - recovered).abs() > 1e-3);
        assert!(larger_q.abs() < recovered.abs());
        let discrete_increment = recover_discrete_time_independent_predictor_effect(
            coefficient,
            1.0,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("discrete TIPREDEFFECT");
        let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
        assert!((discrete_std - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = unit * predictor_variance.sqrt() / total.sqrt();
        assert!((contaminated - recovered).abs() > 1e-3);
        let zero = recover_standardised_asymptotic_time_independent_predictor_effect(
            0.0,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_unstandardised_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
                unit,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_discrete_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
                discrete_std,
                recovered
            ),
            Err(
                PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_contaminated_asymptotic_time_independent_effect_as_standardised_asymptotic_time_independent_effect(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_asymptotic_time_independent_effect_fails_closed_when_unstandardised_is_defined()
    {
        let log_rate = -0.5_f64;
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                1.0,
                0.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance
            )
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                0.0,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance
            )
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                -0.1,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                0.3,
                f64::NAN,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_asymptotic_time_independent_predictor_effect(
                4.0,
                1e308,
                1e-308,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_continuous_time_independent_effect_recovers_driver_page_sixteen_after_positive_variances()
     {
        // Driver et al. (2017, p. 16 TIPREDEFFECTstd; §7.2; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a) and
        // strictly positive v, then B · √v / √p.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_standardised_continuous_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TIPREDEFFECTstd");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        let expected = coefficient * predictor_variance.sqrt() / within.sqrt();
        assert!((recovered - expected).abs() < 1e-15);
        let larger_q = recover_standardised_continuous_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            2.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TIPREDEFFECTstd q=2");
        assert!((larger_q - recovered).abs() > 1e-3);
        assert!(larger_q.abs() < recovered.abs());
        let asymptotic = recover_standardised_asymptotic_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECTstd");
        assert!((asymptotic - recovered).abs() > 1e-3);
        let discrete_increment = recover_discrete_time_independent_predictor_effect(
            coefficient,
            1.0,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("discrete TIPREDEFFECT");
        let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
        assert!((discrete_std - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
        assert!((contaminated - recovered).abs() > 1e-3);
        let zero = recover_standardised_continuous_time_independent_predictor_effect(
            0.0,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_unstandardised_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
                coefficient,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_asymptotic_time_independent_effect_as_standardised_continuous_time_independent_effect(
                asymptotic,
                recovered
            ),
            Err(
                PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_discrete_time_independent_effect_as_standardised_continuous_time_independent_effect(
                discrete_std,
                recovered
            ),
            Err(
                PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_contaminated_continuous_time_independent_effect_as_standardised_continuous_time_independent_effect(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_continuous_time_independent_effect_fails_closed_when_unstandardised_is_defined()
    {
        let log_rate = -0.5_f64;
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                1.0,
                0.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                0.0,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                1.0,
                0.4,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                -0.1,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                0.3,
                f64::NAN,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                f64::NAN,
                1.0,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_independent_predictor_effect(
                4.0,
                1e308,
                1e-308,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_continuous_time_dependent_effect_recovers_driver_page_sixteen_after_positive_variances()
     {
        // Driver et al. (2017, p. 16 TDPREDEFFECTstd; Table 2; footnote 4):
        // form strictly positive asymDIFFUSION = −q / (2 a) and
        // strictly positive v, then m · √v / √p.
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_standardised_continuous_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TDPREDEFFECTstd");
        let within = recover_stationary_latent_variance(diffusion, log_rate, LagClock::EventTime)
            .expect("asymDIFFUSION");
        assert!(within > 0.0);
        let expected = coefficient * predictor_variance.sqrt() / within.sqrt();
        assert!((recovered - expected).abs() < 1e-15);
        let larger_q = recover_standardised_continuous_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            2.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TDPREDEFFECTstd q=2");
        assert!((larger_q - recovered).abs() > 1e-3);
        assert!(larger_q.abs() < recovered.abs());
        let tipred = recover_standardised_continuous_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TIPREDEFFECTstd");
        assert_eq!(tipred.to_bits(), recovered.to_bits());
        let discrete_increment = recover_discrete_time_independent_predictor_effect(
            coefficient,
            1.0,
            log_rate,
            1.0,
            LagClock::EventTime,
        )
        .expect("intercept-style discrete TDPREDEFFECT");
        let discrete_std = discrete_increment * predictor_variance.sqrt() / within.sqrt();
        assert!((discrete_std - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, within)
            .expect("trait+state var");
        let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
        assert!((contaminated - recovered).abs() > 1e-3);
        let zero = recover_standardised_continuous_time_dependent_predictor_effect(
            0.0,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_unstandardised_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
                coefficient,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_continuous_time_independent_effect_as_standardised_continuous_time_dependent_effect(
                tipred,
                recovered
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_discrete_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
                discrete_std,
                recovered
            ),
            Err(
                PsychometricError::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_trait_contaminated_continuous_time_dependent_effect_as_standardised_continuous_time_dependent_effect(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, within),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_continuous_time_dependent_effect_fails_closed_when_unstandardised_is_defined() {
        let log_rate = -0.5_f64;
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                1.0,
                0.0,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                0.0,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance
            )
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                1.0,
                0.4,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::StationaryVarianceRequiresStableDrift)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                1.0,
                0.4,
                log_rate,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                -0.1,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                0.3,
                f64::NAN,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                f64::NAN,
                1.0,
                0.4,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_continuous_time_dependent_predictor_effect(
                4.0,
                1e308,
                1e-308,
                log_rate,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn standardised_initial_time_dependent_effect_recovers_driver_table_three_after_positive_variances()
     {
        // Driver et al. (2017, Table 3 T0TDPREDEFFECTstd; p. 16; footnote 4):
        // form strictly positive free T0VAR p_0 and strictly positive v,
        // then t0_m · √v / √p_0. Affected variance is p_0, not
        // asymDIFFUSION.
        let initial_variance = 1.6_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_standardised_initial_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            initial_variance,
            LagClock::EventTime,
        )
        .expect("T0TDPREDEFFECTstd");
        let expected = coefficient * predictor_variance.sqrt() / initial_variance.sqrt();
        assert!((recovered - expected).abs() < 1e-15);
        let larger_p0 = recover_standardised_initial_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            6.4,
            LagClock::EventTime,
        )
        .expect("T0TDPREDEFFECTstd p_0=6.4");
        assert!((larger_p0 - recovered).abs() > 1e-3);
        assert!(larger_p0.abs() < recovered.abs());
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let continuous = recover_standardised_continuous_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TDPREDEFFECTstd");
        assert!((continuous - recovered).abs() > 1e-3);
        let t0_tipred = recover_standardised_initial_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            initial_variance,
            LagClock::EventTime,
        )
        .expect("T0TIPREDEFFECTstd");
        assert_eq!(t0_tipred.to_bits(), recovered.to_bits());
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, initial_variance)
            .expect("trait+state var");
        let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
        assert!((contaminated - recovered).abs() > 1e-3);
        let zero = recover_standardised_initial_time_dependent_predictor_effect(
            0.0,
            predictor_variance,
            initial_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_unstandardised_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
                coefficient,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_continuous_time_dependent_effect_as_standardised_initial_time_dependent_effect(
                continuous,
                recovered
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_initial_time_independent_effect_as_standardised_initial_time_dependent_effect(
                t0_tipred,
                recovered
            ),
            Err(
                PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_trait_contaminated_initial_time_dependent_effect_as_standardised_initial_time_dependent_effect(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, initial_variance),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_initial_time_dependent_effect_fails_closed_when_unstandardised_is_defined() {
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance
            )
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                0.0,
                1.6,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance
            )
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                1.0,
                1.6,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                -0.1,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                1.0,
                -0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                f64::NAN,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                0.3,
                1.0,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                f64::NAN,
                1.0,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_dependent_predictor_effect(
                4.0,
                1e308,
                1e-308,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn initial_time_dependent_predictor_variance_recovers_analog_of_added_t0_tipred_var() {
        let coefficient = 0.3_f64;
        let predictor_variance = 4.0_f64;
        let recovered = recover_initial_time_dependent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TDPREDVAR analog");
        assert!((recovered - coefficient * coefficient * predictor_variance).abs() < 1e-15);
        let doubled = recover_initial_time_dependent_predictor_variance(
            coefficient,
            8.0,
            LagClock::EventTime,
        )
        .expect("doubled v");
        assert!((doubled - 2.0 * recovered).abs() < 1e-15);
        let negative = recover_initial_time_dependent_predictor_variance(
            -coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("signed coefficient");
        assert_eq!(negative.to_bits(), recovered.to_bits());
        let ti_extra = recover_initial_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TIPREDVAR");
        assert_eq!(ti_extra.to_bits(), recovered.to_bits());
        let standardised = recover_standardised_initial_time_dependent_predictor_effect(
            coefficient,
            predictor_variance,
            1.6,
            LagClock::EventTime,
        )
        .expect("T0TDPREDEFFECTstd");
        assert!((standardised - recovered).abs() > 1e-3);
        let covariance = coefficient * predictor_variance;
        assert!((covariance - recovered).abs() > 1e-3);
        let zero_coefficient = recover_initial_time_dependent_predictor_variance(
            0.0,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero_coefficient.to_bits(), 0.0_f64.to_bits());
        let zero_variance = recover_initial_time_dependent_predictor_variance(
            coefficient,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero variance");
        assert_eq!(zero_variance.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_initial_time_dependent_variance_as_initial_time_independent_variance(
                recovered, ti_extra
            ),
            Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeIndependentVariance)
        );
        assert_eq!(
            refuse_initial_time_dependent_variance_as_standardised_initial_time_dependent_effect(
                recovered, standardised
            ),
            Err(
                PsychometricError::InitialTimeDependentVarianceIsNotStandardisedInitialTimeDependentEffect
            )
        );
        assert_eq!(
            refuse_initial_time_dependent_variance_as_initial_time_dependent_covariance(
                recovered, covariance
            ),
            Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialTimeDependentCovariance)
        );
        assert_eq!(
            refuse_initial_time_dependent_variance_as_initial_latent_variance(recovered, 1.6),
            Err(PsychometricError::InitialTimeDependentVarianceIsNotInitialLatentVariance)
        );
        assert_eq!(
            refuse_initial_time_dependent_variance_as_trait_variance(recovered, 1.0),
            Err(PsychometricError::InitialTimeDependentVarianceIsNotTraitVariance)
        );
    }

    #[test]
    fn initial_time_dependent_predictor_variance_fails_closed_on_non_event_clock_and_overflow() {
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(0.3, 4.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(0.3, -0.1, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(f64::NAN, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(0.3, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(1e308, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_predictor_variance(1e154, 1e154, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let zero_with_overflowing_variance =
            recover_initial_time_dependent_predictor_variance(0.0, 1e308, LagClock::EventTime)
                .expect("zero coefficient keeps zero");
        assert_eq!(zero_with_overflowing_variance.to_bits(), 0.0_f64.to_bits());
        let zero_with_overflowing_coefficient =
            recover_initial_time_dependent_predictor_variance(1e308, 0.0, LagClock::EventTime)
                .expect("zero variance keeps zero");
        assert_eq!(
            zero_with_overflowing_coefficient.to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn initial_time_dependent_observed_variance_recovers_eq5_of_analog_added_t0_tipred_var() {
        let loading = 2.0_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 4.0_f64;
        let extra = recover_initial_time_dependent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TDPREDVAR analog");
        let recovered = recover_initial_time_dependent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("eq5 analog");
        let expected = recover_manifest_observed_variance(loading, extra, 0.0).expect("λ² extra");
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - loading * loading * extra).abs() < 1e-15);
        let doubled = recover_initial_time_dependent_observed_variance(
            loading,
            coefficient,
            8.0,
            LagClock::EventTime,
        )
        .expect("doubled v");
        assert!((doubled - 2.0 * recovered).abs() < 1e-15);
        let negative = recover_initial_time_dependent_observed_variance(
            loading,
            -coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("signed coefficient");
        assert_eq!(negative.to_bits(), recovered.to_bits());
        let ti_observed = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("eq5 addedT0TIPREDVAR");
        assert_eq!(ti_observed.to_bits(), recovered.to_bits());
        let initial_observed =
            recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p_0 + θ");
        assert!((initial_observed - recovered).abs() > 1e-3);
        assert!((extra - recovered).abs() > 1e-3);
        assert!((0.1_f64 - recovered).abs() > 1e-3);
        let zero_loading = recover_initial_time_dependent_observed_variance(
            0.0,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero loading");
        assert_eq!(zero_loading.to_bits(), 0.0_f64.to_bits());
        let zero_coefficient = recover_initial_time_dependent_observed_variance(
            loading,
            0.0,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero_coefficient.to_bits(), 0.0_f64.to_bits());
        let zero_variance = recover_initial_time_dependent_observed_variance(
            loading,
            coefficient,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero variance");
        assert_eq!(zero_variance.to_bits(), 0.0_f64.to_bits());
        let scaled = recover_initial_time_dependent_observed_variance(
            1e308,
            1e-154,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!(scaled.is_finite());
        assert_eq!(
            refuse_initial_time_dependent_observed_variance_as_initial_time_dependent_variance(
                recovered, extra
            ),
            Err(
                PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeDependentVariance
            )
        );
        assert_eq!(
            refuse_initial_time_dependent_observed_variance_as_initial_observed_variance(
                recovered,
                initial_observed
            ),
            Err(
                PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialObservedVariance
            )
        );
        assert_eq!(
            refuse_initial_time_dependent_observed_variance_as_initial_time_independent_observed_variance(
                recovered,
                ti_observed
            ),
            Err(
                PsychometricError::InitialTimeDependentObservedVarianceIsNotInitialTimeIndependentObservedVariance
            )
        );
        assert_eq!(
            refuse_initial_time_dependent_observed_variance_as_measurement_error(recovered, 0.1),
            Err(PsychometricError::InitialTimeDependentObservedVarianceIsNotMeasurementError)
        );
    }

    #[test]
    fn initial_time_dependent_observed_variance_fails_closed_on_non_event_clock_and_overflow() {
        assert_eq!(
            recover_initial_time_dependent_observed_variance(2.0, 0.3, 4.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(2.0, 0.3, -0.1, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(
                f64::NAN,
                0.3,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(
                2.0,
                f64::NAN,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(
                2.0,
                0.3,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(2.0, 1e308, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_dependent_observed_variance(1e308, 0.3, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let zero_with_overflowing_loading =
            recover_initial_time_dependent_observed_variance(1e308, 0.0, 4.0, LagClock::EventTime)
                .expect("zero extra keeps zero");
        assert_eq!(zero_with_overflowing_loading.to_bits(), 0.0_f64.to_bits());
        let zero_with_overflowing_variance =
            recover_initial_time_dependent_observed_variance(2.0, 0.0, 1e308, LagClock::EventTime)
                .expect("zero coefficient keeps zero");
        assert_eq!(zero_with_overflowing_variance.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn standardised_initial_time_independent_effect_recovers_driver_table_three_after_positive_variances()
     {
        // Driver et al. (2017, Table 3 T0TIPREDEFFECTstd; p. 16; footnote 4):
        // form strictly positive free T0VAR p_0 and strictly positive v,
        // then t0_b · √v / √p_0. Affected variance is p_0, not
        // asymDIFFUSION.
        let initial_variance = 1.6_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 1.0_f64;
        let recovered = recover_standardised_initial_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            initial_variance,
            LagClock::EventTime,
        )
        .expect("T0TIPREDEFFECTstd");
        let expected = coefficient * predictor_variance.sqrt() / initial_variance.sqrt();
        assert!((recovered - expected).abs() < 1e-15);
        let larger_p0 = recover_standardised_initial_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            6.4,
            LagClock::EventTime,
        )
        .expect("T0TIPREDEFFECTstd p_0=6.4");
        assert!((larger_p0 - recovered).abs() > 1e-3);
        assert!(larger_p0.abs() < recovered.abs());
        let diffusion = 0.4_f64;
        let log_rate = -0.5_f64;
        let continuous = recover_standardised_continuous_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("TIPREDEFFECTstd");
        assert!((continuous - recovered).abs() > 1e-3);
        let asymptotic = recover_standardised_asymptotic_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            diffusion,
            log_rate,
            LagClock::EventTime,
        )
        .expect("asymTIPREDEFFECTstd");
        assert!((asymptotic - recovered).abs() > 1e-3);
        let trait_variance = 1.0_f64;
        let total = recover_trait_plus_state_latent_variance(trait_variance, initial_variance)
            .expect("trait+state var");
        let contaminated = coefficient * predictor_variance.sqrt() / total.sqrt();
        assert!((contaminated - recovered).abs() > 1e-3);
        let zero = recover_standardised_initial_time_independent_predictor_effect(
            0.0,
            predictor_variance,
            initial_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_unstandardised_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
                coefficient,
                recovered
            ),
            Err(
                PsychometricError::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_continuous_time_independent_effect_as_standardised_initial_time_independent_effect(
                continuous,
                recovered
            ),
            Err(
                PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_standardised_asymptotic_time_independent_effect_as_standardised_initial_time_independent_effect(
                asymptotic,
                recovered
            ),
            Err(
                PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_contaminated_initial_time_independent_effect_as_standardised_initial_time_independent_effect(
                contaminated,
                recovered
            ),
            Err(
                PsychometricError::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_trait_variance_as_standardisation_variance(trait_variance, initial_variance),
            Err(PsychometricError::TraitVarianceIsNotStandardisationVariance)
        );
    }

    #[test]
    fn standardised_initial_time_independent_effect_fails_closed_when_unstandardised_is_defined() {
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                1.0,
                0.0,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance
            )
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                0.0,
                1.6,
                LagClock::EventTime
            ),
            Err(
                PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance
            )
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                1.0,
                1.6,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                -0.1,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                1.0,
                -0.1,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                f64::NAN,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                0.3,
                1.0,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                f64::NAN,
                1.0,
                1.6,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_standardised_initial_time_independent_predictor_effect(
                4.0,
                1e308,
                1e-308,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
    }

    #[test]
    fn initial_time_independent_predictor_variance_recovers_added_t0_tipred_var() {
        let coefficient = 0.3_f64;
        let predictor_variance = 4.0_f64;
        let recovered = recover_initial_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TIPREDVAR");
        assert!((recovered - coefficient * coefficient * predictor_variance).abs() < 1e-15);
        let doubled = recover_initial_time_independent_predictor_variance(
            coefficient,
            8.0,
            LagClock::EventTime,
        )
        .expect("doubled v");
        assert!((doubled - 2.0 * recovered).abs() < 1e-15);
        let negative = recover_initial_time_independent_predictor_variance(
            -coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("signed coefficient");
        assert_eq!(negative.to_bits(), recovered.to_bits());
        let asymptotic = recover_asymptotic_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            -0.5,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        assert!((asymptotic - recovered).abs() > 1e-3);
        assert_eq!(
            recover_asymptotic_time_independent_predictor_variance(
                coefficient,
                predictor_variance,
                0.5,
                LagClock::EventTime,
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        let growing = recover_initial_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("growing a is not an input");
        assert_eq!(growing.to_bits(), recovered.to_bits());
        let standardised = recover_standardised_initial_time_independent_predictor_effect(
            coefficient,
            predictor_variance,
            1.6,
            LagClock::EventTime,
        )
        .expect("T0TIPREDEFFECTstd");
        assert!((standardised - recovered).abs() > 1e-3);
        let zero_coefficient = recover_initial_time_independent_predictor_variance(
            0.0,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero_coefficient.to_bits(), 0.0_f64.to_bits());
        let zero_variance = recover_initial_time_independent_predictor_variance(
            coefficient,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero variance");
        assert_eq!(zero_variance.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            refuse_initial_time_independent_variance_as_asymptotic_time_independent_variance(
                recovered,
                asymptotic
            ),
            Err(
                PsychometricError::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance
            )
        );
        assert_eq!(
            refuse_initial_time_independent_variance_as_standardised_initial_time_independent_effect(
                recovered,
                standardised
            ),
            Err(
                PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect
            )
        );
        assert_eq!(
            refuse_initial_time_independent_variance_as_initial_latent_variance(recovered, 1.6),
            Err(PsychometricError::InitialTimeIndependentVarianceIsNotInitialLatentVariance)
        );
        assert_eq!(
            refuse_initial_time_independent_variance_as_trait_variance(recovered, 1.0),
            Err(PsychometricError::InitialTimeIndependentVarianceIsNotTraitVariance)
        );
    }

    #[test]
    fn initial_time_independent_predictor_variance_fails_closed_on_non_event_clock_and_overflow() {
        assert_eq!(
            recover_initial_time_independent_predictor_variance(0.3, 4.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_variance(0.3, -0.1, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_variance(f64::NAN, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_variance(0.3, f64::NAN, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_variance(1e308, 4.0, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_predictor_variance(1e154, 1e154, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        let zero_with_overflowing_variance =
            recover_initial_time_independent_predictor_variance(0.0, 1e308, LagClock::EventTime)
                .expect("zero coefficient keeps zero");
        assert_eq!(zero_with_overflowing_variance.to_bits(), 0.0_f64.to_bits());
        let zero_with_overflowing_coefficient =
            recover_initial_time_independent_predictor_variance(1e308, 0.0, LagClock::EventTime)
                .expect("zero variance keeps zero");
        assert_eq!(
            zero_with_overflowing_coefficient.to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn initial_time_independent_observed_variance_recovers_eq5_of_added_t0_tipred_var() {
        let loading = 2.0_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 4.0_f64;
        let extra = recover_initial_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TIPREDVAR");
        let recovered = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("eq5 addedT0TIPREDVAR");
        let expected = recover_manifest_observed_variance(loading, extra, 0.0).expect("λ² extra");
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - loading * loading * extra).abs() < 1e-15);
        let doubled = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            8.0,
            LagClock::EventTime,
        )
        .expect("doubled v");
        assert!((doubled - 2.0 * recovered).abs() < 1e-15);
        let negative = recover_initial_time_independent_observed_variance(
            loading,
            -coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("signed coefficient");
        assert_eq!(negative.to_bits(), recovered.to_bits());
        let asymptotic_observed = recover_asymptotic_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            -0.5,
            LagClock::EventTime,
        )
        .expect("λ² (B/a)² v");
        assert!((asymptotic_observed - recovered).abs() > 1e-3);
        let initial_observed =
            recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p_0 + θ");
        assert!((initial_observed - recovered).abs() > 1e-3);
        assert!((extra - recovered).abs() > 1e-3);
        assert!((0.1_f64 - recovered).abs() > 1e-3);
        let growing = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("growing a is not an input");
        assert_eq!(growing.to_bits(), recovered.to_bits());
        let zero_loading = recover_initial_time_independent_observed_variance(
            0.0,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero loading");
        assert_eq!(zero_loading.to_bits(), 0.0_f64.to_bits());
        let zero_coefficient = recover_initial_time_independent_observed_variance(
            loading,
            0.0,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero_coefficient.to_bits(), 0.0_f64.to_bits());
        let zero_variance = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero variance");
        assert_eq!(zero_variance.to_bits(), 0.0_f64.to_bits());
        let scaled = recover_initial_time_independent_observed_variance(
            1e308,
            1e-154,
            1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!(scaled.is_finite());
        assert_eq!(
            refuse_initial_time_independent_observed_variance_as_initial_time_independent_variance(
                recovered, extra
            ),
            Err(
                PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance
            )
        );
        assert_eq!(
            refuse_initial_time_independent_observed_variance_as_initial_observed_variance(
                recovered,
                initial_observed
            ),
            Err(
                PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance
            )
        );
        assert_eq!(
            refuse_initial_time_independent_observed_variance_as_asymptotic_time_independent_observed_variance(
                recovered,
                asymptotic_observed
            ),
            Err(
                PsychometricError::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance
            )
        );
        assert_eq!(
            refuse_initial_time_independent_observed_variance_as_measurement_error(recovered, 0.1),
            Err(PsychometricError::InitialTimeIndependentObservedVarianceIsNotMeasurementError)
        );
    }

    #[test]
    fn initial_time_independent_observed_variance_fails_closed_on_non_event_clock_and_overflow() {
        assert_eq!(
            recover_initial_time_independent_observed_variance(2.0, 0.3, 4.0, LagClock::SystemTime),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(2.0, 0.3, -0.1, LagClock::EventTime),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(
                f64::NAN,
                0.3,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(
                2.0,
                f64::NAN,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(
                2.0,
                0.3,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(
                2.0,
                1e308,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_initial_time_independent_observed_variance(
                1e308,
                0.3,
                4.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        let zero_with_overflowing_loading = recover_initial_time_independent_observed_variance(
            1e308,
            0.0,
            4.0,
            LagClock::EventTime,
        )
        .expect("zero extra keeps zero");
        assert_eq!(zero_with_overflowing_loading.to_bits(), 0.0_f64.to_bits());
        let zero_with_overflowing_variance = recover_initial_time_independent_observed_variance(
            2.0,
            0.0,
            1e308,
            LagClock::EventTime,
        )
        .expect("zero coefficient keeps zero");
        assert_eq!(zero_with_overflowing_variance.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn asymptotic_time_independent_observed_variance_recovers_eq5_of_added_tipred_var() {
        let loading = 2.0_f64;
        let coefficient = 0.3_f64;
        let predictor_variance = 4.0_f64;
        let log_rate = -0.5_f64;
        let extra = recover_asymptotic_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("addedTIPREDVAR");
        let recovered = recover_asymptotic_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("eq5 addedTIPREDVAR");
        let expected = recover_manifest_observed_variance(loading, extra, 0.0).expect("λ² extra");
        assert!((recovered - expected).abs() < 1e-15);
        assert!((recovered - loading * loading * extra).abs() < 1e-15);
        let doubled = recover_asymptotic_time_independent_observed_variance(
            loading,
            coefficient,
            8.0,
            log_rate,
            LagClock::EventTime,
        )
        .expect("doubled v");
        assert!((doubled - 2.0 * recovered).abs() < 1e-15);
        let negative = recover_asymptotic_time_independent_observed_variance(
            loading,
            -coefficient,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("signed coefficient");
        assert_eq!(negative.to_bits(), recovered.to_bits());
        let initial_extra = recover_initial_time_independent_predictor_variance(
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("addedT0TIPREDVAR");
        let initial_observed = recover_initial_time_independent_observed_variance(
            loading,
            coefficient,
            predictor_variance,
            LagClock::EventTime,
        )
        .expect("eq5 addedT0TIPREDVAR");
        assert!((initial_observed - recovered).abs() > 1e-3);
        let stationary_observed =
            recover_manifest_observed_variance(loading, 1.6, 0.1).expect("λ² p + θ");
        assert!((stationary_observed - recovered).abs() > 1e-3);
        assert!((extra - recovered).abs() > 1e-3);
        assert!((initial_extra - recovered).abs() > 1e-3);
        assert!((0.1_f64 - recovered).abs() > 1e-3);
        let zero_loading = recover_asymptotic_time_independent_observed_variance(
            0.0,
            coefficient,
            predictor_variance,
            log_rate,
            LagClock::EventTime,
        )
        .expect("zero loading");
        assert_eq!(zero_loading.to_bits(), 0.0_f64.to_bits());
        let zero_coefficient = recover_asymptotic_time_independent_observed_variance(
            loading,
            0.0,
            predictor_variance,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero coefficient");
        assert_eq!(zero_coefficient.to_bits(), 0.0_f64.to_bits());
        let zero_variance = recover_asymptotic_time_independent_observed_variance(
            loading,
            coefficient,
            0.0,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero variance");
        assert_eq!(zero_variance.to_bits(), 0.0_f64.to_bits());
        let scaled = recover_asymptotic_time_independent_observed_variance(
            1e308,
            1e-154,
            1.0,
            -1.0,
            LagClock::EventTime,
        )
        .expect("scale");
        assert!(scaled.is_finite());
        assert_eq!(
            refuse_asymptotic_time_independent_observed_variance_as_asymptotic_time_independent_variance(
                recovered, extra
            ),
            Err(
                PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance
            )
        );
        assert_eq!(
            refuse_asymptotic_time_independent_observed_variance_as_initial_time_independent_observed_variance(
                recovered,
                initial_observed
            ),
            Err(
                PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance
            )
        );
        assert_eq!(
            refuse_asymptotic_time_independent_observed_variance_as_stationary_observed_variance(
                recovered,
                stationary_observed
            ),
            Err(
                PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance
            )
        );
        assert_eq!(
            refuse_asymptotic_time_independent_observed_variance_as_measurement_error(
                recovered, 0.1
            ),
            Err(PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn asymptotic_time_independent_observed_variance_fails_closed_on_non_event_clock_and_overflow()
    {
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                0.3,
                4.0,
                -0.5,
                LagClock::SystemTime
            ),
            Err(PsychometricError::EventTimeRequired)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                0.3,
                -0.1,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                0.3,
                4.0,
                0.0,
                LagClock::EventTime
            ),
            Err(PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                f64::NAN,
                0.3,
                4.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                f64::NAN,
                4.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                0.3,
                f64::NAN,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                0.3,
                4.0,
                f64::NAN,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                2.0,
                1e308,
                4.0,
                -1e-308,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        assert_eq!(
            recover_asymptotic_time_independent_observed_variance(
                1e308,
                0.3,
                4.0,
                -0.5,
                LagClock::EventTime
            ),
            Err(PsychometricError::InvalidNumericInput)
        );
        let zero_with_overflowing_loading = recover_asymptotic_time_independent_observed_variance(
            1e308,
            0.0,
            4.0,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero extra keeps zero");
        assert_eq!(zero_with_overflowing_loading.to_bits(), 0.0_f64.to_bits());
        let zero_with_overflowing_variance = recover_asymptotic_time_independent_observed_variance(
            2.0,
            0.0,
            1e308,
            0.0,
            LagClock::EventTime,
        )
        .expect("zero coefficient keeps zero");
        assert_eq!(zero_with_overflowing_variance.to_bits(), 0.0_f64.to_bits());
    }
}
