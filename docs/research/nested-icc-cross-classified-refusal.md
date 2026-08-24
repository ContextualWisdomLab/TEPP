# Nested ICC and cross-classified refusal

## Claim boundary

This increment adds a CPU `f64` nested intraclass correlation for one-way clustered membership and refuses to apply that estimator when the active design is cross-classified or multiple-membership. It is not a full MMMC, HLM, ESEM, or DSEM fit, not a causal identification claim, and not a persistence/migration change.

## Authority

Raudenbush, S. W., & Bryk, A. S. (2002). *Hierarchical linear models: Applications and data analysis methods* (2nd ed.). Sage.

Snijders, T. A. B., & Bosker, R. J. (2012). *Multilevel analysis: An introduction to basic and advanced multilevel modeling* (2nd ed.). SAGE.

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124. https://doi.org/10.1177/1471082X0100100202

Beretvas, S. N. (2011). Cross-classified and multiple-membership models. In J. J. Hox & J. K. Roberts (Eds.), *Handbook of advanced multilevel analysis* (pp. 313–334). Routledge.

## Estimator

For a nested design the unbalanced ANOVA estimator is

`ICC = σ²_u / (σ²_u + σ²_e)`,

with `σ²_e = MSW` and `σ²_u = max(0, (MSB − MSW) / n₀)`, where `n₀` is the Snijders–Bosker unbalanced cluster-size factor. Empty, singleton-only, or zero-variance samples fail closed.

A member who is active in two roles is cross-classified. A member who is active in two groups of one role has multiple membership. Neither design may be collapsed into a nested ICC.

## Verification

- balanced four-by-two ANOVA recovers ICC `1/4` with computed RMSE versus that known truth;
- zero within-cluster residual recovers ICC `1`; equal cluster means recover ICC `0`;
- cross-classified and multiple-membership networks return `NestedIccInapplicable`;
- empty, inactive, duplicate, unknown, singleton, and constant samples fail closed.
