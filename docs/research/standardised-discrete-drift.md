# Scalar event-time `discreteDRIFTstd` recovery

## Domain owner

This contract belongs to TEPP's **Longitudinal Modeling** bounded context because it composes a continuous-time drift over a substantive event-time interval. It is not a reusable static psychometric response/dependence kernel and does not establish `psychometric_core` as an owner. Reusable generalized-mixed, LSIRM/MLSIRM/DLSJM numerical kernels remain fast-mlsirm-owned.

## Estimand

For scalar continuous-time drift `a` and positive event interval `Δt`, Driver, Oud, and Voelkle's discrete-time transition is

`φ(Δt) = exp(a Δt)`.

The 2017 ctsem standardisation uses the relevant within-person asymptotic variance. For a scalar stationary process with continuous diffusion intensity `q > 0` and stable `a < 0`,

`p = -q / (2a) > 0`.

The affecting/affected standard-deviation ratio is therefore `sqrt(p) / sqrt(p) = 1`, so the scalar p. 16 `discreteDRIFTstd` is numerically `exp(a Δt)`. Numerical equality does not collapse the named estimands: unstandardised `discreteDRIFT`, `discreteDRIFTstd`, trait-plus-state lagged association, trait variance, process noise and a fitted ctsem/DSEM model remain different contracts.

`recover_event_time_standardised_discrete_drift` accepts only an event interval; it does not accept a generic clock enum. This makes the event-time ownership structural at the API boundary rather than relying on a caller-supplied clock label.

## Identification and admissibility

This deterministic transform requires finite `q`, finite `a`, finite positive `Δt`, `q > 0`, stable `a < 0`, and a representable positive stationary within-person variance. If `exp(a Δt)` underflows to zero, the binary64 result is refused because zero cannot be inverted through the corresponding real log-rate map. The function does not estimate `a`, `q`, latent states, measurement parameters, or uncertainty.

No claim is made for matrix standardisation, time-varying drift, nonstationary state variance, ESEM, DSEM, or ctsem estimation. Those require their own equations, identification, estimators and recovery evidence.

## Recovery evidence

The integration contract uses deterministic known truth over irregular positive intervals and multiple stable drift/diffusion settings. It computes `truth = exp(a Δt)` independently and requires machine-precision RMSE. Additional cases cover zero/underflowed within-person variance, unstable drift, non-positive/non-finite intervals, negative/non-finite diffusion, non-finite/overflowed intermediates, exponential underflow, and named-estimand refusal.

This is exact arithmetic recovery rather than a fitted stochastic estimator, so Monte Carlo interval coverage is not manufactured for this function. Monte Carlo RMSE/bias/coverage remains mandatory for estimators that infer drift, diffusion, latent states or uncertainty from sampled data.

## Traceability

Driver, C. C., Oud, J. H. L., & Voelkle, M. C. (2017). Continuous time structural equation modeling with R package ctsem. *Journal of Statistical Software, 77*(5), 1–35. https://doi.org/10.18637/jss.v077.i05

Relevant evidence: Eq. 3 discrete transition; Table 2 continuous/discrete parameter names; p. 16 standardised output; footnote 4 standardisation rule; §7.1 separation of stable trait and within-person dynamics. The historical #310 branch also records inspection of the 2017-era `summary.ctsemFit.R`; that implementation lineage is preserved in Git history rather than keeping the temporal transform in the wrong bounded context.
