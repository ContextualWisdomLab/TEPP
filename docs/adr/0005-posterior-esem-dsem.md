# ADR 0005: Posterior-Aware ESEM and DSEM

**Status:** Accepted  
**Date:** 2026-08-05

## Decision

Topic proportions are compositional and are not treated as error-free ordinary indicators. TEPP uses logistic-normal latent coordinates or orthonormal log-ratio coordinates and propagates topic posterior uncertainty through plausible values or a joint text-measurement/structural model.

Before ESEM, each higher-order construct is classified as reflective, formative/composite, network, or unresolved. Reflective indicators may use ESEM or set-ESEM; formative structures use composite/formative models; interacting structures use networks. Longitudinal analysis evaluates measurement invariance and separates stable between-episode differences from within-episode change.

Input-process/intervention-outcome paths obey event-time order. Temporal precedence and document linkage alone do not justify causal language.

## Consequences

Raw topic-proportion correlation matrices and naive two-stage point-score SEM are prohibited. Loading, factor, path, indirect-effect, and model-selection results carry posterior and Monte Carlo uncertainty. Method factors may represent language, template, section, source, and copied-report effects.

## Verification

Synthetic studies recover loadings, cross-loadings, factors, lagged paths, indirect effects, irregular-time dynamics, invariance violations, and method effects with RMSE, bias, interval coverage, calibration, and identification diagnostics.
