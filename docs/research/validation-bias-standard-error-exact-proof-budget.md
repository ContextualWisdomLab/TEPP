# Bias-SE exact-proof budget characterization

## Scope

Issue #491 replaces the sample-count staircase in GAP-111 through GAP-125 with an evidence-based resource budget. This note records bounded characterization and production-route evidence. The current implementation attempts the checked O(n) neutral-zero proof for every `n >= 3`; only the O(n²) pairwise reference remains bounded to `n <= 16` after O(n) refusal. This note does not claim buyer-path p95 or turn one benchmark environment into a universal resource guarantee.

## New represented-input boundary evidence

The seventeen-observation residual multiset

`[38_557_579, 48_779_805, 63_558_649, 106_352_599, 139_863_777, 142_786_819, 267_163_239, 275_103_292, 375_678_558, 454_709_869, 484_300_224, 623_646_610, 989_643_121, 1_027_595_814, 1_520_220_488, 1_569_903_156, 1_805_452_085]`

is exactly representable in binary64. Its 136 squared pair distances sum to

`N = 92_549_865_125_191_410_206`.

For `n=17`, the scientific denominator is `17^2(17-1)=4_624`. `gcd(N,4_624)=2`, giving the reduced exact radicand

`46_274_932_562_595_705_103 / 2_312`.

The predecessor public route returned translated-floating-fallback bits `0x41a0_dd77_9ac3_8e98` for this fixture. Parent `6f0063d3f0ebba807bbde8763a5238cd0306702e` independently brackets the exact target between the adjacent binary64 midpoint squares and establishes the correctly rounded public result `0x41a0_dd77_9ac3_8e99`. The current source therefore demonstrates that the O(n) proof route is materially active at `n=17`; this is evidence against a sample-count staircase, not permission to weaken checked proof admission.

`crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` fixes the exact pair-square sum, reduced ratio, algebraic equivalence, pair-record counts, distinct checked-integer envelopes, dyadic-unit normalization, the admitted/refused-set relation between pair and O(n) references, and a dependency-free wider-product reference for narrow-intermediate refusal. Current public-route contracts separately verify exact admission above sixteen and deterministic delegation when the O(n) proof refuses.

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

requires 129 bits and therefore refuses before cancellation. The companion square `(64D)^2 = 340_282_366_920_938_465_824_557_848_866_590_822_400` also requires 129 bits. Their exact difference is the 123-bit pair numerator above. Linear refusal remains distinct from scientific refusal, but the evidence now survives canonical dyadic normalization.

Commit `081000289f5a52e94863026d55696ee2a4daf923` adds a dependency-free `Wide256` characterization reference using four 64-bit limbs for exact `u128 × u128` products, checked two-limb subtraction, and checked dyadic restoration. It proves two separate facts: `(2^128-1)^2` is represented exactly as high limb `2^128-2` and low limb `1`, and the odd `D=2^58+1,n=65` geometry that the narrow O(n) kernel refuses is recovered exactly after 129-bit intermediate cancellation. The wider reference deliberately keeps coefficient and square accumulation in checked `u128`; it is therefore a bounded intermediate-width experiment, not arbitrary precision and not a general admission theorem.

Production uses the checked neutral-zero O(n) identity as a sufficient proof path for every `n >= 3`. It preserves bounded pairwise comparison/reference only when `n <= 16` and the O(n) proof refuses. Above sixteen, O(n) refusal delegates to the established general bias implementation without pair enumeration. This keeps resource behavior bounded without pretending that one proof representation covers every scientifically valid represented-input geometry.

## Checked-u128 envelopes

Two different `u128` bounds must not be conflated.

For a **canonical aligned coefficient diameter** `D` after common power-of-two normalization, both `n * sum(c_i^2)` and `(sum c_i)^2` are bounded above by `n^2 D^2`. With aligned `D = 2^53`, that distribution-independent sufficient intermediate envelope fits through `n=2_047` and reaches the unrepresentable `2^128` boundary at `n=2_048`. This is an envelope on the normalized coefficient grid, not a claim that a raw represented diameter containing a common dyadic factor must be judged at that raw scale.

