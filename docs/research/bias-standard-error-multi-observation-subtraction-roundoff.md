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

All three binary64 subtractions round to `1.0`, so the predecessor's rounded-residual path produced `SE = 0`. The represented-input mean is exactly `1 - 2^-55`. The deviations are therefore `[-2^-55, 0, 2^-55]`; the sample standard deviation is `2^-55`, and the standard error is `2^-55 / sqrt(3)`.

The first public contract accidentally encoded `0x3c72_79a7_4590_331d`, which is the result of evaluating `1 / rounded_sqrt(3)` and then restoring the exact power-of-two scale. Exact high-precision evaluation of the represented-input expression shows that the correctly rounded final binary64 value is instead `0x3c72_79a7_4590_331c` (`0x1.279a74590331cp-56`). Oracle correction `6224320410ccabb1cf16d36cc12f88e2b7a05bb1` makes that distinction executable. This exposed GAP-094: the predecessor standard-error helper separately rounded `sqrt(sample_variance)` and `sqrt(n)`, moving the final standard error by one ULP even after GAP-093 restored the missing low-order dispersion.

This is a Validation Evidence defect: zero uncertainty is materially stronger than small but nonzero uncertainty, and a one-ULP numerical boundary is not interchangeable with the represented-input target when the deterministic CPU `f64` reference claims that boundary. Morris, White, and Crowther (2019) treat bias and simulation uncertainty as performance measures whose uncertainty must be reported rather than silently collapsed by implementation arithmetic.

## Constraints

The repair must preserve the existing finite-residual admission gate, the GAP-092 two-observation exact-difference path, and the normal scaled estimator when rounded residuals do not collapse. It must not introduce arbitrary-precision production arithmetic, a second psychometric arithmetic owner, or an O(n²) pairwise-difference fallback on the general path. Reusable static psychometric estimation remains owned by `fast-mlsirm`; this function remains TEPP Validation Evidence arithmetic.

IEEE 754-2019 remains the active IEEE floating-point standard, and ISO/IEC 60559:2020 remains the published international floating-point standard. The repair therefore treats represented binary64 inputs as the executable numerical boundary and avoids an algebraically unnecessary intermediate rounding when the equivalent normalized expression is bounded.

## Alternatives considered

A general pairwise-difference variance identity would remove the rounded mean, but evaluating every represented-input pair is O(n²) and unnecessarily widens this repair. Arbitrary-precision rationals would make the oracle straightforward but would add a production dependency and a second numerical path that is not justified by this bounded defect. Returning `InvalidInput` whenever an n>2 subtraction has roundoff would fail closed but would reject a standard error that is both scientifically meaningful and representable.

The GAP-093 repair uses the error-free subtraction decomposition already implicit in `subtraction_has_roundoff`. When every rounded residual has the same high part `h`, each exact represented-input residual can be written `r_i = h + l_i`, where `l_i` is the error-free low term. Standard deviation and standard error are translation-invariant, so the common `h` cannot contribute to dispersion. Commit `04c62514a23722d63a62bd5d5af6e3a930cc3147` therefore evaluates the scaled standard error from the `l_i` values in O(n), leaving the GAP-092 n=2 identity and non-collapsed estimator unchanged.

For GAP-094, keeping `sqrt(sample_variance) / sqrt(n)` was rejected because it rounds two square-root operands before the final division. The normalized deviations are bounded by one, so the equivalent `sqrt(sum(d²) / (n * (n - 1)))` can be evaluated without overflow and with one final square root. Commit `8b8f0a21ccc825f355859cadbf20d83f04d2369f` applies that form inside the existing scaled standard-error helper; it does not add a new estimator or weaken fail-closed handling.

## Acceptance and traceability

| Evidence | Exact reference | Acceptance meaning |
|---|---|---|
| GAP-093 public RED | `06a70e1c01629f15f05efef48576a9cadb1f1b98` | Three distinct represented-input residuals collapse to rounded `1.0`; predecessor false zero is rejected. |
| GAP-093 source repair | `04c62514a23722d63a62bd5d5af6e3a930cc3147` | Preserve subtraction low terms and use them only when all rounded residual high parts are equal. |
| GAP-094 oracle correction / RED | `6224320410ccabb1cf16d36cc12f88e2b7a05bb1` | Correct expected represented-input SE from `...331d` to `...331c`, exposing the separate square-root double rounding. |
| GAP-094 source repair | `8b8f0a21ccc825f355859cadbf20d83f04d2369f` | Form normalized SE as `sqrt(sum(d²)/(n(n-1)))` before exact scale restoration. |
| GAP-093 CHANGELOG | `56d091544a5780567a6ef772568d62b0fc651747` | Buyer-visible false-zero repair and scope are recorded. |
| PR authority | `#488` | Validation Evidence landing vehicle; exact current head is maintained in the PR body and queue-authority baseline. |
| Module/API | `crates/validation_core/src/bias.rs::bias_standard_error` | Owner-correct production boundary. |
| Public contract | `crates/validation_core/tests/bias_standard_error_multi_observation_subtraction_roundoff_contract.rs` | Correctly rounded positive case, sign mirror, and equal-residual zero control. |

These repairs do **not** claim globally correctly rounded standard errors for every finite n>2 input. In particular, when rounded residual high parts differ, retained subtraction low terms may still move a nonzero standard error across a final binary64 boundary; that requires an independent represented-input counterexample before widening the production algorithm.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

IEEE Standards Association. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

International Organization for Standardization. (2020). *Information technology—Microprocessor systems—Floating-point arithmetic* (ISO/IEC 60559:2020). https://www.iso.org/standard/80985.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
