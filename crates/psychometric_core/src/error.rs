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
    /// Invariance evidence carried an empty comparison-scope or
    /// model-version label.
    MalformedInvarianceEvidence,
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
    /// Driver §7.2 level-change CINT increment was treated as the
    /// contemporaneous Dirac. `(1 − e^{a Δt}) m x` is not `m x`.
    LevelChangeIncrementIsNotImpulse,
    /// Driver §7.2 level-change CINT increment was treated as `CINT`.
    /// `(1 − e^{a Δt}) m x` is not `κ = −a m x`.
    LevelChangeIncrementIsNotIntercept,
    /// Driver §7.2 level-change CINT increment was treated as the
    /// Eq. 3 process increment. `(1 − e^{a Δt}) m x` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    LevelChangeIncrementIsNotProcessIncrement,
    /// Driver §7.2 extra-process contribution was requested for a
    /// non-negative extra drift. Lasting level change via the extra
    /// latent process requires `ε < 0`. Precisely `ε = 0` causes
    /// computational problems in the printed ctsem specification.
    LevelChangeExtraProcessRequiresNegativeDrift,
    /// Driver §7.2 extra-process contribution was treated as the
    /// contemporaneous Dirac. `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`
    /// is not `m x`.
    LevelChangeExtraProcessIsNotImpulse,
    /// Driver §7.2 extra-process contribution was treated as the
    /// level-change `CINT`. `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`
    /// is not `κ = −a m x`.
    LevelChangeExtraProcessIsNotIntercept,
    /// Driver §7.2 extra-process contribution was treated as the
    /// Eq. 3 level-change increment. `a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a)`
    /// is not `(1 − e^{a Δt}) m x`.
    LevelChangeExtraProcessIsNotIncrement,
    /// Driver Eq. 5 of the evolved mean was treated as the Eq. 5
    /// extra-process observed mean. `τ + λ μ_t` is not
    /// `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`.
    EvolvedObservedMeanIsNotExtraProcessObservedMean,
    /// Driver Eq. 5 of the contemporaneous impulse was treated as
    /// the Eq. 5 extra-process observed mean. `τ + λ(μ_t + m x)` is
    /// not `τ + λ(μ_t + a_{ηξ} x (e^{ε Δt} − e^{a Δt}) / (ε − a))`.
    ImpulseObservedMeanIsNotExtraProcessObservedMean,
    /// Driver §7.2 extra-process contribution was treated as `E(y_t)`.
    /// The contribution is not `τ + λ` of the evolved-plus-contribution
    /// latent mean. The extra process has `LAMBDA` 0 in the printed
    /// specification.
    ExtraProcessContributionIsNotObservedMean,
    /// Driver §7.2 evolved-plus-contribution latent mean was treated
    /// as `E(y_t)`. Equation 5 maps `E(y_t) = τ + λ` of that mean.
    ExtraProcessLatentMeanIsNotObservedMean,
    /// Driver Eq. 5 of the first-occasion extra process was treated
    /// as the Eq. 5 after-t0 extra-process observed mean. `T0TDPREDEFFECT`
    /// on the extra process uses `Δt = t − t0`. `TDPREDEFFECT` after
    /// `t0` uses `t − u` with `t0 < u < t`.
    ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean,
    /// Driver Eq. 5 of the evolved mean was treated as the Eq. 5
    /// after-t0 extra-process observed mean. `τ + λ μ_t` is not
    /// `τ + λ(μ_t + a_{ηξ} x (e^{ε(t−u)} − e^{a(t−u)}) / (ε − a))`.
    EvolvedObservedMeanIsNotAfterExtraProcessObservedMean,
    /// Driver Eq. 5 of the impulse carry was treated as the Eq. 5
    /// after-t0 extra-process observed mean. `e^{a(t−u)} m x` is a
    /// Dirac on the original process. Extra-process `TDPREDEFFECT`
    /// after `t0` drives the original process through `DRIFT`.
    ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean,
    /// Driver §7.2 after-t0 extra-process contribution was treated
    /// as `E(y_t)`. The contribution is not `τ + λ` of the
    /// evolved-plus-after-contribution latent mean.
    AfterExtraProcessContributionIsNotObservedMean,
    /// Driver §7.2 evolved-plus-after-contribution latent mean was
    /// treated as `E(y_t)`. Equation 5 maps `E(y_t) = τ + λ` of
    /// that mean.
    AfterExtraProcessLatentMeanIsNotObservedMean,
    /// Driver §7.2 `asymTIPREDEFFECT` was requested for a non-stable
    /// drift. The expected total change in process means is `-B z / a`
    /// and requires `a < 0`.
    AsymptoticTimeIndependentEffectRequiresStableDrift,
    /// Driver §7.2 `asymTIPREDEFFECT` was treated as `TIPREDEFFECT`.
    /// `-B z / a` is not the coefficient `B`.
    AsymptoticTimeIndependentEffectIsNotCoefficient,
    /// Driver §7.2 `asymTIPREDEFFECT` was treated as the finite-interval
    /// discrete increment. `-B z / a` is not
    /// `A^{-1}[e^{A Δt} − I] B z`.
    AsymptoticTimeIndependentEffectIsNotDiscreteEffect,
    /// Driver §7.2 `asymTIPREDEFFECT` was treated as `CINT`.
    /// `-B z / a` is not `κ`.
    AsymptoticTimeIndependentEffectIsNotContinuousIntercept,
    /// Driver §7.2 `asymTIPREDEFFECT` was treated as the contemporaneous
    /// Dirac. `-B z / a` is not `M x`.
    AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse,
    /// Driver §7.2 `addedTIPREDVAR` was treated as `TRAITVAR`.
    /// `(B / a)² v` is between-subject variance accounted for by a
    /// time-independent predictor, not a zero-drift trait process.
    AsymptoticTimeIndependentVarianceIsNotTraitVariance,
    /// Driver §7.2 `addedTIPREDVAR` was treated as `asymDIFFUSION`.
    /// `(B / a)² v` is not the stationary within-subject variance.
    AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject,
    /// Driver §7.2 `addedTIPREDVAR` was treated as `asymTIPREDEFFECT`.
    /// `(B / a)² v` is a variance, not the expected total change in
    /// process means.
    AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect,
    /// Driver Table 2 `asymCINT` was requested for a non-stable drift.
    /// The expected change in process means for a change in intercept
    /// is `-κ / a` and requires `a < 0`.
    AsymptoticContinuousInterceptRequiresStableDrift,
    /// Driver Table 2 `asymCINT` was treated as `CINT`.
    /// `-κ / a` is not `κ`.
    AsymptoticContinuousInterceptIsNotContinuousIntercept,
    /// Driver Table 2 `asymCINT` was treated as the finite-interval
    /// discrete intercept increment. `-κ / a` is not
    /// `A^{-1}[e^{A Δt} − I] κ`.
    AsymptoticContinuousInterceptIsNotDiscreteIncrement,
    /// Driver Table 2 `asymCINT` was treated as `T0MEANS`.
    /// `-κ / a` is not the first-occasion latent mean.
    AsymptoticContinuousInterceptIsNotInitialLatentMean,
    /// Driver Table 2 `asymCINT` was treated as `asymTIPREDEFFECT`.
    /// `-κ / a` is the intercept contribution. `-B z / a` is the
    /// time-independent predictor contribution. Page 16 of the JSS
    /// article notes that a `T0MEANS` stationarity constraint includes
    /// time-independent predictors; that composition is not this map.
    AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect,
    /// Driver p. 16 stationary `T0MEANS` was treated as free `T0MEANS`.
    /// `-κ / a + −B z / a` is the constrained first-occasion mean, not
    /// the free first-occasion latent mean.
    StationaryInitialLatentMeanIsNotInitialLatentMean,
    /// Driver p. 16 stationary `T0MEANS` was treated as `asymCINT`.
    /// The constraint includes time-independent predictors. `-κ / a`
    /// is not that composition when `B z ≠ 0`.
    StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept,
    /// Driver p. 16 stationary `T0MEANS` was treated as
    /// `asymTIPREDEFFECT`. The constraint includes the intercept
    /// contribution. `-B z / a` is not that composition when `κ ≠ 0`.
    StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect,
    /// Driver p. 16 stationary `T0MEANS` was treated as a finite-
    /// interval discrete latent mean. The constrained first-occasion
    /// mean is not `exp(a Δt) μ_0 + (exp(a Δt) − 1)/a κ`.
    StationaryInitialLatentMeanIsNotDiscreteMean,
    /// Driver Eq. 5 of §4.3 stationary `T0MEANS` was treated as
    /// `MANIFESTMEANS`. `τ + λ(−κ / a + −B z / a)` is not `τ`.
    StationaryInitialObservedMeanIsNotManifestMeans,
    /// Driver §4.3 stationary `T0MEANS` was treated as `E(y_0)`.
    /// `−κ / a + −B z / a` is the constrained latent mean, not the
    /// observed-indicator mean.
    StationaryInitialLatentMeanIsNotObservedMean,
    /// Driver Eq. 5 of a finite-interval evolved mean was treated as
    /// Eq. 5 of §4.3 stationary `T0MEANS`. `τ + λ μ_t` is not
    /// `τ + λ(−κ / a + −B z / a)` when the first occasion is
    /// constrained.
    EvolvedObservedMeanIsNotStationaryInitialObservedMean,
    /// Driver Eq. 5 of `asymCINT` was treated as Eq. 5 of §4.3
    /// stationary `T0MEANS`. `τ + λ(−κ / a)` is not
    /// `τ + λ(−κ / a + −B z / a)` when `B z ≠ 0`.
    AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean,
    /// Driver Eq. 5 of free `T0MEANS` was treated as Eq. 5 of §4.3
    /// stationary `T0MEANS`. `τ + λ μ_0` is not
    /// `τ + λ(−κ / a + −B z / a)`.
    InitialObservedMeanIsNotStationaryInitialObservedMean,
    /// Driver §4.3 / p. 16 stationary `T0VAR` was treated as free
    /// `T0VAR`. `trait + −q / (2 a) + (B / a)² v` is the constrained
    /// first-occasion variance, not the free first-occasion latent
    /// variance.
    StationaryInitialLatentVarianceIsNotInitialLatentVariance,
    /// Driver §4.3 / p. 16 stationary `T0VAR` was treated as
    /// `asymDIFFUSION`. The constraint includes trait variance and
    /// time-independent predictor variance. `-q / (2 a)` is not that
    /// composition when `TRAITVAR` or `addedTIPREDVAR` is nonzero.
    StationaryInitialLatentVarianceIsNotStationaryWithinSubject,
    /// Driver §4.3 / p. 16 stationary `T0VAR` was treated as
    /// `TRAITVAR`. The constraint includes the within-subject
    /// process variance and time-independent predictor variance.
    StationaryInitialLatentVarianceIsNotTraitVariance,
    /// Driver §4.3 / p. 16 stationary `T0VAR` was treated as
    /// `addedTIPREDVAR`. The constraint includes trait variance and
    /// `asymDIFFUSION`. `(B / a)² v` is not that composition when
    /// those contributions are nonzero.
    StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance,
    /// Driver §4.3 / p. 16 stationary `T0VAR` was treated as a
    /// finite-interval discrete latent variance.
    /// `exp(2 a Δt) p + Q_Δt` is not the constrained first-occasion
    /// variance.
    StationaryInitialLatentVarianceIsNotDiscreteVariance,
    /// Driver Eq. 5 of §4.3 stationary `T0VAR` was treated as
    /// `MANIFESTVAR`. `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`
    /// is not `θ`.
    StationaryInitialObservedVarianceIsNotMeasurementError,
    /// Driver §4.3 stationary `T0VAR` was treated as `Var(y_0)`.
    /// `trait + −q / (2 a) + (B / a)² v` is the constrained latent
    /// variance, not the observed-indicator variance.
    StationaryInitialLatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 of a finite-interval evolved variance was treated
    /// as Eq. 5 of §4.3 stationary `T0VAR`. `λ² Var(η_t) + θ` is not
    /// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` when the first
    /// occasion is constrained.
    EvolvedObservedVarianceIsNotStationaryInitialObservedVariance,
    /// Driver Eq. 5 of `asymDIFFUSION` was treated as Eq. 5 of §4.3
    /// stationary `T0VAR`. `λ²(−q / (2 a)) + θ` is not
    /// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` when `TRAITVAR`
    /// or `addedTIPREDVAR` is nonzero.
    StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance,
    /// Driver Eq. 5 of free `T0VAR` was treated as Eq. 5 of §4.3
    /// stationary `T0VAR`. `λ² p_0 + θ` is not
    /// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ`.
    InitialObservedVarianceIsNotStationaryInitialObservedVariance,
    /// Driver §4.3 lagged stationary covariance was treated as
    /// contemporaneous stationary `T0VAR`.
    /// `trait + e^{a Δt}(−q / (2 a)) + (B / a)² v` is not
    /// `trait + −q / (2 a) + (B / a)² v` at a strictly positive lag.
    StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance,
    /// Driver §4.3 lagged stationary covariance was treated as the
    /// decayed total `e^{a Δt}(trait + −q / (2 a) + (B / a)² v)`.
    /// Trait variance and `addedTIPREDVAR` do not decay.
    StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance,
    /// Driver §4.3 trait-plus-state lagged covariance was treated as
    /// lagged stationary `T0VAR`. `trait + e^{a Δt} p` is not that
    /// composition when `addedTIPREDVAR` is nonzero.
    TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance,
    /// Driver §4.3 lagged stationary covariance was treated as
    /// lagged observed covariance. Equation 5 maps
    /// `cov(y_t, y_{t-1}) = λ²` of that covariance plus `ψ`.
    StationaryLaggedLatentCovarianceIsNotObservedCovariance,
    /// Driver Eq. 5 measurement error was treated as lagged
    /// stationary observed covariance. Independent `ε_t` does not
    /// enter `cov(y_t, y_{t-1})`.
    MeasurementErrorIsNotStationaryLaggedObservedCovariance,
    /// Driver Eq. 5 of contemporaneous stationary `T0VAR` was treated
    /// as lagged stationary observed covariance.
    /// `λ²(trait + −q / (2 a) + (B / a)² v) + θ + ψ` includes `θ`
    /// and is not `λ²(trait + e^{a Δt}(−q / (2 a)) + (B / a)² v) + ψ`.
    StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance,
    /// Driver §4.3 later-occasion stationary variance was treated as
    /// lagged stationary covariance. `e^{2 a Δt} p + Q_Δt` of the
    /// within-subject state is not `e^{a Δt} p`.
    StationaryLaterLatentVarianceIsNotLaggedCovariance,
    /// Driver §4.3 later-occasion stationary variance was treated as
    /// the free discrete evolution of the constrained total.
    /// Trait variance and `addedTIPREDVAR` do not enter `Q_Δt`.
    StationaryLaterLatentVarianceIsNotDiscreteVariance,
    /// Driver §4.3 later-occasion stationary variance was treated as
    /// finite-interval process noise. `Q_Δt` is the state residual,
    /// not `trait + e^{2 a Δt} p + Q_Δt + (B / a)² v`.
    StationaryLaterLatentVarianceIsNotProcessNoise,
    /// Driver §4.3 later-occasion stationary variance was treated as
    /// later-occasion observed variance. Equation 5 maps
    /// `Var(y_t) = λ²` of that variance plus `θ + ψ`.
    StationaryLaterLatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 measurement error was treated as later-occasion
    /// stationary observed variance. `θ` is not
    /// `λ²(trait + e^{2 a Δt} p + Q_Δt + (B / a)² v) + θ + ψ`.
    MeasurementErrorIsNotStationaryLaterObservedVariance,
    /// Driver Eq. 5 of lagged §4.3 stationary `T0VAR` was treated as
    /// later-occasion stationary observed variance. Lagged covariance
    /// omits `Q_Δt` and `θ`.
    StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance,
    /// Driver §4.3 predetermined later-occasion variance was treated as
    /// later-occasion stationary `T0VAR`. Free `T0VAR` is not
    /// `−q / (2 a)`.
    PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance,
    /// Driver §4.3 predetermined later-occasion variance was treated as
    /// the free discrete evolution of `trait + p_0 + (B / a)² v`.
    /// Trait variance and `addedTIPREDVAR` do not enter `Q_Δt`.
    PredeterminedLaterLatentVarianceIsNotDiscreteVariance,
    /// Driver §4.3 predetermined later-occasion variance was treated as
    /// free first-occasion `T0VAR`. `e^{2 a Δt} p_0 + Q_Δt` is not `p_0`.
    PredeterminedLaterLatentVarianceIsNotInitialLatentVariance,
    /// Driver §4.3 predetermined later-occasion variance was treated as
    /// predetermined later-occasion observed variance. Equation 5 maps
    /// `Var(y_t) = λ²` of that variance plus `θ + ψ`.
    PredeterminedLaterLatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 measurement error was treated as predetermined
    /// later-occasion observed variance. `θ` is not
    /// `λ²(trait + e^{2 a Δt} p_0 + Q_Δt + (B / a)² v) + θ + ψ`.
    MeasurementErrorIsNotPredeterminedLaterObservedVariance,
    /// Driver Eq. 5 of later-occasion §4.3 stationary `T0VAR` was treated
    /// as predetermined later-occasion observed variance. Stationary
    /// later variance uses `−q / (2 a)`, not free `p_0`.
    StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance,
    /// Driver §4.3 predetermined lagged covariance was treated as lagged
    /// stationary `T0VAR`. Free `T0VAR` is not `−q / (2 a)`.
    PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance,
    /// Driver §4.3 predetermined lagged covariance was treated as
    /// predetermined later-occasion variance. Lagged covariance omits
    /// `Q_Δt` and uses `e^{a Δt} p_0`, not `e^{2 a Δt} p_0`.
    PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance,
    /// Driver §4.3 predetermined lagged covariance was treated as the
    /// decayed total `e^{a Δt}(trait + p_0 + (B / a)² v)`. Trait
    /// variance and `addedTIPREDVAR` do not decay.
    PredeterminedLaggedLatentCovarianceIsNotDecayedTotal,
    /// Driver §4.3 predetermined lagged covariance was treated as free
    /// first-occasion `T0VAR`. `e^{a Δt} p_0` is not `p_0`.
    PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance,
    /// Driver §4.3 predetermined lagged covariance was treated as
    /// predetermined lagged observed covariance. Equation 5 maps
    /// `cov(y_t, y_{t-1}) = λ²` of that covariance plus `ψ`.
    PredeterminedLaggedLatentCovarianceIsNotObservedCovariance,
    /// Driver Eq. 5 measurement error was treated as predetermined
    /// lagged observed covariance. Independent `ε_t` does not enter
    /// `cov(y_t, y_{t-1})`.
    MeasurementErrorIsNotPredeterminedLaggedObservedCovariance,
    /// Driver Eq. 5 of predetermined later-occasion `T0VAR` was treated
    /// as predetermined lagged observed covariance. Later variance
    /// includes `Q_Δt` and `θ`.
    PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance,
    /// Driver Eq. 5 of lagged §4.3 stationary `T0VAR` was treated as
    /// predetermined lagged observed covariance. Stationary lagged
    /// covariance uses `−q / (2 a)`, not free `p_0`.
    StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance,
    /// Driver §4.3 predetermined first-occasion variance was treated as
    /// stationary first-occasion `T0VAR`. Free `T0VAR` is not
    /// `−q / (2 a)`.
    PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance,
    /// Driver §4.3 predetermined first-occasion variance was treated as
    /// free first-occasion `T0VAR`. `trait + p_0 + (B / a)² v` is not
    /// `p_0`.
    PredeterminedInitialLatentVarianceIsNotInitialLatentVariance,
    /// Driver §4.3 predetermined first-occasion variance was treated as
    /// predetermined lagged covariance. First-occasion variance does
    /// not decay the state.
    PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance,
    /// Driver §4.3 predetermined first-occasion variance was treated as
    /// predetermined later-occasion variance. First-occasion variance
    /// omits `Q_Δt`.
    PredeterminedInitialLatentVarianceIsNotLaterLatentVariance,
    /// Driver §4.3 predetermined first-occasion variance was treated as
    /// predetermined first-occasion observed variance. Equation 5 maps
    /// `Var(y_0) = λ²` of that variance plus `θ + ψ`.
    PredeterminedInitialLatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 measurement error was treated as predetermined
    /// first-occasion observed variance. `θ` is not
    /// `λ²(trait + p_0 + (B / a)² v) + θ + ψ`.
    MeasurementErrorIsNotPredeterminedInitialObservedVariance,
    /// Driver Eq. 5 of §4.3 stationary `T0VAR` was treated as
    /// predetermined first-occasion observed variance. Stationary
    /// first-occasion variance uses `−q / (2 a)`, not free `p_0`.
    StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance,
    /// Driver Eq. 5 of predetermined later-occasion `T0VAR` was treated
    /// as predetermined first-occasion observed variance. Later
    /// variance includes `Q_Δt`.
    PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance,
    /// Driver §4.3 later-start lagged covariance of predetermined
    /// `T0VAR` was treated as first-occasion lagged covariance.
    /// Later-start lag includes `e^{a s} Q_u`.
    PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance,
    /// Driver §4.3 later-start lagged covariance of predetermined
    /// `T0VAR` was treated as later-occasion variance. Lagged
    /// covariance is `e^{a s}` of the later state, not that variance.
    PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance,
    /// Driver §4.3 later-start lagged covariance of predetermined
    /// `T0VAR` was treated as lagged stationary `T0VAR`. Free `T0VAR`
    /// is not `−q / (2 a)`.
    PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance,
    /// Driver §4.3 later-start lagged covariance of predetermined
    /// `T0VAR` was treated as the decayed later total. Trait variance
    /// and `addedTIPREDVAR` do not decay.
    PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal,
    /// Driver §4.3 later-start lagged covariance of predetermined
    /// `T0VAR` was treated as later-start lagged observed covariance.
    /// Equation 5 maps `cov(y, y_{lag}) = λ²` of that covariance plus
    /// `ψ`.
    PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance,
    /// Driver Eq. 5 measurement error was treated as later-start lagged
    /// observed covariance of predetermined `T0VAR`. Independent `ε_t`
    /// does not enter.
    MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance,
    /// Driver Eq. 5 of first-occasion lagged predetermined `T0VAR` was
    /// treated as later-start lagged observed covariance. First-occasion
    /// lag omits `e^{a s} Q_u`.
    PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    /// Driver Eq. 5 of lagged §4.3 stationary `T0VAR` was treated as
    /// later-start lagged observed covariance of predetermined `T0VAR`.
    /// Stationary lagged covariance uses `−q / (2 a)`, not free `p_0`.
    StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    /// Driver Eq. 5 of predetermined later-occasion `T0VAR` was treated
    /// as later-start lagged observed covariance. Later variance
    /// includes `Q_Δt` and `θ`.
    PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as later-occasion variance at the later
    /// start. Later-start later-occasion variance adds `Q_s`.
    PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as later-start lagged covariance. Lagged
    /// covariance is `e^{a s}` of the later state and omits `Q_s`.
    PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as later-occasion stationary `T0VAR`. Free
    /// `T0VAR` is not `−q / (2 a)`.
    PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as the evolved later total. Trait variance
    /// and `addedTIPREDVAR` do not enter `Q_s`.
    PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as later-occasion variance over the lag
    /// interval alone. Ignoring `startoffset` omits `e^{2 a s} Q_u`.
    PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance,
    /// Driver §4.3 later-start later-occasion variance of predetermined
    /// `T0VAR` was treated as later-start later-occasion observed
    /// variance. Equation 5 maps `Var(y) = λ²` of that variance plus
    /// `θ + ψ`.
    PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance,
    /// Driver Eq. 5 measurement error was treated as later-start
    /// later-occasion observed variance of predetermined `T0VAR`. `θ`
    /// is not `λ²(trait + e^{2 a s}(e^{2 a u} p_0 + Q_u) + Q_s + (B / a)² v) + θ + ψ`.
    MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance,
    /// Driver Eq. 5 of predetermined later-occasion `T0VAR` was treated
    /// as later-start later-occasion observed variance. Later-occasion
    /// variance at `u` omits `Q_s`.
    PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    /// Driver Eq. 5 of later-start lagged predetermined `T0VAR` was
    /// treated as later-start later-occasion observed variance. Lagged
    /// covariance omits `Q_s` and `θ`.
    PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    /// Driver Eq. 5 of later-occasion §4.3 stationary `T0VAR` was
    /// treated as later-start later-occasion observed variance of
    /// predetermined `T0VAR`. Stationary later variance uses
    /// `−q / (2 a)`, not free `p_0`.
    StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance,
    /// Driver p. 16 `discreteDRIFTstd` was requested with a non-positive
    /// within-subject variance. Footnote 4 standardises `DRIFT` using
    /// only strictly positive `asymDIFFUSION`.
    StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 unstandardised `discreteDRIFT` `e^{a Δt}` was treated
    /// as `discreteDRIFTstd`. Unstandardised `e^{a Δt}` is defined for
    /// growing or zero-diffusion processes; standardised `DRIFT` is not.
    UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift,
    /// Driver §7.1 trait-plus-state autocorrelation was treated as
    /// p. 16 `discreteDRIFTstd`. Footnote 4 uses only `asymDIFFUSION`,
    /// not `TRAITVAR`.
    TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift,
    /// Driver §4.3 / §7.1 trait variance was treated as the p. 16
    /// footnote 4 standardisation variance. `TRAITVAR` is not
    /// `asymDIFFUSION`.
    TraitVarianceIsNotStandardisationVariance,
    /// Driver p. 16 `discreteDIFFUSIONstd` was requested with a
    /// non-positive within-subject variance. Footnote 4 standardises
    /// process noise using only strictly positive `asymDIFFUSION`.
    StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 unstandardised `discreteDIFFUSION` `Q_Δt` was
    /// treated as `discreteDIFFUSIONstd`. Unstandardised `Q_Δt` is
    /// defined for growing or zero-diffusion processes; standardised
    /// `DIFFUSION` is not.
    UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion,
    /// Driver continuous `DIFFUSION` standardisation `q / (−q / (2 a))`
    /// was treated as p. 16 `discreteDIFFUSIONstd`. `−2 a` is not
    /// `Q_Δt / (−q / (2 a))`.
    StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion,
    /// Driver §7.1 trait-contaminated process noise
    /// `Q_Δt / (trait + p + added)` was treated as p. 16
    /// `discreteDIFFUSIONstd`. Footnote 4 uses only `asymDIFFUSION`,
    /// not `TRAITVAR`.
    TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion,
    /// Driver p. 16 `DIFFUSIONstd` was requested with a non-positive
    /// within-subject variance. Footnote 4 standardises continuous
    /// process noise using only strictly positive `asymDIFFUSION`.
    StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 unstandardised `DIFFUSION` `q` was treated as
    /// `DIFFUSIONstd`. Unstandardised `q` is defined for growing or
    /// zero-diffusion processes; standardised `DIFFUSION` is not.
    UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion,
    /// Driver p. 16 `discreteDIFFUSIONstd` `Q_Δt / (−q / (2 a))` was
    /// treated as `DIFFUSIONstd`. `1 − exp(2 a Δt)` is not `−2 a`.
    StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion,
    /// Driver §7.1 trait-contaminated continuous diffusion
    /// `q / (trait + p + added)` was treated as p. 16 `DIFFUSIONstd`.
    /// Footnote 4 uses only `asymDIFFUSION`, not `TRAITVAR`.
    TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion,
    /// Driver p. 16 `DRIFTstd` was requested with a non-positive
    /// within-subject variance. Footnote 4 standardises `DRIFT` using
    /// only strictly positive `asymDIFFUSION`.
    StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 unstandardised `DRIFT` `a` was treated as
    /// `DRIFTstd`. Unstandardised `a` is defined for growing or
    /// zero-diffusion processes; standardised `DRIFT` is not.
    UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift,
    /// Driver p. 16 `discreteDRIFTstd` `e^{a Δt}` was treated as
    /// `DRIFTstd`. The discrete auto-effect is not the continuous
    /// log-rate.
    StandardisedDiscreteDriftIsNotStandardisedContinuousDrift,
    /// Driver §7.1 trait-contaminated continuous drift
    /// `a p / (trait + p + added)` was treated as p. 16 `DRIFTstd`.
    /// Footnote 4 uses only `asymDIFFUSION`, not `TRAITVAR`.
    TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift,
    /// Driver p. 16 `asymTIPREDEFFECTstd` was requested with a
    /// non-positive within-subject variance. Footnote 4 standardises
    /// the affected process using only strictly positive
    /// `asymDIFFUSION`.
    StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 `asymTIPREDEFFECTstd` was requested with a
    /// non-positive predictor variance. Footnote 4 standardises the
    /// affecting predictor using only strictly positive `TIPREDVAR`.
    StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance,
    /// Driver §7.2 unstandardised `asymTIPREDEFFECT` `-B / a` was
    /// treated as p. 16 `asymTIPREDEFFECTstd`. Unstandardised
    /// `-B / a` is defined for a zero coefficient or zero predictor
    /// variance; standardised `asymTIPREDEFFECT` is not.
    UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect,
    /// Driver finite-interval standardised `TIPREDEFFECT`
    /// `A^{-1}[e^{A Δt} − I] B · √v / √p` was treated as p. 16
    /// `asymTIPREDEFFECTstd`. The discrete increment depends on the
    /// event interval; the asymptotic map does not.
    StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect,
    /// Driver §7.1 trait-contaminated asymptotic TI effect
    /// `(-B / a) · √v / √(trait + p + added)` was treated as p. 16
    /// `asymTIPREDEFFECTstd`. Footnote 4 uses only `asymDIFFUSION`,
    /// not `TRAITVAR`.
    TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect,
    /// Driver p. 16 `TIPREDEFFECTstd` was requested with a
    /// non-positive within-subject variance. Footnote 4 standardises
    /// the affected process using only strictly positive
    /// `asymDIFFUSION`.
    StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 `TIPREDEFFECTstd` was requested with a
    /// non-positive predictor variance. Footnote 4 standardises the
    /// affecting predictor using only strictly positive `TIPREDVAR`.
    StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance,
    /// Driver Table 2 unstandardised `TIPREDEFFECT` `B` was treated
    /// as p. 16 `TIPREDEFFECTstd`. Unstandardised `B` is defined for
    /// a zero coefficient or zero predictor variance; standardised
    /// `TIPREDEFFECT` is not.
    UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect,
    /// Driver p. 16 `asymTIPREDEFFECTstd`
    /// `(-B / a) · √v / √(-q / (2 a))` was treated as p. 16
    /// `TIPREDEFFECTstd`. The asymptotic map is the total change, not
    /// the continuous coefficient.
    StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect,
    /// Driver finite-interval standardised `TIPREDEFFECT`
    /// `A^{-1}[e^{A Δt} − I] B · √v / √p` was treated as p. 16
    /// `TIPREDEFFECTstd`. The discrete increment depends on the
    /// event interval; the continuous coefficient does not.
    StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect,
    /// Driver §7.1 trait-contaminated continuous TI effect
    /// `B · √v / √(trait + p + added)` was treated as p. 16
    /// `TIPREDEFFECTstd`. Footnote 4 uses only `asymDIFFUSION`,
    /// not `TRAITVAR`.
    TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect,
    /// Driver Table 3 / p. 16 `T0TIPREDEFFECTstd` was requested with
    /// a non-positive free first-occasion variance. Footnote 4
    /// standardises the affected first-occasion latent using only
    /// strictly positive free `T0VAR`.
    StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance,
    /// Driver Table 3 / p. 16 `T0TIPREDEFFECTstd` was requested with
    /// a non-positive predictor variance. Footnote 4 standardises the
    /// affecting predictor using only strictly positive `TIPREDVAR`.
    StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance,
    /// Driver Table 3 unstandardised `T0TIPREDEFFECT` `t0_b` was
    /// treated as p. 16 `T0TIPREDEFFECTstd`. Unstandardised `t0_b`
    /// is defined for a zero coefficient or zero predictor variance;
    /// standardised `T0TIPREDEFFECT` is not.
    UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect,
    /// Driver p. 16 `TIPREDEFFECTstd`
    /// `B · √v / √(-q / (2 a))` was treated as Table 3 / p. 16
    /// `T0TIPREDEFFECTstd`. The continuous map uses `asymDIFFUSION`;
    /// the first-occasion map uses free `T0VAR`.
    StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect,
    /// Driver p. 16 `asymTIPREDEFFECTstd`
    /// `(-B / a) · √v / √(-q / (2 a))` was treated as Table 3 /
    /// p. 16 `T0TIPREDEFFECTstd`. The asymptotic map is the total
    /// change, not the first-occasion coefficient.
    StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect,
    /// Driver §7.1 trait-contaminated first-occasion TI effect
    /// `t0_b · √v / √(trait + p_0 + added)` was treated as Table 3 /
    /// p. 16 `T0TIPREDEFFECTstd`. Footnote 4 uses only free `T0VAR`,
    /// not `TRAITVAR`.
    TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// §7.2 `addedTIPREDVAR` `(B / a)² v`. The first-occasion extra
    /// variance uses free `T0TIPREDEFFECT`, not `-B / a`.
    InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// Table 3 / p. 16 `T0TIPREDEFFECTstd`. The extra first-occasion
    /// variance is not the standardised coefficient.
    InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// free first-occasion `T0VAR`. `p_0` is the first-occasion
    /// state, not the extra TI variance.
    InitialTimeIndependentVarianceIsNotInitialLatentVariance,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// `TRAITVAR`. Section 4.3 `TRAITVAR` is a zero-drift latent
    /// process, not first-occasion TI extra variance.
    InitialTimeIndependentVarianceIsNotTraitVariance,
    /// Driver Eq. 5 of 2017-era `addedT0TIPREDVAR` `λ² t0_b² v` was
    /// treated as the latent extra `t0_b² v`. The observed extra is
    /// not the latent extra.
    InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance,
    /// Driver Eq. 5 of 2017-era `addedT0TIPREDVAR` `λ² t0_b² v` was
    /// treated as first-occasion observed variance `λ² p_0 + θ`.
    /// The extra is not the full first-occasion `Var(y_0)`.
    InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance,
    /// Driver Eq. 5 of 2017-era `addedT0TIPREDVAR` `λ² t0_b² v` was
    /// treated as Eq. 5 of `addedTIPREDVAR` `λ² (B / a)² v`. The
    /// first-occasion observed extra uses free `T0TIPREDEFFECT`,
    /// not `-B / a`.
    InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance,
    /// Driver Eq. 5 of 2017-era `addedT0TIPREDVAR` `λ² t0_b² v` was
    /// treated as `MANIFESTVAR` `θ`. Measurement error is not extra
    /// observed TI variance.
    InitialTimeIndependentObservedVarianceIsNotMeasurementError,
    /// Driver Eq. 5 of §7.2 `addedTIPREDVAR` `λ² (B / a)² v` was
    /// treated as the latent extra `(B / a)² v`. The observed extra
    /// is not the latent extra.
    AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance,
    /// Driver Eq. 5 of §7.2 `addedTIPREDVAR` `λ² (B / a)² v` was
    /// treated as Eq. 5 of `addedT0TIPREDVAR` `λ² t0_b² v`. The
    /// asymptotic observed extra uses `-B / a`, not free
    /// `T0TIPREDEFFECT`.
    AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance,
    /// Driver Eq. 5 of §7.2 `addedTIPREDVAR` `λ² (B / a)² v` was
    /// treated as stationary observed variance `λ² p + θ`. The extra
    /// is not the full stationary `Var(y)`.
    AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance,
    /// Driver Eq. 5 of §7.2 `addedTIPREDVAR` `λ² (B / a)² v` was
    /// treated as `MANIFESTVAR` `θ`. Measurement error is not extra
    /// observed TI variance.
    AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError,
    /// Driver p. 16 `TDPREDEFFECTstd` was requested with a
    /// non-positive within-subject variance. Footnote 4 standardises
    /// the affected process using only strictly positive
    /// `asymDIFFUSION`.
    StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 `TDPREDEFFECTstd` was requested with a
    /// non-positive predictor variance. Footnote 4 standardises the
    /// affecting predictor using only strictly positive TD predictor
    /// variance.
    StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance,
    /// Driver Table 2 unstandardised `TDPREDEFFECT` `M` was treated
    /// as p. 16 `TDPREDEFFECTstd`. Unstandardised `M` is defined for
    /// a zero coefficient or zero predictor variance; standardised
    /// `TDPREDEFFECT` is not.
    UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect,
    /// Driver p. 16 `TIPREDEFFECTstd` `B · √v / √(-q / (2 a))` was
    /// treated as p. 16 `TDPREDEFFECTstd`. Table 2 names `M`
    /// `TDPREDEFFECT` and `B` `TIPREDEFFECT`. Equal numbers when
    /// `M = B` and the predictor variances match are still distinct
    /// named quantities.
    StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect,
    /// Driver finite-interval standardised `TDPREDEFFECT`
    /// `A^{-1}[e^{A Δt} − I] M · √v / √p` was treated as p. 16
    /// `TDPREDEFFECTstd`. That intercept-style discrete map depends
    /// on the event interval; the continuous Dirac coefficient does
    /// not.
    StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect,
    /// Driver §7.1 trait-contaminated continuous TD effect
    /// `m · √v / √(trait + p + added)` was treated as p. 16
    /// `TDPREDEFFECTstd`. Footnote 4 uses only `asymDIFFUSION`,
    /// not `TRAITVAR`.
    TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect,
    /// Driver Table 3 / p. 16 `T0TDPREDEFFECTstd` was requested with
    /// a non-positive free first-occasion variance. Footnote 4
    /// standardises the affected first-occasion latent using only
    /// strictly positive free `T0VAR`.
    StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance,
    /// Driver Table 3 / p. 16 `T0TDPREDEFFECTstd` was requested with
    /// a non-positive predictor variance. Footnote 4 standardises the
    /// affecting predictor using only strictly positive TD predictor
    /// variance.
    StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance,
    /// Driver Table 3 unstandardised `T0TDPREDEFFECT` `t0_m` was
    /// treated as p. 16 `T0TDPREDEFFECTstd`. Unstandardised `t0_m`
    /// is defined for a zero coefficient or zero predictor variance;
    /// standardised `T0TDPREDEFFECT` is not.
    UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect,
    /// Driver p. 16 `TDPREDEFFECTstd`
    /// `m · √v / √(-q / (2 a))` was treated as Table 3 / p. 16
    /// `T0TDPREDEFFECTstd`. The continuous map uses `asymDIFFUSION`;
    /// the first-occasion map uses free `T0VAR`.
    StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect,
    /// Driver Table 3 / p. 16 `T0TIPREDEFFECTstd`
    /// `t0_b · √v / √p_0` was treated as Table 3 / p. 16
    /// `T0TDPREDEFFECTstd`. Table 3 names different matrices. Equal
    /// numbers when `t0_m = t0_b` are still distinct named
    /// quantities.
    StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect,
    /// Driver §7.1 trait-contaminated first-occasion TD effect
    /// `t0_m · √v / √(trait + p_0 + added)` was treated as Table 3
    /// / p. 16 `T0TDPREDEFFECTstd`. Footnote 4 uses only free
    /// `T0VAR`, not `TRAITVAR`.
    TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect,
    /// Driver p. 16 `T0VARstd` was requested with a non-positive
    /// free first-occasion variance. The 2017-era correlation form
    /// requires strictly positive free `T0VAR`.
    StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance,
    /// Driver Table 2 unstandardised `T0VAR` `p_0` was treated as
    /// p. 16 `T0VARstd`. Unstandardised `p_0` is defined for a zero
    /// first-occasion variance; standardised `T0VAR` is not.
    UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance,
    /// Driver Table 3 / p. 16 `T0TDPREDEFFECTstd`
    /// `t0_m · √v / √p_0` was treated as p. 16 `T0VARstd`. The
    /// effect map depends on `p_0`; the correlation form of free
    /// `T0VAR` does not.
    StandardisedInitialTimeDependentEffectIsNotStandardisedInitialLatentVariance,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// p. 16 `T0VARstd`. Extra TI variance is not the correlation
    /// form of free `T0VAR`.
    InitialTimeIndependentVarianceIsNotStandardisedInitialLatentVariance,
    /// Driver p. 16 `TRAITVARstd` was requested with a non-positive
    /// trait variance. The 2017-era correlation form requires
    /// strictly positive `TRAITVAR` and is not formed when
    /// `TRAITVAR` is zero.
    StandardisedTraitVarianceRequiresPositiveTraitVariance,
    /// Driver Table 2 unstandardised `TRAITVAR` was treated as
    /// p. 16 `TRAITVARstd`. Unstandardised trait variance is
    /// defined for a zero trait; standardised `TRAITVAR` is not.
    UnstandardisedTraitVarianceIsNotStandardisedTraitVariance,
    /// Driver p. 16 `T0VARstd` was treated as p. 16 `TRAITVARstd`.
    /// Equal numbers when both correlations equal 1 are still
    /// distinct named quantities.
    StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// p. 16 `TRAITVARstd`. Extra first-occasion TI variance is not
    /// the correlation form of between-subject `TRAITVAR`.
    InitialTimeIndependentVarianceIsNotStandardisedTraitVariance,
    /// Driver p. 16 `MANIFESTTRAITVARstd` was requested with a
    /// non-positive manifest-trait variance. The 2017-era
    /// correlation form requires strictly positive
    /// `MANIFESTTRAITVAR` and is not formed when
    /// `MANIFESTTRAITVAR` is zero.
    StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance,
    /// Driver Table 2 unstandardised `MANIFESTTRAITVAR` `Ψ_τ` was
    /// treated as p. 16 `MANIFESTTRAITVARstd`. Unstandardised
    /// manifest-trait variance is defined for a zero trait;
    /// standardised `MANIFESTTRAITVAR` is not.
    UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance,
    /// Driver p. 16 `TRAITVARstd` was treated as p. 16
    /// `MANIFESTTRAITVARstd`. Equal numbers when both correlations
    /// equal 1 are still distinct named quantities. `TRAITVAR` is
    /// process-level; `MANIFESTTRAITVAR` is indicator-level.
    StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance,
    /// Driver Table 2 `MANIFESTVAR` `Θ` was treated as p. 16
    /// `MANIFESTTRAITVARstd`. Measurement error is not the
    /// correlation form of indicator-level trait variance.
    MeasurementErrorIsNotStandardisedManifestTraitVariance,
    /// Driver p. 16 `MANIFESTVARstd` was requested with a
    /// non-positive measurement-error variance. The 2017-era
    /// correlation form requires strictly positive `MANIFESTVAR`.
    /// Zero `θ` makes `solve(sqrt(0))` fail in that source.
    StandardisedManifestVarianceRequiresPositiveManifestVariance,
    /// Driver Table 2 unstandardised `MANIFESTVAR` `Θ` was treated
    /// as p. 16 `MANIFESTVARstd`. Unstandardised measurement error
    /// is defined for a zero residual; standardised `MANIFESTVAR`
    /// is not.
    UnstandardisedManifestVarianceIsNotStandardisedManifestVariance,
    /// Driver p. 16 `MANIFESTTRAITVARstd` was treated as p. 16
    /// `MANIFESTVARstd`. Equal numbers when both correlations equal
    /// 1 are still distinct named quantities. `MANIFESTTRAITVAR` is
    /// indicator-level trait variance; `MANIFESTVAR` is
    /// contemporaneous measurement error.
    StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance,
    /// Driver Eq. 5 observed-indicator variance was treated as p. 16
    /// `MANIFESTVARstd`. `λ² Var(η) + θ` is `Var(y)`, not the
    /// correlation form of `Θ`.
    ObservedVarianceIsNotStandardisedManifestVariance,
    /// Driver p. 16 `TIPREDVARstd` was requested with a non-positive
    /// time-independent predictor variance. The 2017-era correlation
    /// form requires strictly positive `TIPREDVAR`. Zero `v` makes
    /// `solve(sqrt(0))` fail in that source.
    StandardisedTimeIndependentPredictorVarianceRequiresPositivePredictorVariance,
    /// Driver Table 2 unstandardised `TIPREDVAR` was treated as p. 16
    /// `TIPREDVARstd`. Unstandardised predictor variance is defined
    /// for a zero predictor; standardised `TIPREDVAR` is not.
    UnstandardisedTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance,
    /// Driver p. 16 `MANIFESTVARstd` was treated as p. 16
    /// `TIPREDVARstd`. Equal numbers when both correlations equal 1
    /// are still distinct named quantities. `MANIFESTVAR` is
    /// contemporaneous measurement error; `TIPREDVAR` is
    /// time-independent predictor variance.
    StandardisedManifestVarianceIsNotStandardisedTimeIndependentPredictorVariance,
    /// Driver §7.2 `addedTIPREDVAR` was treated as p. 16
    /// `TIPREDVARstd`. `(B / a)² v` is extra process variance, not
    /// the correlation form of the predictor covariance.
    AsymptoticTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance,
    /// Driver p. 16 `asymDIFFUSIONstd` was requested with a
    /// non-positive within-subject variance. The 2017-era
    /// correlation form requires strictly positive `asymDIFFUSION`.
    /// Zero `q` makes `solve(sqrt(0))` fail in that source.
    StandardisedAsymptoticDiffusionRequiresPositiveWithinSubjectVariance,
    /// Driver p. 16 unstandardised `asymDIFFUSION` `p` was treated
    /// as `asymDIFFUSIONstd`. Unstandardised within-subject variance
    /// is defined for a zero process; standardised `asymDIFFUSION`
    /// is not.
    UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `TIPREDVARstd` was treated as p. 16
    /// `asymDIFFUSIONstd`. Equal numbers when both correlations
    /// equal 1 are still distinct named quantities. `TIPREDVAR` is
    /// predictor covariance; `asymDIFFUSION` is within-subject
    /// process variance.
    StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `DIFFUSIONstd` `q / p = −2 a` was treated as
    /// `asymDIFFUSIONstd`. Footnote 4 `DIFFUSIONstd` is the
    /// continuous-diffusion ratio, not the correlation of
    /// `asymDIFFUSION`.
    StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `discreteCINTstd` was requested with a
    /// non-positive within-subject variance. Footnote 4
    /// standardisation of the 2017-era `discreteCINT` vector
    /// requires strictly positive `asymDIFFUSION`.
    StandardisedDiscreteContinuousInterceptRequiresPositiveWithinSubjectVariance,
    /// Driver Eq. 3 unstandardised `discreteCINT`
    /// `A^{-1}[e^{A Δt} − I] κ` was treated as `discreteCINTstd`.
    /// Unstandardised discrete intercept is defined for growing
    /// `a ≥ 0` and for zero diffusion; standardised `discreteCINT`
    /// is not.
    UnstandardisedDiscreteContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver p. 16 `CINTstd` analog `κ / √p` was treated as
    /// `discreteCINTstd`. The continuous intercept standardisation
    /// does not depend on the event interval.
    StandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver Table 2 `asymCINT` `/ √p` was treated as
    /// `discreteCINTstd`. `(-κ / a) / √p` is the standardised
    /// total intercept change, not the finite-interval map.
    AsymptoticStandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver p. 16 `asymCINTstd` was requested with a
    /// non-positive within-subject variance. Footnote 4
    /// standardisation of the 2017-era `asymCINT` vector
    /// requires strictly positive `asymDIFFUSION`.
    StandardisedAsymptoticContinuousInterceptRequiresPositiveWithinSubjectVariance,
    /// Driver Table 2 unstandardised `asymCINT` `-κ / a` was
    /// treated as `asymCINTstd`. Unstandardised asymptotic
    /// intercept is defined for a zero process; standardised
    /// `asymCINT` is not.
    UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `CINTstd` analog `κ / √p` was treated as
    /// `asymCINTstd`. The continuous intercept standardisation
    /// is not the standardised total intercept change.
    StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `discreteCINTstd` was treated as
    /// `asymCINTstd`. A finite event interval is not the
    /// `Δt → ∞` intercept change.
    StandardisedDiscreteContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `T0MEANSstd` was requested with a
    /// non-positive first-occasion variance. Footnote 4
    /// standardisation of the 2017-era `T0MEANS` vector
    /// requires strictly positive free `T0VAR`.
    StandardisedInitialLatentMeanRequiresPositiveInitialLatentVariance,
    /// Driver Table 2 unstandardised `T0MEANS` `μ_0` was treated
    /// as `T0MEANSstd`. Unstandardised first-occasion mean is
    /// defined for a zero first-occasion variance; standardised
    /// `T0MEANS` is not.
    UnstandardisedInitialLatentMeanIsNotStandardisedInitialLatentMean,
    /// Driver p. 16 `T0VARstd` was treated as p. 16 `T0MEANSstd`.
    /// Equal numbers when `μ_0 = √p_0` are still distinct named
    /// quantities. `T0VARstd` is the correlation form of free
    /// `T0VAR`; `T0MEANSstd` is the first-occasion mean.
    StandardisedInitialLatentVarianceIsNotStandardisedInitialLatentMean,
    /// Driver p. 16 `T0MEANS` `/ √asymDIFFUSION` was treated as
    /// `T0MEANSstd`. Footnote 4 standardises the first-occasion
    /// mean using free `T0VAR`, not process-dynamics
    /// `asymDIFFUSION`.
    WithinSubjectScaledInitialLatentMeanIsNotStandardisedInitialLatentMean,
    /// Driver p. 16 `MANIFESTMEANSstd` was requested with a
    /// non-positive residual. Footnote 4 standardisation of the
    /// 2017-era `MANIFESTMEANS` vector requires strictly positive
    /// `MANIFESTVAR`.
    StandardisedManifestMeanRequiresPositiveManifestVariance,
    /// Driver Table 2 unstandardised `MANIFESTMEANS` `τ` was treated
    /// as `MANIFESTMEANSstd`. Unstandardised measurement intercept
    /// is defined for a zero residual; standardised `MANIFESTMEANS`
    /// is not.
    UnstandardisedManifestMeanIsNotStandardisedManifestMean,
    /// Driver p. 16 `MANIFESTVARstd` was treated as p. 16
    /// `MANIFESTMEANSstd`. Equal numbers when `τ = √θ` are still
    /// distinct named quantities. `MANIFESTVARstd` is the
    /// correlation form of residual `MANIFESTVAR`;
    /// `MANIFESTMEANSstd` is the measurement intercept.
    StandardisedManifestVarianceIsNotStandardisedManifestMean,
    /// Driver Eq. 5 `τ / √(λ² Var(η) + θ)` was treated as
    /// `MANIFESTMEANSstd`. Footnote 4 standardises the named
    /// intercept using residual `MANIFESTVAR`, not total observed
    /// variance.
    ObservedScaledManifestMeanIsNotStandardisedManifestMean,
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
            Self::MalformedInvarianceEvidence => {
                "invariance evidence requires a non-empty comparison scope and model version"
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
            Self::LevelChangeIncrementIsNotImpulse => {
                "level-change CINT increment is not the contemporaneous impulse"
            }
            Self::LevelChangeIncrementIsNotIntercept => {
                "level-change CINT increment is not the level-change intercept"
            }
            Self::LevelChangeIncrementIsNotProcessIncrement => {
                "level-change CINT increment is not the time-independent process increment"
            }
            Self::LevelChangeExtraProcessRequiresNegativeDrift => {
                "lasting level-change extra process requires strictly negative extra drift"
            }
            Self::LevelChangeExtraProcessIsNotImpulse => {
                "level-change extra-process contribution is not the contemporaneous impulse"
            }
            Self::LevelChangeExtraProcessIsNotIntercept => {
                "level-change extra-process contribution is not the level-change intercept"
            }
            Self::LevelChangeExtraProcessIsNotIncrement => {
                "level-change extra-process contribution is not the level-change increment"
            }
            Self::EvolvedObservedMeanIsNotExtraProcessObservedMean => {
                "evolved observed mean is not the extra-process observed mean"
            }
            Self::ImpulseObservedMeanIsNotExtraProcessObservedMean => {
                "contemporaneous-impulse observed mean is not the extra-process observed mean"
            }
            Self::ExtraProcessContributionIsNotObservedMean => {
                "extra-process contribution is not the extra-process observed mean"
            }
            Self::ExtraProcessLatentMeanIsNotObservedMean => {
                "evolved-plus-contribution latent mean is not the extra-process observed mean"
            }
            Self::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean => {
                "first-occasion extra-process observed mean is not the after-t0 extra-process observed mean"
            }
            Self::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean => {
                "evolved observed mean is not the after-t0 extra-process observed mean"
            }
            Self::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean => {
                "impulse-carry observed mean is not the after-t0 extra-process observed mean"
            }
            Self::AfterExtraProcessContributionIsNotObservedMean => {
                "after-t0 extra-process contribution is not the after-t0 extra-process observed mean"
            }
            Self::AfterExtraProcessLatentMeanIsNotObservedMean => {
                "evolved-plus-after-contribution latent mean is not the after-t0 extra-process observed mean"
            }
            Self::AsymptoticTimeIndependentEffectRequiresStableDrift => {
                "asymptotic time-independent predictor effect requires a stable negative drift"
            }
            Self::AsymptoticTimeIndependentEffectIsNotCoefficient => {
                "asymptotic time-independent predictor effect is not the TIPREDEFFECT coefficient"
            }
            Self::AsymptoticTimeIndependentEffectIsNotDiscreteEffect => {
                "asymptotic time-independent predictor effect is not the finite-interval discrete increment"
            }
            Self::AsymptoticTimeIndependentEffectIsNotContinuousIntercept => {
                "asymptotic time-independent predictor effect is not the continuous intercept"
            }
            Self::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse => {
                "asymptotic time-independent predictor effect is not the contemporaneous impulse"
            }
            Self::AsymptoticTimeIndependentVarianceIsNotTraitVariance => {
                "asymptotic time-independent predictor variance is not trait variance"
            }
            Self::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject => {
                "asymptotic time-independent predictor variance is not the stationary within-subject variance"
            }
            Self::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect => {
                "asymptotic time-independent predictor variance is not the expected total change in process means"
            }
            Self::AsymptoticContinuousInterceptRequiresStableDrift => {
                "asymptotic continuous intercept requires a stable negative drift"
            }
            Self::AsymptoticContinuousInterceptIsNotContinuousIntercept => {
                "asymptotic continuous intercept is not the continuous intercept"
            }
            Self::AsymptoticContinuousInterceptIsNotDiscreteIncrement => {
                "asymptotic continuous intercept is not the finite-interval discrete increment"
            }
            Self::AsymptoticContinuousInterceptIsNotInitialLatentMean => {
                "asymptotic continuous intercept is not the first-occasion latent mean"
            }
            Self::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect => {
                "asymptotic continuous intercept is not the asymptotic time-independent predictor effect"
            }
            Self::StationaryInitialLatentMeanIsNotInitialLatentMean => {
                "stationary first-occasion latent mean is not the free first-occasion latent mean"
            }
            Self::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept => {
                "stationary first-occasion latent mean is not the asymptotic continuous intercept"
            }
            Self::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect => {
                "stationary first-occasion latent mean is not the asymptotic time-independent predictor effect"
            }
            Self::StationaryInitialLatentMeanIsNotDiscreteMean => {
                "stationary first-occasion latent mean is not the finite-interval discrete latent mean"
            }
            Self::StationaryInitialObservedMeanIsNotManifestMeans => {
                "stationary first-occasion observed mean is not the manifest mean"
            }
            Self::StationaryInitialLatentMeanIsNotObservedMean => {
                "stationary first-occasion latent mean is not the first-occasion observed mean"
            }
            Self::EvolvedObservedMeanIsNotStationaryInitialObservedMean => {
                "evolved observed mean is not the stationary first-occasion observed mean"
            }
            Self::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean => {
                "asymptotic-intercept observed mean is not the stationary first-occasion observed mean"
            }
            Self::InitialObservedMeanIsNotStationaryInitialObservedMean => {
                "free first-occasion observed mean is not the stationary first-occasion observed mean"
            }
            Self::StationaryInitialLatentVarianceIsNotInitialLatentVariance => {
                "stationary first-occasion latent variance is not the free first-occasion latent variance"
            }
            Self::StationaryInitialLatentVarianceIsNotStationaryWithinSubject => {
                "stationary first-occasion latent variance is not the asymptotic within-subject variance"
            }
            Self::StationaryInitialLatentVarianceIsNotTraitVariance => {
                "stationary first-occasion latent variance is not the trait variance"
            }
            Self::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance => {
                "stationary first-occasion latent variance is not the asymptotic time-independent predictor variance"
            }
            Self::StationaryInitialLatentVarianceIsNotDiscreteVariance => {
                "stationary first-occasion latent variance is not the finite-interval discrete latent variance"
            }
            Self::StationaryInitialObservedVarianceIsNotMeasurementError => {
                "stationary first-occasion observed variance is not the measurement-error variance"
            }
            Self::StationaryInitialLatentVarianceIsNotObservedVariance => {
                "stationary first-occasion latent variance is not the first-occasion observed variance"
            }
            Self::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance => {
                "evolved observed variance is not the stationary first-occasion observed variance"
            }
            Self::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance => {
                "asymptotic-within-subject observed variance is not the stationary first-occasion observed variance"
            }
            Self::InitialObservedVarianceIsNotStationaryInitialObservedVariance => {
                "free first-occasion observed variance is not the stationary first-occasion observed variance"
            }
            Self::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance => {
                "stationary lagged latent covariance is not the stationary first-occasion latent variance"
            }
            Self::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance => {
                "stationary lagged latent covariance is not the decayed stationary variance"
            }
            Self::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance => {
                "trait-plus-state lagged covariance is not the stationary lagged latent covariance"
            }
            Self::StationaryLaggedLatentCovarianceIsNotObservedCovariance => {
                "stationary lagged latent covariance is not the lagged observed covariance"
            }
            Self::MeasurementErrorIsNotStationaryLaggedObservedCovariance => {
                "measurement-error variance is not the stationary lagged observed covariance"
            }
            Self::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance => {
                "stationary first-occasion observed variance is not the stationary lagged observed covariance"
            }
            Self::StationaryLaterLatentVarianceIsNotLaggedCovariance => {
                "stationary later-occasion latent variance is not the stationary lagged latent covariance"
            }
            Self::StationaryLaterLatentVarianceIsNotDiscreteVariance => {
                "stationary later-occasion latent variance is not the free discrete latent variance"
            }
            Self::StationaryLaterLatentVarianceIsNotProcessNoise => {
                "stationary later-occasion latent variance is not the finite-interval process noise"
            }
            Self::StationaryLaterLatentVarianceIsNotObservedVariance => {
                "stationary later-occasion latent variance is not the later-occasion observed variance"
            }
            Self::MeasurementErrorIsNotStationaryLaterObservedVariance => {
                "measurement-error variance is not the stationary later-occasion observed variance"
            }
            Self::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance => {
                "stationary lagged observed covariance is not the stationary later-occasion observed variance"
            }
            Self::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance => {
                "predetermined later-occasion latent variance is not the stationary later-occasion latent variance"
            }
            Self::PredeterminedLaterLatentVarianceIsNotDiscreteVariance => {
                "predetermined later-occasion latent variance is not the free discrete latent variance"
            }
            Self::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance => {
                "predetermined later-occasion latent variance is not the free first-occasion latent variance"
            }
            Self::PredeterminedLaterLatentVarianceIsNotObservedVariance => {
                "predetermined later-occasion latent variance is not the predetermined later-occasion observed variance"
            }
            Self::MeasurementErrorIsNotPredeterminedLaterObservedVariance => {
                "measurement-error variance is not the predetermined later-occasion observed variance"
            }
            Self::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance => {
                "stationary later-occasion observed variance is not the predetermined later-occasion observed variance"
            }
            Self::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance => {
                "predetermined lagged latent covariance is not the stationary lagged latent covariance"
            }
            Self::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance => {
                "predetermined lagged latent covariance is not the predetermined later-occasion latent variance"
            }
            Self::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal => {
                "predetermined lagged latent covariance is not the decayed predetermined total"
            }
            Self::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance => {
                "predetermined lagged latent covariance is not the free first-occasion latent variance"
            }
            Self::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance => {
                "predetermined lagged latent covariance is not the predetermined lagged observed covariance"
            }
            Self::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance => {
                "measurement-error variance is not the predetermined lagged observed covariance"
            }
            Self::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance => {
                "predetermined later-occasion observed variance is not the predetermined lagged observed covariance"
            }
            Self::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance => {
                "stationary lagged observed covariance is not the predetermined lagged observed covariance"
            }
            Self::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance => {
                "predetermined first-occasion latent variance is not the stationary first-occasion latent variance"
            }
            Self::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance => {
                "predetermined first-occasion latent variance is not the free first-occasion latent variance"
            }
            Self::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance => {
                "predetermined first-occasion latent variance is not the predetermined lagged latent covariance"
            }
            Self::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance => {
                "predetermined first-occasion latent variance is not the predetermined later-occasion latent variance"
            }
            Self::PredeterminedInitialLatentVarianceIsNotObservedVariance => {
                "predetermined first-occasion latent variance is not the predetermined first-occasion observed variance"
            }
            Self::MeasurementErrorIsNotPredeterminedInitialObservedVariance => {
                "measurement-error variance is not the predetermined first-occasion observed variance"
            }
            Self::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance => {
                "stationary first-occasion observed variance is not the predetermined first-occasion observed variance"
            }
            Self::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance => {
                "predetermined later-occasion observed variance is not the predetermined first-occasion observed variance"
            }
            Self::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance => {
                "predetermined later-start lagged latent covariance is not the predetermined first-occasion lagged latent covariance"
            }
            Self::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance => {
                "predetermined later-start lagged latent covariance is not the predetermined later-occasion latent variance"
            }
            Self::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance => {
                "predetermined later-start lagged latent covariance is not the stationary lagged latent covariance"
            }
            Self::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal => {
                "predetermined later-start lagged latent covariance is not the decayed later-occasion total"
            }
            Self::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance => {
                "predetermined later-start lagged latent covariance is not the predetermined later-start lagged observed covariance"
            }
            Self::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance => {
                "measurement-error variance is not the predetermined later-start lagged observed covariance"
            }
            Self::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance => {
                "predetermined first-occasion lagged observed covariance is not the predetermined later-start lagged observed covariance"
            }
            Self::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance => {
                "stationary lagged observed covariance is not the predetermined later-start lagged observed covariance"
            }
            Self::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance => {
                "predetermined later-occasion observed variance is not the predetermined later-start lagged observed covariance"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance => {
                "predetermined later-start later-occasion latent variance is not the predetermined later-occasion latent variance"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance => {
                "predetermined later-start later-occasion latent variance is not the predetermined later-start lagged latent covariance"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance => {
                "predetermined later-start later-occasion latent variance is not the stationary later-occasion latent variance"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal => {
                "predetermined later-start later-occasion latent variance is not the evolved later-occasion total"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance => {
                "predetermined later-start later-occasion latent variance is not the lag-interval later-occasion latent variance"
            }
            Self::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance => {
                "predetermined later-start later-occasion latent variance is not the predetermined later-start later-occasion observed variance"
            }
            Self::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance => {
                "measurement-error variance is not the predetermined later-start later-occasion observed variance"
            }
            Self::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance => {
                "predetermined later-occasion observed variance is not the predetermined later-start later-occasion observed variance"
            }
            Self::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance => {
                "predetermined later-start lagged observed covariance is not the predetermined later-start later-occasion observed variance"
            }
            Self::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance => {
                "stationary later-occasion observed variance is not the predetermined later-start later-occasion observed variance"
            }
            Self::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance => {
                "standardised discrete DRIFT requires strictly positive within-subject variance"
            }
            Self::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift => {
                "unstandardised discrete DRIFT is not standardised discrete DRIFT"
            }
            Self::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift => {
                "trait-plus-state autocorrelation is not standardised discrete DRIFT"
            }
            Self::TraitVarianceIsNotStandardisationVariance => {
                "trait variance is not the standardisation variance"
            }
            Self::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance => {
                "standardised discrete DIFFUSION requires strictly positive within-subject variance"
            }
            Self::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion => {
                "unstandardised discrete DIFFUSION is not standardised discrete DIFFUSION"
            }
            Self::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion => {
                "standardised continuous DIFFUSION is not standardised discrete DIFFUSION"
            }
            Self::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion => {
                "trait-contaminated process noise is not standardised discrete DIFFUSION"
            }
            Self::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance => {
                "standardised continuous DIFFUSION requires strictly positive within-subject variance"
            }
            Self::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion => {
                "unstandardised continuous DIFFUSION is not standardised continuous DIFFUSION"
            }
            Self::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion => {
                "standardised discrete DIFFUSION is not standardised continuous DIFFUSION"
            }
            Self::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion => {
                "trait-contaminated continuous DIFFUSION is not standardised continuous DIFFUSION"
            }
            Self::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance => {
                "standardised continuous DRIFT requires strictly positive within-subject variance"
            }
            Self::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift => {
                "unstandardised continuous DRIFT is not standardised continuous DRIFT"
            }
            Self::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift => {
                "standardised discrete DRIFT is not standardised continuous DRIFT"
            }
            Self::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift => {
                "trait-contaminated continuous DRIFT is not standardised continuous DRIFT"
            }
            Self::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance => {
                "standardised asymptotic time-independent predictor effect requires strictly positive within-subject variance"
            }
            Self::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance => {
                "standardised asymptotic time-independent predictor effect requires strictly positive predictor variance"
            }
            Self::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect => {
                "unstandardised asymptotic time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
            }
            Self::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect => {
                "standardised discrete time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
            }
            Self::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect => {
                "trait-contaminated asymptotic time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
            }
            Self::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance => {
                "standardised continuous time-independent predictor effect requires strictly positive within-subject variance"
            }
            Self::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance => {
                "standardised continuous time-independent predictor effect requires strictly positive predictor variance"
            }
            Self::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect => {
                "unstandardised continuous time-independent predictor effect is not standardised continuous time-independent predictor effect"
            }
            Self::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect => {
                "standardised asymptotic time-independent predictor effect is not standardised continuous time-independent predictor effect"
            }
            Self::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect => {
                "standardised discrete time-independent predictor effect is not standardised continuous time-independent predictor effect"
            }
            Self::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect => {
                "trait-contaminated continuous time-independent predictor effect is not standardised continuous time-independent predictor effect"
            }
            Self::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance => {
                "standardised initial time-independent predictor effect requires strictly positive initial latent variance"
            }
            Self::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance => {
                "standardised initial time-independent predictor effect requires strictly positive predictor variance"
            }
            Self::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect => {
                "unstandardised initial time-independent predictor effect is not standardised initial time-independent predictor effect"
            }
            Self::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect => {
                "standardised continuous time-independent predictor effect is not standardised initial time-independent predictor effect"
            }
            Self::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect => {
                "standardised asymptotic time-independent predictor effect is not standardised initial time-independent predictor effect"
            }
            Self::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect => {
                "trait-contaminated initial time-independent predictor effect is not standardised initial time-independent predictor effect"
            }
            Self::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance => {
                "initial time-independent predictor variance is not asymptotic time-independent predictor variance"
            }
            Self::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect => {
                "initial time-independent predictor variance is not standardised initial time-independent predictor effect"
            }
            Self::InitialTimeIndependentVarianceIsNotInitialLatentVariance => {
                "initial time-independent predictor variance is not initial latent variance"
            }
            Self::InitialTimeIndependentVarianceIsNotTraitVariance => {
                "initial time-independent predictor variance is not trait variance"
            }
            Self::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance => {
                "initial time-independent observed variance is not initial time-independent predictor variance"
            }
            Self::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance => {
                "initial time-independent observed variance is not initial observed variance"
            }
            Self::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance => {
                "initial time-independent observed variance is not asymptotic time-independent observed variance"
            }
            Self::InitialTimeIndependentObservedVarianceIsNotMeasurementError => {
                "initial time-independent observed variance is not measurement-error variance"
            }
            Self::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance => {
                "asymptotic time-independent observed variance is not asymptotic time-independent predictor variance"
            }
            Self::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance => {
                "asymptotic time-independent observed variance is not initial time-independent observed variance"
            }
            Self::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance => {
                "asymptotic time-independent observed variance is not stationary observed variance"
            }
            Self::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError => {
                "asymptotic time-independent observed variance is not measurement-error variance"
            }
            Self::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance => {
                "standardised continuous time-dependent predictor effect requires strictly positive within-subject variance"
            }
            Self::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance => {
                "standardised continuous time-dependent predictor effect requires strictly positive predictor variance"
            }
            Self::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect => {
                "unstandardised continuous time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
            }
            Self::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect => {
                "standardised continuous time-independent predictor effect is not standardised continuous time-dependent predictor effect"
            }
            Self::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect => {
                "standardised discrete time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
            }
            Self::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect => {
                "trait-contaminated continuous time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
            }
            Self::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance => {
                "standardised initial time-dependent predictor effect requires strictly positive initial latent variance"
            }
            Self::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance => {
                "standardised initial time-dependent predictor effect requires strictly positive predictor variance"
            }
            Self::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect => {
                "unstandardised initial time-dependent predictor effect is not standardised initial time-dependent predictor effect"
            }
            Self::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect => {
                "standardised continuous time-dependent predictor effect is not standardised initial time-dependent predictor effect"
            }
            Self::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect => {
                "standardised initial time-independent predictor effect is not standardised initial time-dependent predictor effect"
            }
            Self::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect => {
                "trait-contaminated initial time-dependent predictor effect is not standardised initial time-dependent predictor effect"
            }
            Self::StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance => {
                "standardised initial latent variance requires strictly positive initial latent variance"
            }
            Self::UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance => {
                "unstandardised initial latent variance is not standardised initial latent variance"
            }
            Self::StandardisedInitialTimeDependentEffectIsNotStandardisedInitialLatentVariance => {
                "standardised initial time-dependent predictor effect is not standardised initial latent variance"
            }
            Self::InitialTimeIndependentVarianceIsNotStandardisedInitialLatentVariance => {
                "initial time-independent predictor variance is not standardised initial latent variance"
            }
            Self::StandardisedTraitVarianceRequiresPositiveTraitVariance => {
                "standardised trait variance requires strictly positive trait variance"
            }
            Self::UnstandardisedTraitVarianceIsNotStandardisedTraitVariance => {
                "unstandardised trait variance is not standardised trait variance"
            }
            Self::StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance => {
                "standardised initial latent variance is not standardised trait variance"
            }
            Self::InitialTimeIndependentVarianceIsNotStandardisedTraitVariance => {
                "initial time-independent predictor variance is not standardised trait variance"
            }
            Self::StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance => {
                "standardised manifest-trait variance requires strictly positive manifest-trait variance"
            }
            Self::UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance => {
                "unstandardised manifest-trait variance is not standardised manifest-trait variance"
            }
            Self::StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance => {
                "standardised trait variance is not standardised manifest-trait variance"
            }
            Self::MeasurementErrorIsNotStandardisedManifestTraitVariance => {
                "measurement error is not standardised manifest-trait variance"
            }
            Self::StandardisedManifestVarianceRequiresPositiveManifestVariance => {
                "standardised measurement-error variance requires strictly positive measurement-error variance"
            }
            Self::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance => {
                "unstandardised measurement-error variance is not standardised measurement-error variance"
            }
            Self::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance => {
                "standardised manifest-trait variance is not standardised measurement-error variance"
            }
            Self::ObservedVarianceIsNotStandardisedManifestVariance => {
                "observed-indicator variance is not standardised measurement-error variance"
            }
            Self::StandardisedTimeIndependentPredictorVarianceRequiresPositivePredictorVariance => {
                "standardised time-independent predictor variance requires strictly positive time-independent predictor variance"
            }
            Self::UnstandardisedTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance => {
                "unstandardised time-independent predictor variance is not standardised time-independent predictor variance"
            }
            Self::StandardisedManifestVarianceIsNotStandardisedTimeIndependentPredictorVariance => {
                "standardised measurement-error variance is not standardised time-independent predictor variance"
            }
            Self::AsymptoticTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance => {
                "asymptotic time-independent predictor variance is not standardised time-independent predictor variance"
            }
            Self::StandardisedAsymptoticDiffusionRequiresPositiveWithinSubjectVariance => {
                "standardised asymptotic DIFFUSION requires strictly positive within-subject variance"
            }
            Self::UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion => {
                "unstandardised asymptotic DIFFUSION is not standardised asymptotic DIFFUSION"
            }
            Self::StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion => {
                "standardised time-independent predictor variance is not standardised asymptotic DIFFUSION"
            }
            Self::StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion => {
                "standardised continuous DIFFUSION is not standardised asymptotic DIFFUSION"
            }
            Self::StandardisedDiscreteContinuousInterceptRequiresPositiveWithinSubjectVariance => {
                "standardised discrete continuous intercept requires strictly positive within-subject variance"
            }
            Self::UnstandardisedDiscreteContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept => {
                "unstandardised discrete continuous intercept is not standardised discrete continuous intercept"
            }
            Self::StandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept => {
                "standardised continuous intercept is not standardised discrete continuous intercept"
            }
            Self::AsymptoticStandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept => {
                "asymptotic standardised continuous intercept is not standardised discrete continuous intercept"
            }
            Self::StandardisedAsymptoticContinuousInterceptRequiresPositiveWithinSubjectVariance => {
                "standardised asymptotic continuous intercept requires strictly positive within-subject variance"
            }
            Self::UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "unstandardised asymptotic continuous intercept is not standardised asymptotic continuous intercept"
            }
            Self::StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "standardised continuous intercept is not standardised asymptotic continuous intercept"
            }
            Self::StandardisedDiscreteContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "standardised discrete continuous intercept is not standardised asymptotic continuous intercept"
            }
            Self::StandardisedInitialLatentMeanRequiresPositiveInitialLatentVariance => {
                "standardised initial latent mean requires strictly positive initial latent variance"
            }
            Self::UnstandardisedInitialLatentMeanIsNotStandardisedInitialLatentMean => {
                "unstandardised initial latent mean is not standardised initial latent mean"
            }
            Self::StandardisedInitialLatentVarianceIsNotStandardisedInitialLatentMean => {
                "standardised initial latent variance is not standardised initial latent mean"
            }
            Self::WithinSubjectScaledInitialLatentMeanIsNotStandardisedInitialLatentMean => {
                "within-subject scaled initial latent mean is not standardised initial latent mean"
            }
            Self::StandardisedManifestMeanRequiresPositiveManifestVariance => {
                "standardised manifest mean requires strictly positive measurement error"
            }
            Self::UnstandardisedManifestMeanIsNotStandardisedManifestMean => {
                "unstandardised manifest mean is not standardised manifest mean"
            }
            Self::StandardisedManifestVarianceIsNotStandardisedManifestMean => {
                "standardised manifest variance is not standardised manifest mean"
            }
            Self::ObservedScaledManifestMeanIsNotStandardisedManifestMean => {
                "observed scaled manifest mean is not standardised manifest mean"
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
            PsychometricError::MalformedInvarianceEvidence.to_string(),
            "invariance evidence requires a non-empty comparison scope and model version"
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
        assert_eq!(
            PsychometricError::LevelChangeIncrementIsNotImpulse.to_string(),
            "level-change CINT increment is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::LevelChangeIncrementIsNotIntercept.to_string(),
            "level-change CINT increment is not the level-change intercept"
        );
        assert_eq!(
            PsychometricError::LevelChangeIncrementIsNotProcessIncrement.to_string(),
            "level-change CINT increment is not the time-independent process increment"
        );
        assert_eq!(
            PsychometricError::LevelChangeExtraProcessRequiresNegativeDrift.to_string(),
            "lasting level-change extra process requires strictly negative extra drift"
        );
        assert_eq!(
            PsychometricError::LevelChangeExtraProcessIsNotImpulse.to_string(),
            "level-change extra-process contribution is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::LevelChangeExtraProcessIsNotIntercept.to_string(),
            "level-change extra-process contribution is not the level-change intercept"
        );
        assert_eq!(
            PsychometricError::LevelChangeExtraProcessIsNotIncrement.to_string(),
            "level-change extra-process contribution is not the level-change increment"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotExtraProcessObservedMean.to_string(),
            "evolved observed mean is not the extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseObservedMeanIsNotExtraProcessObservedMean.to_string(),
            "contemporaneous-impulse observed mean is not the extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::ExtraProcessContributionIsNotObservedMean.to_string(),
            "extra-process contribution is not the extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::ExtraProcessLatentMeanIsNotObservedMean.to_string(),
            "evolved-plus-contribution latent mean is not the extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::ExtraProcessObservedMeanIsNotAfterExtraProcessObservedMean
                .to_string(),
            "first-occasion extra-process observed mean is not the after-t0 extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotAfterExtraProcessObservedMean.to_string(),
            "evolved observed mean is not the after-t0 extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::ImpulseCarryObservedMeanIsNotAfterExtraProcessObservedMean
                .to_string(),
            "impulse-carry observed mean is not the after-t0 extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::AfterExtraProcessContributionIsNotObservedMean.to_string(),
            "after-t0 extra-process contribution is not the after-t0 extra-process observed mean"
        );
        assert_eq!(
            PsychometricError::AfterExtraProcessLatentMeanIsNotObservedMean.to_string(),
            "evolved-plus-after-contribution latent mean is not the after-t0 extra-process observed mean"
        );
    }

    #[test]
    fn asymptotic_time_independent_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentEffectRequiresStableDrift.to_string(),
            "asymptotic time-independent predictor effect requires a stable negative drift"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentEffectIsNotCoefficient.to_string(),
            "asymptotic time-independent predictor effect is not the TIPREDEFFECT coefficient"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentEffectIsNotDiscreteEffect.to_string(),
            "asymptotic time-independent predictor effect is not the finite-interval discrete increment"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentEffectIsNotContinuousIntercept.to_string(),
            "asymptotic time-independent predictor effect is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentEffectIsNotTimeDependentImpulse.to_string(),
            "asymptotic time-independent predictor effect is not the contemporaneous impulse"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentVarianceIsNotTraitVariance.to_string(),
            "asymptotic time-independent predictor variance is not trait variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentVarianceIsNotStationaryWithinSubject
                .to_string(),
            "asymptotic time-independent predictor variance is not the stationary within-subject variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentVarianceIsNotAsymptoticEffect.to_string(),
            "asymptotic time-independent predictor variance is not the expected total change in process means"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptRequiresStableDrift.to_string(),
            "asymptotic continuous intercept requires a stable negative drift"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptIsNotContinuousIntercept.to_string(),
            "asymptotic continuous intercept is not the continuous intercept"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptIsNotDiscreteIncrement.to_string(),
            "asymptotic continuous intercept is not the finite-interval discrete increment"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptIsNotInitialLatentMean.to_string(),
            "asymptotic continuous intercept is not the first-occasion latent mean"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptIsNotAsymptoticTimeIndependentEffect
                .to_string(),
            "asymptotic continuous intercept is not the asymptotic time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentMeanIsNotInitialLatentMean.to_string(),
            "stationary first-occasion latent mean is not the free first-occasion latent mean"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticContinuousIntercept
                .to_string(),
            "stationary first-occasion latent mean is not the asymptotic continuous intercept"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentMeanIsNotAsymptoticTimeIndependentEffect
                .to_string(),
            "stationary first-occasion latent mean is not the asymptotic time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentMeanIsNotDiscreteMean.to_string(),
            "stationary first-occasion latent mean is not the finite-interval discrete latent mean"
        );
        assert_eq!(
            PsychometricError::StationaryInitialObservedMeanIsNotManifestMeans.to_string(),
            "stationary first-occasion observed mean is not the manifest mean"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentMeanIsNotObservedMean.to_string(),
            "stationary first-occasion latent mean is not the first-occasion observed mean"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedMeanIsNotStationaryInitialObservedMean.to_string(),
            "evolved observed mean is not the stationary first-occasion observed mean"
        );
        assert_eq!(
            PsychometricError::AsymptoticContinuousInterceptObservedMeanIsNotStationaryInitialObservedMean
                .to_string(),
            "asymptotic-intercept observed mean is not the stationary first-occasion observed mean"
        );
        assert_eq!(
            PsychometricError::InitialObservedMeanIsNotStationaryInitialObservedMean.to_string(),
            "free first-occasion observed mean is not the stationary first-occasion observed mean"
        );
    }

    #[test]
    fn stationary_initial_latent_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotInitialLatentVariance
                .to_string(),
            "stationary first-occasion latent variance is not the free first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotStationaryWithinSubject
                .to_string(),
            "stationary first-occasion latent variance is not the asymptotic within-subject variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotTraitVariance.to_string(),
            "stationary first-occasion latent variance is not the trait variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotAsymptoticTimeIndependentVariance
                .to_string(),
            "stationary first-occasion latent variance is not the asymptotic time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotDiscreteVariance.to_string(),
            "stationary first-occasion latent variance is not the finite-interval discrete latent variance"
        );
    }

    #[test]
    fn stationary_initial_observed_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StationaryInitialObservedVarianceIsNotMeasurementError.to_string(),
            "stationary first-occasion observed variance is not the measurement-error variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialLatentVarianceIsNotObservedVariance.to_string(),
            "stationary first-occasion latent variance is not the first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::EvolvedObservedVarianceIsNotStationaryInitialObservedVariance
                .to_string(),
            "evolved observed variance is not the stationary first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::StationaryWithinSubjectObservedVarianceIsNotStationaryInitialObservedVariance
                .to_string(),
            "asymptotic-within-subject observed variance is not the stationary first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::InitialObservedVarianceIsNotStationaryInitialObservedVariance
                .to_string(),
            "free first-occasion observed variance is not the stationary first-occasion observed variance"
        );
    }

    #[test]
    fn stationary_lagged_covariance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StationaryLaggedLatentCovarianceIsNotStationaryInitialLatentVariance
                .to_string(),
            "stationary lagged latent covariance is not the stationary first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::StationaryLaggedLatentCovarianceIsNotDecayedStationaryVariance
                .to_string(),
            "stationary lagged latent covariance is not the decayed stationary variance"
        );
        assert_eq!(
            PsychometricError::TraitPlusStateLaggedCovarianceIsNotStationaryLaggedLatentCovariance
                .to_string(),
            "trait-plus-state lagged covariance is not the stationary lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::StationaryLaggedLatentCovarianceIsNotObservedCovariance.to_string(),
            "stationary lagged latent covariance is not the lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotStationaryLaggedObservedCovariance.to_string(),
            "measurement-error variance is not the stationary lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialObservedVarianceIsNotStationaryLaggedObservedCovariance
                .to_string(),
            "stationary first-occasion observed variance is not the stationary lagged observed covariance"
        );
    }

    #[test]
    fn stationary_later_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StationaryLaterLatentVarianceIsNotLaggedCovariance.to_string(),
            "stationary later-occasion latent variance is not the stationary lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::StationaryLaterLatentVarianceIsNotDiscreteVariance.to_string(),
            "stationary later-occasion latent variance is not the free discrete latent variance"
        );
        assert_eq!(
            PsychometricError::StationaryLaterLatentVarianceIsNotProcessNoise.to_string(),
            "stationary later-occasion latent variance is not the finite-interval process noise"
        );
        assert_eq!(
            PsychometricError::StationaryLaterLatentVarianceIsNotObservedVariance.to_string(),
            "stationary later-occasion latent variance is not the later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotStationaryLaterObservedVariance.to_string(),
            "measurement-error variance is not the stationary later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotStationaryLaterObservedVariance
                .to_string(),
            "stationary lagged observed covariance is not the stationary later-occasion observed variance"
        );
    }

    #[test]
    fn predetermined_later_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::PredeterminedLaterLatentVarianceIsNotStationaryLaterLatentVariance
                .to_string(),
            "predetermined later-occasion latent variance is not the stationary later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLatentVarianceIsNotDiscreteVariance.to_string(),
            "predetermined later-occasion latent variance is not the free discrete latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLatentVarianceIsNotInitialLatentVariance
                .to_string(),
            "predetermined later-occasion latent variance is not the free first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLatentVarianceIsNotObservedVariance.to_string(),
            "predetermined later-occasion latent variance is not the predetermined later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotPredeterminedLaterObservedVariance.to_string(),
            "measurement-error variance is not the predetermined later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterObservedVariance
                .to_string(),
            "stationary later-occasion observed variance is not the predetermined later-occasion observed variance"
        );
    }

    #[test]
    fn predetermined_lagged_covariance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::PredeterminedLaggedLatentCovarianceIsNotStationaryLaggedCovariance
                .to_string(),
            "predetermined lagged latent covariance is not the stationary lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaggedLatentCovarianceIsNotLaterLatentVariance
                .to_string(),
            "predetermined lagged latent covariance is not the predetermined later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaggedLatentCovarianceIsNotDecayedTotal.to_string(),
            "predetermined lagged latent covariance is not the decayed predetermined total"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaggedLatentCovarianceIsNotInitialLatentVariance
                .to_string(),
            "predetermined lagged latent covariance is not the free first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaggedLatentCovarianceIsNotObservedCovariance
                .to_string(),
            "predetermined lagged latent covariance is not the predetermined lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotPredeterminedLaggedObservedCovariance
                .to_string(),
            "measurement-error variance is not the predetermined lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaggedObservedCovariance
                .to_string(),
            "predetermined later-occasion observed variance is not the predetermined lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaggedObservedCovariance
                .to_string(),
            "stationary lagged observed covariance is not the predetermined lagged observed covariance"
        );
    }

    #[test]
    fn predetermined_initial_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotStationaryInitialLatentVariance
                .to_string(),
            "predetermined first-occasion latent variance is not the stationary first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotInitialLatentVariance
                .to_string(),
            "predetermined first-occasion latent variance is not the free first-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotLaggedLatentCovariance
                .to_string(),
            "predetermined first-occasion latent variance is not the predetermined lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotLaterLatentVariance
                .to_string(),
            "predetermined first-occasion latent variance is not the predetermined later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedInitialLatentVarianceIsNotObservedVariance.to_string(),
            "predetermined first-occasion latent variance is not the predetermined first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotPredeterminedInitialObservedVariance
                .to_string(),
            "measurement-error variance is not the predetermined first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::StationaryInitialObservedVarianceIsNotPredeterminedInitialObservedVariance
                .to_string(),
            "stationary first-occasion observed variance is not the predetermined first-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedInitialObservedVariance
                .to_string(),
            "predetermined later-occasion observed variance is not the predetermined first-occasion observed variance"
        );
    }

    #[test]
    fn predetermined_later_lagged_covariance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotPredeterminedLaggedCovariance
                .to_string(),
            "predetermined later-start lagged latent covariance is not the predetermined first-occasion lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotLaterLatentVariance
                .to_string(),
            "predetermined later-start lagged latent covariance is not the predetermined later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotStationaryLaggedCovariance
                .to_string(),
            "predetermined later-start lagged latent covariance is not the stationary lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotDecayedLaterTotal
                .to_string(),
            "predetermined later-start lagged latent covariance is not the decayed later-occasion total"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedLatentCovarianceIsNotObservedCovariance
                .to_string(),
            "predetermined later-start lagged latent covariance is not the predetermined later-start lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotPredeterminedLaterLaggedObservedCovariance
                .to_string(),
            "measurement-error variance is not the predetermined later-start lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
                .to_string(),
            "predetermined first-occasion lagged observed covariance is not the predetermined later-start lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::StationaryLaggedObservedCovarianceIsNotPredeterminedLaterLaggedObservedCovariance
                .to_string(),
            "stationary lagged observed covariance is not the predetermined later-start lagged observed covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterLaggedObservedCovariance
                .to_string(),
            "predetermined later-occasion observed variance is not the predetermined later-start lagged observed covariance"
        );
    }

    #[test]
    fn predetermined_later_start_later_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLatentVariance
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the predetermined later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLaterLaggedCovariance
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the predetermined later-start lagged latent covariance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotStationaryLaterLatentVariance
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the stationary later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotDecayedLaterTotal
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the evolved later-occasion total"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotLagIntervalLaterLatentVariance
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the lag-interval later-occasion latent variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterStartLaterLatentVarianceIsNotObservedVariance
                .to_string(),
            "predetermined later-start later-occasion latent variance is not the predetermined later-start later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotPredeterminedLaterStartLaterObservedVariance
                .to_string(),
            "measurement-error variance is not the predetermined later-start later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
                .to_string(),
            "predetermined later-occasion observed variance is not the predetermined later-start later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::PredeterminedLaterLaggedObservedCovarianceIsNotPredeterminedLaterStartLaterObservedVariance
                .to_string(),
            "predetermined later-start lagged observed covariance is not the predetermined later-start later-occasion observed variance"
        );
        assert_eq!(
            PsychometricError::StationaryLaterObservedVarianceIsNotPredeterminedLaterStartLaterObservedVariance
                .to_string(),
            "stationary later-occasion observed variance is not the predetermined later-start later-occasion observed variance"
        );
    }

    #[test]
    fn standardised_discrete_drift_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedDiscreteDriftRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised discrete DRIFT requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedDiscreteDriftIsNotStandardisedDiscreteDrift
                .to_string(),
            "unstandardised discrete DRIFT is not standardised discrete DRIFT"
        );
        assert_eq!(
            PsychometricError::TraitPlusStateAutocorrelationIsNotStandardisedDiscreteDrift
                .to_string(),
            "trait-plus-state autocorrelation is not standardised discrete DRIFT"
        );
        assert_eq!(
            PsychometricError::TraitVarianceIsNotStandardisationVariance.to_string(),
            "trait variance is not the standardisation variance"
        );
    }

    #[test]
    fn standardised_discrete_diffusion_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedDiscreteDiffusionRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised discrete DIFFUSION requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedDiscreteDiffusionIsNotStandardisedDiscreteDiffusion
                .to_string(),
            "unstandardised discrete DIFFUSION is not standardised discrete DIFFUSION"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedDiscreteDiffusion
                .to_string(),
            "standardised continuous DIFFUSION is not standardised discrete DIFFUSION"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedProcessNoiseIsNotStandardisedDiscreteDiffusion
                .to_string(),
            "trait-contaminated process noise is not standardised discrete DIFFUSION"
        );
    }

    #[test]
    fn standardised_continuous_diffusion_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedContinuousDiffusionRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised continuous DIFFUSION requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedContinuousDiffusionIsNotStandardisedContinuousDiffusion
                .to_string(),
            "unstandardised continuous DIFFUSION is not standardised continuous DIFFUSION"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteDiffusionIsNotStandardisedContinuousDiffusion
                .to_string(),
            "standardised discrete DIFFUSION is not standardised continuous DIFFUSION"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedContinuousDiffusionIsNotStandardisedContinuousDiffusion
                .to_string(),
            "trait-contaminated continuous DIFFUSION is not standardised continuous DIFFUSION"
        );
    }

    #[test]
    fn standardised_continuous_drift_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedContinuousDriftRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised continuous DRIFT requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedContinuousDriftIsNotStandardisedContinuousDrift
                .to_string(),
            "unstandardised continuous DRIFT is not standardised continuous DRIFT"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteDriftIsNotStandardisedContinuousDrift
                .to_string(),
            "standardised discrete DRIFT is not standardised continuous DRIFT"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedContinuousDriftIsNotStandardisedContinuousDrift
                .to_string(),
            "trait-contaminated continuous DRIFT is not standardised continuous DRIFT"
        );
    }

    #[test]
    fn standardised_asymptotic_time_independent_effect_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised asymptotic time-independent predictor effect requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectRequiresPositivePredictorVariance
                .to_string(),
            "standardised asymptotic time-independent predictor effect requires strictly positive predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
                .to_string(),
            "unstandardised asymptotic time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
                .to_string(),
            "standardised discrete time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedAsymptoticTimeIndependentEffectIsNotStandardisedAsymptoticTimeIndependentEffect
                .to_string(),
            "trait-contaminated asymptotic time-independent predictor effect is not standardised asymptotic time-independent predictor effect"
        );
    }

    #[test]
    fn standardised_continuous_time_independent_effect_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised continuous time-independent predictor effect requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeIndependentEffectRequiresPositivePredictorVariance
                .to_string(),
            "standardised continuous time-independent predictor effect requires strictly positive predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
                .to_string(),
            "unstandardised continuous time-independent predictor effect is not standardised continuous time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
                .to_string(),
            "standardised asymptotic time-independent predictor effect is not standardised continuous time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
                .to_string(),
            "standardised discrete time-independent predictor effect is not standardised continuous time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeIndependentEffect
                .to_string(),
            "trait-contaminated continuous time-independent predictor effect is not standardised continuous time-independent predictor effect"
        );
    }

    #[test]
    fn standardised_initial_time_independent_effect_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositiveInitialLatentVariance
                .to_string(),
            "standardised initial time-independent predictor effect requires strictly positive initial latent variance"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialTimeIndependentEffectRequiresPositivePredictorVariance
                .to_string(),
            "standardised initial time-independent predictor effect requires strictly positive predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
                .to_string(),
            "unstandardised initial time-independent predictor effect is not standardised initial time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
                .to_string(),
            "standardised continuous time-independent predictor effect is not standardised initial time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedAsymptoticTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
                .to_string(),
            "standardised asymptotic time-independent predictor effect is not standardised initial time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedInitialTimeIndependentEffectIsNotStandardisedInitialTimeIndependentEffect
                .to_string(),
            "trait-contaminated initial time-independent predictor effect is not standardised initial time-independent predictor effect"
        );
    }

    #[test]
    fn initial_time_independent_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotAsymptoticTimeIndependentVariance
                .to_string(),
            "initial time-independent predictor variance is not asymptotic time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialTimeIndependentEffect
                .to_string(),
            "initial time-independent predictor variance is not standardised initial time-independent predictor effect"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotInitialLatentVariance.to_string(),
            "initial time-independent predictor variance is not initial latent variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotTraitVariance.to_string(),
            "initial time-independent predictor variance is not trait variance"
        );
    }

    #[test]
    fn initial_time_independent_observed_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialTimeIndependentVariance
                .to_string(),
            "initial time-independent observed variance is not initial time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotInitialObservedVariance
                .to_string(),
            "initial time-independent observed variance is not initial observed variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentObservedVariance
                .to_string(),
            "initial time-independent observed variance is not asymptotic time-independent observed variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentObservedVarianceIsNotMeasurementError
                .to_string(),
            "initial time-independent observed variance is not measurement-error variance"
        );
    }

    #[test]
    fn asymptotic_time_independent_observed_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotAsymptoticTimeIndependentVariance
                .to_string(),
            "asymptotic time-independent observed variance is not asymptotic time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotInitialTimeIndependentObservedVariance
                .to_string(),
            "asymptotic time-independent observed variance is not initial time-independent observed variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotStationaryObservedVariance
                .to_string(),
            "asymptotic time-independent observed variance is not stationary observed variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentObservedVarianceIsNotMeasurementError
                .to_string(),
            "asymptotic time-independent observed variance is not measurement-error variance"
        );
    }

    #[test]
    fn standardised_continuous_time_dependent_effect_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised continuous time-dependent predictor effect requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeDependentEffectRequiresPositivePredictorVariance
                .to_string(),
            "standardised continuous time-dependent predictor effect requires strictly positive predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
                .to_string(),
            "unstandardised continuous time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeIndependentEffectIsNotStandardisedContinuousTimeDependentEffect
                .to_string(),
            "standardised continuous time-independent predictor effect is not standardised continuous time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
                .to_string(),
            "standardised discrete time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedContinuousTimeDependentEffectIsNotStandardisedContinuousTimeDependentEffect
                .to_string(),
            "trait-contaminated continuous time-dependent predictor effect is not standardised continuous time-dependent predictor effect"
        );
    }

    #[test]
    fn standardised_initial_time_dependent_effect_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositiveInitialLatentVariance
                .to_string(),
            "standardised initial time-dependent predictor effect requires strictly positive initial latent variance"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialTimeDependentEffectRequiresPositivePredictorVariance
                .to_string(),
            "standardised initial time-dependent predictor effect requires strictly positive predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
                .to_string(),
            "unstandardised initial time-dependent predictor effect is not standardised initial time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
                .to_string(),
            "standardised continuous time-dependent predictor effect is not standardised initial time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialTimeIndependentEffectIsNotStandardisedInitialTimeDependentEffect
                .to_string(),
            "standardised initial time-independent predictor effect is not standardised initial time-dependent predictor effect"
        );
        assert_eq!(
            PsychometricError::TraitContaminatedInitialTimeDependentEffectIsNotStandardisedInitialTimeDependentEffect
                .to_string(),
            "trait-contaminated initial time-dependent predictor effect is not standardised initial time-dependent predictor effect"
        );
    }

    #[test]
    fn standardised_initial_latent_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance
                .to_string(),
            "standardised initial latent variance requires strictly positive initial latent variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance
                .to_string(),
            "unstandardised initial latent variance is not standardised initial latent variance"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialTimeDependentEffectIsNotStandardisedInitialLatentVariance
                .to_string(),
            "standardised initial time-dependent predictor effect is not standardised initial latent variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedInitialLatentVariance
                .to_string(),
            "initial time-independent predictor variance is not standardised initial latent variance"
        );
    }

    #[test]
    fn standardised_trait_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedTraitVarianceRequiresPositiveTraitVariance.to_string(),
            "standardised trait variance requires strictly positive trait variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedTraitVarianceIsNotStandardisedTraitVariance
                .to_string(),
            "unstandardised trait variance is not standardised trait variance"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance
                .to_string(),
            "standardised initial latent variance is not standardised trait variance"
        );
        assert_eq!(
            PsychometricError::InitialTimeIndependentVarianceIsNotStandardisedTraitVariance
                .to_string(),
            "initial time-independent predictor variance is not standardised trait variance"
        );
    }

    #[test]
    fn standardised_manifest_trait_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance
                .to_string(),
            "standardised manifest-trait variance requires strictly positive manifest-trait variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance
                .to_string(),
            "unstandardised manifest-trait variance is not standardised manifest-trait variance"
        );
        assert_eq!(
            PsychometricError::StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance
                .to_string(),
            "standardised trait variance is not standardised manifest-trait variance"
        );
        assert_eq!(
            PsychometricError::MeasurementErrorIsNotStandardisedManifestTraitVariance.to_string(),
            "measurement error is not standardised manifest-trait variance"
        );
    }

    #[test]
    fn standardised_manifest_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedManifestVarianceRequiresPositiveManifestVariance
                .to_string(),
            "standardised measurement-error variance requires strictly positive measurement-error variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedManifestVarianceIsNotStandardisedManifestVariance
                .to_string(),
            "unstandardised measurement-error variance is not standardised measurement-error variance"
        );
        assert_eq!(
            PsychometricError::StandardisedManifestTraitVarianceIsNotStandardisedManifestVariance
                .to_string(),
            "standardised manifest-trait variance is not standardised measurement-error variance"
        );
        assert_eq!(
            PsychometricError::ObservedVarianceIsNotStandardisedManifestVariance.to_string(),
            "observed-indicator variance is not standardised measurement-error variance"
        );
    }

    #[test]
    fn standardised_time_independent_predictor_variance_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedTimeIndependentPredictorVarianceRequiresPositivePredictorVariance
                .to_string(),
            "standardised time-independent predictor variance requires strictly positive time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance
                .to_string(),
            "unstandardised time-independent predictor variance is not standardised time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::StandardisedManifestVarianceIsNotStandardisedTimeIndependentPredictorVariance
                .to_string(),
            "standardised measurement-error variance is not standardised time-independent predictor variance"
        );
        assert_eq!(
            PsychometricError::AsymptoticTimeIndependentPredictorVarianceIsNotStandardisedTimeIndependentPredictorVariance
                .to_string(),
            "asymptotic time-independent predictor variance is not standardised time-independent predictor variance"
        );
    }

    #[test]
    fn standardised_asymptotic_diffusion_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedAsymptoticDiffusionRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised asymptotic DIFFUSION requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "unstandardised asymptotic DIFFUSION is not standardised asymptotic DIFFUSION"
        );
        assert_eq!(
            PsychometricError::StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "standardised time-independent predictor variance is not standardised asymptotic DIFFUSION"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "standardised continuous DIFFUSION is not standardised asymptotic DIFFUSION"
        );
    }

    #[test]
    fn standardised_discrete_continuous_intercept_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedDiscreteContinuousInterceptRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised discrete continuous intercept requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedDiscreteContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
                .to_string(),
            "unstandardised discrete continuous intercept is not standardised discrete continuous intercept"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
                .to_string(),
            "standardised continuous intercept is not standardised discrete continuous intercept"
        );
        assert_eq!(
            PsychometricError::AsymptoticStandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept
                .to_string(),
            "asymptotic standardised continuous intercept is not standardised discrete continuous intercept"
        );
    }

    #[test]
    fn standardised_asymptotic_continuous_intercept_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedAsymptoticContinuousInterceptRequiresPositiveWithinSubjectVariance
                .to_string(),
            "standardised asymptotic continuous intercept requires strictly positive within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
                .to_string(),
            "unstandardised asymptotic continuous intercept is not standardised asymptotic continuous intercept"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
                .to_string(),
            "standardised continuous intercept is not standardised asymptotic continuous intercept"
        );
        assert_eq!(
            PsychometricError::StandardisedDiscreteContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
                .to_string(),
            "standardised discrete continuous intercept is not standardised asymptotic continuous intercept"
        );
    }

    #[test]
    fn standardised_initial_latent_mean_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedInitialLatentMeanRequiresPositiveInitialLatentVariance
                .to_string(),
            "standardised initial latent mean requires strictly positive initial latent variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedInitialLatentMeanIsNotStandardisedInitialLatentMean
                .to_string(),
            "unstandardised initial latent mean is not standardised initial latent mean"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedInitialLatentMean
                .to_string(),
            "standardised initial latent variance is not standardised initial latent mean"
        );
        assert_eq!(
            PsychometricError::WithinSubjectScaledInitialLatentMeanIsNotStandardisedInitialLatentMean
                .to_string(),
            "within-subject scaled initial latent mean is not standardised initial latent mean"
        );
    }

    #[test]
    fn standardised_manifest_mean_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedManifestMeanRequiresPositiveManifestVariance.to_string(),
            "standardised manifest mean requires strictly positive measurement error"
        );
        assert_eq!(
            PsychometricError::UnstandardisedManifestMeanIsNotStandardisedManifestMean.to_string(),
            "unstandardised manifest mean is not standardised manifest mean"
        );
        assert_eq!(
            PsychometricError::StandardisedManifestVarianceIsNotStandardisedManifestMean
                .to_string(),
            "standardised manifest variance is not standardised manifest mean"
        );
        assert_eq!(
            PsychometricError::ObservedScaledManifestMeanIsNotStandardisedManifestMean.to_string(),
            "observed scaled manifest mean is not standardised manifest mean"
        );
    }
}
