# Mean bias: mixed-sign cancellation roundoff

## Problem

TEPP defines signed bias as `mean(recovered - truth)`. After GAP-086, overflowing pairwise residuals can be algebraically expanded and opposing extreme terms cancel before scale reduction. A separate finite-input defect remained inside that shared cancellation helper: repeatedly adding an opposite-sign term smaller than half an ULP of the current residual could round back to the same high part and silently discard the low term each time.

For represented inputs

- residuals: `[1, -2^-54, -2^-54, -2^-54, -2^-54]`
- equivalent public pair input: `truth=[0, 2^-54, 2^-54, 2^-54, 2^-54]`, `recovered=[1, 0, 0, 0, 0]`

an implementation that materializes each cancellation as ordinary binary64 addition can keep the running high part at exactly `1.0` four times. The four discarded low terms sum to `2^-52`, so the exact represented-input numerator is `1 - 2^-52`; dividing by five rounds to bits `0x3fc9999999999998`, not ordinary `0.2` (`0x3fc999999999999a`). The final scientific estimand is representable, so discarding the low terms is an arithmetic defect rather than a justified fail-closed boundary.

Morris, White, and Crowther (2019) treat bias against known truth as a simulation performance measure. TEPP therefore preserves the represented-input estimand rather than making the rounding behavior of an avoidable pairwise cancellation step authoritative.

This remains Validation Evidence arithmetic. It does not move reusable psychometric estimation ownership from `fast-mlsirm` and does not introduce a second public bias definition.

## Public RED

Commit `63913727a3dfc2dd65c2508fe7e4acfdd6498508` adds `crates/validation_core/tests/bias_cancellation_roundoff_contract.rs`.

The contract fixes both signs of the same scientific boundary. Four quarter-ULP opposing residuals must collectively affect the represented mean bias even though each individual subtraction from the running high part is too small to change that high part by itself.

The RED commit is preserved as source-level reproducer lineage. Any later hosted run cancelled or superseded by subsequent source pushes is not treated as current-head RED execution evidence.

## Causal repair

Commit `5697cca51df2ec49e44a04730a14fd77656b48a5` changes only the shared deterministic cancellation path in `validation_core::numeric`.

Each opposite-sign addition now uses an error-free TwoSum decomposition: the rounded high part continues to drive the existing magnitude-ordered cancellation, while any nonzero low term is retained. If every cancellation is exact, the predecessor same-sign scale-reduction path is unchanged. Only when low terms exist are the final high remainders and those low terms scale-normalized together and accumulated with the existing deterministic compensated summation before division by the original scientific denominator.

This is narrower than replacing all mean arithmetic with arbitrary precision. It addresses the demonstrated loss of collectively material roundoff while preserving full-range cancellation, exact zero, the explicit denominator required by GAP-086, and fail-closed behavior for nonzero results outside or below binary64 range.

CHANGELOG commit: `53f5c912fbca415172f621c909b626ea1db66582`.

## Alternatives rejected

Always summing normalized original inputs was rejected because extreme cancellation such as `MAX + (-MAX) + MIN_SUBNORMAL` can lose the tiny surviving term during normalization before the large terms cancel. Pairwise Kahan/Neumaier accumulation over the original unscaled inputs was rejected because same-sign extreme intermediates may overflow even when the final mean is representable. Arbitrary-precision production arithmetic was rejected as unnecessary for this bounded counterexample and would add a new runtime dependency and performance surface.

Ignoring the low terms because each is individually below one ULP was rejected because the public RED proves their combined represented mass changes the final binary64 bias by two ULPs.

## Scope and residual risk

The repair claims only that opposite-sign cancellation no longer drops representable TwoSum low terms one at a time before their aggregate can affect the final mean. It does not claim globally correctly rounded summation for every possible binary64 sequence, nor does it alter `bias_standard_error`, whose estimand still requires individually representable signed residual dispersion.

A future stronger summation rule requires an independent represented-input counterexample plus a bounded causal repair. Algebraic suspicion alone is not sufficient to create another Validation gap.

## Traceability

- Bounded context: Validation Evidence
- Public API: `crates/validation_core/src/bias.rs` / `mean_bias`
- Shared arithmetic: `crates/validation_core/src/numeric.rs` / `deterministic_representable_sum_over_count`
- Public RED: `63913727a3dfc2dd65c2508fe7e4acfdd6498508`
- Causal repair: `5697cca51df2ec49e44a04730a14fd77656b48a5`
- CHANGELOG: `53f5c912fbca415172f621c909b626ea1db66582`
- Contract test: `crates/validation_core/tests/bias_cancellation_roundoff_contract.rs`
- Landing vehicle: PR #488; only its latest exact head after this documentation commit is authoritative for hosted checks and review.

## Normative and methodological references

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

As of 2026-09-05, IEEE 754-2019 remains an active published standard; IEEE P754 remains an active revision PAR rather than a published replacement. ISO/IEC 60559:2020 remains published at stage 60.60. AERA continues to publish the 2014 Testing Standards while the AERA/APA/NCME Joint Committee revises that edition; the unpublished revision is not treated as current normative authority.
