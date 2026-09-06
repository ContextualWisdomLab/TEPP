# Bias standard-error wide-linear admission bound

## Problem

Issue #491 originally treated the production `n=4..=16` exact bias-standard-error proof as a sample-count staircase. The accumulated evidence separates three questions: whether represented data admit an exact proof, whether the arithmetic representation is wide enough to carry that proof, and whether the resource cost is acceptable for production. None is resolved by incrementing `n` alone.

For an earlier characterization that subtracts the represented minimum and produces nonnegative integer coefficients `c_i` on a common exact dyadic unit, define `P = sum_{i<j}(c_i-c_j)^2`, `S1 = sum_i c_i`, and `S2 = sum_i c_i^2`. Because at least one coefficient is zero and every coefficient is a nonnegative integer, `S1 <= S2 <= P`. Thus a pair-admitted `P <= u128::MAX` cannot fail a wider O(n) reference solely because `S1` or `S2` overflowed. This theorem is scoped to that nonnegative minimum-anchor representation; production exact-anchor coordinates may be signed.

## Full-width product and rounding bounds

Odd diameter `D=2^58+1,n=65` is the canonical narrow-width witness: both `n*S2` and `S1^2` require 129 bits while the exact pair numerator requires 123 bits. RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` makes the two-limb capacity theorem executable. RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` → repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` adds measured `narrow O(n) -> Wide256 O(n) -> pair` route telemetry.

Represented-input reachability `5a19b6334487b43fb630abba7e487d7cf4c49960` reaches the wider numerator route at `n=4096` on `{0,1,2^53}` and recovers `P=664289479338799435974172876300357631`. At represented `n=2050`, characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d` finds `P=332306998946228931332463617650984961`, denominator `8_610_922_500`, but 136-bit candidate-square and 140-bit adjacent-midpoint comparison operands. `aab9fe9115cee97225f2aa81e54a55ceafb23336` shows those comparisons can remain bounded as `Wide256` mantissa plus signed dyadic exponent. RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` → repair `e4a85f53a611922be7492fe906d62ce65787c18e` integrates that comparator into production; `1240ace8eb41a01fa72a4bb99df842fd550a1288` fixes both exact tie-to-even parity directions.

IEEE 754-2019 and ISO/IEC 60559:2020 remain the published floating-point basis; IEEE P754 is an active revision project rather than a published replacement.

## Represented proof equivalence and anchor admission

Characterization `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` compares an actual O(n²) pair authority with an independent Wide256 O(n) identity for represented residual classes `{0,1,2^53}` at `n=4,16,17,65,257,2050`. Wherever both admit, they produce the same common unit, exact pair numerator, and reduced ratio. Its minimum-anchor rule is scoped to that input family, not a universal production policy.

Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` shows why: residuals `[0,1,2^-54,2]` have exact coordinates from anchor `0`, while non-anchor subtraction `1-2^-54` rounds. On common unit `2^-54`, coordinates `[0,2^54,1,2^55]` give `P=3569704090242693886528325169446915`. The generic fallback happens to return the same public result, so this is an admission distinction rather than a public defect.

Source RED `fd9f9ff2c5c395e4cc13042232f4deef018adb48` then shows a public defect with `[0,1,2,-2^53]`. The represented minimum `-2^53` cannot exactly translate `1`, but represented anchor `0` preserves every coordinate. Signed unit-one coordinates give `P=243388915243820099130562543878155`, denominator `48`, and correctly rounded result `0x4320000000000001`; the predecessor translated floating-moment fallback returns `0x4320000000000000`. The RED Actions runs were cancelled by the immediate successor and are not hosted RED evidence.

Initial repair `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f` searched every represented residual as a candidate exact anchor. Follow-up RED `9f403194a2ec1636531c2dfe9229cfb34b73d747` proves that set is still incomplete. With represented residuals `[1,2^-54,2,3]`, no observed residual is a universal exact anchor: each nonzero candidate loses the tiny represented component in at least one subtraction. Neutral dyadic anchor `0`, although not an observed residual in this fixture, preserves every coordinate exactly.

On common unit `2^-54`, those coordinates are `[2^54,1,2^55,3*2^54]` and

`P = 6490371073168534319490338297741315`.

The exact scientific denominator is `48 * 2^108`, equivalently the exact-rounding route receives numerator `P`, denominator `48`, unit exponent `-54`. Correct binary64 rounding is `0x3fe4a7e9cb8a3491`. The predecessor translated floating-moment fallback returns adjacent upper `0x3fe4a7e9cb8a3492`; this is a second public one-ULP defect and demonstrates that “search observed anchors” is not a complete scientific admission policy.

