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

`crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` fixes the exact pair-square sum, reduced ratio, current fallback boundary, algebraic equivalence, pair-record counts, and the distinct checked-integer envelopes below. The test intentionally makes any future widening of production admission update this characterization rather than silently inheriting a stale fallback assumption.

## O(n²) reference versus O(n) exact accumulator

For exact dyadic coefficients `c_i` on one shared unit,

`sum_{i<j} (c_i-c_j)^2 = n * sum_i c_i^2 - (sum_i c_i)^2`.

The characterization test computes both sides with checked `u128` on the seventeen-observation fixture and requires exact equality to `N`. This demonstrates that an O(n) integer accumulator is algebraically capable of preserving the same pair-distance numerator once its admission proof establishes a common exact dyadic grid and exact represented-input geometry.

That admission proof is the material constraint. A prospective linear path may choose the minimum represented residual as an anchor, prove every anchor-relative subtraction exact, align all offsets on the minimum dyadic exponent, and require the aligned diameter to remain within a checked-integer budget. This condition is deliberately stronger than the current O(n²) reference unless a separate proof establishes that enumerated pairwise exactness is unnecessary; refusal must fall back rather than change scientific meaning.

## Checked-u128 envelopes

Two different `u128` bounds must not be conflated.

For the current minimum-shifted O(n) identity, both `n * sum(c_i^2)` and `(sum c_i)^2` are bounded above by `n^2 D^2` for aligned diameter `D`. With `D = 2^53`, that **implementation-intermediate** bound fits through `n=2_047` and reaches the unrepresentable `2^128` boundary at `n=2_048`.

The exact pair-square numerator itself has the tighter extremal bound `floor(n^2/4) D^2`, attained by placing the represented coefficients at the two diameter endpoints as evenly as possible. At the same `D = 2^53`, that final exact numerator can still fit `u128` through `n=4_095` and crosses the `2^128` boundary at `n=4_096`. Therefore `2_047` is not an intrinsic exact-pair numerator ceiling; it is a ceiling of the presently characterized minimum-shifted linear intermediate.

The scientific denominator `n^2(n-1)` is a separate bounded-proof input. Its unreduced value remains at or below the exact binary64-integer bound `2^53` through `n=208_064` and exceeds it at `n=208_065`. The production midpoint proof checks the **reduced** denominator after GCD reduction, so this unreduced threshold is a sufficient envelope marker rather than a universal refusal count.

None of these arithmetic thresholds is a production sample-count budget. Production still needs release-mode CPU and allocation evidence and, where the estimator appears on an HTTP buyer path, applicable end-to-end p95 evidence against the TEPP `<=20 ms` target.

## Allocation finding

The current bounded O(n²) implementation stores every pair as `Option<(u128, i32)>` before the second accumulation pass. The number of stored pair records is exactly `n(n-1)/2`: 120 at `n=16`, 136 at `n=17`, 2,096,128 at `n=2,048`, and **4,997,541** at `n=3,162`. The predecessor note incorrectly recorded the last count as 4,997,500; the Rust characterization now locks the exact integer count.

Exact byte cost is target-layout dependent and must not be inferred by adding field widths. The measurement harness now obtains `size_of::<Option<(u128, i32)>>()` on the executing target, records the actual `Vec` element capacity after `with_capacity`, and reports their product as scratch payload bytes. Allocator bookkeeping and whole-process RSS are still outside that number and must be recorded separately if they become release evidence.

A two-pass O(n²) reference can remove the pair-record allocation without changing the pair-enumeration proof shape: the first pass establishes the common dyadic unit, and the second recomputes each pair record and accumulates the checked square. The harness now measures this allocation-free quadratic alternative alongside the buffered pair layout. The O(n) identity removes pair enumeration as well, but production activation still requires its admitted/refused-set proof and exact-head verification.

## Measurement harness

`crates/validation_core/examples/bias_se_exact_proof_budget.rs` is a standard-library-only release-mode characterization harness. For deterministic compact-dyadic coefficients it compares three kernels:

- `quadratic_buffered`: production-layout-shaped `Vec<Option<(u128, i32)>>` pair records plus aligned checked-square accumulation;
- `quadratic_two_pass`: the same pair enumeration and dyadic alignment without pair-record storage;
- `linear`: `n*sum(c_i^2) - (sum c_i)^2` on minimum-shifted coefficients.

Before timing, the harness restores the buffered/two-pass unit exponent and requires all three kernels to equal the same exact pair-square numerator. It emits CSV columns

`sample_count,kernel,p95_ns,timing_samples,unit_exponent,scratch_records,scratch_payload_bytes,pair_record_size_bytes`

for 16, 64, 256, 1,024, and 2,047 observations. The buffered timing includes pair-vector allocation and consumption; the two-pass and linear rows report zero pair-record scratch payload. This is still a kernel harness, not the full public API: binary64 residual admission, endpoint serialization, networking, scheduler effects, and allocator metadata/RSS remain outside the measurement.

No release-mode timing result is recorded in this document yet. A valid timing record must include CPU, OS, Rust toolchain, exact commit, release build mode, raw sample count, and raw CSV. It cannot substitute for an applicable API buyer-path p95 measurement.

## Decision and rejected alternatives

Production admission stays `n=4..=16`. Extending to `n=17` alone is rejected because GAP-111 through GAP-125 plus the seventeen-observation evidence show that the integer cutoff is not a scientific boundary. Removing the cutoff entirely is rejected because the current production implementation still enumerates and stores O(n²) pair evidence. Treating `n<=2_047`, `n<=4_095`, or the unreduced denominator threshold as the production budget is rejected because arithmetic representability is not latency or memory evidence.

A two-pass O(n²) allocation-removal path remains a candidate because it can preserve the current pair-proof shape while eliminating pair-record storage. The O(n) identity remains the stronger candidate for CPU scaling, subject to an admitted/refused-set proof. Arbitrary-precision production arithmetic is deferred: it may be useful as an independent wider reference, but adding it to the production dependency surface requires measured benefit, supply-chain review, and an explicit owner/resource decision.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Predecessor scientific repair | GAP-125; RED `5da82b2d651706c191ca191c6c077d916cbfda25`; repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Current production module | `crates/validation_core/src/bias_se.rs` |
| Public API | `validation_core::bias_standard_error` |
| Exact characterization | `crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` |
| CPU/layout harness | `crates/validation_core/examples/bias_se_exact_proof_budget.rs` |
| CHANGELOG evidence | `CHANGELOG.d/validation-bias-exact-proof-budget-characterization.md` |
| Resource/merge rule | Production cutoff remains `n<=16` pending measured exact-proof budget |

## Follow-up evidence required by #491

Run the harness in release mode on a recorded CPU/toolchain and retain raw timing CSV. Record allocator/RSS evidence in addition to the harness's exact pair-record payload layout. Prototype the stronger O(n) dyadic-grid admission behind tests and compare its admitted/refused set against the existing pairwise proof. Evaluate a wider-integer/reference alternative without making it production authority. Only after those results establish a resource budget should production admission change; any such change needs a realistic public RED, exact-head Rust/rustdoc/coverage evidence, and applicable buyer-path p95 evidence.
