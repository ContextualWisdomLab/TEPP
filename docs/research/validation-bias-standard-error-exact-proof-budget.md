# Bias-SE exact-proof budget characterization

## Scope

Issue #491 replaces the sample-count staircase in GAP-111 through GAP-125 with an evidence-based resource budget. This note records bounded characterization evidence. It does **not** widen production admission beyond `n=16`, does not claim buyer-path p95, and does not make a benchmark result from an unmeasured environment authoritative.

## New represented-input boundary evidence

The seventeen-observation residual multiset

`[38_557_579, 48_779_805, 63_558_649, 106_352_599, 139_863_777, 142_786_819, 267_163_239, 275_103_292, 375_678_558, 454_709_869, 484_300_224, 623_646_610, 989_643_121, 1_027_595_814, 1_520_220_488, 1_569_903_156, 1_805_452_085]`

is exactly representable in binary64. Its 136 squared pair distances sum to

`N = 92_549_865_125_191_410_206`.

For `n=17`, the scientific denominator is `17^2(17-1)=4_624`. `gcd(N,4_624)=2`, giving the reduced exact radicand

`46_274_932_562_595_705_103 / 2_312`.

The current public API intentionally remains on the established translated floating fallback at `n=17`; for this fixture it returns bits `0x41a0_dd77_9ac3_8e98`. Independent high-precision rational-square-root evaluation gives the adjacent correctly rounded binary64 target `0x41a0_dd77_9ac3_8e99`. This extends the demonstrated failure class beyond the current cutoff, but it is evidence for a systemic budget decision rather than justification for another one-count production patch.

`crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` fixes the exact pair-square sum, reduced ratio, current fallback boundary, algebraic equivalence, pair-record counts, distinct checked-integer envelopes, dyadic-unit normalization, and the admitted/refused-set relation between the current pair reference and the candidate O(n) accumulator. The test intentionally makes any future widening of production admission update this characterization rather than silently inheriting a stale fallback assumption.

## O(n²) reference versus O(n) exact accumulator

For exact dyadic coefficients `c_i` on one shared unit,

`sum_{i<j} (c_i-c_j)^2 = n * sum_i c_i^2 - (sum_i c_i)^2`.

The characterization test computes both sides with checked `u128` on the seventeen-observation fixture and requires exact equality to `N`. It also checks deterministic compact grids at `n=4,16,17,32,64,128,256`. Whenever the O(n) kernel admits those grids, it must equal the O(n²) pair numerator exactly.

The shared dyadic unit is part of the proof, not an optional optimization. After selecting an exact anchor, the candidate O(n) path must remove the largest common power-of-two factor from all nonzero anchor-relative coefficients before it judges checked-intermediate overflow. Otherwise the admission decision depends on an arbitrary integer scale rather than on the represented dyadic geometry.

The predecessor characterization violated that requirement. It used one coefficient at zero and the rest at `D=2^58`, but evaluated the O(n) intermediates on raw coefficients. At `n=65`, that raw representation makes `65*64*D^2` and `(64D)^2` overflow `u128`, while the pair numerator remains `64D^2 = 2^122`; the predecessor therefore labeled this geometry a linear refusal. That refusal is spurious. All nonzero coefficients share the exact unit `2^58`; after normalization the coefficients are one zero and sixty-four ones, the two O(n) intermediates are `4_160` and `4_096`, their difference is `64`, and restoring the squared unit yields the same `2^122` pair numerator. RED contract `4f1bd2c343cf2d54905a07c257a570a89dc575d3` exposes this scale-dependent refusal. Characterization repair `d423b57797b6f7f127e61e0679f9ee9841525c77` removes the common power-of-two unit before checked accumulation.

The corrected admission-set characterization still proves that the normalized checked-`u128` O(n) kernel is **sufficient but not admission-equivalent** to the pair reference; it simply requires a counterexample whose common dyadic unit cannot erase the problematic scale. Let `D=2^58+1`, an odd integer, and use one coefficient at zero with every remaining coefficient at `D`. Because `D` is odd, the canonical common dyadic unit is one. At `n=64`, both kernels fit. At `n=65`, the exact pair numerator is

`64 * (2^58 + 1)^2 = 5_316_911_983_139_663_528_508_716_388_540_481_600`,

which is a 123-bit `u128` value, while the first O(n) intermediate

`65 * 64 * (2^58 + 1)^2 = 345_599_278_904_078_129_353_066_565_255_131_304_000`

