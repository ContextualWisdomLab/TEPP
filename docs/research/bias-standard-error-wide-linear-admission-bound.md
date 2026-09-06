# Bias standard-error wide-linear admission bound

## Problem

Issue #491 originally treated the production `n=4..=16` exact bias-standard-error proof as a sample-count staircase. The accumulated evidence separates three questions: whether represented data admit an exact proof, whether the arithmetic representation is wide enough to carry that proof, and whether the resource cost is acceptable for production. None is resolved by incrementing `n` alone.

An earlier characterization subtracted the represented minimum and produced nonnegative integer coefficients `c_i` on a common exact dyadic unit. For that historical representation, `P = sum_{i<j}(c_i-c_j)^2`, `S1 = sum_i c_i`, and `S2 = sum_i c_i^2` satisfy `S1 <= S2 <= P` because at least one coefficient is zero. That theorem remains useful for its bounded input family, but it is not the production admission rule: production now uses signed coordinates around neutral zero and computes `P = n*S2-S1^2` with separate positive and negative coefficient mass.

## Full-width product and rounding bounds

Odd diameter `D=2^58+1,n=65` is the canonical narrow-width witness: both `n*S2` and `S1^2` require 129 bits while the exact pair numerator requires 123 bits. RED `f74d9ac11cb0acf3eb8fdd9ad79ac3d2e9180993` → repair `e9a7dee29afb97542bfe2965f850c8ab5a34368e` makes the two-limb capacity theorem executable. RED `3136739460ef0c8e13c044a7e5b04891e4f4e23d` → repair `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5` adds measured `narrow O(n) -> Wide256 O(n) -> pair` route telemetry to the characterization harness.

Represented-input reachability `5a19b6334487b43fb630abba7e487d7cf4c49960` reaches the wider numerator route at `n=4096` on `{0,1,2^53}` and recovers `P=664289479338799435974172876300357631`. At represented `n=2050`, characterization `a8423173188fa53a26a16d3afdafeb76e114cc1d` finds `P=332306998946228931332463617650984961`, denominator `8_610_922_500`, but 136-bit candidate-square and 140-bit adjacent-midpoint comparison operands. `aab9fe9115cee97225f2aa81e54a55ceafb23336` shows those comparisons can remain bounded as `Wide256` mantissa plus signed dyadic exponent. RED `f7717361ad8c5f0592688c1514c104cc1b4adabe` → repair `e4a85f53a611922be7492fe906d62ce65787c18e` integrates that comparator into production; `1240ace8eb41a01fa72a4bb99df842fd550a1288` fixes both exact tie-to-even parity directions.

IEEE 754-2019 and ISO/IEC 60559:2020 remain the published floating-point basis. Rechecked on 2026-09-07, ISO lists ISO/IEC 60559:2020 at published stage 60.60, while IEEE lists P754 as an Active PAR approved 2024-06-06 that supersedes 754-2019 when completed; P754 is therefore a revision project, not a published replacement.

## Represented proof equivalence and admission

Historical characterization `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943` compared an O(n²) pair authority with a minimum-anchor Wide256 O(n) identity for represented residual classes `{0,1,2^53}` at `n=4,16,17,65,257,2050`. Wherever both admitted, they produced the same common unit, exact pair numerator, and reduced ratio. That helper ceased to represent production after the conditioned-anchor route was removed.

Characterization `2bc1d2284d75154e020640adb573c1cfadf005fb` shows why pairwise-f64 exactness cannot define represented-input admission. Residuals `[0,1,2^-54,2]` have exact coordinates from neutral zero, while non-anchor subtraction `1-2^-54` rounds. On common unit `2^-54`, coordinates `[0,2^54,1,2^55]` give `P=3569704090242693886528325169446915`. The generic fallback happens to return the same public result, so this fixture is an admission distinction rather than a public defect.

Source RED `fd9f9ff2c5c395e4cc13042232f4deef018adb48` then shows a public defect with `[0,1,2,-2^53]`. The represented minimum `-2^53` cannot exactly translate `1`, but neutral zero preserves every represented residual exactly. Signed unit-one coordinates give `P=243388915243820099130562543878155`, denominator `48`, and correctly rounded result `0x4320000000000001`; the predecessor translated floating-moment fallback returns `0x4320000000000000`. The RED Actions runs were cancelled by the immediate successor and are not hosted RED evidence.

Initial repair `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f` searched every represented residual as a candidate exact anchor. Follow-up RED `9f403194a2ec1636531c2dfe9229cfb34b73d747` proved that set incomplete: with represented residuals `[1,2^-54,2,3]`, no observed residual is a universal exact anchor, while neutral zero preserves every represented residual exactly. On common unit `2^-54`, the coordinates are `[2^54,1,2^55,3*2^54]` and `P=6490371073168534319490338297741315`; correctly rounded public bits are `0x3fe4a7e9cb8a3491`, while the predecessor translated floating-moment fallback returned adjacent upper `0x3fe4a7e9cb8a3492`.

Repair `6cf30eeb549c0df0377bda1111cf46396e8282a3` therefore expanded the candidate set to neutral zero plus represented residual anchors. That fixed the demonstrated correctness defects but still evaluated pairwise proof first and then scanned the sample for every candidate, leaving production O(n²).

Neutral zero is not synthetic evidence. It is a deterministic coordinate origin for the translation-invariant pair-distance identity; subtracting `0` from a finite binary64 residual reproduces that represented residual exactly. The source observations remain unchanged.

## Production route repair

The production resource defect is repaired on active PR #488 without widening the sample budget.

