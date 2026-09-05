# Wilson all-covered exact-count scaling under extreme finite critical values

## Problem

`WilsonCoverageEvidenceV1` retains `sample_count` and `covered_count` as exact `u64` provenance and recomputes the canonical Wilson score endpoints from those counts. The reciprocal-scale path introduced for sample counts above binary64's exact-integer range correctly avoids pre-rounding the durable denominator. Its all-covered branch, however, always evaluated the lower endpoint through the complementary miss mass

`1 - (z² / n) / (1 + z² / n)`.

That algebraic form is stable when `z² / n` is small because it keeps a finite miss mass that would otherwise disappear when `1 + z² / n` rounds to `1.0`. It is not stable when `z² / n` is large: the miss fraction itself rounds to exact `1.0`, so subtracting it produces a false exact-zero lower endpoint even when the Wilson lower endpoint is an ordinary positive binary64 value.

## Public RED

Commit `059ce70d3dd497f486214137bca6a1f1b2e8b3cd` adds `crates/validation_core/tests/wilson_all_covered_inexact_count_extreme_z_contract.rs`.

The contract fixes:

- `sample_count = covered_count = 2^53 + 1 = 9_007_199_254_740_993`;
- `z = 1e20`, so represented `z²` is finite;
- empirical coverage `1.0`;
- represented Wilson lower endpoint `0x1.16c262777579dp-80`, decimal `9.007199254740993e-25`.

The predecessor reciprocal path computes `z² / n`, then `(z² / n)/(1 + z² / n)`. At this scale the latter rounds to exact `1.0`; subtracting from one therefore returns `0.0` and destroys representable finite-sample uncertainty.

## Repair lineage and self-review RCA

Commit `0f4783929b8d067eecb91696e1ad5761cd315b1e` first repaired the catastrophic boundary collapse by switching algebraic projection for the all-covered lower endpoint when the durable `u64` denominator is not exactly representable as binary64:

- if `z² / n <= 1`, evaluate `1 - (z² / n)/(1 + z² / n)` so tiny uncertainty below one survives;
- if `z² / n > 1`, evaluate `1/(1 + z² / n)` so a large complementary miss mass cannot round to one before subtraction.

Commit `8e2058f2fc6ab42af8f732f03e7ae1dcee3e873d` then removed a transient duplicated implementation from `coverage_evidence.rs`, restoring the Wilson projection to the single canonical writer in `coverage.rs`.

Pre-GREEN exact-oracle review found that this was still incomplete. The first scale-switch implementation formed `z² / n` as `z² * round(1/n)`. For the RED case that yields `0x1.d6329f1c35ca5p+79` and the direct reciprocal lower endpoint `0x1.16c262777579cp-80`, one ULP below the public oracle. The catastrophic false zero was gone, but the exact durable denominator had still been rounded once before multiplication.

Commit `93d5d2089db1fcf4c6167930adb362333a2ed809` is the causal correction for that residual double rounding. For the inexact-`u64` all-covered path, `coverage.rs` now decodes represented finite `z²` into its exact binary significand and power-of-two exponent, computes the significand/`sample_count` unit ratio through the existing integer ties-to-even routine, and restores the power-of-two scale. For the RED case this produces `z²/n = 0x1.d6329f1c35ca4p+79`, after which the large-scale reciprocal yields the required `0x1.16c262777579dp-80` endpoint.

The new private helper is restricted to this exact-count projection path. It does not redefine a psychometric estimator or make a claim that every arbitrary algebraic rearrangement of the Wilson formula is globally correctly rounded. The public contract is the exposed scientific boundary: exact durable count provenance must not be replaced by a pre-rounded denominator or a rounded-reciprocal product that changes the represented endpoint in the tested case.

## Boundary and owner decision

The existing `n = 2^55 + 3, z = 1.96` durable all-covered contract exercises the small-`z²/n` side and still requires `next_down(1.0)`. The extreme-`z` contract exercises the large side and requires the exact public-oracle bit pattern rather than false exact zero or the one-ULP-low intermediate repair. The transition at `z²/n = 1` changes only which algebraically equivalent expression is numerically conditioned; it does not alter interval sidedness, critical-value semantics, retained counts, or the Wilson score estimand.

This remains TEPP Validation Evidence provenance/projection behavior. No reusable static psychometric estimator is added, so there is no fast-mlsirm migration. No LLM is involved, and no contextual-orchestrator dependency is introduced.

## Traceability

- Public RED: `059ce70d3dd497f486214137bca6a1f1b2e8b3cd`
- Boundary scale-switch repair: `0f4783929b8d067eecb91696e1ad5761cd315b1e`
- Single-writer cleanup: `8e2058f2fc6ab42af8f732f03e7ae1dcee3e873d`
- Exact-denominator quotient correction: `93d5d2089db1fcf4c6167930adb362333a2ed809`
- Changelog correction: `43b6562c4c2ad3af51970040c127c784cccff228`
- Module: `crates/validation_core/src/coverage.rs`
- Durable carrier: `crates/validation_core/src/coverage_evidence.rs`
- Public contract: `crates/validation_core/tests/wilson_all_covered_inexact_count_extreme_z_contract.rs`
- Landing vehicle: PR #488

## References

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

IEEE P754 is an active revision project superseding IEEE 754-2019; it is not treated here as a published replacement standard.
