//! Fail-closed psychometric input and recovery errors.

use std::fmt;

/// A fail-closed psychometric-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PsychometricError {
    /// Raw simplex proportions were offered as Euclidean indicators.
    RawProportionForbidden,
    /// Empty, unequal-length, or non-finite numeric input.
    InvalidNumericInput,
    /// A predictor or indicator vector has zero variance.
    SingularDesign,
    /// A good global fit was used to reinterpret a formative or network
    /// construct as reflective.
    FormativeReinterpretationForbidden,
    /// Temporal precedence, linkage, tracking, or prediction was treated as
    /// causal identification.
    CausalUnderidentified,
    /// The construct class is unresolved and cannot support a reflective
    /// interpretation.
    UnresolvedConstruct,
    /// Latent-mean or path comparison was requested without invariance
    /// evidence.
    InvarianceRequired,
    /// A structural lag or local log-rate was requested on a non-event clock.
    EventTimeRequired,
    /// The Voelkle–Oud difference quotient was offered as a continuous-time
    /// rate.
    DifferenceQuotientForbidden,
    /// Discrete lags from unequal event intervals were treated as one coefficient.
    UnequalIntervalPoolingForbidden,
    /// A time-varying predictor was mapped with unmatched sampling and
    /// constancy intervals. Oud and Jansen (2000) is unread.
    UnmatchedTimeVaryingInterval,
    /// Fewer than two clusters were supplied for a within/between decomposition.
    InsufficientClusters,
    /// A membership or survey weight is empty, negative, or non-finite.
    InvalidWeight,
    /// An event-time interval is non-positive.
    NonPositiveInterval,
    /// Fewer than two posterior draws were supplied for Rubin combining.
    InsufficientDraws,
    /// Latent-mean comparison was requested at metric/weak invariance.
    StrongInvarianceRequired,
    /// Driver Eq. 3 process noise was treated as the unconditional latent
    /// variance. `Q_Δt` is `cov(η_ti | η_{t-1,i})`.
    ProcessNoiseIsConditionalVariance,
    /// Stationary within-subject variance was requested for a non-stable
    /// drift. Driver et al. (2017, Eq. 4 as `Δt → ∞`; §4.3; p. 16
    /// `asymDIFFUSION`) require `a < 0`.
    StationaryVarianceRequiresStableDrift,
    /// Finite-interval Driver Eq. 3 process noise was treated as the
    /// asymptotic within-subject variance. `Q_Δt` at a finite `Δt` is not
    /// `asymDIFFUSION`.
    FiniteIntervalProcessNoiseIsNotStationary,
    /// Driver §4.3 trait variance was treated as process noise or diffusion.
    /// A stable trait has `DRIFT` and `DIFFUSION` fixed to zero.
    TraitVarianceIsNotProcessNoise,
    /// Driver §4.3 trait variance was treated as the stationary
    /// within-subject variance. `TRAITVAR` is between-subject and
    /// time-invariant; `asymDIFFUSION` is the `Δt → ∞` state variance.
    TraitVarianceIsNotStationaryWithinSubject,
    /// Driver Eq. 5 measurement-error variance was treated as the
    /// observed-indicator variance. Table 2 (p. 12) names
    /// `MANIFESTVAR` as `Θ`, not `Var(y)`.
    MeasurementErrorIsNotObservedVariance,
    /// Driver Eq. 5 latent variance was treated as the observed-indicator
    /// variance. `Var(η)` is not `Var(y)`.
    LatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 `MANIFESTTRAITVAR` was treated as `MANIFESTVAR`.
    /// Table 2 (p. 12) names `Ψ_τ` separately from `Θ`.
    ManifestTraitVarianceIsNotMeasurementError,
    /// Driver Eq. 3–4 lagged latent covariance was treated as the
    /// lagged observed-indicator covariance. Equation 5 maps
    /// `cov(y_t, y_{t-1}) = λ² cov(η_t, η_{t-1}) + ψ`.
    LatentLaggedCovarianceIsNotObservedCovariance,
    /// Driver Eq. 5 measurement-error variance was treated as the
    /// lagged observed-indicator covariance. Independent `ε` does
    /// not enter `cov(y_t, y_{t-1})`.
    MeasurementErrorIsNotLaggedObservedCovariance,
    /// Driver Eq. 5 `MANIFESTMEANS` was treated as `E(y)`. Table 2
    /// (p. 12) names `τ` the expected intercept `Γ`, not `τ + λ μ`.
    ManifestMeansIsNotObservedMean,
    /// Driver Eq. 5 latent mean was treated as `E(y)`. `E(η)` is
    /// not `τ + λ E(η)`.
    LatentMeanIsNotObservedMean,
    /// Driver Table 2 `CINT` was treated as `MANIFESTMEANS`. `κ` is
    /// the latent continuous intercept, not the expected `Γ`.
    ContinuousInterceptIsNotManifestMeans,
    /// Driver Table 2 `T0MEANS` was treated as the evolved latent mean.
    /// Equation 3 maps `μ_t = exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`.
    InitialLatentMeanIsNotEvolvedMean,
    /// Driver Table 2 `CINT` was treated as the discrete mean increment.
    /// `κ` is not `A^{-1}[e^{A Δt} − I] κ`.
    ContinuousInterceptIsNotDiscreteMeanIncrement,
    /// Driver Table 2 `CINT` was treated as `T0MEANS`. `κ` is not
    /// the first-occasion latent mean.
    ContinuousInterceptIsNotInitialLatentMean,
    /// Driver Eq. 5 of the first-occasion mean was treated as
    /// `E(y_t)`. `τ + λ μ_0` is not `τ + λ μ_t`.
    InitialObservedMeanIsNotEvolvedObservedMean,
    /// Driver Eq. 3 fourth-summand impulse was treated as `CINT`.
    /// Table 2 names `M` `TDPREDEFFECT`, not `κ`.
    TimeDependentImpulseIsNotContinuousIntercept,
    /// Driver Eq. 3 fourth-summand impulse was treated as the
    /// time-independent discrete effect. `M x` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    TimeDependentImpulseIsNotTimeIndependentEffect,
    /// Driver Eq. 3 fourth-summand impulse was treated as Voelkle
    /// et al. (2012, Eq. 14). `M x` is not `a_{yx} Δt`.
    TimeDependentImpulseIsNotTimeVaryingDiscreteEffect,
    /// Driver Eq. 3 time-independent discrete effect was treated as
    /// `CINT`. `A^{-1}[e^{A Δt} − I] B z` is not `κ`.
    TimeIndependentEffectIsNotContinuousIntercept,
    /// Driver Eq. 3 time-independent discrete effect was treated as
    /// the fourth-summand impulse. `A^{-1}[e^{A Δt} − I] B z` is not
    /// `M x`.
    TimeIndependentEffectIsNotTimeDependentImpulse,
    /// Driver Eq. 3 time-independent discrete effect was treated as
    /// Voelkle et al. (2012, Eq. 14). `A^{-1}[e^{A Δt} − I] B z` is
    /// not `a_{yx} Δt`.
    TimeIndependentEffectIsNotTimeVaryingDiscreteEffect,
    /// Driver Table 2 `TIPREDEFFECT` was treated as the discrete
    /// increment. `B` is not `A^{-1}[e^{A Δt} − I] B z`.
    TimeIndependentCoefficientIsNotDiscreteEffect,
    /// Driver Eq. 1–2 within-interval impulse carry was treated as
    /// the contemporaneous Dirac. `e^{A(t−u)} M x` for `t0 < u < t`
    /// is not `M x`.
    TimeDependentImpulseCarryIsNotContemporaneousImpulse,
    /// Driver Eq. 1–2 within-interval impulse carry was treated as
    /// `CINT`. `e^{A(t−u)} M x` is not `κ`.
    TimeDependentImpulseCarryIsNotContinuousIntercept,
    /// Driver Eq. 1–2 within-interval impulse carry was treated as
    /// the time-independent discrete effect. `e^{A(t−u)} M x` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    TimeDependentImpulseCarryIsNotTimeIndependentEffect,
    /// Driver Eq. 1–2 within-interval impulse carry was treated as
    /// Voelkle et al. (2012, Eq. 14). `e^{A(t−u)} M x` is not
    /// `a_{yx} Δt`.
    TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect,
    /// Driver Eq. 5 of the Eq. 3 evolved mean was treated as
    /// Equation 5 of the Eq. 1–2 carried latent mean.
    /// `τ + λ μ_t` is not `τ + λ(μ_t + e^{a(t−u)} m x)`.
    EvolvedObservedMeanIsNotImpulseCarryObservedMean,
    /// Driver Eq. 5 of the Eq. 3 evolved mean was treated as
    /// Equation 5 of the contemporaneous impulse.
    /// `τ + λ μ_t` is not `τ + λ(μ_t + m x)`.
    EvolvedObservedMeanIsNotImpulseObservedMean,
    /// Driver Eq. 5 of the contemporaneous impulse was treated as
    /// Equation 5 of the Eq. 1–2 carried latent mean.
    /// `τ + λ(μ_t + m x)` is not `τ + λ(μ_t + e^{a(t−u)} m x)`.
    ImpulseObservedMeanIsNotImpulseCarryObservedMean,
    /// Driver Eq. 5 of the Eq. 3 evolved mean was treated as
    /// Equation 5 of the time-independent predictor.
    /// `τ + λ μ_t` is not `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`.
    EvolvedObservedMeanIsNotTimeIndependentObservedMean,
    /// Driver Eq. 5 of the contemporaneous impulse was treated as
    /// Equation 5 of the time-independent predictor.
    /// `τ + λ(μ_t + m x)` is not `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`.
    ImpulseObservedMeanIsNotTimeIndependentObservedMean,
    /// Driver Eq. 5 of the Eq. 1–2 carried latent mean was treated as
    /// Equation 5 of the time-independent predictor.
    /// `τ + λ(μ_t + e^{a(t−u)} m x)` is not
    /// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)`.
    ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean,
    /// Driver Table 3 `T0TIPREDEFFECT` first-occasion shift was treated
    /// as the Eq. 3 process increment. `t0_b z` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    InitialTimeIndependentEffectIsNotProcessIncrement,
    /// Driver Eq. 3 carry of `T0TIPREDEFFECT` was treated as the
    /// first-occasion shift. `e^{A Δt} t0_b z` is not `t0_b z`.
    InitialTimeIndependentCarryIsNotInitialEffect,
    /// Driver Table 3 `T0TIPREDEFFECT` first-occasion shift was treated
    /// as `CINT`. `t0_b z` is not `κ`.
    InitialTimeIndependentEffectIsNotContinuousIntercept,
    /// Driver Table 3 `T0TIPREDEFFECT` first-occasion shift was treated
    /// as the fourth-summand impulse. `t0_b z` is not `M x`.
    InitialTimeIndependentEffectIsNotTimeDependentImpulse,
    /// Driver Table 3 `T0TIPREDEFFECT` was treated as the first-occasion
    /// shift. The coefficient is not `t0_b z`.
    InitialTimeIndependentCoefficientIsNotInitialEffect,
    /// Driver Eq. 5 of the Eq. 3 evolved mean was treated as
    /// Equation 5 of the Table 3 first-occasion TI predictor.
    /// `τ + λ μ_t` is not `τ + λ(μ_t + e^{a Δt} t0_b z)`.
    EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean,
    /// Driver Eq. 5 of the Eq. 3 process increment was treated as
    /// Equation 5 of the Table 3 first-occasion TI predictor.
    /// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not
    /// `τ + λ(μ_t + e^{a Δt} t0_b z)`.
    TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean,
    /// Driver Eq. 5 of the contemporaneous impulse was treated as
    /// Equation 5 of the Table 3 first-occasion TI predictor.
    /// `τ + λ(μ_t + m x)` is not `τ + λ(μ_t + e^{a Δt} t0_b z)`.
    ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean,
    /// Driver Eq. 5 of the Eq. 1–2 carried latent mean was treated as
    /// Equation 5 of the Table 3 first-occasion TI predictor.
    /// `τ + λ(μ_t + e^{a(t−u)} m x)` is not
    /// `τ + λ(μ_t + e^{a Δt} t0_b z)`.
    ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean,
    /// Driver Table 3 `T0TDPREDEFFECT` first-occasion shift was treated
    /// as the contemporaneous Dirac. `t0_m x0` is not `M x`.
    InitialTimeDependentEffectIsNotContemporaneousImpulse,
    /// Driver Eq. 3 carry of `T0TDPREDEFFECT` was treated as the
    /// first-occasion shift. `e^{A Δt} t0_m x0` is not `t0_m x0`.
    InitialTimeDependentCarryIsNotInitialEffect,
    /// Driver Table 3 `T0TDPREDEFFECT` first-occasion shift was treated
    /// as `CINT`. `t0_m x0` is not `κ`.
    InitialTimeDependentEffectIsNotContinuousIntercept,
    /// Driver Table 3 `T0TDPREDEFFECT` first-occasion shift was treated
    /// as the Eq. 3 process increment. `t0_m x0` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    InitialTimeDependentEffectIsNotProcessIncrement,
    /// Driver Table 3 `T0TDPREDEFFECT` first-occasion shift was treated
    /// as the Table 3 `T0TIPREDEFFECT` shift. `t0_m x0` is not `t0_b z`.
    InitialTimeDependentEffectIsNotInitialTimeIndependentEffect,
    /// Driver Table 3 `T0TDPREDEFFECT` was treated as the first-occasion
    /// shift. The coefficient is not `t0_m x0`.
    InitialTimeDependentCoefficientIsNotInitialEffect,
    /// Driver Eq. 3 carry of `T0TDPREDEFFECT` was treated as the
    /// within-interval impulse carry. `e^{A Δt} t0_m x0` is not
    /// `e^{A(t−u)} M x` for `t0 < u < t`.
    InitialTimeDependentCarryIsNotImpulseCarry,
    /// Driver Eq. 5 of the Eq. 3 evolved mean was treated as
    /// Equation 5 of the Table 3 first-occasion TD predictor.
    /// `τ + λ μ_t` is not `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
    EvolvedObservedMeanIsNotInitialTimeDependentObservedMean,
    /// Driver Eq. 5 of the Eq. 3 process increment was treated as
    /// Equation 5 of the Table 3 first-occasion TD predictor.
    /// `τ + λ(μ_t + A^{-1}[e^{A Δt} − I] B z)` is not
    /// `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
    TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean,
    /// Driver Eq. 5 of the contemporaneous impulse was treated as
    /// Equation 5 of the Table 3 first-occasion TD predictor.
    /// `τ + λ(μ_t + m x)` is not `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
    ImpulseObservedMeanIsNotInitialTimeDependentObservedMean,
    /// Driver Eq. 5 of the Eq. 1–2 carried latent mean was treated as
    /// Equation 5 of the Table 3 first-occasion TD predictor.
    /// `τ + λ(μ_t + e^{a(t−u)} m x)` is not
    /// `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
    ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean,
    /// Driver Eq. 5 of Table 3 `T0TIPREDEFFECT` was treated as
    /// Equation 5 of Table 3 `T0TDPREDEFFECT`.
    /// `τ + λ(μ_t + e^{a Δt} t0_b z)` is not
    /// `τ + λ(μ_t + e^{a Δt} t0_m x0)`.
    InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean,
    /// Driver §7.2 level-change `CINT` was requested for a non-stable
    /// drift. Lasting level change via `CINT = TDPREDEFFECT × (−DRIFT)`
    /// requires `a < 0` so `−κ / a = m x` is an equilibrium offset.
    LevelChangeRequiresStableDrift,
    /// Driver §7.2 level-change `CINT` was treated as the
    /// contemporaneous Dirac. `−a m x` is not `m x`.
    LevelChangeInterceptIsNotImpulse,
    /// Driver §7.2 level-change `CINT` was treated as a free `CINT`.
    /// `−a m x` is not an arbitrary `κ`.
    LevelChangeInterceptIsNotFreeContinuousIntercept,
    /// Driver §7.2 level-change `CINT` was treated as the Eq. 3
    /// process increment. `−a m x` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    LevelChangeInterceptIsNotProcessIncrement,
}

