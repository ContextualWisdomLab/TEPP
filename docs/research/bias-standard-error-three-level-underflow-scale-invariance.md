# GAP-110 — three-level standard error underflow-scale invariance

## Finding

`bias_standard_error` could return exact zero for a represented three-observation residual sample with nonzero, representable dispersion when the exact three-level proof evaluated raw products before normalization.

The public regression sample is

- `truth = [0, 0, 0]`
- `recovered = [0, f64::MIN_POSITIVE, 2 * f64::MIN_POSITIVE]`
- expected `SE(mean)` bits: `0x0009_3cd3_a2c8_198e`

The canonical exact translation is `[-m, 0, m]`, where `m = 2^-1022`. For three represented observations `[0, x, y]`,

`SE(mean)^2 = (x^2 + y^2 - xy) / 9`.

With `x = -m` and `y = m`, the exact result is `m / sqrt(3)`, which is a nonzero subnormal binary64 value. The predecessor GAP-109 path formed `x*x`, `y*y`, and `x*y` at the raw scale. All three exact products lie below the binary64 minimum-subnormal magnitude and round to signed zero. The subsequent FMA residual checks also round those exact products to zero, so a zero product could be mistaken for an error-free product. The proof then admitted a zero radicand and returned `SE(mean) = 0`.

This is deterministic performance-measure arithmetic error, not Monte Carlo uncertainty. The represented observations and target estimand are fixed; permutation or sign reflection must not change whether a nonzero standard error survives the arithmetic path.

## Causal repair

Public RED: `fe013e7dbd9b6c99371fe28ff3f6aa2cb2915408`.

Rust repair: `d3660d44f1bc315e2e34ecfcfa74c26b8f1cd257` in `crates/validation_core/src/bias.rs`.

The repair does not broaden exact-three-level admission. It extends the existing GAP-109 normalization trigger from raw product overflow to raw product loss of represented range: a nonzero square or cross-product that rounds to zero is treated as an inability to prove the identity at that magnitude. The same exactly reversible `exact_power_of_two_scale` retry is used. The normalized products must still be finite, nonzero where the source product is mathematically nonzero, exactly reconstruct the represented offsets after scale restoration, satisfy the existing FMA/error-free sum and subtraction checks, and produce an exactly represented square root before the direct identity can be admitted.

For the GAP-110 sample, normalization produces offsets `[-1, 1]`. Its radicand is `3`, whose square root is not exactly represented, so the bounded identity correctly declines admission. Control returns to the existing translated second-moment path, which already normalizes the represented geometry before squaring and returns the correctly rounded nonzero subnormal result `0x0009_3cd3_a2c8_198e`.

The public contract covers all six permutations and their sign mirrors in `crates/validation_core/tests/bias_standard_error_three_level_underflow_scale_contract.rs`. An internal unit contract also asserts that the bounded exact-three-level helper returns `None` for `(-f64::MIN_POSITIVE, f64::MIN_POSITIVE)` instead of manufacturing an exact zero proof.

## Alternatives rejected

Returning `InvalidInput` whenever a raw square underflows was rejected because the requested final standard error can remain representable, as it does here. Treating FMA residual zero as sufficient proof was rejected because an exact result below the destination format can round to zero in both the product and the residual check. Applying unconditional normalization to every three-level sample was rejected because GAP-108/GAP-109 already preserve a narrower finite-product fast path with explicit exactness evidence. Arbitrary-precision production arithmetic remains out of scope for this TEPP Validation Evidence repair and would cross the reusable numerical-owner boundary without a demonstrated need.

## Scientific and standards trace

IEEE 754-2019 remains the active published IEEE floating-point standard. IEEE P754 is an active revision PAR, approved 2024-06-06, intended to supersede 754-2019 but is not a published replacement as of 2026-09-05. ISO/IEC 60559:2020 remains the published international adoption of IEEE 754-2019. The repair therefore documents destination-format underflow and exact-operation evidence against the current published standard rather than a draft revision.

Morris, White, and Crowther's ADEMP framework distinguishes the deterministic definition and computation of a performance measure from Monte Carlo standard error caused by a finite number of simulation repetitions. GAP-110 concerns the former: the same represented sample cannot legitimately move from nonzero dispersion to zero because an intermediate product falls outside binary64 range.

AERA, APA, and NCME continue to publish the 2014 *Standards for Educational and Psychological Testing* while the Joint Committee is revising that edition. TEPP therefore keeps the 2014 edition as the normative testing-standards authority and treats revision materials as development evidence only.

## Traceability

- Bounded context: Validation Evidence
- Aggregate/API: `bias_standard_error`
- Production module: `crates/validation_core/src/bias.rs`
- Public contract: `crates/validation_core/tests/bias_standard_error_three_level_underflow_scale_contract.rs`
- RED: `fe013e7dbd9b6c99371fe28ff3f6aa2cb2915408`
- Causal repair: `d3660d44f1bc315e2e34ecfcfa74c26b8f1cd257`
- CHANGELOG: `CHANGELOG.d/validation-bias-standard-error-three-level-underflow-scale.md`
- Predecessor: GAP-109 `1eaac11e715144127048fc20033719777528c692`
- Owner boundary: reusable static psychometric arithmetic remains in `fast-mlsirm`; no mutable `contextual-orchestrator` contract is consumed.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

ISO/IEC. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
