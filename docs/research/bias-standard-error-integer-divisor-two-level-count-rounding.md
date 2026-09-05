# Exact integer-divisor two-level bias standard error

## Problem

`validation_core::bias_standard_error` already recognizes exactly translated two-level samples before using the general translated second-moment path. GAP-102 preserved the subset whose count factor is a reciprocal power of two. Fresh review found that the scientific identity is broader: some non-singleton two-level count geometries reduce to the reciprocal square of an integer that is not a power of two.

For two residual levels separated by represented gap `g`, with counts `m` and `n-m`,

`SE(mean)^2 = g^2 * m(n-m) / (n^2(n-1))`.

For `n=9`, `m=3`, the count factor is

`3*6 / (9^2*8) = 18/648 = 1/36`,

so the represented-input target is exactly `SE(mean) = |g|/6` before the final binary64 rounding.

The public RED uses `g = 0x1.ffffffffffffdp-1` (three representable steps below `1.0`) with three residuals at `0` and six at `g`. The exact quotient `g/6` rounds to bits `0x3fc5_5555_5555_5553`. The predecessor translated sum/square/FMA/square-root reconstruction returned the adjacent upper value `0x3fc5_5555_5555_5554`.

## Constraints

- The repair must remain inside TEPP Validation Evidence performance-measure arithmetic; it does not create a reusable static psychometric estimator.
- Admission must still prove exact anchor-relative residual translation before any two-level shortcut is used.
- Count algebra must be exact and overflow checked.
- The fix must not claim correctly rounded standard errors for arbitrary `n>2` or arbitrary multi-level residual distributions.
- A mathematically nonzero result that cannot be represented in binary64 must fail closed rather than become false perfect recovery.

## Decision

RED `d793f7f9ada68d5976effa6539182ba7037bf8d0` adds the nine-observation 3/6 contract, permutation invariance, sign symmetry, and minimum-subnormal false-zero refusal.

Causal repair `0a4bfcd52defe8912afa8269576395803d451bb3` replaces the power-of-two-only count predicate with an exact integer-divisor predicate. Using checked `u128` arithmetic, TEPP forms `n^2(n-1)`, verifies divisibility by `m(n-m)`, takes the integer square root of the quotient, verifies the square exactly, and admits the shortcut only when the resulting integer divisor is exactly representable as binary64. The final metric is then one division `|g|/divisor`.

This preserves GAP-102 (`6/10` of `16` gives divisor `8`) and additionally admits exact cases such as `3/6` of `9` giving divisor `6`. Count factors that are not exact reciprocal integer squares continue through the bounded translated second-moment path.

## Rejected alternatives

A payload-specific `n=9, m=3` branch was rejected because the scientific condition is the exact count identity, not this fixture. Keeping the power-of-two restriction was rejected because it makes an implementation convenience narrower than the proved estimator algebra. Replacing all two-level samples with a floating count-ratio formula was rejected because it would broaden the changed rounding surface to cases whose square root is irrational or whose count ratio is itself rounded. Arbitrary-precision production arithmetic was rejected because this defect has a narrower exact integer repair and does not justify a new runtime dependency or owner boundary.

## Risk and follow-up

This change does not prove global correct rounding of `bias_standard_error`. Two-level factors that do not reduce to an exact reciprocal integer square, and general multi-level samples, retain the current translated moment path and remain candidates for separately reproduced scientific findings. Hosted exact-head Rust, coverage, security, documentation, and independent review evidence remain required after the source mutation.

## Traceability

- RED: `d793f7f9ada68d5976effa6539182ba7037bf8d0`
- causal source repair: `0a4bfcd52defe8912afa8269576395803d451bb3`
- release-note fragment: `9651dfd123b71b14c58e34dfca4800a92e298a99`
- module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error`
- executable contract: `crates/validation_core/tests/bias_standard_error_two_level_integer_divisor_rounding_contract.rs`

IEEE 754-2019 and ISO/IEC 60559:2020 define the binary floating-point arithmetic model used by this deterministic `f64` reference, including division and square root. The relevant engineering point here is not that every composed expression is globally correctly rounded, but that an exact algebraic `g/6` target should not be unnecessarily reconstructed through additional rounded sums, products, and square root operations.

Morris, White, and Crowther (2019) treat bias and related performance measures as explicit estimand-linked quantities in simulation studies and recommend defining performance measures unambiguously and reporting Monte Carlo uncertainty. TEPP therefore treats a one-ULP change caused by avoidable arithmetic reconstruction as a Validation Evidence defect when the represented-input target is algebraically known.

## References

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic*.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
