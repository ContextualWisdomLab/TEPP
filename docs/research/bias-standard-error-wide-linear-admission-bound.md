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

This is a width theorem, not by itself a production-equivalence claim. Normalization, full-width multiplication/subtraction, dyadic restoration, reduced denominator/midpoint rounding, and upstream represented-residual admission still have to compose without introducing a stricter refusal than the existing pair proof.

RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` made that product-capacity theorem executable by adding a characterization test that referenced a not-yet-defined proof helper. Repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` adds the helper and checks the 256-bit extremum together with the known compact, common-power, odd-boundary, and exhaustive small-integer composition geometries.

## Represented-input reachability

The odd `D = 2^58 + 1` integer boundary is useful for arithmetic width, but by itself it does not establish that the wider route is needed by a residual set that passes the represented binary64 subtraction gates. A separate characterization supplies such a case without weakening those gates.

At `n = 4096`, take represented residuals with three distinct values: one `0`, one `1`, and 4094 copies of `2^53`. With truth fixed at represented zero, every residual construction is exact. The only distinct pairwise subtractions are `1`, `2^53`, and `2^53 - 1`; all three are exactly representable in binary64, so the production pair-subtraction roundoff predicate accepts the fixture's distinct subtraction classes. Because coefficient `1` is present, the canonical common power-of-two shift is zero rather than an artifact that removes the width pressure.

The canonical sums `S1` and `S2` still fit `u128`, but both narrow cancellation products `n*S2` and `S1^2` exceed `u128`. The exact pair numerator remains only 119 bits:

`P = 664289479338799435974172876300357631`.

The two-limb product/subtraction recovers that value exactly. The unreduced scientific denominator is `4096^2 * 4095 = 68702699520`, which is also below the current `2^53` exact-denominator gate. This closes the narrower question of whether Wide256 recovery is reachable from represented residuals; it does not authorize changing the production sample-count admission.

## Exact-rounding width is a second boundary

Widening only the O(n) numerator identity is not sufficient for end-to-end exact admission. Before repair `e4a85f53a611922be7492fe906d62ce65787c18e`, the exact candidate/midpoint proof in `bias_se.rs` formed scaled `u128` products when comparing the exact rational target against a binary64 candidate square and the adjacent midpoint square.

A smaller represented fixture with the same three residual classes at `n = 2050` reaches that boundary. Its residual and pairwise-subtraction classes remain exact and its common dyadic shift is zero. Both narrow O(n) products require 129 bits, while the exact pair numerator is still only 118 bits:

`P = 332306998946228931332463617650984961`.

The unreduced denominator is `8610922500`, below `2^53`. The normal binary64 ratio/square-root seed is `0x4296998e1aff78de`. Its compact dyadic significand/exponent are `3180642552495215 * 2^-9`. Exact candidate-square comparison therefore needs both `P * 2^18` and `denominator * significand^2`; each is 136 bits. Comparing with the exact midpoint to the upward neighbor requires 140-bit operands. Wide256 comparison shows the exact target is above the candidate square and below the midpoint square, proving that the original candidate is the nearest binary64 result.

This is the causal resource finding that motivated the production primitive repair: exact-rounding comparison width must cover the same represented cases as the wider numerator path. Pairwise proof remains the fail-closed comparison authority until the whole represented-input route is proven admission-equivalent.

Executable evidence for both represented boundaries is `crates/validation_core/tests/bias_standard_error_represented_wide_recovery_characterization.rs`: reachability was introduced at `5a19b6334487b43fb630abba7e487d7cf4c49960`, and the exact-midpoint width characterization at `a8423173188fa53a26a16d3afdafeb76e114cc1d`.

## Exponent-safe exact comparison

The 136/140-bit finding does not require an arbitrary-precision integer for the comparison itself. Characterization `aab9fe9115cee97225f2aa81e54a55ceafb23336` adds a two-limb comparison reference that keeps each nonzero integer mantissa in `Wide256` and keeps its power-of-two scale as a signed exponent. It first compares the absolute top-bit positions. Only when those positions tie does it compare significand bits aligned from the common top bit. No `2^k` factor is materialized.

