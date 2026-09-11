# Wilson all-covered exact-count large-z denominator absorption

## Problem and scientific boundary

TEPP's Validation Evidence owner computes the Wilson score interval from retained coverage counts and a finite positive standard-normal critical value. For an all-covered sample, Wilson's lower endpoint is

`L = n / (n + z²)`.

The estimator, sidedness, retained counts, and critical-value meaning do not change here. The defect is binary64 evaluation of that same represented-input endpoint when `n` is exactly representable but `z²` is so large that forming `n + z²` rounds back to `z²`.

For `n = 3` and represented `z = 0x1.fffffffffffffp+29`, binary64 multiplication produces represented `z² = 0x1.ffffffffffffep+59`. The rounded denominator `n + z²` equals `z²`, so the predecessor evaluates a quotient with the finite sample-count contribution missing from its denominator. The resulting lower endpoint is `0x1.8000000000002p-59`. The exact rational endpoint formed from the exact integer `3` and the represented binary64 `z²` rounds instead to `0x1.8000000000001p-59`, one ULP lower.

This is not a new confidence-interval estimator and does not move reusable psychometric arithmetic into TEPP. It is numerical representation of TEPP Validation Evidence around the canonical Wilson producer in `validation_core`; reusable static psychometric estimation remains fast-mlsirm-owned.

## RED and causal repair

Public RED `3f3c9f2e16303791eaa0554979366dc68e2e63ff` adds `crates/validation_core/tests/wilson_all_covered_exact_count_large_z_rounding_contract.rs` and fixes the expected lower endpoint to bits `0x3c48000000000001` for the `n = 3` counterexample.

Causal repair `07766cb3df3a4c788c95fa6ec13bfd3de072185b` keeps the existing exact-count direct path and its separate near-one complementary repair. Only when the rounded denominator equals `z²` exactly does the implementation recover the lost addition term with a TwoSum-style residual. It then computes the quotient residual with `f64::mul_add` and applies the residual correction to the direct quotient. The branch is therefore limited to complete denominator absorption rather than imposing a new global evaluation order on ordinary Wilson endpoints.

Boundary reinforcement `3a6b243e3dae786fcd154674dffe9a890a7099f5` adds `z = 2^30` at the same `n = 3` absorption scale. In that neighboring case the ordinary quotient already has the correct represented endpoint `0x1.8000000000000p-59`; residual compensation must preserve that value rather than force an unconditional one-ULP decrement.

CHANGELOG trace: `8a2f6cee2dd8b490d2124a6f4255897925fd7cc0`.

## Decision record

Problem: an exactly representable sample count could be lost when a large represented `z²` absorbed it in `n + z²`, and that intermediate rounding could move the final all-covered Wilson lower endpoint by one ULP.

Constraints:

- preserve Wilson's score interval and the existing standard-normal/two-sided evidence contract;
- preserve exact retained `u64` count provenance;
- keep `coverage.rs` as the single Wilson arithmetic writer;
- do not replace the already-correct near-one or inexact-count paths;
- do not claim globally correctly rounded Wilson arithmetic beyond the demonstrated boundary.

Alternatives considered:

- Always rewrite the endpoint as `1 / (1 + z² / n)`. Rejected because it introduces an additional rounded division and addition and does not, by itself, recover the exact represented-input rational endpoint.
- Always evaluate `(n / z²) / (1 + n / z²)`. Rejected for the same double-rounding reason; the exposed counterexample remains one ULP high under that simple rearrangement.
- Introduce arbitrary-precision arithmetic for every Wilson endpoint. Rejected as disproportionate to the demonstrated defect and inconsistent with the smallest causal repair requirement for the Rust `f64` reference path.
- Recover the lost denominator term and quotient residual only when `n + z² == z²`. Selected because it directly repairs the demonstrated intermediate-rounding failure while leaving ordinary evaluation unchanged.

Risk: the repair does not prove global correct rounding for every exactly represented `n` and finite `z²`. It closes the demonstrated complete-denominator-absorption boundary. A different counterexample outside that boundary requires its own exact represented-input oracle and RED before the evaluation contract is broadened.

## Traceability

- Bounded context: Validation Evidence
- Production writer: `crates/validation_core/src/coverage.rs`
- Public contract: `crates/validation_core/tests/wilson_all_covered_exact_count_large_z_rounding_contract.rs`
- Durable consumers: `WilsonCoverageEvidenceV1` and `ValidationEvidenceV1` continue to consume the canonical Wilson writer rather than duplicate endpoint arithmetic.
- RED: `3f3c9f2e16303791eaa0554979366dc68e2e63ff`
- Causal fix: `07766cb3df3a4c788c95fa6ec13bfd3de072185b`
- Boundary reinforcement: `3a6b243e3dae786fcd154674dffe9a890a7099f5`
- CHANGELOG: `8a2f6cee2dd8b490d2124a6f4255897925fd7cc0`

## References

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

IEEE. (2019). *IEEE Standard for Floating-Point Arithmetic (IEEE Std 754-2019).* https://standards.ieee.org/ieee/754/6210/

ISO/IEC. (2020). *ISO/IEC 60559:2020 Information technology—Microprocessor systems—Floating-point arithmetic.* https://www.iso.org/standard/80985.html

As checked in September 2026, IEEE 754-2019 is an active standard and ISO/IEC 60559:2020 remains published; IEEE P754 is an active revision project rather than a published replacement.
