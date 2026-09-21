# Wilson sample-count scale in durable coverage evidence

## Problem

`WilsonCoverageEvidenceV1` stores `sample_count` and `covered_count` as fixed-width `u64` provenance and validates its projected Wilson score interval by recomputation. The preceding large-count repairs made `covered_count / sample_count` correctly rounded and stopped one uncovered observation above `2^53` from collapsing into an all-covered state. One denominator path nevertheless remained lossy: `wilson_coverage_interval_from_counts` converted the exact `sample_count` to binary64 before using it in the Wilson scale terms.

That conversion is harmless only when the integer count itself is exactly representable. For `sample_count = 9_007_199_254_740_1013`, `sample_count as f64` is a neighboring even binary64 integer rather than the retained denominator. With `covered_count = 1_286_742_750_677_287` and `z = 1.96`, the exact integer ratio still projects to ordinary finite coverage `0.1428571428571428`, but using the rounded denominator moves the Wilson lower endpoint one binary64 ULP upward. The represented endpoint obtained from the exact denominator is `0x1.2492482c43beap-3` (`0.14285713563046382`).

The same loss is buyer-visible at a boundary. For all-covered evidence with `sample_count = 2^55 + 3 = 36_028_797_018_963_971` and `z = 1.96`, the exact Wilson lower root `n / (n + z²)` rounds to `next_down(1.0)`. Materializing the count as `f64` first can make the endpoint exact `1.0` and therefore erase representable finite-sample uncertainty even though the durable carrier still owns the exact denominator.

## Repair

The canonical count-based Wilson authority now distinguishes whether the fixed-width denominator is exactly representable in binary64. Exact counts keep the established `n: f64` path so existing small-count and extreme-`z` contracts retain their evaluated arithmetic. Counts that are not exactly representable use `correctly_rounded_unit_ratio(1, sample_count)` and evaluate the same Wilson score algebra through the reciprocal scale `1 / n`, avoiding a pre-rounded integer denominator.

For a positive strict-interior represented proportion `p`, the rationalized lower root is evaluated as

`2 p² / [z²/n + 2p + z sqrt(z²/n² + 4p(1-p)/n)]`.

The upper root uses the same reciprocal scale in its denominator, center, and radical. Near all-covered evidence still uses score-interval complement symmetry with the smaller uncovered proportion. For exact all-covered evidence on the inexact-denominator path, the implementation evaluates the complementary miss mass `(z²/n) / (1 + z²/n)` and subtracts that from one; this avoids losing a representable `next_down(1.0)` endpoint when `1 + z²/n` itself rounds to one.

This is not a new confidence-interval estimand and does not introduce arbitrary higher-precision output. The durable contract already retained exact integer provenance. The repair prevents an avoidable binary64 pre-rounding of that provenance before the existing Wilson score projection. Public results remain binary64 and deterministic.

## RED → repair trace

- `f89e246790a835ca1a520c8071401a1e4fd6892f`: public RED for the strict-interior denominator case, fixing exact `u64` counts and the one-ULP lower-endpoint oracle.
- `1a5180b22c755c793d3878e644192e7d7cbbff47`: strengthen the RED with the exact all-covered `2^55 + 3` boundary whose finite-sample lower endpoint must remain `next_down(1.0)`.
- `6254c4989a5fb8922e88bff3d6e5d5b42b4f88e0`: correct the predecessor non-boundary large-count fixture; its empirical count-ratio oracle was already exact, but its Wilson lower endpoint still encoded rounded-denominator arithmetic.
- `73bbb5cf262f710d35b2fffa793076ae6a173947`: causal production repair using an exact-integer representability check and reciprocal-scale Wilson evaluation only when the retained denominator cannot be represented exactly.
- `f2b5768b1632d647959c72a1799160bf16cab5c6`: release-facing CHANGELOG fragment.

Every later source or documentation push invalidates predecessor exact-head workflow/review evidence. Landing still requires current-head hosted Rust, owned line/branch coverage, documentation, security/SAST/supply-chain checks, resolved review threads, and a qualifying independent approval.

## Bounded-context ownership

This change belongs to TEPP Validation Evidence. `WilsonCoverageEvidenceV1` is a durable provenance/projection carrier around the Wilson coverage producer and its exact fixed-width counts; the issue is not reusable static psychometric estimation. No fast-mlsirm source is copied, no Longitudinal Modeling semantics move into `validation_core`, and no contextual-orchestrator behavior is involved.

## Standards and primary source

Wilson's original paper remains the primary source for the score-interval family. Binary64 representation and round-to-nearest behavior are grounded in the currently published IEEE 754-2019 and its ISO/IEC 60559:2020 adoption. IEEE P754 is an active revision project rather than a published replacement. The current published AERA/APA/NCME testing standards remain the 2014 edition while the Joint Committee proceeds with revision.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization, & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). ISO. https://www.iso.org/standard/80985.html

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

## Verification contract

`crates/validation_core/tests/wilson_sample_count_rounding_contract.rs` fixes both the strict-interior and exact all-covered large-denominator oracles. `crates/validation_core/tests/wilson_coverage_ratio_rounding_contract.rs` protects the corrected earlier ratio case. Private unit coverage verifies binary64 integer-representability classification across spacing changes. Existing small-denominator, extreme-`z`, complement, endpoint-cancellation, carrier-serde, and durable-envelope contracts remain required and are not replaced by these tests.