For represented `n = 2050`, that reference orders the 136-bit candidate-square operands as target greater than candidate square and the 140-bit upward-midpoint operands as target less than midpoint square, reproducing the exact nearest-binary64 decision from the earlier characterization. It also verifies equality and strict ordering across exponent pairs `-2148/-2149` and `2046/2047`, where a direct `u128` shift-factor construction is not a viable representation. The full-width edge `(2^128 - 1)^2` remains exactly represented as high limb `2^128 - 2`, low limb `1`.

RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` moved this finding onto the authoritative path by requiring `correctly_rounded_scaled_sqrt_ratio` to return `0x4296998e1aff78de` for the represented `n=2050` reduced ratio; the previous bounded comparator returned `None` before it could make the exact decision. Repair `e4a85f53a611922be7492fe906d62ce65787c18e` integrates the two-limb product and signed-exponent ordering into `crates/validation_core/src/bias_se.rs`. Candidate and adjacent-midpoint denominator products are now formed as exact `Wide256` values and compared without materializing an oversized power-of-two factor.

This is a production primitive integration, not a sample-count admission change. `exact_pair_distance_standard_error` remains deliberately bounded to `n=4..=16`; the `n=2050` ratio is a private exact-rounding contract that prevents a future wider numerator route from inheriting the former false refusal. Exact-head Rust/rustdoc/line+branch/security/documentation GREEN and represented-input route equivalence remain required before admission can widen.

The standards basis remains the current published floating-point standards, IEEE 754-2019 and ISO/IEC 60559:2020. IEEE currently lists 754-2019 as an active standard and ISO lists ISO/IEC 60559:2020 as the published international standard; IEEE P754 is an active revision project and is not treated as a published replacement in this decision.

## Represented route-equivalence slice

Characterization `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` adds an actual two-pass O(n²) pairwise authority comparison against the two-limb O(n) identity for represented residual classes `{0, 1, 2^53}` at `n = 4, 16, 17, 65, 257, 2050`. It does not use the earlier analytic pair-numerator formula for the comparison: represented pair differences are checked for exact binary64 subtraction, normalized to a common dyadic unit, squared, and accumulated by the pairwise route. Independently, the wide-linear route selects the represented minimum as anchor, verifies every anchor-relative subtraction, derives the same dyadic unit, and computes `n*S2 - S1^2` with full-width products and subtraction.

For every characterized sample count, both routes must produce the same unit exponent, exact pair numerator, and GCD-reduced `(numerator, denominator, unit_exponent)` tuple presented to the exact rounder. The `n = 2050` fixture is the resource boundary rather than a small-only identity check: both narrow cancellation products must overflow, while the pairwise and Wide256 routes must agree on `P = 332306998946228931332463617650984961`, denominator `8610922500`, divisor `1`, and unit exponent `0`. The characterization also reverses the `n = 65` represented sequence to verify that selecting the minimum exact anchor makes the wide-linear result independent of observation order.

This closes a deterministic represented-input arithmetic-equivalence slice, not the whole production admission decision. The test still carries a test-only copy of the candidate arithmetic and does not route `bias_standard_error` samples above `n=16` through the wider implementation. Production integration therefore still requires one implementation of canonical represented normalization and Wide256 numerator routing in the owning module, same-head exact-rounding/tie-to-even evidence, and comparison against the pairwise authority before the fallback can be demoted or the cutoff widened.

## Anchor-exact admission is broader than pairwise-f64 admission

Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` isolates a proof-admission distinction inside the existing `n=4` production sample budget. Use represented residuals `[0, 1, 2^-54, 2]` with truth fixed at represented zero. Every residual and every subtraction from the minimum anchor `0` is exact. The non-anchor subtraction `1 - 2^-54`, however, rounds in binary64; the current O(n²) pair proof therefore refuses before exact integer pair accumulation.

That refusal is a limitation of the reference path, not evidence that the represented geometry lacks an exact pair-distance proof. With common unit `2^-54`, the anchor-relative integer coefficients are `[0, 2^54, 1, 2^55]`. Direct integer pair distances and the linear identity agree exactly on

`P = 3569704090242693886528325169446915`.

The public metric already returns the correctly represented standard error `0x3fdea33e2c83c140` through its generic translated fallback, so this fixture is not a public numerical defect. It demonstrates that requiring every non-anchor pair subtraction to be exact in binary64 is sufficient for the current O(n²) authority but is not a scientific prerequisite for canonical anchor-linear exact admission. A production O(n) route should therefore be evaluated for two properties separately: equality with the pair authority wherever both admit, and intentional strictly broader admission for anchor-exact geometries such as this one. The pairwise path remains fail-closed comparison evidence during that migration; the finding does not justify widening `n=4..=16`.

