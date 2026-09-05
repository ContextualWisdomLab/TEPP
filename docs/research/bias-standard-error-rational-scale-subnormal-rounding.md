# Bias standard error rational-scale subnormal rounding

## Problem

GAP-104 established an exact two-level shortcut when the reduced count factor is a rational square. The surviving implementation still routed that exact rational scale through `deterministic_representable_sum_over_count`, whose same-sign path first normalizes by an exact power of two. That normalization is harmless while the restored quotient remains normal, but it can introduce a second binary64 rounding boundary when the restored scientific result is subnormal.

The public counterexample uses `n = 33`, with six represented residuals at `0` and 27 at `g = f64::from_bits(0x004a_2c74_6ac3_028e)`.

For a two-level sample,

`SE(mean)^2 = g^2 * m(n-m) / (n^2(n-1))`.

Here

`6 * 27 / (33^2 * 32) = 162 / 34848 = 9 / 1936 = (3 / 44)^2`,

so the represented-input target is exactly `3 * |g| / 44`. Correct rounding to binary64 gives bits `0x000e_46cb_22f6_0165`. The GAP-104 predecessor normalizes the rational numerator, rounds the normalized quotient, then restores the power-of-two scale into the subnormal range and returns adjacent lower bits `0x000e_46cb_22f6_0164`.

This is not a disagreement about the estimand. It is an avoidable floating-point projection error after the exact two-level count geometry has already been proved.

## Constraints

- TEPP owns Validation Evidence performance-measure arithmetic; reusable static psychometric estimators remain in `fast-mlsirm`.
- Production numerical arithmetic remains Rust-first and deterministic.
- The repair preserves the existing exact translated-residual admission and does not send arbitrary multi-level or irrational count geometry through a new path.
- A mathematically nonzero result below binary64 range fails closed rather than silently becoming zero.
- No arbitrary-precision production dependency is introduced.

## Decision

For exact rational-square two-level geometry, the implementation attempts a bounded exact subnormal projection before the existing represented sum-over-count path.

A positive finite binary64 magnitude has an integer significand of at most 53 bits. Expressed in units of the minimum positive subnormal, a normal value with encoded exponent `e` has exact unit count `significand * 2^(e-1)`; a subnormal value uses its stored fraction directly. The rational scale therefore has exact unit numerator

`significand * numerator * 2^(e-1)`

for normal inputs, or `significand * numerator` for subnormal inputs. On supported targets, the significand and `usize` rational factor fit the bounded `u128` product. The exponent shift is checked; if it cannot fit, the result is outside this subnormal projection and the existing normal overflow-safe path remains authoritative. Division by the exact integer denominator is rounded once with round-to-nearest, ties-to-even. Results above the normal/subnormal boundary fall back to the existing path; a nonzero exact result that rounds below one minimum-subnormal unit returns `ValidationError::InvalidInput`.

This preserves the existing overflow-safe path for normal results while removing the double-rounding surface at the subnormal boundary.

## Alternatives rejected

1. **Keep normalized rational scaling and accept one-ULP drift.** Rejected because the exact count geometry is already known and the drift is an implementation artifact, not simulation uncertainty.
2. **Special-case the `6/27 of 33` payload.** Rejected because the defect is the normal-to-subnormal restoration boundary, not those counts.
3. **Apply arbitrary-precision arithmetic to all Validation Evidence metrics.** Rejected as substantially broader than the causal defect and contrary to the bounded Rust reference-path design.
4. **Replace all standard-error arithmetic with a closed form.** Rejected because general multi-level samples and non-square count factors do not share this exact rational identity.

## Evidence and traceability

- Public RED: `8b7995d2320cf256b3a38991ae1f8a230ca00146`
- Causal source repair: `ab0f0df1b8f36647f67239a5c628daed9023210e`
- Public boundary coverage: `4daeb65daee98b25fdc5a29744a710752e229a50`
- Production branch-coverage hardening: `8f6916dda1bc241e6bfd5dab0840e621769f995b`
- CHANGELOG fragment: `900b2091d572c9a984e350d2795ec58bfdd3177c`
- Module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error`
- Public contract: `crates/validation_core/tests/bias_standard_error_rational_scale_subnormal_rounding_contract.rs`
- Expected represented result: `0x000e_46cb_22f6_0165`
- Predecessor result: `0x000e_46cb_22f6_0164`
- Edge contract: a minimum-subnormal gap with the same exact `3/44` count scale is mathematically nonzero but below binary64 range and returns `InvalidInput`.

The public contract fixes permutation and sign-mirror invariance, both ties-to-even directions in minimum-subnormal units, exact rounding onto `f64::MIN_POSITIVE`, and fail-closed underflow. The crate-private branch tests cover invalid helper admission, normal/subnormal significand decoding, midpoint increment/non-increment, zero refusal, normal-boundary return, above-boundary fallback, and exponent-shift fallback. Hosted exact-head CI remains authoritative for GREEN; branch-local arithmetic proof or predecessor workflow results are not transferred as current-head CI evidence.

## Standards and methodological basis

IEEE 754-2019 remains the active IEEE floating-point standard. IEEE P754, approved as a PAR on June 6, 2024, is an active revision project that supersedes 754-2019 only when a replacement standard is actually published. ISO/IEC 60559:2020 remains a published international standard adopting the same floating-point arithmetic model. These sources support explicit control of destination-format rounding and distinguish arithmetic semantics from application-level statistical uncertainty.

For Validation Evidence, Morris, White, and Crowther (2019) treat bias and empirical standard error as simulation performance measures and require Monte Carlo uncertainty to be reported as simulation uncertainty. A deterministic one-ULP arithmetic projection error in a represented performance measure is therefore not something to absorb into Monte Carlo error.

The currently published AERA/APA/NCME *Standards for Educational and Psychological Testing* remains the 2014 edition. AERA, APA, and NCME announced a Joint Committee in 2024 to revise that edition; the in-progress revision is not treated here as published normative authority.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization, & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
