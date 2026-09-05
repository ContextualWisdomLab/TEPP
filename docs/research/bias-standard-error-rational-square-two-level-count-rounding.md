# Exact rational-square two-level bias standard error

## Problem

`validation_core::bias_standard_error` already proves exact anchor-relative translation before recognizing two-level residual samples. GAP-103 widened the direct count path from reciprocal powers of two to reciprocal integer squares, but that predicate is still narrower than the estimator algebra: the reduced two-level count factor can be the square of a non-unit rational.

For two represented residual levels separated by `g`, with counts `m` and `n-m`,

`SE(mean)^2 = g^2 * m(n-m) / (n^2(n-1))`.

For `n=10`, `m=2`, the count factor is

`2*8 / (10^2*9) = 16/900 = 4/225 = (2/15)^2`,

so the represented-input target is `SE(mean) = 2*|g|/15` before final binary64 rounding. The public RED uses `g = 0x1.ffffffffffffep-1`, with two residuals at `0` and eight at `g`. The exact represented rational target rounds to bits `0x3fc1_1111_1111_1110`; the GAP-103 predecessor rejects the non-unit numerator, falls through to translated sum/square/FMA/square-root reconstruction, and returns adjacent upper bits `0x3fc1_1111_1111_1111`.

## Constraints

- The repair remains inside TEPP Validation Evidence performance-measure arithmetic and does not create a reusable static psychometric estimator.
- Exact anchor-relative translation remains a prerequisite; no shortcut is admitted from rounded residual labels alone.
- Count algebra is checked in `u128`; overflow or a non-square reduced factor falls back to the bounded translated second-moment path.
- The repair must preserve GAP-102 and GAP-103 reciprocal-integer cases.
- A mathematically nonzero result below binary64 support fails closed rather than becoming false zero.
- The change does not claim globally correctly rounded standard errors for arbitrary `n>2`, multi-level residuals, or irrational count factors.

## Decision

RED `6dc8116c89fa44a7ff1d58a8f9a51c876993d33f` adds the 10-observation 2/8 contract, permutation invariance, sign symmetry, and minimum-subnormal false-zero refusal.

Causal repair `8f2803c874568877e20e4c0f267ec5ce613daa3d` reduces `m(n-m) / (n^2(n-1))` by the exact integer greatest common divisor, verifies that both reduced numerator and denominator are perfect squares, and returns their integer square roots as the rational scale. When the non-singleton two-level path proves such a scale, TEPP reuses `deterministic_representable_sum_over_count` with `numerator` copies of `|gap|` and the exact integer `denominator`. This preserves a single deterministic represented-rational rounding boundary without multiply-first overflow or divide-first subnormal loss.

The predecessor cases remain admitted: `3/6` of `9` reduces to `(1/6)^2`, and `6/10` of `16` reduces to `(1/8)^2`. The new 2/8-of-10 case reduces to `(2/15)^2`. Count factors that are not rational squares retain the existing translated second-moment path.

## Rejected alternatives

A fixture-specific `n=10,m=2` branch was rejected because the scientific condition is the reduced rational-square identity, not the example. Retaining the reciprocal-integer restriction was rejected because it excludes exact algebraic targets such as `2|g|/15`. A generic floating `sqrt(m(n-m)/(n^2(n-1)))` shortcut was rejected because it changes the rounding surface for irrational count factors and reintroduces the same composed-rounding problem. Direct `(|g| * numerator) / denominator` was rejected because the multiplication can overflow although the final scaled result is representable; direct `(|g| / denominator) * numerator` can lose subnormal mass before the numerator is restored. Arbitrary-precision production arithmetic was rejected because the exact reduced-count predicate and the existing deterministic represented-sum divisor are sufficient for this defect.

## Risk and follow-up

This repair proves only the exact rational-square two-level subset after exact residual translation. Rationally non-square two-level factors and general multi-level samples remain on the translated moment implementation and require their own reproduced scientific finding before any broader arithmetic change. Hosted exact-head Rust, coverage, documentation, security, and independent review evidence remain required after this source mutation.

## Traceability

- RED: `6dc8116c89fa44a7ff1d58a8f9a51c876993d33f`
- causal source repair: `8f2803c874568877e20e4c0f267ec5ce613daa3d`
- release-note fragment: `f0bb7af91c08b7766f43fec4c8d7984b68a3599f`
- module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error`
- executable contract: `crates/validation_core/tests/bias_standard_error_two_level_rational_scale_rounding_contract.rs`

IEEE 754-2019 remains the active IEEE floating-point standard, and ISO/IEC 60559:2020 remains the published international adoption. They define the binary floating-point arithmetic model relevant to this deterministic `f64` reference. The engineering conclusion here is narrower than global correct rounding: when the represented-input statistical identity is already an exact rational scale, reconstructing it through additional rounded sums, products, FMA, division, and square root is avoidable.

Morris, White, and Crowther (2019) frame simulation evaluation around known truth, explicit estimands and performance measures, and Monte Carlo uncertainty. TEPP therefore treats a reproducible one-ULP displacement in a declared Validation Evidence performance measure as a scientific arithmetic defect when the represented-input target is analytically known.

## References

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic*.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
