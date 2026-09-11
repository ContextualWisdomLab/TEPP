# Mean bias: compensated numerator division rounding

## Problem

TEPP defines signed bias as `mean(recovered - truth)`. GAP-087 retained error-free low terms from opposite-sign cancellation, but the mixed-remainder path still collapsed the compensated normalized numerator to one binary64 value before dividing by the scientific observation count. That extra rounding can change the final represented mean even when the compensated numerator carries enough information to avoid it.

A public three-observation boundary is:

- truth: `[0, 0, 0]`;
- recovered/residuals: `[2^-53, -2^-52, -(1 + 2^-52)]`.

The exact represented-input numerator is

`2^-53 - 2^-52 - (1 + 2^-52) = -(1 + 3*2^-53)`.

Its exact mean is

`-9007199254740995 / 27021597764222976`,

which rounds to binary64 `-0x1.5555555555557p-2` (bits `0xbfd5555555555557`). The predecessor Neumaier path retained a high part and correction, but `sum + correction` rounded the numerator first to `-0x1.0000000000002p+0`; dividing that rounded numerator by three produced `-0x1.5555555555558p-2`, one ULP away from the represented-input mean. The mirrored positive case has the same defect.

This is a Validation Evidence arithmetic defect. It does not change the scientific definition of bias, create a new estimator, or move reusable static psychometric ownership out of `fast-mlsirm`.

## Public RED

Commit `8b4d19d161cb4322db3a143b2e34125d3bcc08f1` adds `crates/validation_core/tests/bias_compensated_division_contract.rs` and fixes both signs of the boundary. The contract requires `mean_bias` to return bits `0xbfd5555555555557` and `0x3fd5555555555557` from public pair inputs.

The RED is preserved as source-level reproducer evidence. Hosted runs superseded or cancelled by later source pushes are not promoted as current-head GREEN or RED execution evidence.

## Causal repair

Commit `7a33212b0c0f079a9fb138b6a8564881bc22fc9e` keeps the existing magnitude-ordered cancellation and GAP-087 low-term retention. It factors the canonical Neumaier pass into a private `(sum, correction)` result only for internal reuse. The existing `deterministic_compensated_sum` public-to-crate behavior remains `sum + correction` for callers that request a sum.

Only `mixed_remainder_mean_over_total` changes its mean formation. It now:

1. preserves the normalized Neumaier high part and correction separately;
2. divides the high part by the original scientific denominator;
3. uses binary64 FMA to recover the division residual of that high part;
4. combines that residual with the retained compensation before the final denominator division;
5. restores the same exact power-of-two scale and keeps the existing fail-closed non-finite/zero-underflow boundary.

For the RED, the retained correction is `+2^-53` relative to the rounded negative high part. Carrying that mass through division moves the result from predecessor bits `0xbfd5555555555558` to the represented-input result `0xbfd5555555555557`.

The repair intentionally does not alter the same-sign path or `bias_standard_error`; neither is implicated by this counterexample. CHANGELOG commit: `bad698dd686ce69a96aee1ca748bd7dc8e63aa1b`.

## Alternatives rejected

Replacing all Validation arithmetic with arbitrary precision was rejected because this bounded defect is caused by one avoidable rounding boundary and does not justify a new production dependency or latency surface.

Dividing `sum + correction` was rejected because that is the defective double-rounding sequence demonstrated by the RED. Dividing `sum` and `correction` independently and simply adding the quotients was also not adopted as the contract: the chosen FMA step additionally recovers the high-part division residual before the retained correction is consumed.

Changing the same-sign mean path or `bias_standard_error` without a represented-input counterexample was rejected as scope expansion. A future change there requires its own RED and causal proof.

## Scope and residual risk

This repair claims the demonstrated property that retained mixed-sign compensation is not forced through an additional numerator rounding before the original scientific count division. It does not claim globally correctly rounded summation or division for every binary64 sequence, every possible `usize` count, or every Validation metric.

A later numerical gap requires an independent public counterexample in which the current canonical producer differs from the represented-input estimand. Algebraic suspicion or an oracle-only mismatch outside a declared public contract is not sufficient by itself.

## Traceability

- Bounded context: Validation Evidence
- Public API: `crates/validation_core/src/bias.rs` / `mean_bias`
- Shared arithmetic: `crates/validation_core/src/numeric.rs` / `deterministic_representable_sum_over_count`
- Public RED: `8b4d19d161cb4322db3a143b2e34125d3bcc08f1`
- Causal repair: `7a33212b0c0f079a9fb138b6a8564881bc22fc9e`
- CHANGELOG: `bad698dd686ce69a96aee1ca748bd7dc8e63aa1b`
- Contract test: `crates/validation_core/tests/bias_compensated_division_contract.rs`
- Landing vehicle: PR #488; only its latest exact head after this documentation commit is authoritative for hosted checks and review.

## Normative and methodological references

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.
