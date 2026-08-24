# ADR 0005 — Posterior-aware ESEM/DSEM and structural interpretation

**Decision status:** Accepted
**Implementation maturity:** active-PR — `longitudinal_core` separates unit means from within residuals and refuses between-as-within change; remaining ESEM/DSEM fit remains accepted-target
**Date:** 2026-08-05
**Supersedes:** None. ADR 0012 governs upstream topic measurement/network coordinates; this ADR governs higher-order psychometric structure and longitudinal interpretation.

## Context

Topic outputs are uncertain and compositional. Treating raw topic proportions as error-free Euclidean indicators, attaching a conventional SEM after point estimation, or forcing every relationship into a reflective factor model can create invalid loadings, correlations, and longitudinal conclusions.

TEPP also needs to distinguish stable between-unit differences from within-unit change, accommodate irregular observation intervals, and evaluate whether language/time/template/source changes alter what is being measured before comparing factor means or structural paths.

## Decision

Topic proportions are not treated as error-free ordinary indicators. Compositional parts are not ordinary Euclidean measurements (Aitchison, 1982). TEPP uses logistic-normal latent coordinates or valid orthonormal log-ratio coordinates and propagates topic posterior uncertainty through plausible values or a joint text-measurement/structural model.

Before ESEM/SEM interpretation, each higher-order construct is classified as reflective, formative/composite, network, or unresolved. Reflective indicators may use exploratory structural equation modeling (Asparouhov & Muthén, 2009; Marsh et al., 2014); formative structures use composite/formative models; interacting structures use network models. A good global fit statistic is not authority to reinterpret a formative/network structure as reflective.

Longitudinal analysis evaluates measurement invariance at the level needed for the claimed comparison (American Educational Research Association, American Psychological Association, & National Council on Measurement in Education, 2014), supports partial/approximate or time-varying loadings where scientifically justified, separates stable between-unit components from within-unit temporal change, and handles irregular intervals through dynamic structural equation models (Asparouhov et al., 2018).

This ADR remains **accepted-target**. Naming ESEM/DSEM and compositional coordinates as the model-family contract is not a protected-main implementation claim.

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

## References

Aitchison, J. (1982). The statistical analysis of compositional data. *Journal of the Royal Statistical Society: Series B, 44*(2), 139–177. https://doi.org/10.1111/j.2517-6161.1982.tb01195.x

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural equation models. *Structural Equation Modeling, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803

Asparouhov, T., & Muthén, B. (2009). Exploratory structural equation modeling. *Structural Equation Modeling, 16*(3), 397–438. https://doi.org/10.1080/10705510903008204

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory structural equation modeling: An integration of the best features of exploratory and confirmatory factor analysis. *Annual Review of Clinical Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700
