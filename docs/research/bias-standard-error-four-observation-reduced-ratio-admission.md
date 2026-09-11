# Bias standard error: reduced exact four-observation ratio admission

## Finding

GAP-111 added a bounded exact four-observation pair-distance path so that `SE(mean)` is rounded from the exact rational radicand instead of from a binary64-rounded ratio. Its admission still tested the *unreduced* pair-square numerator against the binary64 exact-integer bound. That made proof admission depend on an algebraically irrelevant representation of the same rational number.

A represented-input counterexample is

`r = [0, 14_099_687, 16_729_100, 94_045_527]`.

Every residual and every pairwise difference is exactly represented in binary64. The six squared pair distances are

`198801173497969`, `279862786810000`, `8844561148707729`, `6913812724569`, `6391337333305600`, and `5977829884046329`,

which sum exactly to

`N = 21699306139092196`.

For four observations,

`SE(mean)^2 = N / 48`.

The exact rational reduces by `gcd(N, 48) = 4` to

`5424826534773049 / 12`.

The unreduced numerator is greater than `2^53`, so GAP-111 refused its bounded midpoint-square proof and returned to the translated floating path. That fallback returns adjacent-lower bits `0x4174_46e5_76f8_7444`. The *reduced* numerator `5424826534773049` is below `2^53`; applying the same exact midpoint comparison to the identical rational gives the correctly rounded represented-input target `0x4174_46e5_76f8_7445`.

This is a deterministic Validation Evidence arithmetic defect. No data, estimator target, or probabilistic assumption changes.

## RED and causal repair

Public RED `4f2231354fa0ad3e0c646bbb100b3cc83566d033` adds `crates/validation_core/tests/bias_standard_error_four_observation_reduced_ratio_contract.rs`. It fixes several permutations of the exact residual multiset and their sign mirrors at bits `0x4174_46e5_76f8_7445`.

Causal source repair `ed1a8763c198fbe478de5ec8be72e3436e5918e3` changes only the GAP-111 bounded admission in `crates/validation_core/src/bias_se.rs`. After the exact pair-square sum is constructed in checked `u128`, the implementation computes the integer greatest common divisor with the scientific denominator `48`, divides numerator and denominator by that divisor, and then invokes the existing exact adjacent-midpoint square comparison. The rational radicand is unchanged.

The admission boundary remains narrow:

- exactly four observations;
- every represented residual is finite and subtraction-error-free;
- every pairwise residual difference is finite and subtraction-error-free;
- the dyadic pair-distance numerator fits the existing checked `u128` construction;
- after exact rational reduction, numerator and denominator fit the existing bounded binary64-integer proof;
- exact candidate/neighbor midpoint-square comparison can complete without integer overflow.

Any failed proof still returns to `crates/validation_core/src/bias.rs`. No arbitrary-precision runtime, mutable sibling dependency, or new reusable psychometric arithmetic owner is introduced.

## Alternatives rejected

Keeping the unreduced numerator and increasing the `2^53` admission limit was rejected because `numerator as f64` would cease to be exact and would invalidate the proof that the candidate is derived from the represented rational without an extra integer-conversion rounding.

Special-casing this residual payload was rejected because the defect is rational representation, not the particular values.

Changing every four-observation evaluation to a different floating formula was rejected because algebraic rearrangement alone does not provide a correct-rounding proof and would widen the rounding surface beyond the established GAP-111 contract.

Arbitrary-precision production arithmetic was rejected as disproportionate to the finding and outside TEPP's owner boundary. Reusable static psychometric arithmetic remains owned by `fast-mlsirm`; this code is a bounded Validation Evidence admission proof.

GAP-112 does not claim globally correctly rounded `bias_standard_error` for arbitrary `n > 2`, nor does it admit four-observation samples whose exactness or bounded integer proof cannot be established.

## Standards and methodological trace

IEEE 754-2019 remains the published IEEE floating-point standard used for the binary64 destination-format reasoning here. IEEE P754 is the active revision project rather than a published replacement. ISO/IEC 60559:2020 remains the corresponding published international floating-point standard. The relevant engineering consequence is that an exact rational identity does not make two sequences of rounded operations representation-equivalent; proof admission must preserve the exact value being compared with destination-format rounding boundaries.

The AERA/APA/NCME public testing authority remains the 2014 *Standards for Educational and Psychological Testing* while revision work proceeds. The Validation Evidence interpretation therefore stays tied to the published edition rather than an unpublished revision.

Morris, White, and Crowther (2019) distinguish deterministic performance-measure calculation from Monte Carlo uncertainty due to finite simulation repetitions. GAP-112 concerns the former: for fixed represented inputs, a one-ULP discrepancy caused by an unnecessary proof fallback is not Monte Carlo error.

### References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019).

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020).

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

## Traceability

- bounded context: Validation Evidence;
- public API: `validation_core::bias_standard_error`;
- exact admission: `crates/validation_core/src/bias_se.rs`;
- established fallback: `crates/validation_core/src/bias.rs`;
- public RED: `crates/validation_core/tests/bias_standard_error_four_observation_reduced_ratio_contract.rs` at `4f2231354fa0ad3e0c646bbb100b3cc83566d033`;
- causal source repair: `ed1a8763c198fbe478de5ec8be72e3436e5918e3`;
- CHANGELOG: `CHANGELOG.d/validation-bias-standard-error-four-observation-reduced-ratio.md` beginning at `28dde0f1e8e3a0f8e8614ed449733b98f2d65c1c`;
- landing vehicle: PR #488;
- predecessor retained: GAP-111 and all inherited Validation Evidence lineages remain in ancestry.
