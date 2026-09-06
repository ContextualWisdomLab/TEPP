# Bias standard-error wide-linear admission bound

## Problem

Issue #491 previously required a fixture where the two-limb O(n) exact pair-numerator reference would refuse because the normalized coefficient sum or normalized square sum overflowed `u128` while the O(n²) pair numerator still fit. That search target is inconsistent with the canonical anchor-relative integer representation used by the characterization.

Let `c_i` be canonical nonnegative integer coefficients after subtracting the minimum represented value and removing the greatest common power-of-two unit. At least one `c_i` is zero. Define

`P = sum_{i<j} (c_i - c_j)^2`,
`S1 = sum_i c_i`, and
`S2 = sum_i c_i^2`.

For every nonnegative integer `c_i`, `c_i <= c_i^2`. Therefore `S1 <= S2`. Because at least one anchor coefficient is zero, the pair terms against that anchor contain every `c_i^2`, and all remaining pair terms are nonnegative. Therefore `S2 <= P`. Hence

`S1 <= S2 <= P`.

If the exact canonical pair numerator `P` fits `u128`, both O(n) accumulators necessarily fit `u128`. A pair-admitted fixture cannot fail the wider O(n) reference solely because `S1` or `S2` overflowed.

## Full-width product bound

The normalized narrow O(n) implementation can still refuse while `P` fits because the products `n*S2` and `S1^2` may require more than 128 bits before cancellation. The odd-diameter fixture `D = 2^58 + 1`, `n = 65` demonstrates that case: both products require 129 bits while the exact pair numerator is 123 bits.

That narrow refusal does not imply that a wider product requires arbitrary precision. On supported targets the sample count is converted from `usize` to `u128`; pair admission gives `S2 <= P <= u128::MAX` and `S1 <= S2 <= u128::MAX`. Consequently each exact cancellation product is a product of two `u128` values. Its maximum width is 256 bits: `(2^128 - 1)^2 < 2^256`. The dependency-free two-limb `Wide256` product representation is therefore wide enough for every canonical coefficient set whose exact pair numerator is admitted by `u128`.

This is a width theorem, not yet a production-equivalence claim. The implementation still has to prove that normalization, full-width multiplication/subtraction, dyadic restoration, reduced denominator/midpoint rounding, and upstream represented-residual admission compose without introducing a stricter refusal than the existing pair proof.

RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` made that product-capacity theorem executable by adding a characterization test that referenced a not-yet-defined proof helper. Repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` adds the helper and checks the 256-bit extremum together with the known compact, common-power, odd-boundary, and exhaustive small-integer composition geometries.

## Represented-input reachability

The odd `D = 2^58 + 1` integer boundary is useful for arithmetic width, but by itself it does not establish that the wider route is needed by a residual set that passes the represented binary64 subtraction gates. A separate characterization now supplies such a case without weakening those gates.

At `n = 4096`, take represented residuals with three distinct values: one `0`, one `1`, and 4094 copies of `2^53`. With truth fixed at represented zero, every residual construction is exact. The only distinct pairwise subtractions are `1`, `2^53`, and `2^53 - 1`; all three are exactly representable in binary64, so the production pair-subtraction roundoff predicate accepts the fixture's distinct subtraction classes. Because coefficient `1` is present, the canonical common power-of-two shift is zero rather than an artifact that removes the width pressure.

The canonical sums `S1` and `S2` still fit `u128`, but both narrow cancellation products `n*S2` and `S1^2` exceed `u128`. The exact pair numerator remains only 119 bits:

`P = 664289479338799435974172876300357631`.

The two-limb product/subtraction recovers that value exactly. The unreduced scientific denominator is `4096^2 * 4095 = 68702699520`, which is also below the current `2^53` exact-denominator gate. This closes the narrower question of whether Wide256 recovery is reachable from represented residuals; it does not yet prove the full production route through dyadic restoration and midpoint rounding, and it does not authorize changing the production sample-count admission.

Executable evidence is `crates/validation_core/tests/bias_standard_error_represented_wide_recovery_characterization.rs`, introduced at `5a19b6334487b43fb630abba7e487d7cf4c49960`.

## Consequence for #491

The existing candidate route remains `narrow O(n) -> Wide256 O(n) -> buffered pair fail-closed fallback`, introduced by RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` and repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`. The predecessor narrow-to-pair hybrid remains a comparison baseline.

For odd `D = 2^58 + 1`, `n = 65`, the predecessor hybrid must still select the pair fallback because its narrow products overflow. The newer candidate must select the wider-product route, recover the same exact restored numerator as both O(n²) references, and report `used_wide_product=true` with `used_pairwise_fallback=false`. The power-of-two-normalized `D=2^58,n=65` geometry and odd `n=64` geometry remain narrow-path admissions. Separate route observability matters because a correct exact numerator does not by itself show whether the candidate avoided O(n²) allocation.

The new width theorem makes a post-Wide256 pair fallback look redundant within the canonical `u128` coefficient domain: product width alone cannot cause Wide256 refusal where the pair numerator is admissible. It is nevertheless retained as a fail-closed comparison reference until represented-input admission equivalence and exact-head execution demonstrate that no other stage creates a legitimate wider-route refusal.

The executable accumulator characterization is `crates/validation_core/tests/bias_standard_error_wide_linear_admission_bound_characterization.rs`. The represented-input reachability characterization is `crates/validation_core/tests/bias_standard_error_represented_wide_recovery_characterization.rs`. The executable timing/layout vehicle is `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it records `used_wide_product` and `used_pairwise_fallback` independently so route selection can be audited from raw CSV.

## Decision

Do not promote the wider O(n) route into production solely from the arithmetic proof or characterization. Production `validation_core::bias_standard_error` remains bounded to `n=4..=16`. A production change requires recorded Rust 1.98.0 `--release` raw CSV, CPU/OS/build metadata, allocator/RSS evidence, represented-input admission comparison through the complete exact-rounding path, applicable full buyer-path p95 evidence, exact-head Rust/rustdoc/100% line+branch coverage/security/documentation GREEN, and qualifying independent current-head review.

## Traceability

- Issue: #491
- Landing PR: #488
- Accumulator-bound characterization: `b7e4da353ac58069afd73ee7c0e8427d49993fdb`
- Wide-product capacity RED: `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993`
- Wide-product capacity repair: `e9a7dee29afb97542bfe2965f850c8ab5a34368e`
- Represented-input Wide256 reachability: `5a19b6334487b43fb630abba7e487d7cf4c49960`
- Narrow-wide-pair RED: `3136739460ef0c8e13c044a7e5b04891e4f4e23d`
- Narrow-wide-pair repair: `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-standard-error-wide-linear-admission-bound.md`
- Production module under decision: `crates/validation_core/src/bias_se.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