requires 129 bits and therefore refuses before cancellation. The pair reference can still prove the exact numerator. Linear refusal remains distinct from scientific refusal, but the evidence now survives canonical dyadic normalization.

This rules out a drop-in replacement of the current pair proof with the normalized checked-`u128` identity. A production O(n) path can preserve current scientific admission only as a sufficient fast path followed by the existing pairwise proof on refusal, or by adopting a wider checked-integer representation with its own resource and supply-chain evidence.

## Checked-u128 envelopes

Two different `u128` bounds must not be conflated.

For a **canonical aligned coefficient diameter** `D` after common power-of-two normalization, both `n * sum(c_i^2)` and `(sum c_i)^2` are bounded above by `n^2 D^2`. With aligned `D = 2^53`, that distribution-independent sufficient intermediate envelope fits through `n=2_047` and reaches the unrepresentable `2^128` boundary at `n=2_048`. This is an envelope on the normalized coefficient grid, not a claim that a raw represented diameter containing a common dyadic factor must be judged at that raw scale.

The exact pair-square numerator itself has the tighter extremal bound `floor(n^2/4) D^2`, attained by placing the aligned coefficients at the two diameter endpoints as evenly as possible. At the same aligned `D = 2^53`, that final exact numerator can still fit `u128` through `n=4_095` and crosses the `2^128` boundary at `n=4_096`. Therefore `2_047` is not an intrinsic exact-pair numerator ceiling; it is only the conservative all-distribution envelope of the normalized O(n) intermediates.

The scientific denominator `n^2(n-1)` is a separate bounded-proof input. Its unreduced value remains at or below the exact binary64-integer bound `2^53` through `n=208_064` and exceeds it at `n=208_065`. The production midpoint proof checks the **reduced** denominator after GCD reduction, so this unreduced threshold is a sufficient envelope marker rather than a universal refusal count.

None of these arithmetic thresholds is a production sample-count budget. Production still needs release-mode CPU and allocation evidence and, where the estimator appears on an HTTP buyer path, applicable end-to-end p95 evidence against the TEPP `<=20 ms` target.

## Allocation finding

The current bounded O(n²) implementation stores every pair as `Option<(u128, i32)>` before the second accumulation pass. The number of stored pair records is exactly `n(n-1)/2`: 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and **4,997,541** at `n=3,162`. The predecessor note incorrectly recorded the last count as 4,997,500; the Rust characterization now locks the exact integer count.

Exact byte cost is target-layout dependent and must not be inferred by adding field widths. The measurement harness obtains `size_of::<Option<(u128, i32)>>()` on the executing target, records the actual `Vec` element capacity after `with_capacity`, and reports their product as scratch payload bytes. Allocator bookkeeping and whole-process RSS are still outside that number and must be recorded separately if they become release evidence.

A two-pass O(n²) reference can remove the pair-record allocation without changing the pair-enumeration proof shape: the first pass establishes the common dyadic unit, and the second recomputes each pair record and accumulates the checked square. The harness measures this allocation-free quadratic alternative alongside the buffered pair layout. The normalized O(n) identity removes pair enumeration as well, but the admission-set characterization shows that its checked-`u128` form must remain a sufficient fast path with pairwise fallback unless wider intermediates are justified.

## Measurement harness

`crates/validation_core/examples/bias_se_exact_proof_budget.rs` is a standard-library-only release-mode characterization harness. It compares four kernels:

- `quadratic_buffered`: production-layout-shaped `Vec<Option<(u128, i32)>>` pair records plus aligned checked-square accumulation;
- `quadratic_two_pass`: the same pair enumeration and dyadic alignment without pair-record storage;
- `linear`: minimum-anchor coefficients, normalized by their largest shared power-of-two unit, then `n*sum(c_i^2) - (sum c_i)^2`;
- `hybrid`: the viable resource shape, using the normalized checked O(n) accumulator when it admits and otherwise falling back to the production-layout-shaped buffered pair proof.

Before timing, the harness restores the dyadic unit and requires every applicable kernel to equal the same exact pair-square numerator. Compact deterministic fixtures at 16, 64, 256, 1,024, and 2,047 observations exercise ordinary admitting geometry. Three boundary geometries separate normalization from true refusal:

