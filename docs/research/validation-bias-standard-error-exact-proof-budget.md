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

`crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` fixes the exact pair-square sum, reduced ratio, current fallback boundary, algebraic equivalence, pair-record counts, distinct checked-integer envelopes, and the admitted/refused-set relation between the current pair reference and the candidate O(n) accumulator. The test intentionally makes any future widening of production admission update this characterization rather than silently inheriting a stale fallback assumption.

## O(n²) reference versus O(n) exact accumulator

For exact dyadic coefficients `c_i` on one shared unit,

`sum_{i<j} (c_i-c_j)^2 = n * sum_i c_i^2 - (sum_i c_i)^2`.

The characterization test computes both sides with checked `u128` on the seventeen-observation fixture and requires exact equality to `N`. It also checks deterministic compact grids at `n=4,16,17,32,64,128,256`. Whenever the minimum-shifted O(n) kernel admits those grids, it must equal the O(n²) pair numerator exactly. This demonstrates that an O(n) integer accumulator is algebraically capable of preserving the same pair-distance numerator once its admission proof establishes a common exact dyadic grid and its checked intermediates remain representable.

That admission proof is the material constraint. A prospective linear path may choose the minimum represented residual as an anchor, prove every anchor-relative subtraction exact, align all offsets on the minimum dyadic exponent, and require its checked integer intermediates to remain representable. This condition is deliberately stronger than the current O(n²) reference unless a separate proof establishes equivalence; refusal must fall back rather than change scientific meaning.

The admission-set characterization proves that the present `u128` linear kernel is **sufficient but not admission-equivalent** to the pair reference. Let `D=2^58` and use one coefficient at zero with every remaining coefficient at `D`. At `n=64`, the linear first term is `64*63*D^2 = 4032*2^116`, so both kernels fit and return the same exact numerator `63*D^2`. At `n=65`, the exact pair numerator is still only `64*D^2 = 2^122`, but the unreduced linear first term becomes `65*64*D^2 = 4160*2^116`, which exceeds the `u128` range before subtracting `(sum c_i)^2`. The checked O(n) kernel therefore refuses a geometry that the checked O(n²) pair reference can still prove exactly.

This rules out a drop-in replacement of the current pair proof with the minimum-shifted `u128` identity. A production O(n) path can preserve current scientific admission only as a sufficient fast path followed by the existing pairwise proof on refusal, or by adopting a wider checked-integer representation with its own resource and supply-chain evidence. A refusal from the linear path is not evidence that the represented-input estimator is scientifically unprovable.

## Checked-u128 envelopes

Two different `u128` bounds must not be conflated.

For the current minimum-shifted O(n) identity, both `n * sum(c_i^2)` and `(sum c_i)^2` are bounded above by `n^2 D^2` for aligned diameter `D`. With `D = 2^53`, that **distribution-independent sufficient intermediate envelope** fits through `n=2_047` and reaches the unrepresentable `2^128` boundary at `n=2_048`. It is not the exact admission frontier for every coefficient distribution; the `D=2^58`, `n=64/65` characterization above shows that actual checked-intermediate admission depends on the coefficient distribution as well as `n` and diameter.

The exact pair-square numerator itself has the tighter extremal bound `floor(n^2/4) D^2`, attained by placing the represented coefficients at the two diameter endpoints as evenly as possible. At the same `D = 2^53`, that final exact numerator can still fit `u128` through `n=4_095` and crosses the `2^128` boundary at `n=4_096`. Therefore `2_047` is not an intrinsic exact-pair numerator ceiling; it is only the conservative all-distribution envelope of the presently characterized minimum-shifted linear intermediates.

The scientific denominator `n^2(n-1)` is a separate bounded-proof input. Its unreduced value remains at or below the exact binary64-integer bound `2^53` through `n=208_064` and exceeds it at `n=208_065`. The production midpoint proof checks the **reduced** denominator after GCD reduction, so this unreduced threshold is a sufficient envelope marker rather than a universal refusal count.

None of these arithmetic thresholds is a production sample-count budget. Production still needs release-mode CPU and allocation evidence and, where the estimator appears on an HTTP buyer path, applicable end-to-end p95 evidence against the TEPP `<=20 ms` target.

## Allocation finding

The current bounded O(n²) implementation stores every pair as `Option<(u128, i32)>` before the second accumulation pass. The number of stored pair records is exactly `n(n-1)/2`: 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and **4,997,541** at `n=3,162`. The predecessor note incorrectly recorded the last count as 4,997,500; the Rust characterization now locks the exact integer count.

Exact byte cost is target-layout dependent and must not be inferred by adding field widths. The measurement harness obtains `size_of::<Option<(u128, i32)>>()` on the executing target, records the actual `Vec` element capacity after `with_capacity`, and reports their product as scratch payload bytes. Allocator bookkeeping and whole-process RSS are still outside that number and must be recorded separately if they become release evidence.

