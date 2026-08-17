# ADR 0005 — Posterior-aware ESEM/DSEM and structural interpretation

**Decision status:** Accepted
**Implementation maturity:** partial — construct classification, valid log-ratio/logistic-normal indicator gates, CPU `f64` OLS and posterior-draw loading point-estimate averaging, Rubin `T_m = Ū_m + (1+1/m) B_m` on draw-level OLS loadings, cluster-mean within/between OLS with the CWC contextual effect and Kish ESS WLS, event-time discrete lag-1 and exact scalar local log-rate, exact scalar forward map and unequal-interval remapping, exact scalar discrete effect of a constant predictor, first-order discrete effect of a time-varying predictor with matched sampling and constancy intervals (Voelkle et al., 2012, Eq. 14), CWC-then-event-time residual lag, irregular already-centered residual log-rate, and strong/strict-gated two-group OLS latent-mean difference are implemented on the stacked psychometric PR and are not implemented-main until exact-head checks, review, and protected-main integration complete; full ESEM/set-ESEM, formative composites, DSEM, and matrix continuous-time dynamics remain accepted-target
**Date:** 2026-08-05
**Supersedes:** None. ADR 0012 governs upstream topic measurement/network coordinates; this ADR governs higher-order psychometric structure and longitudinal interpretation.

## Context

Topic outputs are uncertain and compositional. Treating raw topic proportions as error-free Euclidean indicators, attaching a conventional SEM after point estimation, or forcing every relationship into a reflective factor model can create invalid loadings, correlations, and longitudinal conclusions.

TEPP also needs to distinguish stable between-unit differences from within-unit change, accommodate irregular observation intervals, and evaluate whether language/time/template/source changes alter what is being measured before comparing factor means or structural paths.

## Decision

Topic proportions are not treated as error-free ordinary indicators. TEPP uses logistic-normal latent coordinates or valid orthonormal log-ratio coordinates and propagates topic posterior uncertainty through plausible values or a joint text-measurement/structural model.
The current executable slice averages loading point estimates across posterior draws and, separately, combines those complete-data OLS loadings with Rubin total variance. The point-estimate helper still does not by itself satisfy the full posterior-propagation decision. The Rubin helper uses complete-data OLS sampling variances; it does not treat the draws as Mislevy person-level plausible values.

Before ESEM/SEM interpretation, each higher-order construct is classified as reflective, formative/composite, network, or unresolved. Reflective indicators may use ESEM/set-ESEM; formative structures use composite/formative models; interacting structures use network models. A good global fit statistic is not authority to reinterpret a formative/network structure as reflective.

Longitudinal analysis evaluates measurement invariance at the level needed for the claimed comparison, supports partial/approximate or time-varying loadings where scientifically justified, separates stable between-unit components from within-unit temporal change, and handles irregular intervals through appropriate discrete- or continuous-time dynamics.

The executable multilevel slice is cluster-mean centering (CWC) plus within/between OLS, the CWC contextual effect (`between − within`), and Kish-weighted slopes. Enders and Tofighi (2007, Table 2, pp. 124–127) show that the CWC cluster-mean coefficient is the contextual effect, not the between-cluster effect. It is not DSEM and not RI-CLPM. The executable temporal slice maps a discrete lag through the exact scalar exponential `a = ln(φ) / Δt` on event time only (Voelkle, Oud, Davidov, & Schmidt, 2012, Eq. 7; Driver, Oud, & Voelkle, 2017, Eq. 3), recovers the forward map `φ(Δt) = exp(a Δt)`, remaps a discrete lag onto another event interval through that log-rate, recovers the exact scalar discrete effect of a constant predictor (Voelkle et al., 2012, Eq. 12) as `a_yx (expm1(z) / a_xx)` with `z = a_xx Δt` so a finite result is not lost when `z` overflows to `-∞` or when `a_yx Δt` overflows, and in log space when `expm1(z)` overflows at a finite `z` (`a_yx = 0` is exactly zero; an overflowing `a_yx/a_xx` rewrite term fails closed), recovers the first-order discrete effect of a time-varying predictor with matched sampling and constancy intervals as `a_yx Δt` (Voelkle et al., 2012, Eq. 14; ZORA accepted manuscript p. 21; not Eq. 12; unmatched intervals fail closed because Oud & Jansen, 2000, is unread), and refuses the difference quotient, pooling discrete lags from unequal intervals, and a binary64 underflow of that exponential to `+0` (not a discrete lag). The first-order product `a_yx Δt` is Eq. 14 and the underflow limit of Eq. 12, not the general constant-predictor discrete effect. Already-centered residuals may have irregular event intervals. Subtracting the person-specific mean from a raw autoregressive series is not the lagged within-person residual (Curran & Bauer, 2011, pp. 607–608). Metric/weak invariance licenses shared metric meaning only. Latent-mean comparison requires strong (equal loading and intercept) or strict invariance. This two-group OLS gate is not MGCFA.

Input/process/intervention/outcome paths obey event-time order. Temporal precedence, document linkage, event tracking, or model prediction alone do not justify causal language.

## Non-goals

- do not use raw topic-proportion Pearson correlations as the default psychometric input;
- do not force every topic/factor relation into a reflective model;
- do not claim mean/path comparability without the required invariance evidence;
- do not interpret time precedence as causal identification.

## Alternatives considered

1. **Two-stage point estimates followed by ordinary SEM** — rejected because text-model uncertainty and compositional constraints are ignored.
2. **One reflective ESEM for every construct** — rejected because formative/composite/network structures have different semantics.
3. **Posterior-aware structural modeling with explicit construct class, invariance, and longitudinal decomposition** — accepted.

## Consequences

Loading, factor, path, indirect-effect, correlation, and model-selection results carry posterior/Monte Carlo uncertainty. Method factors can represent language, template, section, source, and copied-report effects. Longitudinal claims can distinguish real within-unit change from stable group differences and measurement drift.

## Failure and recovery

Non-identification, poor convergence, inadmissible loadings/variances, failed invariance, insufficient occasions, unstable posterior propagation, or causal under-identification produces unresolved/restricted interpretation rather than a forced model. Recovery changes the structural model only through explicit scientific rationale and preserved versioned evidence.

## Security, privacy, and governance impact

Factor/trajectory outputs can be sensitive derived data even when raw identifiers are absent. Access/export follows ADR 0009. Scientific claim promotion follows ADR 0014; LLM-generated narrative cannot override estimator diagnostics or claim scope.

## Compatibility and migration

Structural model definitions, factor labels, loading constraints, invariance rules, time units, and posterior-propagation method are versioned. Upstream topic-model changes under ADR 0012 require compatibility/recovery evidence before existing ESEM/DSEM interpretation is reused.

## Verification

Synthetic studies recover loadings, cross-loadings, factors, lagged/direct/indirect paths, within/between components, irregular/continuous-time dynamics, invariance violations, method effects, and multiple-membership/multilevel effects with bias, RMSE, interval coverage, convergence, calibration, and identification diagnostics. Negative tests ensure unsupported causal or invariance claims remain blocked.

## Rollback and supersession

Rollback selects the last validated structural-model version and compatible upstream model artifact. Supersede only if a later decision preserves explicit construct classification, uncertainty propagation, invariance/longitudinal evidence, and claim discipline or deliberately changes those estimands with a PRD update.
