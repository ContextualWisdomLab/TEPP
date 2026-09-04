# Bias standard error: repeated-level three-observation rounding

## Problem

Validation Evidence computes the standard error of the mean signed bias from represented binary64 recovery residuals. After GAP-095–098, the exact-translated path preserves represented residual geometry and uses a power-of-two scale, but it still evaluates the general second-moment expression through squared binary64 values and a square root.

A bounded three-observation counterexample remains. Let

- `a = next_down(1.0) = 0x1.fffffffffffffp-1 = 1 - 2^-53`,
- `truth = [0, 0, 0]`, and
- `recovered = [0, a, a]`.

All three represented residuals and anchor-relative translated deltas are exact. For the residual vector `[0, a, a]`, the sample standard error of the mean simplifies algebraically to

`SE(mean) = |a| / 3`.

The correctly rounded binary64 result is `0x1.5555555555555p-2` (`0x3fd5_5555_5555_5555`). The predecessor exact-translated implementation normalized on a power-of-two scale, squared the normalized `a` values, formed `n * sum(x^2) - sum(x)^2`, divided, and then took a square root. That sequence returns the adjacent lower float `0x1.5555555555554p-2` for this represented input. The discrepancy is one ULP and is not caused by temporal composition, sampling design, or reusable psychometric estimation; it is Validation Evidence binary64 projection error.

Public RED: `fbcdb7fac40744c697debbbe6184d4e0ffd5e32a` (`bias_standard_error_repeated_level_rounding_contract.rs`).

## Constraints

The repair must preserve the current owner and numerical boundaries:

- TEPP Validation Evidence owns this performance-measure decision arithmetic.
- reusable static psychometric estimation remains fast-mlsirm-owned;
- no arbitrary-precision production dependency is introduced merely to eliminate a one-ULP bounded projection error;
- exact translated-delta admission and fail-closed behavior remain unchanged;
- a local counterexample does not justify claiming globally correctly rounded `n > 2` standard errors.

## Alternatives

### Keep the general second-moment route

Rejected for the proven shape. The general formula is mathematically valid, but the represented operation sequence introduces avoidable square and square-root projections after the residual geometry is already exact.

### Replace all larger-sample standard errors with arbitrary precision

Rejected. It widens the production dependency and performance surface far beyond the demonstrated defect and would duplicate a numerical owner without a demonstrated buyer/scientific need.

### Add a generic two-level closed form for every sample size

Not adopted in this repair. For arbitrary counts the coefficient includes a square root and still needs separate rounding analysis. The current counterexample proves only the `n = 3`, two-equal-level identity where the standard error reduces exactly to one represented gap divided by three.

### Evaluate the exact three-observation two-level identity directly

Selected. After exact translated-residual admission, any three-observation sample with exactly two equal levels is translation-equivalent to `[0, 0, d]` or `[0, d, d]`. In either case `SE(mean) = |d| / 3`. The gap is already a represented exact translated delta, so one correctly rounded binary64 division is the narrow causal operation required by the scientific identity.

Causal repair: `e0f2445d825f12817631ca8e5ef5fed77fcd113a`.

## Risk and follow-up

This repair does not establish global correct rounding for general translated second moments. Independent counterexamples involving three distinct levels, larger `n`, square accumulation, division, or square root remain separate findings and require their own represented-input RED before the algorithm is widened again. If the direct identity produces zero from a nonzero represented gap, the result remains fail closed as unrepresentable rather than being reported as zero uncertainty.

## Traceability

- Bounded context: Validation Evidence.
- Module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error` / `exact_translated_residual_standard_error`.
- Public contract: `crates/validation_core/tests/bias_standard_error_repeated_level_rounding_contract.rs`.
- RED: `fbcdb7fac40744c697debbbe6184d4e0ffd5e32a`.
- Repair: `e0f2445d825f12817631ca8e5ef5fed77fcd113a`.
- Release note: `CHANGELOG.d/validation-bias-standard-error-repeated-level-rounding.md`.

## References

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). Institute of Electrical and Electronics Engineers. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