impl fmt::Display for PsychometricError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::RawProportionForbidden => {
                "raw topic proportions are forbidden psychometric indicators"
            }
            Self::InvalidNumericInput => "invalid psychometric numeric input",
            Self::SingularDesign => "singular psychometric design matrix",
            Self::FormativeReinterpretationForbidden => {
                "formative or network constructs cannot be reinterpreted as reflective"
            }
            Self::CausalUnderidentified => "temporal precedence is not causal identification",
            Self::UnresolvedConstruct => "construct class is unresolved",
            Self::InvarianceRequired => "latent-mean comparison requires invariance evidence",
            Self::EventTimeRequired => {
                "discrete lag and local log-rate require event time, not another clock"
            }
            Self::DifferenceQuotientForbidden => {
                "the difference quotient is not the local continuous-time rate"
            }
            Self::UnequalIntervalPoolingForbidden => {
                "discrete lags from unequal event intervals are not one coefficient"
            }
            Self::UnmatchedTimeVaryingInterval => {
                "time-varying predictor discrete effect requires matching sampling and constancy intervals"
            }
            Self::InsufficientClusters => "within/between recovery requires at least two clusters",
            Self::InvalidWeight => "invalid non-negative finite psychometric weight",
            Self::NonPositiveInterval => "event-time interval must be strictly positive",
            Self::InsufficientDraws => {
                "Rubin total variance requires at least two complete-data draws"
            }
            Self::StrongInvarianceRequired => {
                "latent-mean comparison requires strong or strict invariance; metric/weak is not enough"
            }
            Self::ProcessNoiseIsConditionalVariance => {
                "discrete process noise is the conditional residual variance, not the unconditional latent variance"
            }
            Self::StationaryVarianceRequiresStableDrift => {
                "stationary within-subject variance requires a stable negative drift"
            }
            Self::FiniteIntervalProcessNoiseIsNotStationary => {
                "finite-interval process noise is not the asymptotic within-subject variance"
            }
            Self::TraitVarianceIsNotProcessNoise => {
                "trait variance is not process noise and is not a diffusion"
            }
            Self::TraitVarianceIsNotStationaryWithinSubject => {
                "trait variance is not the stationary within-subject variance"
            }
            Self::MeasurementErrorIsNotObservedVariance => {
                "measurement-error variance is not the observed-indicator variance"
            }
            Self::LatentVarianceIsNotObservedVariance => {
                "latent variance is not the observed-indicator variance"
            }
            Self::ManifestTraitVarianceIsNotMeasurementError => {
                "manifest-trait variance is not measurement-error variance"
            }
            Self::LatentLaggedCovarianceIsNotObservedCovariance => {
                "lagged latent covariance is not the lagged observed-indicator covariance"
            }
            Self::MeasurementErrorIsNotLaggedObservedCovariance => {
                "measurement-error variance is not the lagged observed-indicator covariance"
            }
            Self::ManifestMeansIsNotObservedMean => {
                "manifest means are not the observed-indicator mean"
            }
            Self::LatentMeanIsNotObservedMean => "latent mean is not the observed-indicator mean",
            Self::ContinuousInterceptIsNotManifestMeans => {
                "continuous intercept is not the manifest mean"
            }
            Self::InitialLatentMeanIsNotEvolvedMean => {
                "initial latent mean is not the evolved latent mean"
            }
            Self::ContinuousInterceptIsNotDiscreteMeanIncrement => {
                "continuous intercept is not the discrete mean increment"
            }
            Self::ContinuousInterceptIsNotInitialLatentMean => {
                "continuous intercept is not the initial latent mean"
            }
            Self::InitialObservedMeanIsNotEvolvedObservedMean => {
                "first-occasion observed mean is not the evolved observed mean"
            }
            Self::TimeDependentImpulseIsNotContinuousIntercept => {
                "time-dependent predictor impulse is not the continuous intercept"
            }
            Self::TimeDependentImpulseIsNotTimeIndependentEffect => {
                "time-dependent predictor impulse is not the time-independent discrete effect"
            }
            Self::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect => {
                "time-dependent predictor impulse is not the time-varying discrete effect"
            }
            Self::TimeIndependentEffectIsNotContinuousIntercept => {
                "time-independent predictor effect is not the continuous intercept"
            }
            Self::TimeIndependentEffectIsNotTimeDependentImpulse => {
                "time-independent predictor effect is not the time-dependent impulse"
            }
            Self::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect => {
                "time-independent predictor effect is not the time-varying discrete effect"
            }
            Self::TimeIndependentCoefficientIsNotDiscreteEffect => {
                "time-independent predictor coefficient is not the discrete effect"
            }
            Self::TimeDependentImpulseCarryIsNotContemporaneousImpulse => {
                "time-dependent predictor impulse carry is not the contemporaneous impulse"
            }
            Self::TimeDependentImpulseCarryIsNotContinuousIntercept => {
                "time-dependent predictor impulse carry is not the continuous intercept"
            }
            Self::TimeDependentImpulseCarryIsNotTimeIndependentEffect => {
                "time-dependent predictor impulse carry is not the time-independent discrete effect"
            }
            Self::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect => {
                "time-dependent predictor impulse carry is not the time-varying discrete effect"
            }
            Self::EvolvedObservedMeanIsNotImpulseCarryObservedMean => {
                "evolved observed mean is not the impulse-carry observed mean"
            }
            Self::EvolvedObservedMeanIsNotImpulseObservedMean => {
                "evolved observed mean is not the contemporaneous-impulse observed mean"
            }
            Self::ImpulseObservedMeanIsNotImpulseCarryObservedMean => {
                "contemporaneous-impulse observed mean is not the impulse-carry observed mean"
            }
            Self::EvolvedObservedMeanIsNotTimeIndependentObservedMean => {
                "evolved observed mean is not the time-independent-predictor observed mean"
            }
            Self::ImpulseObservedMeanIsNotTimeIndependentObservedMean => {
                "contemporaneous-impulse observed mean is not the time-independent-predictor observed mean"
            }
            Self::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean => {
                "impulse-carry observed mean is not the time-independent-predictor observed mean"
            }
            Self::InitialTimeIndependentEffectIsNotProcessIncrement => {
                "first-occasion time-independent predictor shift is not the process increment"
            }
            Self::InitialTimeIndependentCarryIsNotInitialEffect => {
                "carried first-occasion time-independent predictor is not the first-occasion shift"
            }
            Self::InitialTimeIndependentEffectIsNotContinuousIntercept => {
                "first-occasion time-independent predictor shift is not the continuous intercept"
            }
            Self::InitialTimeIndependentEffectIsNotTimeDependentImpulse => {
                "first-occasion time-independent predictor shift is not the time-dependent impulse"
            }
            Self::InitialTimeIndependentCoefficientIsNotInitialEffect => {
                "first-occasion time-independent predictor coefficient is not the first-occasion shift"
            }
            Self::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean => {
                "evolved observed mean is not the first-occasion time-independent-predictor observed mean"
            }
            Self::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean => {
                "time-independent-predictor observed mean is not the first-occasion time-independent-predictor observed mean"
            }
            Self::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean => {
                "contemporaneous-impulse observed mean is not the first-occasion time-independent-predictor observed mean"
            }
            Self::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean => {
                "impulse-carry observed mean is not the first-occasion time-independent-predictor observed mean"
            }
            Self::InitialTimeDependentEffectIsNotContemporaneousImpulse => {
                "first-occasion time-dependent predictor shift is not the contemporaneous impulse"
            }
            Self::InitialTimeDependentCarryIsNotInitialEffect => {
                "carried first-occasion time-dependent predictor is not the first-occasion shift"
            }
            Self::InitialTimeDependentEffectIsNotContinuousIntercept => {
                "first-occasion time-dependent predictor shift is not the continuous intercept"
            }
            Self::InitialTimeDependentEffectIsNotProcessIncrement => {
                "first-occasion time-dependent predictor shift is not the process increment"
            }
            Self::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect => {
                "first-occasion time-dependent predictor shift is not the first-occasion time-independent predictor shift"
            }
            Self::InitialTimeDependentCoefficientIsNotInitialEffect => {
                "first-occasion time-dependent predictor coefficient is not the first-occasion shift"
            }
            Self::InitialTimeDependentCarryIsNotImpulseCarry => {
                "carried first-occasion time-dependent predictor is not the impulse carry"
            }
            Self::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean => {
                "evolved observed mean is not the first-occasion time-dependent-predictor observed mean"
            }
            Self::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean => {
                "time-independent-predictor observed mean is not the first-occasion time-dependent-predictor observed mean"
            }
            Self::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean => {
                "contemporaneous-impulse observed mean is not the first-occasion time-dependent-predictor observed mean"
            }
            Self::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean => {
                "impulse-carry observed mean is not the first-occasion time-dependent-predictor observed mean"
            }
            Self::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean => {
                "first-occasion time-independent-predictor observed mean is not the first-occasion time-dependent-predictor observed mean"
            }
            Self::LevelChangeRequiresStableDrift => {
                "lasting level-change CINT requires stable negative drift"
            }
            Self::LevelChangeInterceptIsNotImpulse => {
                "level-change CINT is not the contemporaneous impulse"
            }
            Self::LevelChangeInterceptIsNotFreeContinuousIntercept => {
                "level-change CINT is not a free continuous intercept"
            }
            Self::LevelChangeInterceptIsNotProcessIncrement => {
                "level-change CINT is not the time-independent process increment"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PsychometricError {}

#[cfg(test)]
mod tests {
    use super::PsychometricError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            PsychometricError::RawProportionForbidden.to_string(),
            "raw topic proportions are forbidden psychometric indicators"
        );
        assert_eq!(
            PsychometricError::InvalidNumericInput.to_string(),
            "invalid psychometric numeric input"
        );
        assert_eq!(
            PsychometricError::SingularDesign.to_string(),
            "singular psychometric design matrix"
        );
        assert_eq!(
            PsychometricError::FormativeReinterpretationForbidden.to_string(),
            "formative or network constructs cannot be reinterpreted as reflective"
        );
        assert_eq!(
            PsychometricError::CausalUnderidentified.to_string(),
            "temporal precedence is not causal identification"
        );
        assert_eq!(
            PsychometricError::UnresolvedConstruct.to_string(),
            "construct class is unresolved"
        );
        assert_eq!(
            PsychometricError::InvarianceRequired.to_string(),
            "latent-mean comparison requires invariance evidence"
        );
        assert_eq!(
            PsychometricError::EventTimeRequired.to_string(),
            "discrete lag and local log-rate require event time, not another clock"
        );
        assert_eq!(
            PsychometricError::DifferenceQuotientForbidden.to_string(),
            "the difference quotient is not the local continuous-time rate"
        );
        assert_eq!(
            PsychometricError::UnequalIntervalPoolingForbidden.to_string(),
            "discrete lags from unequal event intervals are not one coefficient"
        );
        assert_eq!(
            PsychometricError::UnmatchedTimeVaryingInterval.to_string(),
            "time-varying predictor discrete effect requires matching sampling and constancy intervals"
        );
        assert_eq!(
            PsychometricError::InsufficientClusters.to_string(),
            "within/between recovery requires at least two clusters"
        );
        assert_eq!(
            PsychometricError::InvalidWeight.to_string(),
            "invalid non-negative finite psychometric weight"
        );
        assert_eq!(
            PsychometricError::NonPositiveInterval.to_string(),
            "event-time interval must be strictly positive"
        );
        assert_eq!(
            PsychometricError::InsufficientDraws.to_string(),
            "Rubin total variance requires at least two complete-data draws"
        );
        assert_eq!(
            PsychometricError::StrongInvarianceRequired.to_string(),
            "latent-mean comparison requires strong or strict invariance; metric/weak is not enough"
        );
        assert_eq!(
            PsychometricError::ProcessNoiseIsConditionalVariance.to_string(),
            "discrete process noise is the conditional residual variance, not the unconditional latent variance"
        );
        assert_eq!(
            PsychometricError::StationaryVarianceRequiresStableDrift.to_string(),
            "stationary within-subject variance requires a stable negative drift"
        );
        assert_eq!(
            PsychometricError::FiniteIntervalProcessNoiseIsNotStationary.to_string(),
            "finite-interval process noise is not the asymptotic within-subject variance"
        );
        assert_eq!(
            PsychometricError::TraitVarianceIsNotProcessNoise.to_string(),
            "trait variance is not process noise and is not a diffusion"
        );
        assert_eq!(
            PsychometricError::TraitVarianceIsNotStationaryWithinSubject.to_string(),
            "trait variance is not the stationary within-subject variance"
        );
    }

    #[test]
    fn observed_indicator_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotObservedVariance.to_string(),
            "measurement-error variance is not the observed-indicator variance"
        );
        assert_eq!(
            PsychometricError::LatentVarianceIsNotObservedVariance.to_string(),
            "latent variance is not the observed-indicator variance"
        );
        assert_eq!(
            PsychometricError::ManifestTraitVarianceIsNotMeasurementError.to_string(),
            "manifest-trait variance is not measurement-error variance"
        );
        assert_eq!(
            PsychometricError::LatentLaggedCovarianceIsNotObservedCovariance.to_string(),
            "lagged latent covariance is not the lagged observed-indicator covariance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotLaggedObservedCovariance.to_string(),
            "measurement-error variance is not the lagged observed-indicator covariance"
        );
        assert_eq!(
            PsychometricError::ManifestMeansIsNotObservedMean.to_string(),
            "manifest means are not the observed-indicator mean"
        );
        assert_eq!(
            PsychometricError::LatentMeanIsNotObservedMean.to_string(),
            "latent mean is not the observed-indicator mean"
        );
        assert_eq!(
            PsychometricError::ContinuousInterceptIsNotManifestMeans.to_string(),
            "continuous intercept is not the manifest mean"
        );
        assert_eq!(
            PsychometricError::InitialLatentMeanIsNotEvolvedMean.to_string(),
            "initial latent mean is not the evolved latent mean"
        );
        assert_eq!(
            PsychometricError::ContinuousInterceptIsNotDiscreteMeanIncrement.to_string(),
            "continuous intercept is not the discrete mean increment"
        );
        assert_eq!(
            PsychometricError::ContinuousInterceptIsNotInitialLatentMean.to_string(),
            "continuous intercept is not the initial latent mean"
        );
        assert_eq!(
            PsychometricError::InitialObservedMeanIsNotEvolvedObservedMean.to_string(),
            "first-occasion observed mean is not the evolved observed mean"
        );
    }

    #[test]
    fn time_dependent_impulse_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::TimeDependentImpulseIsNotContinuousIntercept.to_string(),
            "time-dependent predictor impulse is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseIsNotTimeIndependentEffect.to_string(),
            "time-dependent predictor impulse is not the time-independent discrete effect"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseIsNotTimeVaryingDiscreteEffect.to_string(),
            "time-dependent predictor impulse is not the time-varying discrete effect"
        );
        assert_eq!(
            PsychometricError::TimeIndependentEffectIsNotContinuousIntercept.to_string(),
            "time-independent predictor effect is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::TimeIndependentEffectIsNotTimeDependentImpulse.to_string(),
            "time-independent predictor effect is not the time-dependent impulse"
        );
        assert_eq!(
            PsychometricError::TimeIndependentEffectIsNotTimeVaryingDiscreteEffect.to_string(),
            "time-independent predictor effect is not the time-varying discrete effect"
        );
        assert_eq!(
            PsychometricError::TimeIndependentCoefficientIsNotDiscreteEffect.to_string(),
            "time-independent predictor coefficient is not the discrete effect"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseCarryIsNotContemporaneousImpulse.to_string(),
            "time-dependent predictor impulse carry is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseCarryIsNotContinuousIntercept.to_string(),
            "time-dependent predictor impulse carry is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseCarryIsNotTimeIndependentEffect.to_string(),
            "time-dependent predictor impulse carry is not the time-independent discrete effect"
        );
        assert_eq!(
            PsychometricError::TimeDependentImpulseCarryIsNotTimeVaryingDiscreteEffect.to_string(),
            "time-dependent predictor impulse carry is not the time-varying discrete effect"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotImpulseCarryObservedMean.to_string(),
            "evolved observed mean is not the impulse-carry observed mean"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotImpulseObservedMean.to_string(),
            "evolved observed mean is not the contemporaneous-impulse observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseObservedMeanIsNotImpulseCarryObservedMean.to_string(),
            "contemporaneous-impulse observed mean is not the impulse-carry observed mean"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotTimeIndependentObservedMean.to_string(),
            "evolved observed mean is not the time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseObservedMeanIsNotTimeIndependentObservedMean.to_string(),
            "contemporaneous-impulse observed mean is not the time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseCarryObservedMeanIsNotTimeIndependentObservedMean.to_string(),
            "impulse-carry observed mean is not the time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentEffectIsNotProcessIncrement.to_string(),
            "first-occasion time-independent predictor shift is not the process increment"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentCarryIsNotInitialEffect.to_string(),
            "carried first-occasion time-independent predictor is not the first-occasion shift"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentEffectIsNotContinuousIntercept.to_string(),
            "first-occasion time-independent predictor shift is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentEffectIsNotTimeDependentImpulse.to_string(),
            "first-occasion time-independent predictor shift is not the time-dependent impulse"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentCoefficientIsNotInitialEffect.to_string(),
            "first-occasion time-independent predictor coefficient is not the first-occasion shift"
        );
    }

    #[test]
    fn initial_time_independent_observed_mean_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotInitialTimeIndependentObservedMean
                .to_string(),
            "evolved observed mean is not the first-occasion time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeIndependentObservedMean
                .to_string(),
            "time-independent-predictor observed mean is not the first-occasion time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseObservedMeanIsNotInitialTimeIndependentObservedMean
                .to_string(),
            "contemporaneous-impulse observed mean is not the first-occasion time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeIndependentObservedMean
                .to_string(),
            "impulse-carry observed mean is not the first-occasion time-independent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentEffectIsNotContemporaneousImpulse.to_string(),
            "first-occasion time-dependent predictor shift is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentCarryIsNotInitialEffect.to_string(),
            "carried first-occasion time-dependent predictor is not the first-occasion shift"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentEffectIsNotContinuousIntercept.to_string(),
            "first-occasion time-dependent predictor shift is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentEffectIsNotProcessIncrement.to_string(),
            "first-occasion time-dependent predictor shift is not the process increment"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentEffectIsNotInitialTimeIndependentEffect
                .to_string(),
            "first-occasion time-dependent predictor shift is not the first-occasion time-independent predictor shift"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentCoefficientIsNotInitialEffect.to_string(),
            "first-occasion time-dependent predictor coefficient is not the first-occasion shift"
        );
        assert_eq!(
            PsychometricError::InitialTimeDependentCarryIsNotImpulseCarry.to_string(),
            "carried first-occasion time-dependent predictor is not the impulse carry"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotInitialTimeDependentObservedMean.to_string(),
            "evolved observed mean is not the first-occasion time-dependent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::TimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean
                .to_string(),
            "time-independent-predictor observed mean is not the first-occasion time-dependent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseObservedMeanIsNotInitialTimeDependentObservedMean.to_string(),
            "contemporaneous-impulse observed mean is not the first-occasion time-dependent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseCarryObservedMeanIsNotInitialTimeDependentObservedMean
                .to_string(),
            "impulse-carry observed mean is not the first-occasion time-dependent-predictor observed mean"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentObservedMeanIsNotInitialTimeDependentObservedMean
                .to_string(),
            "first-occasion time-independent-predictor observed mean is not the first-occasion time-dependent-predictor observed mean"
        );
    }

    #[test]
    fn level_change_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::LevelChangeRequiresStableDrift.to_string(),
            "lasting level-change CINT requires stable negative drift"
        );
        assert_eq!(
            PsychometricError::LevelChangeInterceptIsNotImpulse.to_string(),
            "level-change CINT is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::LevelChangeInterceptIsNotFreeContinuousIntercept.to_string(),
            "level-change CINT is not a free continuous intercept"
        );
        assert_eq!(
            PsychometricError::LevelChangeInterceptIsNotProcessIncrement.to_string(),
            "level-change CINT is not the time-independent process increment"
        );
    }
}
