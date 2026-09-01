# Longitudinal within/between decomposition (doctoring)

## Scope

`longitudinal_core` decomposes occasion scores into unit means (between)
and occasion residuals (within). A between-unit component cannot be scored
as within-unit change. Recovered components are scored with computed RMSE
against known truth.

This slice does not fit DSEM, claim lagged or causal paths, or treat
irregular intervals as equally spaced. CWC-then-irregular residual
log-rate lives in the same crate; see
`docs/research/cwc-irregular-residual-log-rate.md`.

## Authority

### Normative TEPP contract

- `docs/adr/0005-posterior-esem-dsem.md` — longitudinal analysis must
  separate stable between-unit components from within-unit temporal change.

### Supporting literature

Hamaker et al. (2015) show that a between-unit difference is not
within-unit change; pooling occasions around a grand mean confounds the
two. Asparouhov et al. (2018) place that separation inside a DSEM program.
They do **not** authorize treating a unit mean as occasion-level change.

Hamaker, E. L., Kuiper, R. M., & Grasman, R. P. P. P. (2015). A critique of
the cross-lagged panel model. *Psychological Methods, 20*(1), 102–116.
https://doi.org/10.1037/a0038889

Asparouhov, T., Hamaker, E. L., & Muthén, B. (2018). Dynamic structural
equation models. *Structural Equation Modeling: A Multidisciplinary
Journal, 25*(3), 359–388. https://doi.org/10.1080/10705511.2017.1406803
