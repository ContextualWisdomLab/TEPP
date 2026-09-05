# Bias-SE exact-proof budget characterization

## Scope

Issue #491 replaces the sample-count staircase in GAP-111 through GAP-125 with an evidence-based resource budget. This note records the first bounded characterization step. It does **not** widen production admission beyond `n=16`, does not claim buyer-path p95, and does not make a benchmark result from an unmeasured environment authoritative.

## New represented-input boundary evidence

The seventeen-observation residual multiset

`[38_557_579, 48_779_805, 63_558_649, 106_352_599, 139_863_777, 142_786_819, 267_163_239, 275_103_292, 375_678_558, 454_709_869, 484_300_224, 623_646_610, 989_643_121, 1_027_595_814, 1_520_220_488, 1_569_903_156, 1_805_452_085]`

is exactly representable in binary64. Its 136 squared pair distances sum to

`N = 92_549_865_125_191_410_206`.

For `n=17`, the scientific denominator is `17^2(17-1)=4_624`. `gcd(N,4_624)=2`, giving the reduced exact radicand

`46_274_932_562_595_705_103 / 2_312`.

The current public API intentionally remains on the established translated floating fallback at `n=17`; for this fixture it returns bits `0x41a0_dd77_9ac3_8e98`. Independent high-precision rational-square-root evaluation gives the adjacent correctly rounded binary64 target `0x41a0_dd77_9ac3_8e99`. This extends the demonstrated failure class beyond the current cutoff, but it is evidence for a systemic budget decision rather than justification for another one-count production patch.

`crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` fixes the exact pair-square sum, reduced ratio, current fallback boundary, and the algebraic equivalence below. The test intentionally makes any future widening of production admission update this characterization rather than silently inheriting a stale fallback assumption.

## O(n²) reference versus O(n) exact accumulator

For exact dyadic coefficients `c_i` on one shared unit,

`sum_{i<j} (c_i-c_j)^2 = n * sum_i c_i^2 - (sum_i c_i)^2`.

The new characterization test computes both sides with checked `u128` on the seventeen-observation fixture and requires exact equality to `N`. This demonstrates that an O(n) integer accumulator is algebraically capable of preserving the same pair-distance numerator once its admission proof establishes a common exact dyadic grid and exact pairwise differences.

That admission proof is the material constraint. A prospective linear path may choose the minimum represented residual as an anchor, prove every anchor-relative subtraction exact, align all offsets on the minimum dyadic exponent, and require the aligned diameter to be at most `2^53` units. Under that sufficient condition, every pair coefficient difference is an exact integer no larger than `2^53`, so pairwise binary64 subtraction exactness follows without enumerating every pair. The condition is deliberately stronger than the current O(n²) reference and therefore may refuse inputs that the pairwise proof could admit; refusal must fall back rather than change scientific meaning.

## Checked-u128 envelope

For nonnegative aligned coefficients bounded by diameter `D`, both `n * sum(c_i^2)` and `(sum c_i)^2` are bounded above by `n^2 D^2`. With `D <= 2^53`, the worst-case checked-`u128` product is strictly below `2^128` for `n <= 2_047`; at `n=2_048` the bound reaches `2^128` and cannot be represented by `u128`. The characterization test fixes this arithmetic ceiling and also verifies that `2_047^2 * 2_046` remains below `2^53`, so the existing exact binary64 denominator-seed bound is not the limiting factor in this compact-diameter case.

This is a worst-case arithmetic envelope, **not** a production sample-count budget. A production budget also needs release-mode CPU and allocation evidence and, where the estimator appears on an HTTP buyer path, applicable end-to-end p95 evidence against the TEPP `<=20 ms` target.

## Allocation finding

The current bounded O(n²) implementation stores every pair as `Option<(u128, i32)>` before the second accumulation pass. The number of stored pair records is exactly `n(n-1)/2`: 120 at `n=16`, 2,096,128 at `n=2_048`, and 4,997,500 at `n=3_162`. Exact byte cost depends on the compiled Rust layout and must be measured rather than inferred from field widths.

A two-pass O(n²) reference can remove that pair-record allocation without changing proof semantics: the first pass establishes exact pairwise subtraction and the minimum unit exponent; the second recomputes each exact pair difference and accumulates its checked integer square. The proposed O(n) sufficient-admission path would reduce both pair enumeration and pair storage, but neither optimization is activated on production input until exact-head Rust verification and resource measurements support it.

## Measurement harness

`crates/validation_core/examples/bias_se_exact_proof_budget.rs` is a standard-library-only harness for reproducible relative timing of the quadratic and linear checked-integer kernels on deterministic compact-dyadic coefficients. It emits CSV `sample_count,kernel,p95_ns,timing_samples` rows for 16, 64, 256, 1,024, and 2,047 observations and asserts exact equality between kernels before timing.

This harness deliberately excludes binary64 residual admission, public API composition, networking, and process scheduling. Its output therefore characterizes the integer kernel only. Release-mode results must record CPU/OS/toolchain/commit and cannot substitute for an applicable API buyer-path p95 measurement.

## Decision and rejected alternatives

Production admission stays `n=4..=16` in this commit. Extending to `n=17` alone is rejected because GAP-111 through GAP-125 plus the new seventeen-observation evidence show that the integer cutoff is not a scientific boundary. Removing the cutoff entirely is rejected because the existing implementation is O(n²) in both pair work and pair-record storage. Treating the `n<=2_047` checked-integer envelope as the production budget is rejected because arithmetic representability is not latency evidence. Arbitrary-precision production arithmetic is also deferred: it may be useful as an independent reference alternative, but adding it to the production dependency surface requires measured benefit, supply-chain review, and an explicit owner/resource decision.

## Traceability

| Item | Evidence |
|---|---|
| Domain owner | TEPP Validation Evidence |
| Systemic issue | #491 |
| Predecessor scientific repair | GAP-125; RED `5da82b2d651706c191ca191c6c077d916cbfda25`; repair `a509ae9e46c8ffc2cc3ef4f0e904774ad2516e1f` |
| Current production module | `crates/validation_core/src/bias_se.rs` |
| Public API | `validation_core::bias_standard_error` |
| Exact characterization | `crates/validation_core/tests/bias_standard_error_exact_proof_budget_characterization.rs` |
| CPU kernel harness | `crates/validation_core/examples/bias_se_exact_proof_budget.rs` |
| Resource/merge rule | Production cutoff remains `n<=16` pending measured exact-proof budget |

## Follow-up evidence required by #491

Run the new harness in release mode on a recorded CPU/toolchain and repeat enough independent samples to retain raw timing evidence. Measure allocation/layout rather than estimating bytes. Prototype the stronger O(n) dyadic-grid admission behind tests and compare its admitted/refused set against the existing pairwise proof. Evaluate a wider-integer/reference alternative without making it production authority. Only after those results establish a resource budget should production admission change; any such change needs a realistic public RED, exact-head Rust/rustdoc/coverage evidence, and applicable buyer-path p95 evidence.
