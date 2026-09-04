# Wilson all-covered exact-count scaling under extreme finite critical values

## Problem

`WilsonCoverageEvidenceV1` retains `sample_count` and `covered_count` as exact `u64` provenance and recomputes the canonical Wilson score endpoints from those counts. The reciprocal-scale path introduced for sample counts above binary64's exact-integer range correctly avoids pre-rounding the durable denominator. Its all-covered branch, however, always evaluated the lower endpoint through the complementary miss mass

`1 - (z² / n) / (1 + z² / n)`.

That algebraic form is stable when `z² / n` is small because it keeps a finite miss mass that would otherwise disappear when `1 + z² / n` rounds to `1.0`. It is not stable when `z² / n` is large: the miss fraction itself rounds to exact `1.0`, so subtracting it produces a false exact-zero lower endpoint even when the Wilson lower endpoint is an ordinary positive binary64 value.

## Public RED

Commit `059ce70d3dd497f486214137bca6a1f1b2e8b3cd` adds `crates/validation_core/tests/wilson_all_covered_inexact_count_extreme_z_contract.rs`.

The contract fixes:

- `sample_count = covered_count = 2^53 + 1 = 9_007_199_254_740_993`;
- `z = 1e20`, so `z²` is finite;
- empirical coverage `1.0`;
- canonical Wilson lower endpoint `n / (n + z²) = 0x1.16c262777579dp-80`, decimal `9.007199254740993e-25`.

The predecessor reciprocal path computes `z² / n`, then `(z² / n)/(1 + z² / n)`. At this scale the latter rounds to exact `1.0`; subtracting from one therefore returns `0.0` and destroys representable finite-sample uncertainty.

## Causal repair

Commit `0f4783929b8d067eecb91696e1ad5761cd315b1e` keeps the Wilson estimand and the exact-count reciprocal path. It changes only the algebraic projection used for the all-covered lower endpoint when the durable `u64` denominator is not exactly representable as binary64:

- if `z² / n <= 1`, evaluate `1 - (z² / n)/(1 + z² / n)` so tiny uncertainty below one survives;
- if `z² / n > 1`, evaluate `1/(1 + z² / n)` so a large complementary miss mass cannot round to one before subtraction.

Both expressions are algebraically identical to `n/(n+z²)`. The branch is numerical conditioning, not an estimator change or a new confidence-interval policy. Commit `8e2058f2fc6ab42af8f732f03e7ae1dcee3e873d` removes a transient duplicated implementation from `coverage_evidence.rs` so the Wilson projection remains single-writer in the canonical `coverage.rs` producer.

## Boundary and owner decision

The existing `n = 2^55 + 3, z = 1.96` durable all-covered contract exercises the small-`z²/n` side and still requires `next_down(1.0)`. The new extreme-`z` contract exercises the large side and requires a positive endpoint rather than false exact zero. The transition at `z²/n = 1` changes only which algebraically equivalent expression is numerically conditioned; it does not alter interval sidedness, critical-value semantics, retained counts, or the Wilson score estimand.

This remains TEPP Validation Evidence provenance/projection behavior. No reusable static psychometric estimator is added, so there is no fast-mlsirm migration. No LLM is involved, and no contextual-orchestrator dependency is introduced.

## Traceability

- Public RED: `059ce70d3dd497f486214137bca6a1f1b2e8b3cd`
- Canonical producer repair: `0f4783929b8d067eecb91696e1ad5761cd315b1e`
- Single-writer cleanup: `8e2058f2fc6ab42af8f732f03e7ae1dcee3e873d`
- Changelog: `8a73cb64d15e5412e5afabbae80603a66c12a7f0`
- Module: `crates/validation_core/src/coverage.rs`
- Durable carrier: `crates/validation_core/src/coverage_evidence.rs`
- Public contract: `crates/validation_core/tests/wilson_all_covered_inexact_count_extreme_z_contract.rs`
- Landing vehicle: PR #488

## References

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

IEEE P754 is an active revision project superseding IEEE 754-2019; it is not treated here as a published replacement standard.
