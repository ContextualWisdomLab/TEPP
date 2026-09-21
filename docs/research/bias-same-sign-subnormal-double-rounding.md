# Same-sign subnormal mean double rounding

## Decision status

Proposed evidence for the Validation bounded context on PR #488. This note defines the represented-input boundary repaired by GAP-090. It does not claim globally correctly rounded binary64 summation or division.

## Problem

`same_sign_mean_over_total` normalized same-sign residuals by an exact power of two, formed a compensated normalized mean, and then restored the scale. That is safe from overflow, but when every represented residual is subnormal the restore step is itself a rounding to the coarser subnormal grid. A correctly rounded normalized intermediate can therefore be rounded a second time to the wrong final binary64 neighbor.

Public RED `91abdb496ef13229ed95bcf8854770a2cc71b4b8` fixes three positive represented residuals whose subnormal-unit counts are `2^52 - 32`, `2^52 - 12`, and `2^52 - 20`. Their exact represented-input mean is

`(3 * 2^52 - 64) / 3 = 2^52 - 21 - 1/3`

minimum-subnormal units. Round-to-nearest, ties-to-even therefore selects bits `0x000f_ffff_ffff_ffeb` (`2^52 - 21`). The predecessor normalize-then-rescale path returned adjacent bits `0x000f_ffff_ffff_ffea` (`2^52 - 22`). The public contract includes the sign mirror.

## Constraints

- Keep the public estimand `mean(recovered - truth)` and the original recovery-unit denominator unchanged.
- Keep Validation Evidence arithmetic in TEPP; do not move this boundary into fast-mlsirm.
- Do not add arbitrary-precision runtime arithmetic. Binary64 subnormals already have an exact integer-unit representation at `2^-1074`.
- Preserve overflow-safe normalization for normal-scale inputs and for explicit-denominator cases whose surviving term count can exceed the divisor.
- Continue to fail closed when a mathematically nonzero represented mean rounds to zero under the existing admission policy.

## Alternatives considered

Keeping normalize-then-rescale was rejected because the RED demonstrates a real one-ULP double-rounding error at the public metric boundary. A first repair attempt, commit `e89dc34616dfba86683c8fc05611a7ac31a2d2c3`, bypassed scaling for all-subnormal same-sign inputs but still used floating-point compensated accumulation before division. Exact-unit checking found a separate halfway payload where that direct-float path selected the odd adjacent unit rather than ties-to-even. It was therefore not retained as the causal repair.

Always replacing the mean path with exact integer or rational arithmetic was also rejected. The defect is specific to a bounded subnormal domain where each represented input is already an integer multiple of `2^-1074`; normal-scale and mixed-remainder behavior retains the existing deterministic binary64 reference.

## Decision

Corrected repair `1c0df8a77c4b65583e9d1945864f0a72bc598a71` handles the bounded case in represented subnormal units when `max_magnitude < f64::MIN_POSITIVE` and the scientific divisor is at least the surviving term count. Each magnitude contributes its exact 52-bit subnormal unit count. The total fits in `u128` on supported `usize` widths, the exact integer quotient and remainder are computed against the original divisor, and one final round-to-nearest, ties-to-even decision selects the binary64 subnormal result. Other paths retain the predecessor normalization and compensation policy.

Edge contract `f79a1b9a1b299b34f87babbbdd766e4be6bd60df` covers both odd-floor and even-floor halfway cases. Follow-up fixture `eb29a79a3c51034dbff8d29782d8e71fe65012b6` records the halfway payload that invalidated the discarded direct-float repair: its exact unit sum leaves remainder 8 on division by 16, so the lower even unit `0x0009_2df1_1e7d_d9b8` must be selected.

## Expected effect and remaining risk

The affected same-sign all-subnormal `mean_bias` path now has a single final rounding decision in the represented unit system instead of a normalized rounding followed by subnormal rescaling. Positive and negative sign mirrors share the same magnitude rule. Normal-scale inputs, mixed-sign cancellation with retained roundoff, RMSE, coverage, Monte Carlo summaries, and `bias_standard_error` are unchanged.

The repair is intentionally bounded. Explicit-denominator paths with a divisor smaller than the surviving term count still use the previous normalization path, because their quotient can leave the subnormal grid. Any defect there requires its own represented-input counterexample before widening this implementation.

## Traceability

- PR: #488, `fix/validation-bias-overflow-safe-mean`
- Public RED: `91abdb496ef13229ed95bcf8854770a2cc71b4b8`
- Discarded first repair: `e89dc34616dfba86683c8fc05611a7ac31a2d2c3`
- Corrected production repair: `1c0df8a77c4b65583e9d1945864f0a72bc598a71`
- Halfway edge coverage: `f79a1b9a1b299b34f87babbbdd766e4be6bd60df`
- Direct-float regression fixture: `eb29a79a3c51034dbff8d29782d8e71fe65012b6`
- Changelog: `65ff502b8f46f4043ca44983722e0f9722105a08`
- Production module: `crates/validation_core/src/numeric.rs`
- Public contract: `crates/validation_core/tests/bias_same_sign_subnormal_double_rounding_contract.rs`

## Standards and methodological authority

IEEE 754-2019 remains the active IEEE floating-point standard as verified on 2026-09-05; IEEE P754 is an active revision PAR approved 2024-06-06 and is not a published replacement. ISO/IEC 60559:2020 remains a published International Standard at stage 60.60. AERA, APA, and NCME continue to revise the 2014 *Standards for Educational and Psychological Testing*; the Joint Committee announced in 2024 is charged with revising that edition, so no unpublished revision is treated as normative authority. Morris, White, and Crowther (2019) remains the methodological basis for defining simulation performance measures against known truth; it does not prescribe this floating-point implementation.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
