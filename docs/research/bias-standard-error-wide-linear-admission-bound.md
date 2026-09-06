# Bias standard-error wide-linear admission bound

## Problem

Issue #491 originally treated the production `n=4..=16` exact bias-standard-error proof as a sample-count staircase. The accumulated evidence shows that three different questions must remain separate: whether represented data admit an exact proof, whether the arithmetic representation is wide enough to carry that proof, and whether the resource cost is acceptable for production. None is resolved by incrementing `n` alone.

For an earlier characterization that subtracts the represented minimum and produces nonnegative integer coefficients `c_i` on a common exact dyadic unit, define

`P = sum_{i<j}(c_i-c_j)^2`, `S1 = sum_i c_i`, and `S2 = sum_i c_i^2`.

Because at least one coefficient is zero and every coefficient is a nonnegative integer, `S1 <= S2 <= P`. Thus a pair-admitted `P <= u128::MAX` cannot fail a wider O(n) reference solely because `S1` or `S2` overflowed. This theorem is scoped to that nonnegative minimum-anchor representation; the production exact-anchor repair below may use signed coordinates because the represented minimum is not always an exact universal anchor.

## Full-width product bound

The narrow O(n) identity can still refuse while `P` fits because `n*S2` and `S1^2` may exceed 128 bits before cancellation. Odd diameter `D=2^58+1,n=65` is the canonical arithmetic witness: both products require 129 bits while the exact pair numerator requires 123 bits.

RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` makes the capacity theorem executable. With `S1,S2,n` each bounded by `u128`, every cancellation product fits below `2^256`; dependency-free two-limb `Wide256` is therefore sufficient for the product width in the characterized domain. This theorem does not by itself prove end-to-end production admission.

RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` → repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` adds a measured characterization route `narrow O(n) -> Wide256 O(n) -> buffered pair fail-closed fallback` and records `used_wide_product` separately from `used_pairwise_fallback`. A correct numerator alone does not establish which resource path ran.

## Represented-input reachability

Characterization `5a19b6334487b43fb630abba7e487d7cf4c49960` makes Wide256 recovery reachable from represented binary64 inputs rather than synthetic integer-only coefficients. At `n=4096`, represented residual classes `{0,1,2^53}` have exact residual construction and exact distinct pair differences. The common dyadic shift is zero, narrow products overflow, and Wide256 recovers

`P = 664289479338799435974172876300357631`.

The unreduced denominator is `68_702_699_520`. This establishes reachability, not production admission.

## Exact-rounding width is a separate boundary

At represented `n=2050`, characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d` reaches a second boundary. The exact pair numerator is

`P = 332306998946228931332463617650984961`,

with denominator `8_610_922_500`. The normal binary64 ratio/square-root seed is `0x4296998e1aff78de`; exact candidate-square comparison needs 136-bit operands and the adjacent upward midpoint comparison needs 140-bit operands.

Characterization `aab9fe9115cee97225f2aa81e54a55ceafb23336` shows that arbitrary precision is unnecessary for these comparisons: represent each nonzero operand as a `Wide256` mantissa plus signed dyadic exponent, compare absolute top-bit positions, then aligned significand bits only if the top positions tie. It also covers extreme exponent metadata `-2148/-2149` and `2046/2047` without materializing `2^k`.

RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` → repair `e4a85f53a611922be7492fe906d62ce65787c18e` moves the signed-exponent two-limb comparison into `crates/validation_core/src/bias_se.rs`. `1240ace8eb41a01fa72a4bb99df842fd550a1288` fixes both exact-midpoint tie-to-even parity directions. The binary64 division/sqrt remains only a candidate seed; the returned value is authorized by the exact candidate/midpoint comparison.

The standards basis remains IEEE 754-2019 and ISO/IEC 60559:2020. IEEE P754 is an active revision project, not a published replacement.

## Pair versus Wide256 represented equivalence

Characterization `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` compares an actual two-pass O(n²) pairwise authority with an independent Wide256 O(n) identity for represented residual classes `{0,1,2^53}` at `n=4,16,17,65,257,2050`. Wherever both admit, they must produce the same common unit exponent, exact pair numerator, and GCD-reduced `(numerator, denominator, unit_exponent)` tuple presented to the exact rounder. At `n=2050`, both narrow products overflow while the two exact routes agree on the 118-bit `P` above.

That test originally used the represented minimum as anchor because its characterized input family made the minimum exact. It is not a universal production anchor rule.

## Pairwise-f64 admission is broader than neither science nor exact-anchor admission

Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` isolates a distinction inside the existing `n=4` budget. Residuals `[0,1,2^-54,2]` have exact minimum-anchor coordinates, but non-anchor subtraction `1-2^-54` rounds in binary64. The O(n²) pairwise-f64 proof therefore refuses. On common unit `2^-54`, integer coordinates `[0,2^54,1,2^55]` give

`P = 3569704090242693886528325169446915`

through both direct integer pair distances and `n*S2-S1^2`. The generic translated fallback happens to return the same correctly represented public result `0x3fdea33e2c83c140`. This showed that exact non-anchor pair subtraction is a sufficient reference-path condition, not a scientific prerequisite.

