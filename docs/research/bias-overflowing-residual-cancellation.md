# Mean bias: overflowing residual cancellation

## Problem

TEPP defines signed bias as `mean(recovered - truth)`. The predecessor required every pairwise signed residual to be representable before computing the mean. That made an intermediate binary64 limitation authoritative even when the final scientific estimand was representable.

For finite represented inputs

- `truth = [-f64::MAX, f64::MAX, 0]`
- `recovered = [f64::MAX, -f64::MAX, 3 * f64::MIN_SUBNORMAL]`

the first two mathematical residuals are `+2*MAX` and `-2*MAX`, so direct binary64 subtraction overflows in both directions. Those terms cancel exactly in the mean numerator. The third residual is three minimum subnormals, leaving an exact represented-input mean bias of one minimum subnormal. Rejecting this payload because the two intermediate residuals are non-finite discards a representable recovery metric.

Morris, White, and Crowther (2019) treat bias as a performance measure relative to known simulation truth. The numerical implementation therefore has to preserve the declared estimand rather than silently replace it with the representability of one avoidable intermediate expression.

This remains TEPP Validation Evidence arithmetic. It does not move reusable psychometric estimation ownership from `fast-mlsirm`.

## Public RED

Commit `04e6e74507a1adb89aac4b2a58d3682746da30ea` adds `crates/validation_core/tests/bias_overflowing_residual_cancellation_contract.rs`.

The contract fixes three boundaries:

1. two overflowing signed residuals with exact cancellation must yield bias `0.0`;
2. the same extreme cancellation plus three minimum subnormals over three observations must preserve one minimum-subnormal mean bias; and
3. a one-sided `2*MAX` mean bias remains unrepresentable and must fail closed.

The RED-head GitHub workflows were superseded by subsequent source commits and cancelled, so they are not promoted as hosted RED execution evidence.

## Causal repair

Commit `28b1d186ce2bd08b63ca267c8b98b2eae45d2da7` factors the existing deterministic cancellation path into crate-private `deterministic_representable_sum_over_count(values, total_count)`. The divisor is explicit so an algebraically expanded numerator can retain the original scientific observation count. The helper keeps the existing sign-cancellation and exact-power-of-two scaling strategy, adds an explicit non-finite final-result rejection because the divisor can now differ from the number of terms, and preserves fail-closed behavior for nonzero results below binary64 range.

Commit `d1cd54615ddc311491e26fa427e956b5b5379e1a` changes only `mean_bias` admission. When all pairwise residuals are finite, the existing direct residual path remains authoritative. If at least one finite-input pairwise subtraction overflows, the fallback evaluates the same numerator as recovered values plus negated truth values and divides by the original paired-observation count. Opposing extreme terms can therefore cancel before scale reduction without creating a second public bias definition.

`bias_standard_error` is intentionally unchanged. Its requested dispersion depends on individual signed residual magnitudes; an unrepresentable residual is therefore not merely an avoidable intermediate for that API.

CHANGELOG commit: `d68dde34c2c63e610a859d45e528c2134b3c0f91`.

## Alternatives rejected

Returning zero whenever positive and negative residual overflows coexist was rejected because an additional finite residual can leave a nonzero representable bias. Clamping overflowing residuals to `f64::MAX` was rejected because it changes the estimand and can reverse cancellation. Computing `mean(recovered) - mean(truth)` as the universal implementation was rejected because independently rounded means introduce a different rounding contract and can lose a representable small bias. Replacing all current mean arithmetic with arbitrary-precision production code was rejected as unnecessary scope expansion.

## Scope and residual risk

The fallback is entered only for finite paired inputs whose direct signed residual path contains an overflow. Ordinary finite residuals retain the predecessor implementation. The repair claims preservation of representable signed mean bias under that intermediate-overflow condition; it does not claim arbitrary-precision bias, a changed standard-error target, or a new psychometric estimator.

The branch still requires current-head Rust tests, 100% owned line/branch coverage, documentation/security/SAST gates, independent review, and protected-main integration before delivery.

## Traceability

- Bounded context: Validation Evidence
- Module/API: `crates/validation_core/src/bias.rs` / `mean_bias`
- Shared crate-private arithmetic: `crates/validation_core/src/numeric.rs` / `deterministic_representable_sum_over_count`
- Public RED: `04e6e74507a1adb89aac4b2a58d3682746da30ea`
- Shared arithmetic repair: `28b1d186ce2bd08b63ca267c8b98b2eae45d2da7`
- Bias causal repair: `d1cd54615ddc311491e26fa427e956b5b5379e1a`
- CHANGELOG: `d68dde34c2c63e610a859d45e528c2134b3c0f91`
- Contract test: `crates/validation_core/tests/bias_overflowing_residual_cancellation_contract.rs`
- Landing vehicle: PR #488; only its latest exact head after this documentation commit is authoritative for hosted checks and review.

## Normative and methodological references

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE Standards Association. https://doi.org/10.1109/IEEESTD.2019.8766229

International Organization for Standardization & International Electrotechnical Commission. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

As of 2026-09-05, IEEE 754-2019 remains an active published standard; IEEE P754 remains an active revision PAR rather than a published replacement. ISO/IEC 60559:2020 remains published at stage 60.60. The AERA/APA/NCME Joint Committee is revising the 2014 testing Standards; the unpublished revision is not treated as current normative authority.
