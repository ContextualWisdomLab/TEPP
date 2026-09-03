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

Exact zero within residuals also have one public identity. IEEE 754 binary64
has distinct `+0.0` and `-0.0` encodings, and subtraction can therefore leave
`-0.0` when a negative-zero observed score equals a canonical zero unit mean.
That sign bit does not represent positive versus negative within-person change:
the deviation is exactly zero. The public decomposition boundary therefore
canonicalizes only validated exact-zero residuals to `+0.0`; private numerical
intermediates remain free to retain signed zero where it is diagnostically
meaningful. IEEE Std 754-2019 remains the active published floating-point
standard while IEEE P754 is the active revision project as of 2026-09-03.

Signed-zero traceability:

- RED `aeb008a38bc333ba0f1bec0651739e361426e66d` —
  `crates/longitudinal_core/tests/decomposition_signed_zero_contract.rs`
  exercises the public decomposition API with `-0.0` and `+0.0` observations.
- Causal repair `a9a70baa5c2a40ec8bf3fc77748bc3a5eaf92cd8` —
  `crates/longitudinal_core/src/decompose.rs` canonicalizes an exact-zero
  within residual only after finite-result validation.
- Standard authority — IEEE. (2019). *IEEE standard for floating-point
  arithmetic* (IEEE Std 754-2019). IEEE. The canonical repository register is
  `docs/research/standards-and-literature.md`.

Known-truth recovery metrics have the same fail-closed representability
boundary. `component_root_mean_square_error` may return exact zero only when
all admitted matched component residuals are exactly zero. If at least one
residual is nonzero but the positive real-valued RMSE falls below binary64
range, returning `0.0` would convert a numerical limitation into false perfect
recovery. For five matched components with one minimum-subnormal residual `u`
and four exact-zero residuals, the mathematical RMSE is `u / sqrt(5) > 0`
but binary64 rounds that final product to zero. That case must therefore be
reported as `InvalidComponentPayload`, consistent with the public contract
that a non-representable final RMSE fails closed.

Recovery-metric traceability:

- RED `496583c6b62cbe0ad1be0e65b51f01d7f72acd5a` —
  `crates/longitudinal_core/tests/component_rmse_underflow_contract.rs` drives
  the public `component_root_mean_square_error` API with identity-matched
  within components and one minimum-subnormal nonzero recovery error.
- Causal repair `a82b383b5940126a0139180d66729d2e6aa4baf7` —
  `crates/longitudinal_core/src/component.rs` keeps exact-zero recovery on the
  existing `scale == 0` path and rejects a later rounded `rmse == 0` once a
  nonzero residual scale has already been established.
- Acceptance boundary — an unrepresentable nonzero error is not a recovered
  parameter and cannot count toward perfect RMSE or scientific claim promotion.

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