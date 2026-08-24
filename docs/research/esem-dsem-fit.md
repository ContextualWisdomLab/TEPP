# ESEM loading recovery and DSEM event-time lags (doctoring)

## Scope

`psychometric_fit` recovers an exploratory loading matrix by ordinary least
squares of each indicator on at most two factor-score series. Recovery is the
computed RMSE against known loadings. A DSEM lagged path is admitted only when
the predictor occasion is strictly earlier in event time than the outcome.

This slice does not implement rotation, posterior pooling, invariance testing,
or the `psychometric_core` input-gate crate owned by PR #49. It does not
allocate migration `0007` or `0008`.

## Authority

### Normative TEPP contract

- `docs/adr/0005-posterior-esem-dsem.md` — logistic-normal or valid log-ratio
  coordinates; construct class before interpretation; event-time order for
  lagged paths; a global fit statistic cannot reclassify formative or network
  constructs as reflective.
- `docs/adr/0001-rust-first-modular-msa.md` — production psychometric
  arithmetic is a CPU `f64` reference path.

### Supporting literature

Asparouhov and Muthén (2009) introduce exploratory structural equation
modeling so indicators may have cross-loadings rather than a strict
confirmatory zero pattern. This crate recovers those cross-loadings by OLS; it
does not implement their full ESEM estimator or rotation.

Asparouhov, Hamaker, and Muthén (2018) specify dynamic structural equation
models on a time-ordered series. The crate enforces the event-time order of a
lagged path and does not implement their Bayesian DSEM sampler.

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling.
*Structural Equation Modeling: A Multidisciplinary Journal, 16*(3), 397–438.
https://doi.org/10.1080/10705510903008204

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural
equation models. *Structural Equation Modeling: A Multidisciplinary Journal,
25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803