Source-level RED `e0b324864e48a503e2aba0d2a487a0b95f5276ed` required a linear neutral-zero proof before quadratic work. Repair `2b62bd46eb0c391327d2285c2244a76f5a1e0449` added `exact_neutral_zero_linear_pair_square_sum`. It scans represented residuals once to select the common dyadic exponent and a second time to accumulate positive/negative integer coefficient mass and `S2`. It then evaluates `n*S2-S1^2` with exact `Wide256` products and subtraction. The proof kernel is O(n) time and O(1) proof storage after the residual vector and allocates no pair records.

A later retention criterion required the conditioned observed-anchor fallback to demonstrate unique bounded admission after neutral-zero refusal. The checked-in production and unit-test corpus supplied no such represented fixture. Source-level RED `d40bfbf98b36562164d15d505f8f8825fd1c1349` rejected continued production reliance on the unsupported route, and repair `14e7862f4ddccce54f3b93d4dac89adbf047ba77` removed `exact_anchor_linear_pair_square_sum`. Production order is now `neutral_zero_linear -> pairwise_reference -> generic_fallback`. This is a consolidation decision under current evidence, not a theorem that pairwise is globally redundant.

Commit `da703fc50e136ed79e12262909c9f400a3945621` repairs the remaining equivalence characterization so it mirrors the production signed neutral-zero arithmetic rather than the removed minimum-anchor helper. For the common represented family `{0,1,2^53}`, it retains exact numerator/reduced-ratio equality against pairwise at `n=4,16,17,65,257,2050` and order invariance. It also adds the mixed-sign represented geometry `[-2^53,0,1,2^53]`: pairwise subtraction correctly refuses the inexact `-2^53` versus `+1` difference, while neutral-zero coordinates admit unit exponent `0`, `S1` magnitude `1`, `S2=2^107+1`, `P=2^109+3`, and the public exact route rounds to `0x432a20bd700c2c3e`. This is production-route admission evidence, not permission to widen `n`.

Production sample admission remains `n=4..=16`.

## Resource budget remains unresolved

Exact pair records are 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2048`, and 4,997,541 at `n=3162`. For the older aligned nonnegative diameter characterization `D=2^53`, narrow checked products fit through `n=2047`, exact pair-numerator extremum through `n=4095`, and unreduced `n²(n-1)` stays at or below `2^53` through `n=208064`. GCD reduction and represented geometry mean none is a universal production cutoff.

The timing vehicle remains `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; it records characterization routes rather than authoritative production execution. No Rust 1.98.0 `--release` raw CPU timing, allocator/RSS, or applicable buyer-path p95 evidence is authoritative yet. Production route telemetry must distinguish `neutral_zero_linear`, `pairwise_reference`, and `generic_fallback`; identical output bits are not route evidence.

## Decision

Keep production `validation_core::bias_standard_error` at `n=4..=16`. Use the neutral-zero two-pass exact proof before quadratic proof work inside that budget, keep pairwise O(n²) as the fail-closed comparison/reference path, and fall back to the established general implementation when bounded exact proof refuses. Do not reintroduce observed-anchor scanning without a represented fixture that proves unique scientific admission value.

Before widening beyond 16, require exact-head Rust 1.98.0 fmt/clippy/nextest/rustdoc and owned-production 100% line/branch coverage, same-head security/documentation GREEN and qualifying independent review, broad pair/neutral-zero equality wherever both admit, explicit neutral-zero-only represented fixtures, exact candidate stepping/midpoint/tie-to-even and permutation invariance, fail-closed overflow/range behavior, truthful production route telemetry, recorded release-mode raw CPU/allocator/RSS and applicable buyer-path p95 evidence, and current CHANGELOG/TRACEABILITY/TEST_STRATEGY/OPERABILITY/operator baseline.

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
- Historical minimum-anchor pair/Wide256 equivalence: `7a2ab0a1ef7a72d8cc9b9253d7f92c493e578943`
- Pairwise-f64-strict characterization: `2bc1d2284d75154e020640adb573c1cfadf005fb`
- Non-minimum represented-anchor RED / repair: `fd9f9ff2c5c395e4cc13042232f4deef018adb48` / `81ba770cc4812c8fbeb4b3529f0a73b41abbed0f`
- Neutral-anchor RED / correctness repair: `9f403194a2ec1636531c2dfe9229cfb34b73d747` / `6cf30eeb549c0df0377bda1111cf46396e8282a3`
- Neutral-zero route-order RED / production resource repair: `e0b324864e48a503e2aba0d2a487a0b95f5276ed` / `2b62bd46eb0c391327d2285c2244a76f5a1e0449`
- Unsupported conditioned-anchor retention RED / removal: `d40bfbf98b36562164d15d505f8f8825fd1c1349` / `14e7862f4ddccce54f3b93d4dac89adbf047ba77`
- Production-aligned neutral-zero equivalence and mixed-sign admission: `da703fc50e136ed79e12262909c9f400a3945621`
- Narrow-wide-pair characterization RED / repair: `3136739460ef0c8e13c044a7e5b04891e4f4e23d` / `ce4ed2722e160eb0ca0ee2d636a5eca55e3ff2d5`
- CHANGELOG fragment: `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md`
- Production module: `crates/validation_core/src/bias_se.rs`
- Route-order contract: `crates/validation_core/tests/bias_standard_error_neutral_zero_route_order_contract.rs`
- Production-aligned equivalence characterization: `crates/validation_core/tests/bias_standard_error_represented_route_equivalence_characterization.rs`
- Public regression: `crates/validation_core/tests/bias_standard_error_nonminimum_anchor_exact_rounding_contract.rs`
- Exact-proof budget harness: `crates/validation_core/examples/bias_se_exact_proof_budget.rs`