## Consequence for #491

The candidate numerator route remains `narrow O(n) -> Wide256 O(n) -> buffered pair fail-closed fallback`, introduced by RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` and repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`. The predecessor narrow-to-pair hybrid remains a comparison baseline.

For odd `D = 2^58 + 1`, `n = 65`, the predecessor hybrid must still select the pair fallback because its narrow products overflow. The newer candidate must select the wider-product route, recover the same exact restored numerator as both O(n²) references, and report `used_wide_product=true` with `used_pairwise_fallback=false`. The power-of-two-normalized `D=2^58,n=65` geometry and odd `n=64` geometry remain narrow-path admissions. Separate route observability matters because a correct exact numerator does not by itself show whether the candidate avoided O(n²) allocation.

The product-width theorem makes a post-Wide256 pair fallback look redundant for the numerator identity within the canonical `u128` coefficient domain. The represented `n=2050` midpoint finding showed why this did not extend automatically to the whole exact-rounding proof. The exponent-safe comparison is now integrated into the authoritative rounding primitive, `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` demonstrates equality of the pairwise and wide-linear exact-ratio inputs across a deterministic represented-input slice, and `2bc1d2284d75154e020640adb573c1cfadf005fb` demonstrates an intended anchor-linear-only admission slice. The remaining equivalence work moves outward to production canonicalization/routing and fail-closed decisions across the represented domain, not to another sample-count staircase.

The executable accumulator characterization is `crates/validation_core/tests/bias_standard_error_wide_linear_admission_bound_characterization.rs`. The represented-input width characterization is `crates/validation_core/tests/bias_standard_error_represented_wide_recovery_characterization.rs`. The represented route-equivalence characterization is `crates/validation_core/tests/bias_standard_error_represented_route_equivalence_characterization.rs`. The anchor-linear admission characterization is `crates/validation_core/tests/bias_standard_error_anchor_linear_admission_characterization.rs`. The exponent-safe comparison characterization is `crates/validation_core/tests/bias_standard_error_wide_scaled_comparison_characterization.rs`. The timing/layout vehicle is `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it records `used_wide_product` and `used_pairwise_fallback` independently so route selection can be audited from raw CSV.

## Decision

Do not widen production `validation_core::bias_standard_error` solely because the exact-rounding primitive now supports wider comparison products, because the deterministic represented route-equivalence slice passes, or because anchor-linear proof admission is strictly broader than pairwise-f64 subtraction admission. Production admission remains `n=4..=16`. A budget change requires production canonical anchor-relative normalization and Wide256 routing, equality against the pairwise authority wherever both admit, explicit tests for intended anchor-linear-only admissions, exact tie-to-even preservation, recorded Rust 1.98.0 `--release` raw CSV, CPU/OS/build metadata, allocator/RSS evidence, applicable full buyer-path p95 evidence, exact-head Rust/rustdoc/100% line+branch coverage/security/documentation GREEN, and qualifying independent current-head review.

## Traceability

- Issue: #491
- Landing PR: #488
- Accumulator-bound characterization: `b7e4da353ac58069afd73ee7c0e8427d49993fdb`
- Wide-product capacity RED: `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993`
- Wide-product capacity repair: `e9a7dee29afb97542bfe2965f850c8ab5a34368e`
- Represented-input Wide256 reachability: `5a19b6334487b43fb630abba7e487d7cf4c49960`
- Represented exact-midpoint width characterization: `a8423173188fa53a26a16d3afdafeb76e114cc1d`
- Exponent-safe scaled comparison characterization: `aab9fe9115cee97225f2aa81e54a55ceafb23336`
- Production scaled-comparison RED: `f7717361ad8c5f0592688c1514c104cc1b4adabe`
- Production scaled-comparison repair: `e4a85f53a611922be7492fe906d62ce65787c18e`
- Represented pair/wide exact-ratio equivalence: `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943`
- Anchor-linear-only represented admission: `2bc1d2284d75154e020640adb573c1cfadf005fb`
- Narrow-wide-pair RED: `3136739460ef0c8e13c044a7e5b04891e4f4e23d`
- Narrow-wide-pair repair: `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md`
- Production module under decision: `crates/validation_core/src/bias_se.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
