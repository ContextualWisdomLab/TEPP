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
//! `Var(η_ti) = A_Δt Var(η_{t-1,i}) A_Δt⊤ + Q_Δt`. The JSS article
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
/// The JSS article has no numbered §2.2.
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
        map_discrete_lag_across_event_intervals, recover_discrete_constant_predictor_effect,
        recover_discrete_lag_from_log_rate, recover_discrete_lag_one,
        recover_discrete_lagged_latent_covariance, recover_discrete_latent_variance,
        recover_discrete_process_noise, recover_discrete_time_varying_predictor_effect,
        recover_event_series_mean_log_rate, recover_event_time_discrete_lag_and_log_rate,
        recover_irregular_centered_residual_log_rate, recover_local_log_rate,
        recover_within_residual_event_time_log_rate, refuse_difference_quotient_as_local_rate,
        refuse_pooled_discrete_lag_across_unequal_intervals,
        refuse_process_noise_as_unconditional_variance,
        refuse_unmatched_time_varying_predictor_interval,
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
}