## Non-minimum exact anchor is a public correctness requirement

A stronger represented fixture found after that characterization proves that selecting the represented minimum as a mandatory anchor is also too strict. Fix truth at represented zero and use residuals

`[0,1,2,-2^53]`.

The represented minimum is `-2^53`. Mathematical difference `1-(-2^53)=2^53+1` is not representable in binary64, so neither the pairwise-f64 proof nor a minimum-anchor-only linear proof can preserve this geometry exactly. Anchor `0`, however, gives exact signed coordinates `[0,1,2,-2^53]` on unit `1`.

Their translation-invariant exact pair numerator is

`P = 243388915243820099130562543878155`,

so `SE(mean)^2 = P/48`. Correct binary64 rounding is `0x4320000000000001`. The predecessor translated floating-moment fallback returns the adjacent lower `0x4320000000000000`; this is a public one-ULP defect rather than only a resource/admission observation.

Source-level RED `fd9f9ff2c5c395e4cc13042232f4deef018adb48` adds forward and reversed public contracts for `0x4320000000000001`. Its Actions runs were cancelled by the immediate successor push, so it is not claimed as hosted RED evidence.

Production repair `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f` keeps exact pairwise accumulation as the first authority. When pair subtraction cannot be proven exact, it searches every represented residual as a candidate translation anchor, requires every anchor-relative coordinate to be error-free, chooses the candidate with the smallest exact maximum translated magnitude and a represented-value tie-break, and converts the signed translated coordinates to a common dyadic grid. Positive and negative coefficient mass are accumulated separately; `n*Σc_i²` and `(Σc_i)²` are formed in `Wide256`, subtracted exactly, and downcast only if the final numerator fits the existing bounded `u128` rounder contract. The same exact candidate/midpoint/tie-to-even rounder then authorizes the result.

This anchor policy intentionally mirrors the permutation-invariant principle already used by `bias.rs`: observation arrival order is not scientific evidence, and the represented minimum is not privileged when it cannot translate the geometry exactly. Unsupported coordinate accumulation, Wide256 subtraction/downcast, denominator, or exact-rounding cases continue to fail closed to the established generic implementation.

The repair also promotes `[0,1,2^-54,2]` from generic fallback to exact anchor admission. It does **not** widen the production sample-count budget: `exact_pair_distance_standard_error` remains bounded to `n=4..=16`.

## Resource budget remains unresolved

Exact pair records are 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2048`, and 4,997,541 at `n=3162`. For the older aligned nonnegative diameter characterization `D=2^53`, narrow checked products fit through `n=2047`, the exact pair-numerator extremum fits through `n=4095`, and unreduced `n²(n-1)` stays at or below `2^53` through `n=208064`. Because production reduces the denominator by GCD and represented geometry varies, none is a universal refusal count or production budget.

The production anchor repair changes the question from “can O(n) replace O(n²)?” to “what exact represented geometries can be admitted deterministically and at what measured cost?” Same-domain pair equivalence, intended anchor-only admission, exact rounder behavior, and fail-closed refusal all need to survive on one current head before any pair fallback can be demoted or the sample cutoff can move.

The timing/layout vehicle remains `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it records wide-product and pair-fallback selection separately. No Rust 1.98.0 `--release` CPU/raw CSV, allocator/RSS, or applicable buyer-path p95 evidence is claimed yet.

## Decision

Keep production `validation_core::bias_standard_error` admission at `n=4..=16`. Accept the deterministic exact-anchor/Wide256 repair within that existing budget because the non-minimum-anchor fixture is a public one-ULP correctness defect, not a speculative performance optimization. Do not infer from that repair that a larger sample budget is safe.

Before any budget change beyond 16, require: exact-head Rust 1.98.0 fmt/clippy/nextest/rustdoc and owned production 100% line/branch coverage; same-head security/documentation GREEN and qualifying independent review; pair/anchor equality wherever both admit; explicit intended anchor-only admissions including `[0,1,2^-54,2]` and `[0,1,2,-2^53]`; exact candidate stepping/midpoint/tie-to-even and permutation invariance; fail-closed overflow/range behavior; recorded release-mode raw CPU/allocator/RSS and applicable buyer-path p95 evidence; and current CHANGELOG/TRACEABILITY/TEST_STRATEGY/OPERABILITY/operator baseline.

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
- Exact midpoint tie-to-even edge contract: `1240ace8eb41a01fa72a4bb99df842fd550a1288`
- Represented pair/Wide256 exact-ratio equivalence: `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943`
- Anchor-linear-only represented admission characterization: `2bc1d2284d75154e020640adb573c1cfadf005fb`
- Non-minimum-anchor public RED: `fd9f9ff2c5c395e4cc13042232f4deef018adb48`
- Deterministic exact-anchor production repair: `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f`
- Narrow-wide-pair RED: `3136739460ef0c8e13c044a7e5b04891e4f4e23d`
- Narrow-wide-pair repair: `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md`
- Production module: `crates/validation_core/src/bias_se.rs`
- Public regression: `crates/validation_core/tests/bias_standard_error_nonminimum_anchor_exact_rounding_contract.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
