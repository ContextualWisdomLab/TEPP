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
    /// Driver p. 16 `CINTstd` was requested without a strictly positive
    /// `asymDIFFUSION`. Footnote 4 standardises using only the
    /// relevant variance; zero `q` has no positive process SD.
    StandardisedContinuousInterceptRequiresPositiveStationaryVariance,
    /// Driver Table 2 unstandardised `CINT` `κ` was treated as
    /// `CINTstd`. Unstandardised intercept is defined for growing
    /// `a ≥ 0` and for zero diffusion; standardised `CINT` is not.
    UnstandardisedContinuousInterceptIsNotStandardisedContinuousIntercept,
    /// Driver p. 16 `asymCINTstd` `(-κ / a) / √p` was treated as
    /// `CINTstd`. The asymptotic map is the total change, not the
    /// continuous intercept.
    AsymptoticStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept,
    /// Driver p. 16 `discreteCINTstd` `A^{-1}[e^{A Δt} − I] κ / √p`
    /// was treated as `CINTstd`. The finite-interval map depends on
    /// `Δt` and is not the continuous intercept.
    DiscreteStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept,
    /// `κ / √(trait + p + added)` was treated as `CINTstd`.
    /// Footnote 4 uses only `asymDIFFUSION`, not total variance.
    /// `TRAITVAR` is not the standardisation variance.
    TraitScaledContinuousInterceptIsNotStandardisedContinuousIntercept,
    /// Driver p. 16 `MANIFESTMEANSstd` was requested with a non-positive
    /// residual `MANIFESTVAR`. Footnote 4 standardises using only the
    /// relevant variance; zero `θ` has no positive SD.
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
    /// Driver p. 16 `T0VARstd` was requested with a non-positive
    /// first-occasion variance. Footnote 4 standardisation of the
    /// 2017-era `T0VAR` matrix requires strictly positive free
    /// `T0VAR`.
    StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance,
    /// Driver Table 2 unstandardised `T0VAR` `p_0` was treated as
    /// p. 16 `T0VARstd`. Unstandardised `p_0` is defined for a zero
    /// first-occasion variance; standardised `T0VAR` is not.
    UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance,
    /// Driver p. 16 `T0MEANSstd` was treated as p. 16 `T0VARstd`.
    /// Equal numbers when `μ_0 = √p_0` are still distinct named
    /// quantities. `T0VARstd` is the correlation form of free
    /// `T0VAR`; `T0MEANSstd` is the first-occasion mean.
    StandardisedInitialLatentMeanIsNotStandardisedInitialLatentVariance,
    /// Driver p. 16 `asymDIFFUSIONstd` was treated as p. 16
    /// `T0VARstd`. Equal numbers of 1 after a strictly positive
    /// relevant variance are still distinct named quantities.
    /// `T0VARstd` is the correlation form of free `T0VAR`;
    /// `asymDIFFUSIONstd` is the correlation form of process-
    /// dynamics `asymDIFFUSION`.
    StandardisedAsymptoticDiffusionIsNotStandardisedInitialLatentVariance,
    /// Driver p. 16 `asymDIFFUSIONstd` was requested without a
    /// strictly positive `asymDIFFUSION`. Footnote 4
    /// standardisation of the 2017-era `asymDIFFUSION` matrix
    /// requires strictly positive `−q / (2 a)`.
    StandardisedAsymptoticDiffusionRequiresPositiveStationaryVariance,
    /// Driver p. 16 unstandardised `asymDIFFUSION` `p` was treated
    /// as `asymDIFFUSIONstd`. Unstandardised `p` is defined for a
    /// zero process; standardised `asymDIFFUSION` is not.
    UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `T0VARstd` was treated as p. 16
    /// `asymDIFFUSIONstd`. Equal numbers of 1 after a strictly
    /// positive relevant variance are still distinct named
    /// quantities. `asymDIFFUSIONstd` is the correlation form of
    /// process-dynamics `asymDIFFUSION`; `T0VARstd` is the
    /// correlation form of free `T0VAR`.
    StandardisedInitialLatentVarianceIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `DIFFUSIONstd` `q / p = −2 a` was treated as
    /// `asymDIFFUSIONstd`. The continuous-diffusion ratio is not
    /// the correlation form of `asymDIFFUSION`.
    StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion,
    /// Driver p. 16 `TIPREDVARstd` was treated as p. 16
    /// `asymDIFFUSIONstd`. Equal numbers of 1 after a strictly
    /// positive relevant variance are still distinct named
    /// quantities. `asymDIFFUSIONstd` is the correlation form of
    /// process-dynamics `asymDIFFUSION`; `TIPREDVARstd` is the
    /// correlation form of `TIPREDVAR`.
    StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion,

    /// Driver p. 16 `TRAITVARstd` was requested with a non-positive
    /// trait variance. The 2017-era source skips forming
    /// `TRAITVARstd` when `TRAITVAR == 0`; footnote 4
    /// standardisation requires strictly positive `TRAITVAR`.
    StandardisedTraitVarianceRequiresPositiveTraitVariance,
    /// Driver Table 2 unstandardised `TRAITVAR` was treated as
    /// p. 16 `TRAITVARstd`. Unstandardised trait variance is
    /// defined for a zero trait; standardised `TRAITVAR` is not.
    UnstandardisedTraitVarianceIsNotStandardisedTraitVariance,
    /// Driver p. 16 `T0VARstd` was treated as p. 16 `TRAITVARstd`.
    /// Equal numbers when both correlations equal 1 are still
    /// distinct named quantities. `TRAITVARstd` is the correlation
    /// form of between-subject `TRAITVAR`; `T0VARstd` is the
    /// correlation form of free first-occasion `T0VAR`.
    StandardisedInitialLatentVarianceIsNotStandardisedTraitVariance,
    /// Driver 2017-era `addedT0TIPREDVAR` `t0_b² v` was treated as
    /// p. 16 `TRAITVARstd`. Extra first-occasion TI variance is not
    /// the correlation form of between-subject `TRAITVAR`.
    InitialTimeIndependentVarianceIsNotStandardisedTraitVariance,

    /// Driver p. 16 `discreteCINTstd` was requested without a strictly
    /// positive `asymDIFFUSION`. Footnote 4 standardises using only the
    /// relevant variance; zero `q` has no positive process SD.
    StandardisedDiscreteContinuousInterceptRequiresPositiveStationaryVariance,
    /// Unstandardised `discreteCINT` `A^{-1}[e^{A Δt} − I] κ` was
    /// treated as `discreteCINTstd`. Unstandardised increment is
    /// defined for growing `a ≥ 0` and for zero diffusion;
    /// standardised discrete intercept is not.
    UnstandardisedDiscreteContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver p. 16 `CINTstd` `κ / √p` was treated as
    /// `discreteCINTstd`. The continuous intercept does not depend on
    /// the event interval.
    StandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver p. 16 `asymCINTstd` `(-κ / a) / √p` was treated as
    /// `discreteCINTstd`. The asymptotic map is the total change, not
    /// the finite-interval intercept.
    AsymptoticStandardisedContinuousInterceptIsNotStandardisedDiscreteContinuousIntercept,
    /// Driver p. 16 `asymCINTstd` was requested without a strictly
    /// positive `asymDIFFUSION`. Footnote 4 standardises using only
    /// the relevant variance; zero `q` has no positive process SD.
    StandardisedAsymptoticContinuousInterceptRequiresPositiveStationaryVariance,
    /// Driver Table 2 unstandardised `asymCINT` `-κ / a` was treated
    /// as `asymCINTstd`. Unstandardised asymptotic intercept is
    /// defined for a zero process; standardised `asymCINT` is not.
    UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `CINTstd` `κ / √p` was treated as `asymCINTstd`.
    /// The continuous intercept is not the standardised total change.
    StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `discreteCINTstd`
    /// `A^{-1}[e^{A Δt} − I] κ / √p` was treated as `asymCINTstd`.
    /// A finite event interval is not the `Δt → ∞` intercept change.
    DiscreteStandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept,
    /// Driver p. 16 `MANIFESTTRAITVARstd` was requested with a
    /// non-positive manifest-trait variance. The 2017-era source
    /// skips forming `MANIFESTTRAITVARstd` when
    /// `MANIFESTTRAITVAR == 0`; footnote 4 standardisation requires
    /// strictly positive `MANIFESTTRAITVAR`.
    StandardisedManifestTraitVarianceRequiresPositiveManifestTraitVariance,
    /// Driver Table 2 unstandardised `MANIFESTTRAITVAR` was treated
    /// as p. 16 `MANIFESTTRAITVARstd`. Unstandardised manifest-trait
    /// variance is defined for a zero trait; standardised
    /// `MANIFESTTRAITVAR` is not.
    UnstandardisedManifestTraitVarianceIsNotStandardisedManifestTraitVariance,
    /// Driver p. 16 `TRAITVARstd` was treated as p. 16
    /// `MANIFESTTRAITVARstd`. Equal numbers when both correlations
    /// equal 1 are still distinct named quantities.
    /// `MANIFESTTRAITVARstd` is the correlation form of
    /// indicator-level `MANIFESTTRAITVAR`; `TRAITVARstd` is the
    /// correlation form of process-level `TRAITVAR`.
    StandardisedTraitVarianceIsNotStandardisedManifestTraitVariance,
    /// Driver Table 2 `MANIFESTVAR` `θ` was treated as p. 16
    /// `MANIFESTTRAITVARstd`. Measurement error is not the
    /// correlation form of indicator-level trait variance.
    MeasurementErrorIsNotStandardisedManifestTraitVariance,
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
            Self::StandardisedContinuousInterceptRequiresPositiveStationaryVariance => {
                "standardised continuous intercept requires strictly positive stationary within-subject variance"
            }
            Self::UnstandardisedContinuousInterceptIsNotStandardisedContinuousIntercept => {
                "unstandardised continuous intercept is not standardised continuous intercept"
            }
            Self::AsymptoticStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept => {
                "asymptotic standardised continuous intercept is not standardised continuous intercept"
            }
            Self::DiscreteStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept => {
                "discrete standardised continuous intercept is not standardised continuous intercept"
            }
            Self::TraitScaledContinuousInterceptIsNotStandardisedContinuousIntercept => {
                "trait-scaled continuous intercept is not standardised continuous intercept"
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
            Self::StandardisedInitialLatentVarianceRequiresPositiveInitialLatentVariance => {
                "standardised initial latent variance requires strictly positive initial latent variance"
            }
            Self::UnstandardisedInitialLatentVarianceIsNotStandardisedInitialLatentVariance => {
                "unstandardised initial latent variance is not standardised initial latent variance"
            }
            Self::StandardisedInitialLatentMeanIsNotStandardisedInitialLatentVariance => {
                "standardised initial latent mean is not standardised initial latent variance"
            }
            Self::StandardisedAsymptoticDiffusionIsNotStandardisedInitialLatentVariance => {
                "standardised asymptotic diffusion is not standardised initial latent variance"
            }
            Self::StandardisedAsymptoticDiffusionRequiresPositiveStationaryVariance => {
                "standardised asymptotic diffusion requires strictly positive stationary within-subject variance"
            }
            Self::UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion => {
                "unstandardised asymptotic diffusion is not standardised asymptotic diffusion"
            }
            Self::StandardisedInitialLatentVarianceIsNotStandardisedAsymptoticDiffusion => {
                "standardised initial latent variance is not standardised asymptotic diffusion"
            }
            Self::StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion => {
                "standardised continuous diffusion is not standardised asymptotic diffusion"
            }
            Self::StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion => {
                "standardised time-independent predictor variance is not standardised asymptotic diffusion"
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



            Self::StandardisedDiscreteContinuousInterceptRequiresPositiveStationaryVariance => {
                "standardised discrete continuous intercept requires strictly positive stationary within-subject variance"
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
            Self::StandardisedAsymptoticContinuousInterceptRequiresPositiveStationaryVariance => {
                "standardised asymptotic continuous intercept requires strictly positive stationary within-subject variance"
            }
            Self::UnstandardisedAsymptoticContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "unstandardised asymptotic continuous intercept is not standardised asymptotic continuous intercept"
            }
            Self::StandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "standardised continuous intercept is not standardised asymptotic continuous intercept"
            }
            Self::DiscreteStandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept => {
                "discrete standardised continuous intercept is not standardised asymptotic continuous intercept"
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
    fn standardised_continuous_intercept_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedContinuousInterceptRequiresPositiveStationaryVariance
                .to_string(),
            "standardised continuous intercept requires strictly positive stationary within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
                .to_string(),
            "unstandardised continuous intercept is not standardised continuous intercept"
        );
        assert_eq!(
            PsychometricError::AsymptoticStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
                .to_string(),
            "asymptotic standardised continuous intercept is not standardised continuous intercept"
        );
        assert_eq!(
            PsychometricError::DiscreteStandardisedContinuousInterceptIsNotStandardisedContinuousIntercept
                .to_string(),
            "discrete standardised continuous intercept is not standardised continuous intercept"
        );
        assert_eq!(
            PsychometricError::TraitScaledContinuousInterceptIsNotStandardisedContinuousIntercept
                .to_string(),
            "trait-scaled continuous intercept is not standardised continuous intercept"
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
            PsychometricError::StandardisedInitialLatentMeanIsNotStandardisedInitialLatentVariance
                .to_string(),
            "standardised initial latent mean is not standardised initial latent variance"
        );
        assert_eq!(
            PsychometricError::StandardisedAsymptoticDiffusionIsNotStandardisedInitialLatentVariance
                .to_string(),
            "standardised asymptotic diffusion is not standardised initial latent variance"
        );
    }

    #[test]
    fn standardised_asymptotic_diffusion_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedAsymptoticDiffusionRequiresPositiveStationaryVariance
                .to_string(),
            "standardised asymptotic diffusion requires strictly positive stationary within-subject variance"
        );
        assert_eq!(
            PsychometricError::UnstandardisedAsymptoticDiffusionIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "unstandardised asymptotic diffusion is not standardised asymptotic diffusion"
        );
        assert_eq!(
            PsychometricError::StandardisedInitialLatentVarianceIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "standardised initial latent variance is not standardised asymptotic diffusion"
        );
        assert_eq!(
            PsychometricError::StandardisedContinuousDiffusionIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "standardised continuous diffusion is not standardised asymptotic diffusion"
        );
        assert_eq!(
            PsychometricError::StandardisedTimeIndependentPredictorVarianceIsNotStandardisedAsymptoticDiffusion
                .to_string(),
            "standardised time-independent predictor variance is not standardised asymptotic diffusion"
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

    fn standardised_discrete_continuous_intercept_boundary_messages_are_stable() {
        assert_eq!(
            PsychometricError::StandardisedDiscreteContinuousInterceptRequiresPositiveStationaryVariance
                .to_string(),
            "standardised discrete continuous intercept requires strictly positive stationary within-subject variance"
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
            PsychometricError::StandardisedAsymptoticContinuousInterceptRequiresPositiveStationaryVariance
                .to_string(),
            "standardised asymptotic continuous intercept requires strictly positive stationary within-subject variance"
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
            PsychometricError::DiscreteStandardisedContinuousInterceptIsNotStandardisedAsymptoticContinuousIntercept
                .to_string(),
            "discrete standardised continuous intercept is not standardised asymptotic continuous intercept"
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
}
