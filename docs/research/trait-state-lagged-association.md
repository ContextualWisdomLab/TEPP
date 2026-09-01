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

`longitudinal_core::recover_event_time_lagged_correlation` accepts a lagged covariance, the earlier marginal variance, the later marginal variance, and an admitted `EventTimeInterval`. The typed value object is carried through the public boundary into the internal association implementation rather than being erased back to a bare duration. The operation performs only temporal association standardization and does not infer state variance, process noise, or a psychometric response kernel. The function computes the correlation after an exact binary64 covariance-bound check, so floating-point rounding in `sqrt(variance)` cannot admit a covariance that is actually one ULP above the represented bound.

This preserves the DDD ownership boundary:

- reusable static/generalized-mixed/dependence psychometric kernels remain owned by `ContextualWisdomLab/fast-mlsirm`;
- TEPP owns event-time composition and longitudinal semantics;
- measurement occasion, rater, and method facets are not substitutes for substantive event time;
- callers must assemble occasion-specific marginals from an identified temporal model before asking for a correlation.

The function fails closed when either marginal is non-positive, covariance or marginal inputs are non-finite, or the supplied covariance violates the Cauchy–Bunyakovsky–Schwarz covariance bound. `EventTimeInterval` itself fails admission for non-finite or non-positive durations. Pearson standardization does not estimate or transform time; the event-time value object exists to preserve clock ownership through the API.

## Regression evidence

The regression suite includes a nonstationary case with earlier variance `1`, later variance `4`, and lagged covariance `1.5`. The retired one-sided ratio would be `1.5`; correct standardization yields `0.75`. It also verifies exact `±1` boundaries at ordinary, `f64::MAX`, and minimum-subnormal scales; rejects one-ULP over-bound covariance for both signs; classifies gross subnormal violations before division; rejects invalid event-time value construction; and avoids forming `Var_t * Var_{t+Δ}` directly.

The scalar `discreteDRIFTstd` regressions separately require monotone temporal ordering for stable negative drift and fail closed when a finite negative drift multiplied by a positive admitted event interval underflows to signed zero. That case must not silently become `exp(-0.0) == 1.0`.

These are arithmetic and contract regressions, not an estimator-recovery study. A future production autocorrelation derived from a specific DSEM or continuous-time state model must additionally identify and recover the state transition, process-noise contribution, both marginal variances, uncertainty, and leakage-safe rolling-origin performance. Production promotion therefore requires model-specific true-parameter RMSE, bias, interval coverage, irregular-gap behavior, delayed/retrospective-report handling, and temporal-ordering evidence rather than borrowing these standardization tests as scientific-estimator acceptance (Driver et al., 2017).

## Research basis

Bouniakowsky, V. (1859). Sur quelques inégalités concernant les intégrales ordinaires et les intégrales aux différences finies. *Mémoires de l’Académie Impériale des Sciences de Saint-Pétersbourg, VIIe Série, 1*(9), 1–18.

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Pearson, K. (1895). Notes on regression and inheritance in the case of two parents. *Proceedings of the Royal Society of London, 58*, 240–242.

Rodgers, J. L., & Nicewander, W. A. (1988). Thirteen ways to look at the correlation coefficient. *The American Statistician, 42*(1), 59–66. https://doi.org/10.1080/00031305.1988.10475524