A two-pass O(n²) reference can remove the pair-record allocation without changing the pair-enumeration proof shape: the first pass establishes the common dyadic unit, and the second recomputes each pair record and accumulates the checked square. The harness measures this allocation-free quadratic alternative alongside the buffered pair layout. The O(n) identity removes pair enumeration as well, but the admission-set characterization shows that its present checked-`u128` form must remain a sufficient fast path with pairwise fallback unless wider intermediates are justified.

## Measurement harness

`crates/validation_core/examples/bias_se_exact_proof_budget.rs` is a standard-library-only release-mode characterization harness. It now compares four kernels:

- `quadratic_buffered`: production-layout-shaped `Vec<Option<(u128, i32)>>` pair records plus aligned checked-square accumulation;
- `quadratic_two_pass`: the same pair enumeration and dyadic alignment without pair-record storage;
- `linear`: `n*sum(c_i^2) - (sum c_i)^2` on minimum-shifted coefficients;
- `hybrid`: the viable resource shape, using the checked O(n) accumulator when it admits and otherwise falling back to the production-layout-shaped buffered pair proof.

Before timing, the harness restores the dyadic unit and requires every applicable kernel to equal the same exact pair-square numerator. The compact deterministic fixtures at 16, 64, 256, 1,024, and 2,047 observations must keep the hybrid on its O(n) fast path. A separate `D=2^58` boundary pair measures both sides of the characterized admission relation: `n=64` must keep the hybrid on the linear path, while `n=65` must make the linear kernel refuse and the hybrid use the buffered pair fallback while preserving the exact numerator `2^122`.

The CSV schema is now

`geometry,sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes,used_pairwise_fallback`.

The `geometry` field distinguishes compact admitting fixtures, the `n=64` boundary-admitting fixture, and the `n=65` pair-fallback fixture. `used_pairwise_fallback` makes it observable whether a hybrid row actually exercised the expensive proof path instead of inferring that fact from sample count. The buffered and hybrid-fallback rows include pair-vector allocation and consumption; the two-pass and admitted linear/hybrid rows report zero pair-record scratch payload.

The hybrid harness landed in commit `c6b237e0bb1388cccd7bcb71a0df5cbf837a07c5`. It remains measurement tooling only. No release-mode timing result is recorded in this document yet because the current execution environment does not provide a Rust toolchain and the hosted exact-head jobs have not produced measurement artifacts. A valid timing record must include CPU, OS, Rust toolchain, exact commit, release build mode, raw sample count, raw CSV, allocator/RSS evidence, and the cold/warm procedure. Kernel timing cannot substitute for an applicable API buyer-path p95 measurement.

## Decision and rejected alternatives

Production admission stays `n=4..=16`. Extending to `n=17` alone is rejected because GAP-111 through GAP-125 plus the seventeen-observation evidence show that the integer cutoff is not a scientific boundary. Removing the cutoff entirely is rejected because the current production implementation still enumerates and stores O(n²) pair evidence. Treating `n<=2_047`, `n<=4_095`, or the unreduced denominator threshold as the production budget is rejected because arithmetic representability is not latency or memory evidence.

A two-pass O(n²) allocation-removal path remains a candidate because it can preserve the current pair-proof admission shape while eliminating pair-record storage. The O(n) identity remains the stronger CPU-scaling candidate, but its checked-`u128` form is proven to be a strict sufficient subset of the pair reference. Replacing the pair proof with that kernel alone is rejected because it would silently narrow exact-proof admission. The hybrid harness now measures the viable bounded shape—O(n) fast admission with buffered O(n²) fallback—without making it production behavior. An admission-equivalent wider-integer O(n) proof or the allocation-free two-pass pair reference remain alternatives if measurements justify them. Arbitrary-precision production arithmetic remains deferred pending measured benefit, supply-chain review, and an explicit owner/resource decision.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Predecessor scientific repair | GAP-125; RED `5da82b2d651706c191ca191c6c077d916cbfda25`; repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Current production module | `crates/validation_core/src/bias_se.rs` |
| Public API | `validation_core::bias_standard_error` |
| Exact characterization | `crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` |
| CPU/layout/hybrid harness | `crates/validation_core/examples/bias_se_exact_proof_budget.rs`; hybrid commit `c6b237e0bb1388cccd7bcb71a0df5cbf837a07c5` |
| CHANGELOG evidence | `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md` |
| Resource/merge rule | Production cutoff remains `n<=16` pending measured exact-proof budget |

## Follow-up evidence required by #491

Run the current hybrid-capable harness in release mode on a recorded CPU/toolchain and retain raw timing CSV. Record allocator/RSS evidence in addition to the harness's exact pair-record payload layout. Evaluate a wider-integer/reference alternative without making it production authority. Only after those results establish a resource budget should production admission change; any such change needs a realistic public RED, exact-head Rust/rustdoc/coverage evidence, and applicable buyer-path p95 evidence.
