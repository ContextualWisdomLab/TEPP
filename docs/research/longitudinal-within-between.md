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

## Numerical authority

Unit means in `decompose_within_between` use the same Longitudinal-local
`scaled_compensated_mean` primitive as CWC and occasion-mean composition.
The decomposition path must not maintain a shadow running-mean algorithm.

This matters at binary64 boundaries. With minimum positive subnormal ULP
`u`, the unit scores `[u, 2u]` have exact mean `1.5u`, which IEEE 754
round-to-nearest, ties-to-even represents as `2u`. The predecessor running
update `mean += (value - mean) / count` rounded the half-ULP update to zero
and returned `u`; the derived within residuals were therefore shifted from
`[-u, 0]` to `[0, u]`. The public decomposition contract now pins the
single-rounding result.

Traceability:

- RED `7dc87aa8ad4de4a73a502646c5667d01656e9dbd` —
  `crates/longitudinal_core/tests/decomposition_mean_rounding_contract.rs`.
- Causal repair `97c8ad35d1a1a483d8feb2d617e5fbc035c5ead9` —
  `crates/longitudinal_core/src/decompose.rs` delegates unit means to the
  existing Longitudinal-local compensated mean authority.
- Public API under test — `decompose_within_between` with
  `OccasionObservation`; between/within component identity remains unchanged.

This consolidation is local to Longitudinal Modeling. It does not create a
second reusable psychometric arithmetic owner and does not move static
psychometric truth out of fast-mlsirm.

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