The exact pair-square numerator itself has the tighter extremal bound `floor(n^2/4) D^2`, attained by placing the aligned coefficients at the two diameter endpoints as evenly as possible. At the same aligned `D = 2^53`, that final exact numerator can still fit `u128` through `n=4_095` and crosses the `2^128` boundary at `n=4_096`. Therefore `2_047` is not an intrinsic exact-pair numerator ceiling; it is only the conservative all-distribution envelope of the normalized O(n) intermediates.

The scientific denominator `n^2(n-1)` is a separate bounded-proof input. Its unreduced value remains at or below the exact binary64-integer bound `2^53` through `n=208_064` and exceeds it at `n=208_065`. The production midpoint proof checks the **reduced** denominator after GCD reduction, so this unreduced threshold is a sufficient envelope marker rather than a universal refusal count.

None of these arithmetic thresholds is a production sample-count budget. Production still needs bounded CPU and allocation evidence across its supported represented-input domain and, where the estimator appears on an HTTP buyer path, applicable end-to-end p95 evidence against the TEPP `<=20 ms` target.

## Allocation finding

The bounded O(n²) reference stores every pair as `Option<(u128, i32)>` before the second accumulation pass. The number of stored pair records is exactly `n(n-1)/2`: 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and **4,997,541** at `n=3,162`. Counts above sixteen characterize the reference implementation; the current public route does not allocate this pair buffer above sixteen because O(n) refusal returns before the pairwise call.

Exact byte cost is target-layout dependent and must not be inferred by adding field widths. The measurement harness obtains `size_of::<Option<(u128, i32)>>()` on the executing target, records the actual `Vec` element capacity after `with_capacity`, and reports their product as scratch payload bytes. Allocator bookkeeping and whole-process RSS are measured separately when used as resource evidence.

A two-pass O(n²) reference can remove the pair-record allocation without changing the pair-enumeration proof shape: the first pass establishes the common dyadic unit, and the second recomputes each pair record and accumulates the checked square. That remains characterization work rather than a reason to invoke quadratic proof above sixteen.

## Measurement harness

`crates/validation_core/examples/bias_se_exact_proof_budget.rs` is a standard-library-only release-mode characterization harness. It compares five kernels:

- `quadratic_buffered`: production-layout-shaped `Vec<Option<(u128, i32)>>` pair records plus aligned checked-square accumulation;
- `quadratic_two_pass`: the same pair enumeration and dyadic alignment without pair-record storage;
- `linear`: minimum-anchor coefficients, normalized by their largest shared power-of-two unit, then checked-`u128` `n*sum(c_i^2) - (sum c_i)^2`;
- `linear_wide_product_reference`: the same normalized coefficient/square accumulators, but the two final products and cancellation use dependency-free two-limb 256-bit arithmetic before the exact result is required to fit the existing `u128` pair-numerator domain;
- `hybrid`: a characterization shape using the narrow normalized checked O(n) accumulator when it admits and otherwise falling back to the buffered pair proof.

Before timing, the harness restores the dyadic unit and requires every applicable kernel to equal the same exact pair-square numerator. Compact deterministic fixtures at 16, 64, 256, 1,024, and 2,047 observations exercise ordinary admitting geometry. Three boundary geometries separate normalization from true narrow-intermediate refusal:

1. `power_of_two_normalized_admit`: `n=65`, `D=2^58`. The narrow linear, wider-product reference, and hybrid paths must factor the common dyadic unit and admit; pair fallback here would reproduce the predecessor characterization defect.
2. `odd_boundary_admit`: `n=64`, `D=2^58+1`. With no removable common dyadic factor, normalized narrow O(n) still fits.
3. `odd_boundary_pair_fallback`: `n=65`, `D=2^58+1`. The exact pair numerator fits, normalized narrow O(n) checked intermediates refuse, the wider-product reference still recovers the exact numerator, and the characterization hybrid executes the buffered pair fallback.

The CSV schema remains

`geometry,sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes,used_pairwise_fallback`.

`unit_exponent` is material evidence in the corrected harness: it distinguishes the common-power geometry, where the linear path reports exponent 58 and a small aligned numerator, from the odd-diameter geometry, where the canonical unit exponent is zero. `used_pairwise_fallback` records whether a hybrid row actually exercised the expensive proof path instead of inferring that fact from sample count. The `kernel` field makes the wider-product reference directly comparable to the narrow O(n), pair, and hybrid timing rows in the same release-mode run.