1. `power_of_two_normalized_admit`: `n=65`, `D=2^58`. The linear and hybrid paths must factor the common dyadic unit and admit; pair fallback here would reproduce the predecessor characterization defect.
2. `odd_boundary_admit`: `n=64`, `D=2^58+1`. With no removable common dyadic factor, normalized O(n) still fits.
3. `odd_boundary_pair_fallback`: `n=65`, `D=2^58+1`. The exact pair numerator fits, normalized O(n) checked intermediates refuse, and the hybrid must execute the buffered pair fallback.

The CSV schema remains

`geometry,sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes,used_pairwise_fallback`.

`unit_exponent` is material evidence in the corrected harness: it distinguishes the common-power geometry, where the linear path reports exponent 58 and a small aligned numerator, from the odd-diameter geometry, where the canonical unit exponent is zero. `used_pairwise_fallback` records whether a hybrid row actually exercised the expensive proof path instead of inferring that fact from sample count.

The original hybrid harness landed in `c6b237e0bb1388cccd7bcb71a0df5cbf837a07c5`; the dyadic-normalization correction is `4a3d988702593b2b8d59be6dcfb1601ca1a0d610`. It remains measurement tooling only. No release-mode timing result is recorded in this document yet because the current execution environment does not provide Rust 1.98.0 and the hosted exact-head jobs have not produced measurement artifacts. A valid timing record must include CPU, OS, Rust toolchain, exact commit, release build mode, raw sample count, raw CSV, allocator/RSS evidence, and the cold/warm procedure. Kernel timing cannot substitute for an applicable API buyer-path p95 measurement.

## Decision and rejected alternatives

Production admission stays `n=4..=16`. Extending to `n=17` alone is rejected because GAP-111 through GAP-125 plus the seventeen-observation evidence show that the integer cutoff is not a scientific boundary. Removing the cutoff entirely is rejected because the current production implementation still enumerates and stores O(n²) pair evidence.

The former `D=2^58, n=65` raw-scale refusal is explicitly rejected as admission evidence because it disappears under the same common-power dyadic normalization already required by the proposed O(n) proof. Retaining it would make the resource budget representation-dependent. The odd `D=2^58+1` boundary replaces it as the normalized refusal fixture.

Treating `n<=2_047`, `n<=4_095`, or the unreduced denominator threshold as the production budget is also rejected because arithmetic representability is not latency or memory evidence. A two-pass O(n²) allocation-removal path remains a candidate because it can preserve the current pair-proof admission shape while eliminating pair-record storage. The normalized O(n) identity remains the stronger CPU-scaling candidate, but its checked-`u128` form is still a strict sufficient subset of the pair reference. Replacing the pair proof with that kernel alone is rejected because it would silently narrow exact-proof admission. The hybrid harness measures the viable bounded shape—normalized O(n) fast admission with buffered O(n²) fallback—without making it production behavior. An admission-equivalent wider-integer O(n) proof or the allocation-free two-pass pair reference remain alternatives if measurements justify them. Arbitrary-precision production arithmetic remains deferred pending measured benefit, supply-chain review, and an explicit owner/resource decision.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Predecessor scientific repair | GAP-125; RED `5da82b2d651706c191ca191c6c077d916cbfda25`; repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Characterization RED | `4f1bd2c343cf2d54905a07c257a570a89dc575d3` — common `2^58` scale must normalize before O(n) overflow admission |
| Characterization repair | `d423b57797b6f7f127e61e0679f9ee9841525c77` — normalized test kernel plus odd-diameter refusal fixture |
| Harness repair | `4a3d988702593b2b8d59be6dcfb1601ca1a0d610` — normalized O(n)/hybrid measurement geometries |
| Current production module | `crates/validation_core/src/bias_se.rs` |
| Public API | `validation_core::bias_standard_error` |
| Exact characterization | `crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` |
| CPU/layout/hybrid harness | `crates/validation_core/examples/bias_se_exact_proof_budget.rs` |
| CHANGELOG evidence | `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md` |
| Resource/merge rule | Production cutoff remains `n<=16` pending measured exact-proof budget |

## Follow-up evidence required by #491

Run the corrected hybrid-capable harness in release mode on a recorded CPU/toolchain and retain raw timing CSV. Record allocator/RSS evidence in addition to the harness's exact pair-record payload layout. Evaluate a wider-integer/reference alternative without making it production authority. Only after those results establish a resource budget should production admission change; any such change needs a realistic public RED, exact-head Rust/rustdoc/coverage evidence, and applicable buyer-path p95 evidence.
