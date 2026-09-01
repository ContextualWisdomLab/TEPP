# Trait-plus-state lagged association boundary

## Status

Research boundary note for the Longitudinal Modeling bounded context. This note does not activate a production scoring or estimator target.

## Scientific correction

A lagged covariance divided only by the earlier marginal variance is not, in general, an autocorrelation. For event times `t` and `t + Δ`, a Pearson correlation requires both marginals:

\[
\rho_{t,t+\Delta}=
\frac{\operatorname{Cov}(Y_t,Y_{t+\Delta})}
{\sqrt{\operatorname{Var}(Y_t)\operatorname{Var}(Y_{t+\Delta})}}.
\]

When the process is nonstationary, `Var(Y_t)` and `Var(Y_{t+Δ})` need not be equal. Consequently, a one-sided ratio such as

\[
\frac{\text{stable variance}+\text{lagged state covariance}}
{\text{stable variance}+\text{earlier state variance}}
\]

can exceed one and must not be published as an autocorrelation. Driver, Oud, and Voelkle (2017) provide the continuous-time state-transition and covariance ingredients used by ctsem, but they do not print or validate that one-sided ratio as an autocorrelation.

The unmerged implementation that introduced a public `expected_autocorrelation` API is therefore retired rather than patched with an arbitrary restriction such as rejecting only positive drift. That restriction would still be insufficient for a nonstationary initial state with negative drift because the later marginal variance remains necessary.

## TEPP contract

`longitudinal_core::recover_event_time_lagged_correlation` accepts a lagged covariance, the earlier marginal variance, the later marginal variance, and a strictly positive event-time interval. It performs only the temporal association standardization and does not infer state variance, process noise, or a psychometric response kernel.

This preserves the DDD ownership boundary:

- reusable static/generalized-mixed/dependence psychometric kernels remain owned by `ContextualWisdomLab/fast-mlsirm`;
- TEPP owns event-time composition and longitudinal semantics;
- measurement occasion, rater, and method facets are not substitutes for substantive event time;
- callers must assemble occasion-specific marginals from an identified temporal model before asking for a correlation.

The function fails closed when either marginal is non-positive, the interval is non-positive, inputs are non-finite, or the supplied covariance violates the Cauchy-Schwarz bound.

## Regression evidence

The regression suite includes a nonstationary case with earlier variance `1`, later variance `4`, and lagged covariance `1.5`. The retired one-sided ratio would be `1.5`; correct standardization yields `0.75`. The suite also verifies exact `±1` boundaries, rejects incompatible covariance, rejects non-event intervals, and exercises very large representable variances without forming `Var_t * Var_t+Δ` directly.

This is intentionally narrower than a state estimator. A future production autocorrelation derived from a specific DSEM/continuous-time state model must additionally identify and recover the state transition, process-noise contribution, both marginal variances, uncertainty, and rolling-origin leakage-safe performance before activation.

## Research basis

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05