The original hybrid harness landed in `c6b237e0bb1388cccd7bcb71a0df5cbf837a07c5`; the dyadic-normalization correction is `4a3d988702593b2b8d59be6dcfb1601ca1a0d610`; the wider-product measurement extension is `0bd805d4b0304cf1f76344ae14b7f079b3dade17`. Current isolated `n=2,047`, odd-dyadic resource evidence measures candidate and established fallback in separate `/usr/bin/time -v` processes on identical geometry: candidate p95 `23,306 ns`, max RSS `2,176 kB`, zero pair-record scratch; fallback p95 `15,385,815 ns`, max RSS `100,152 kB`, `2,094,081` pair records and `100,515,888` bytes explicit scratch. These figures are specimen-specific kernel/process measurements, not a portable performance guarantee and not buyer-facing HTTP p95.

## Decision and rejected alternatives

Keep the checked neutral-zero O(n) proof eligible for every `n >= 3`. Keep pairwise O(n²) only as the bounded `n <= 16` comparison/fail-closed reference after O(n) refusal. Above sixteen, a refused O(n) proof delegates directly to the established general implementation without quadratic scratch. Reintroducing a hard production cutoff at sixteen is rejected because the current source and independent `n=17` midpoint evidence demonstrate scientifically useful exact admission above that count.

The former `D=2^58, n=65` raw-scale refusal is explicitly rejected as admission evidence because it disappears under the same common-power dyadic normalization required by the O(n) proof. Retaining it would make the resource budget representation-dependent. The odd `D=2^58+1` boundary remains a normalized refusal fixture.

Treating `n<=2_047`, `n<=4_095`, or the unreduced denominator threshold as a production budget is also rejected because arithmetic representability is not latency or memory evidence. The normalized O(n) identity remains a sufficient checked proof route rather than a full-domain replacement for every possible proof representation. Wider-product/arbitrary-precision expansion remains deferred pending measured benefit, supply-chain review, complete represented-input evidence, and an explicit owner/resource decision.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Predecessor scientific repair | GAP-125; RED `5da82b2d651706c191ca191c6c077d916cbfda25`; repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Characterization RED | `4f1bd2c343cf2d54905a07c257a570a89dc575d3` — common `2^58` scale must normalize before O(n) overflow admission |
| Characterization repair | `d423b57797b6f7f127e61e0679f9ee9841525c77` — normalized test kernel plus odd-diameter refusal fixture |
| Narrow-reference overflow hardening | `96f17c02edba0792f61e0e92167703a6ae4e40d0` — checked restored-scale multiplication |
| Wider-product characterization | `081000289f5a52e94863026d55696ee2a4daf923` — dependency-free two-limb products/cancellation and odd-boundary recovery |
| Wider-product harness | `0bd805d4b0304cf1f76344ae14b7f079b3dade17` — release-mode comparison row for the wider intermediate reference |
| CHANGELOG fragment | `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md` |
| Current production module | `crates/validation_core/src/bias_se.rs` |
| Public API | `validation_core::bias_standard_error` |
| Exact characterization | `crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` |
| CPU/layout/hybrid harness | `crates/validation_core/examples/bias_se_exact_proof_budget.rs` |
| n=17 midpoint proof | `6f0063d3f0ebba807bbde8763a5238cd0306702e` |
| n=17 proof-width refusal | `bace14c6836014c37be67f0505d797ff28f0f643` |
| Current route rule | O(n) neutral-zero proof eligible for every `n >= 3`; O(n²) pair reference only after refusal at `n <= 16`; refusal above sixteen delegates directly to established fallback |

## Follow-up evidence required by #491

Retain exact-current release-mode resource measurements across representative admitted and refused geometries, including CPU/toolchain/build identity, raw timing samples, allocator/RSS and explicit scratch. Extend deterministic/reference/refusal/permutation evidence across the supported represented-input and target-width domain so a bounded proof refusal cannot be mistaken for scientific invalidity. Release acceptance still requires exact-head Rust/rustdoc/owned-production 100% line+branch/security/documentation GREEN, current TRACEABILITY/research/operator doctoring, qualifying independent current-head review, and applicable buyer-path p95 evidence where this estimator is exposed on an HTTP path.