# Common-high subtraction low terms must not inherit a rounded-mean dispersion geometry

## Problem

`bias_standard_error` already retained error-free subtraction low terms when several represented-input residuals rounded to one common binary64 high part. The predecessor then computed a representable mean of those low terms and centered each low term on that rounded mean before evaluating dispersion.

For `truth = [2^-54, 0, 0]` and `recovered = [1, 1, 1]`, all three binary64 subtractions return `1.0`, but the represented-input residuals are exactly `[1 - 2^-54, 1, 1]`. The common high part is translation-invariant and contributes no dispersion. The retained low terms are `[-2^-54, 0, 0]`, whose represented-input standard error is exactly `2^-54 / 3` before final binary64 rounding, bits `0x3c75_5555_5555_5555`.

The predecessor first rounded the low-term mean `-2^-54 / 3`, then formed three deviations from that rounded mean. That changes the represented dispersion geometry and returns the adjacent higher binary64 value. The sign-mirrored input has the same standard error and exposes the same defect.

## Constraints and rejected alternatives

The repair stays inside TEPP Validation Evidence. It does not create a reusable psychometric estimator, copy fast-mlsirm arithmetic, or change Longitudinal Modeling composition. It also does not claim globally correctly rounded standard errors for arbitrary binary64 samples.

Replacing the whole standard-error implementation with arbitrary-precision production arithmetic was rejected because the counterexample only requires preserving a translation-invariant represented-input identity already used by the current Validation path. Keeping the rounded low-term mean as authoritative was rejected because the public RED proves that its intermediate rounding changes the target quantity. Removing the common-high path altogether was also rejected: when low-term anchor differences cannot be proven exactly representable, the existing bounded fallback still preserves earlier nonzero-dispersion behavior better than collapsing to the rounded high parts.

## Decision

When all rounded residual highs are equal and subtraction roundoff is present, first run the existing exact translated-residual second-moment calculation over the retained low terms with zero secondary roundoff. This path is admitted only when every anchor-relative low-term delta is exactly representable under the same proof used by GAP-095/096. If that proof fails, retain the predecessor scaled low-term mean/deviation fallback.

Public contract: `crates/validation_core/tests/bias_standard_error_common_high_mean_roundoff_contract.rs`.

RED commit: `d48e2515d62dfbe0a807b5dba40fbb7034d4fa9d`.

Causal repair: `a63b8d7a8e79146cbb17ceb17855bcca312535a1`.

CHANGELOG: `6fc8e82839389f2f8c07e1cf7ba78a29f19d2510`.

## Scientific trace

Bias and its Monte Carlo uncertainty are performance measures against known truth; preserving the represented-input statistic is therefore part of Validation Evidence rather than an LLM or projection judgment. Binary64 rounding behavior follows the published IEEE floating-point arithmetic contract.

IEEE Computer Society. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

## Remaining risk

This repair proves the common-high counterexample and its sign mirror only under exact anchor-relative low-term translation. If those translated low-term deltas themselves require rounding, the bounded predecessor fallback remains. A wider change requires a separate represented-input counterexample and RED rather than extrapolation from this case.
