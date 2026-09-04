# Bias standard error after multi-observation subtraction collapse

## Problem

`validation_core::bias_standard_error` admitted only finite `recovered - truth` residuals, but until GAP-093 it treated the rounded binary64 residual as authoritative for samples larger than two observations. That is not sufficient when distinct represented inputs produce the same rounded subtraction result. A standard error of exactly zero then asserts no sampling dispersion even though the represented-input residuals differ.

The public RED in commit `06a70e1c01629f15f05efef48576a9cadb1f1b98` uses

- `truth = [2^-54, 2^-55, 0]`,
- `recovered = [1, 1, 1]`.

The exact represented-input residuals are

- `r1 = 1 - 2^-54`,
- `r2 = 1 - 2^-55`,
- `r3 = 1`.

All three binary64 subtractions round to `1.0`, so the predecessor's rounded-residual path produced `SE = 0`. The represented-input mean is exactly `1 - 2^-55`. The deviations are therefore `[-2^-55, 0, 2^-55]`; the sample standard deviation is `2^-55`, and the standard error is `2^-55 / sqrt(3)`, which rounds to binary64 bits `0x3c72_79a7_4590_331d`. The sign-mirrored payload has the same standard error. An equal-residual control remains exactly zero.

This is a Validation Evidence defect: zero uncertainty is a materially stronger scientific claim than small but nonzero uncertainty. Morris, White, and Crowther (2019) treat bias and Monte Carlo uncertainty as performance measures whose uncertainty must be reported rather than silently collapsed by implementation arithmetic.

## Constraints

The repair must preserve the existing finite-residual admission gate, the GAP-092 two-observation exact-difference path, and the normal scaled estimator when rounded residuals do not collapse. It must not introduce arbitrary-precision production arithmetic, a second psychometric arithmetic owner, or an O(n²) pairwise-difference fallback on the general path. Reusable static psychometric estimation remains owned by `fast-mlsirm`; this function remains TEPP Validation Evidence arithmetic.

IEEE 754-2019 remains the active IEEE floating-point standard, and ISO/IEC 60559:2020 remains the published international floating-point standard. The repair therefore treats the represented binary64 inputs and their specified arithmetic as the executable numerical boundary rather than assuming real-number subtraction was retained automatically.

## Alternatives considered

A general pairwise-difference variance identity would remove the rounded mean, but evaluating every represented-input pair is O(n²) and unnecessarily widens this repair. Arbitrary-precision rationals would make the oracle straightforward but would add a production dependency and a second numerical path that is not justified by this bounded defect. Returning `InvalidInput` whenever an n>2 subtraction has roundoff would fail closed but would reject a standard error that is both scientifically meaningful and representable.

The selected repair uses the error-free subtraction decomposition already implicit in `subtraction_has_roundoff`. When every rounded residual has the same high part `h`, each exact represented-input residual can be written `r_i = h + l_i`, where `l_i` is the error-free low term. Standard deviation and standard error are translation-invariant, so the common `h` cannot contribute to dispersion. The canonical scaled standard-error path can therefore operate on the `l_i` values in O(n) time. Commit `04c62514a23722d63a62bd5d5af6e3a930cc3147` implements this boundary and leaves the GAP-092 n=2 identity and non-collapsed estimator unchanged.

## Acceptance and traceability

| Evidence | Exact reference | Acceptance meaning |
|---|---|---|
| Public RED | `06a70e1c01629f15f05efef48576a9cadb1f1b98` | Three distinct represented-input residuals collapse to rounded `1.0`; expected nonzero SE is `0x3c72_79a7_4590_331d`. |
| Causal source repair | `04c62514a23722d63a62bd5d5af6e3a930cc3147` | Preserve subtraction low terms and use them only when all rounded residual high parts are equal. |
| CHANGELOG | `56d091544a5780567a6ef772568d62b0fc651747` | Buyer-visible numerical behavior and scope are recorded. |
| PR authority | `#488` | Validation Evidence landing vehicle; exact current head is maintained in the PR body and queue-authority baseline. |
| Module/API | `crates/validation_core/src/bias.rs::bias_standard_error` | Owner-correct production boundary. |
| Public contract | `crates/validation_core/tests/bias_standard_error_multi_observation_subtraction_roundoff_contract.rs` | Positive case, sign mirror, and equal-residual zero control. |

This repair does **not** claim globally correctly rounded standard errors for every finite n>2 input. In particular, when rounded residual high parts differ, retained low terms may still move a nonzero standard error across a final binary64 boundary; that requires an independent represented-input counterexample before widening the production algorithm.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE Standards Association. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
