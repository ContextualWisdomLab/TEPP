# Bias standard error: dyadic two-level count geometry

## Problem

`validation_core::bias_standard_error` already preserves exact anchor-relative residual translations before evaluating larger-sample dispersion. The remaining two-level path still reconstructed dispersion from translated sums and squared values unless one residual level occurred exactly once. That reconstruction can round an otherwise exact count identity before the final square root.

For an exactly translated sample with `m` observations at residual level `0`, `n-m` observations at residual level `g`, and `n > 1`, the represented-input sample standard error of the mean is

`SE(mean) = |g| * sqrt(m(n-m) / (n^2(n-1)))`.

The identity follows directly from the two-level sample mean and centered squared-deviation sum; no estimated latent quantity or LLM judgment is involved.

## RED

Public RED: `3bc43da21784d3bf2f506c2ffdaa66c90a76d85d` with `crates/validation_core/tests/bias_standard_error_two_level_count_rounding_contract.rs`.

Let `g = next_down(1.0) = 0x1.fffffffffffffp-1`, `n = 16`, and use six residuals at `0` plus ten residuals at `g`. Then

`m(n-m)/(n-1) = 6*10/15 = 4`, so

`SE(mean) = |g| * sqrt(4 / 16^2) = |g| / 8`.

Because division by eight is an exact binary exponent shift for this normal represented gap, the required binary64 result is `0x1.fffffffffffffp-4`, bits `0x3fbf_ffff_ffff_ffff`. The predecessor translated sum/square/square-root path returned `0x1.0000000000000p-3`, bits `0x3fc0_0000_0000_0000`, moving the standard error upward by one ULP. The contract also fixes permutation invariance, sign-mirrored dispersion, and fail-closed behavior when the same nonzero dyadic result would underflow from a minimum-subnormal gap.

The hosted workflows created for the RED commit were cancelled after the branch advanced; they are not represented as completed RED execution evidence. The mathematical oracle and executable public contract remain the reproducer.

## Causal repair

Repair: `77ba10026d23252314f04d95a67ce1cfeb5e54a0`.

After the existing exact translated-residual admission proves two represented residual levels, TEPP now checks the level counts with exact integer arithmetic. A direct path is used only when the count-only factor satisfies

`m(n-m) * d^2 = n^2(n-1)`

for a power-of-two divisor `d`. In that bounded case the requested standard error is exactly `|g| / d`, so the code applies the dyadic scaling before any rounded moment reconstruction. The existing singleton-level identity remains unchanged. Non-singleton two-level samples whose count geometry does not prove a reciprocal power-of-two factor continue through the prior exact-translated second-moment path.

The repair uses checked `u128` count arithmetic. Failure to prove the dyadic relation is a fallback condition, not permission to approximate the count identity. A nonzero exact dyadic result that becomes binary64 zero remains `ValidationError::InvalidInput` under the existing no-false-perfect-recovery policy.

## Alternatives rejected

A general two-level closed form using a rounded floating-point count ratio was rejected for this change. It removes translated-moment cancellation in some cases but introduces a different rounding projection in others and therefore does not establish a stronger represented-input contract without a separate correctly-rounded square-root/product proof.

A payload-specific `n = 16, m = 6` branch was rejected because sample-size constants are not a scientific abstraction. The accepted predicate is the algebraic dyadic count relation itself.

Arbitrary-precision runtime arithmetic was rejected because this bounded binary64 identity is exactly decidable from integer counts plus power-of-two scaling. Adding a second numerical runtime for this case would increase production complexity without changing the estimand.

## Scope and risk

This repair does not claim globally correctly rounded `bias_standard_error` for every `n > 2` sample, nor does it redefine reusable static psychometric estimation owned by `fast-mlsirm`. It changes only TEPP Validation Evidence arithmetic after exact residual translation has already been established. Non-dyadic two-level count factors and general multi-level residual samples retain their prior bounded path and therefore remain candidates for separately demonstrated RED findings rather than implicit expansion of GAP-102.

## Traceability

- Public contract: `crates/validation_core/tests/bias_standard_error_two_level_count_rounding_contract.rs`.
- Production module/API: `crates/validation_core/src/bias.rs` → `bias_standard_error` and `exact_translated_residual_standard_error`.
- RED: `3bc43da21784d3bf2f506c2ffdaa66c90a76d85d`.
- Repair: `77ba10026d23252314f04d95a67ce1cfeb5e54a0`.
- CHANGELOG: `CHANGELOG.d/validation-bias-standard-error-dyadic-two-level-count.md` at `f165914eaceb17780370ca4f2e446c22efcf15ea`.

## References

IEEE Computer Society. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
