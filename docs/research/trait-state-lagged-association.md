# Trait-plus-state lagged association boundary

## Status

Research boundary note for the Longitudinal Modeling bounded context. This note does not activate a production scoring or estimator target.

## Scientific correction

A Pearson correlation standardizes covariance by both marginal standard deviations; this normalization is the defining feature of the coefficient and is not replaceable by a one-sided variance ratio (Pearson, 1895; Rodgers & Nicewander, 1988). For event times `t` and `t + Δ`:

\[
\rho_{t,t+\Delta}=
\frac{\operatorname{Cov}(Y_t,Y_{t+\Delta})}
{\sqrt{\operatorname{Var}(Y_t)\operatorname{Var}(Y_{t+\Delta})}}.
\]

The Cauchy–Bunyakovsky–Schwarz inequality gives the corresponding covariance bound. Bouniakowsky (1859, pp. 3–4) states the integral form that bounds a squared cross-product integral by the product of the two squared-function integrals; applying that inner-product inequality to centered random variables yields `Cov² <= Var_t Var_{t+Δ}`. TEPP therefore treats a supplied covariance outside the exact binary64 representation of that bound as incompatible input rather than rounding it into range. Rodgers and Nicewander (1988) remain supplementary statistical context, not the primary source for the inequality.

Continuous-time state models need not have equal occasion-specific marginal variances when the initial distribution is not stationary or when time-varying inputs alter state uncertainty. Driver, Oud, and Voelkle (2017, §§4.3, 7.1–7.2) explicitly distinguish initial variance, asymptotic diffusion, trait variance, predictor-related variance, and time evolution. Consequently, a one-sided ratio such as

\[
\frac{\text{stable variance}+\text{lagged state covariance}}
{\text{stable variance}+\text{earlier state variance}}
\]

can exceed one and must not be published as an autocorrelation. Driver et al. (2017) provide the continuous-time state-transition and covariance ingredients used by ctsem, but they do not print or validate that one-sided ratio as an autocorrelation.

The unmerged implementation that introduced a public `expected_autocorrelation` API is therefore retained only as RED/scientific-failure lineage. Rejecting only positive drift would not repair the definition: a nonstationary initial state can still require a distinct later marginal variance even under negative drift (Driver et al., 2017, §§4.3, 7.1).

## TEPP contract

`longitudinal_core::recover_event_time_lagged_correlation` accepts a lagged covariance, the earlier marginal variance, the later marginal variance, and an admitted `EventTimeInterval`. The typed value object is carried through the public boundary into the internal association implementation rather than being erased back to a bare duration. The operation performs only temporal association standardization and does not infer state variance, process noise, or a psychometric response kernel.

Admissibility and perfect-correlation endpoint claims both use the exact binary64 covariance relation. The implementation decomposes the covariance and marginal variances into integer significands and powers of two, compares `Cov²` with `Var_t Var_{t+Δ}` without a rounded square-root product, and records whether the relation is strict or exact. Rounded `sqrt`/division arithmetic is then used only to form an interior representable coefficient. Exact binary64 equality at the covariance boundary is itself authoritative for the `±1` endpoint, so floating-point square-root rounding may neither invent nor weaken perfect association.

This distinction is necessary in both directions. For the exact binary64 inputs

- earlier variance `f64::from_bits(4_607_182_418_800_016_408)`,
- later variance `f64::from_bits(4_607_182_418_800_016_427)`, and
- covariance magnitude `f64::from_bits(4_607_182_418_800_016_417)`,

`Cov²` is strictly smaller than the exact product of the two supplied marginal variances, yet the rounded square roots followed by the two divisions produce `1.0`. Returning that endpoint would convert an interior association into a scientifically stronger perfect-correlation claim. RED `683b28eeeda3ad72ac11f5317c5aea54f34e0692` fixes both covariance signs through the public API in `crates/longitudinal_core/tests/correlation_false_perfect_contract.rs`; causal repair `9eeb373df2cd333fe7543df2197ea0cc0c492780` prevents a rounded strict-interior endpoint from being promoted.

The inverse rounding failure also occurs on an exact boundary. With represented inputs `Var_t = 2`, `Var_{t+Δ} = 8`, and `|Cov| = 4`, exact binary arithmetic gives `Cov² = Var_t Var_{t+Δ}` and therefore `|ρ| = 1`. The predecessor square-root/division path evaluates the positive coefficient as `0x1.fffffffffffffp-1`, one ULP below one, which would weaken an exactly perfect represented association. RED `c25000901eb429a43817552f8b76cf4aae04e522` adds positive and negative public-API cases; causal repair `d06259ec1e036558d8d2f775c266b2b9db4e42c4` returns the exact sign endpoint directly when the exact integer covariance relation is on the boundary and reserves floating-point standardization for strict-interior coefficients.

This preserves the DDD ownership boundary:

- reusable static/generalized-mixed/dependence psychometric kernels remain owned by `ContextualWisdomLab/fast-mlsirm`;
- TEPP owns event-time composition and longitudinal semantics;
- measurement occasion, rater, and method facets are not substitutes for substantive event time;
- callers must assemble occasion-specific marginals from an identified temporal model before asking for a correlation.

The function fails closed when either marginal is non-positive, covariance or marginal inputs are non-finite, the supplied covariance violates the Cauchy–Bunyakovsky–Schwarz covariance bound, a nonzero exact coefficient would collapse to binary64 zero, or a strict interior covariance would round to exact `±1`. `EventTimeInterval` itself fails admission for non-finite or non-positive durations. Pearson standardization does not estimate or transform time; the event-time value object exists to preserve clock ownership through the API.

## Regression evidence

The regression suite includes a nonstationary case with earlier variance `1`, later variance `4`, and lagged covariance `1.5`. The retired one-sided ratio would be `1.5`; correct standardization yields `0.75`. It also verifies exact `±1` boundaries at ordinary, unequal-marginal (`2`, `8`, `4`), `f64::MAX`, and minimum-subnormal scales; rejects one-ULP over-bound covariance for both signs; rejects strict-interior covariances whose rounded standardization would otherwise become false exact `±1`; classifies gross subnormal violations before division; rejects invalid event-time value construction; and avoids forming `Var_t * Var_{t+Δ}` directly.

The scalar `discreteDRIFTstd` regressions separately require monotone temporal ordering for stable negative drift and fail closed when a finite negative drift multiplied by a positive admitted event interval underflows to signed zero. That case must not silently become `exp(-0.0) == 1.0`.

These are arithmetic and contract regressions, not an estimator-recovery study. A future production autocorrelation derived from a specific DSEM or continuous-time state model must additionally identify and recover the state transition, process-noise contribution, both marginal variances, uncertainty, and leakage-safe rolling-origin performance. Production promotion therefore requires model-specific true-parameter RMSE, bias, interval coverage, irregular-gap behavior, delayed/retrospective-report handling, and temporal-ordering evidence rather than borrowing these standardization tests as scientific-estimator acceptance (Driver et al., 2017).

## Research basis

Bouniakowsky, V. (1859). Sur quelques inégalités concernant les intégrales ordinaires et les intégrales aux différences finies. *Mémoires de l’Académie Impériale des Sciences de Saint-Pétersbourg, VIIe Série, 1*(9), 1–18.

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Pearson, K. (1895). Notes on regression and inheritance in the case of two parents. *Proceedings of the Royal Society of London, 58*, 240–242.

Rodgers, J. L., & Nicewander, W. A. (1988). Thirteen ways to look at the correlation coefficient. *The American Statistician, 42*(1), 59–66. https://doi.org/10.1080/00031305.1988.10475524
