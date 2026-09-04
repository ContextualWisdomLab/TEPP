# Wilson all-covered exact-count partial-denominator rounding

## Problem

For an all-covered binomial sample with an exactly representable retained count `n`, the Wilson lower endpoint reduces to

\[
L = \frac{n}{n + z^2}.
\]

The prior exact-count boundary repairs covered two special cases: a tiny positive `z²` whose denominator addition collapses to `n`, and a large `z²` that completely absorbs `n`. Fresh exact-rational review found a third state in the ordinary path: `n + z²` can round to a nearby finite binary64 value without either operand being completely absorbed, and the subsequent correctly rounded hardware division is then correct for the *rounded denominator* rather than for the represented-input denominator sum.

The public RED uses `n = 1` and represented `z = 3 * 2^-28` (`0x1.8p-27`). Binary64 multiplication gives `z² = 9 * 2^-56` (`0x1.2p-53`). The exact represented-input endpoint is `1 / (1 + 9 * 2^-56)`, whose nearest binary64 value is `0x1.fffffffffffffp-1`. Forming the denominator first rounds `1 + z²` upward to `0x1.0000000000001p+0`, and direct division then produces `0x1.ffffffffffffep-1`, one ULP below the correct represented-input result.

This is numerical evidence corruption, not an estimator-target change. A durable Validation Evidence artifact must not depend on a denominator rounding state that is distinguishable from the exact arithmetic of its already represented inputs.

## Constraints and rejected alternatives

`coverage.rs` remains the canonical Wilson writer. The repair must not duplicate Wilson arithmetic in a report/projection layer, move reusable static psychometric arithmetic into TEPP, consume mutable code from `fast-mlsirm`, or use an LLM as numerical authority.

Always switching to `1 - z² / (n + z²)` was rejected. That algebraic form is useful at the near-one boundary but has its own rounding behavior outside that boundary. Replacing all Wilson evaluation with an arbitrary alternative formula would change stable paths without causal evidence.

Treating hardware division as sufficient was also rejected. IEEE 754 makes the basic operation deterministic for the floating-point operands presented to it; it does not restore information already lost when `n + z²` was rounded before division.

## Selected repair

For the exactly representable-count, all-covered path, TEPP now obtains the error-free TwoSum residual of the denominator addition:

\[
D = \operatorname{fl}(n + z^2), \qquad n + z^2 = D + \delta_D.
\]

When `δ_D != 0`, the direct quotient `q = fl(n / D)` is corrected with a fused residual for `n - qD` and the denominator residual contribution `-qδ_D`. The correction reuses the same compensated quotient mechanism already justified by the complete large-`z²` absorption repair; this change broadens its causal trigger from complete absorption to any demonstrably inexact exact-count denominator sum. Exact denominator sums remain on the direct path. The earlier false-exact-one complementary branch remains boundary-local because it also protects the case in which the quotient has already collapsed to `1.0`.

The change does **not** claim that every possible floating-point rearrangement of the Wilson interval is globally correctly rounded. It closes the demonstrated partial-denominator state and retains bit-level public contracts for the near-one, partial-rounding, large-`z²`, and correctly rounded control boundaries.

## Evidence and traceability

- Public RED: `06e556538e171e675c4d8a8287d75052ffc2c4c3`, `crates/validation_core/tests/wilson_all_covered_exact_count_partial_denominator_rounding_contract.rs`.
- Causal production repair: `6c084dbe607e6c415288c77fa41a4270947cd51e`, `crates/validation_core/src/coverage.rs`.
- Canonical API: `validation_core::wilson_coverage_interval`.
- Predecessor complete-absorption contract retained: `crates/validation_core/tests/wilson_all_covered_exact_count_large_z_rounding_contract.rs`.
- Owner boundary: TEPP Validation Evidence numerical representation/admission. No reusable static psychometric estimator was added.

## Standards and primary literature status checked 2026-09-04

Wilson's original score-interval source remains the primary statistical reference. IEEE 754-2019 is currently listed by IEEE SA as an Active Standard; P754 is an Active PAR to revise/supersede it, not a published replacement. ISO/IEC 60559:2020 remains a published International Standard adopting the floating-point arithmetic specification. The current published AERA/APA/NCME *Standards for Educational and Psychological Testing* remains the 2014 edition; the sponsoring organizations' Joint Committee is revising that edition, and AERA lists the Standards task-force roster as of 2026-08-31. These statuses are recorded without treating an unpublished revision as current normative authority.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE Computer Society. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953