Repair `6cf30eeb549c0df0377bda1111cf46396e8282a3` expands the production candidate set to neutral `0` plus every represented residual. Every candidate must preserve all translated coordinates exactly. Among admitted candidates, the implementation chooses the smallest maximum translated magnitude and breaks ties by represented anchor value, preserving permutation invariance while preferring a smaller exact dynamic range when an observed anchor is useful. Signed dyadic coordinates keep positive and negative coefficient mass separately; `n*Σc_i²` and `(Σc_i)²` are formed in `Wide256`, subtracted exactly, and downcast only if the final numerator fits the bounded exact-rounder contract. Unsupported coordinate construction, integer accumulation, Wide256 subtraction/downcast, denominator, or exact-rounding cases fail closed to the established generic implementation.

Neutral zero is not synthetic evidence. It is a deterministic translation origin for the translation-invariant pair-distance identity; subtracting `0` from a finite binary64 residual reproduces that represented residual exactly. The source observations remain unchanged. The repair still does not prove that zero plus observed residuals is a globally resource-optimal anchor set for every representable geometry; it closes the demonstrated correctness gaps without turning an unproved optimization claim into admission policy.

Production sample admission remains `n=4..=16`.

## Resource budget remains unresolved

Exact pair records are 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2048`, and 4,997,541 at `n=3162`. For the older aligned nonnegative diameter characterization `D=2^53`, narrow checked products fit through `n=2047`, exact pair-numerator extremum through `n=4095`, and unreduced `n²(n-1)` stays at or below `2^53` through `n=208064`. GCD reduction and represented geometry mean none is a universal production cutoff.

The timing vehicle remains `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it records wide-product and pair-fallback selection separately. No Rust 1.98.0 `--release` CPU/raw CSV, allocator/RSS, or applicable buyer-path p95 evidence is authoritative yet.

## Decision

Keep production `validation_core::bias_standard_error` at `n=4..=16`. Accept neutral-zero-plus-observed-anchor exact proof inside that existing budget because `9f403194... -> 6cf30eeb...` repairs a demonstrated public one-ULP defect. Do not infer that a larger sample budget is safe or that pairwise authority can yet be removed.

Before widening beyond 16, require exact-head Rust 1.98.0 fmt/clippy/nextest/rustdoc and owned production 100% line/branch coverage, same-head security/documentation GREEN and qualifying independent review, pair/anchor equality wherever both admit, explicit anchor-only fixtures `[0,1,2^-54,2]`, `[0,1,2,-2^53]`, and `[1,2^-54,2,3]`, exact candidate stepping/midpoint/tie-to-even and permutation invariance, fail-closed overflow/range behavior, recorded release-mode raw CPU/allocator/RSS and applicable buyer-path p95 evidence, and current CHANGELOG/TRACEABILITY/TEST_STRATEGY/OPERABILITY/operator baseline.

## Traceability

- Issue: #491
- Landing PR: #488
- Accumulator-bound characterization: `b7e4da353ac58069afd73ee7c0e8427d49993fdb`
- Wide-product capacity RED / repair: `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` / `e9a7dee29afb97542bfe2965f850c8ab5a34368e`
- Represented-input Wide256 reachability: `5a19b6334487b43fb630abba7e487d7cf4c49960`
- Represented exact-midpoint width: `a8423173188fa53a26a16d3afdafeb76e114cc1d`
- Exponent-safe scaled comparison: `aab9fe9115cee97225f2aa81e54a55ceafb23336`
- Production scaled-comparison RED / repair: `f7717361ad8c5f0592688c1514c104cc1b4adabe` / `e4a85f53a611922be7492fe906d62ce65787c18e`
- Exact midpoint tie-to-even: `1240ace8eb41a01fa72a4bb99df842fd550a1288`
- Represented pair/Wide256 exact-ratio equivalence: `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943`
- Pairwise-f64-strict characterization: `2bc1d2284d75154e020640adb573c1cfadf005fb`
- Non-minimum represented-anchor RED / repair: `fd9f9ff2c5c395e4cc13042232f4deef018adb48` / `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f`
- Neutral-anchor RED / repair: `9f403194a2ec1636531c2dfe9229cfb34b73d747` / `6cf30eeb549c0df0377bda1111cf46396e8282a3`
- Narrow-wide-pair RED / repair: `3136739460ef0c8e13c044a7e5b04891e4f4e23d` / `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md`
- Production module: `crates/validation_core/src/bias_se.rs`
- Public regression: `crates/validation_core/tests/bias_standard_error_nonminimum_anchor_exact_rounding_contract.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
