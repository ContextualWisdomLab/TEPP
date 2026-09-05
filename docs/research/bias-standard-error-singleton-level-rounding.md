# Bias standard-error singleton-level rounding

## Decision scope

This note records a Validation Evidence numerical finding for `validation_core::bias_standard_error`. It does not create a reusable psychometric estimator and does not move static psychometric arithmetic from the fast-mlsirm owner. The relevant scientific estimand is the standard error of the mean signed recovery bias under the existing independent-observation contract.

## Finding

Let `a = next_down(1.0) = 0x1.fffffffffffffp-1`, with represented inputs

- `truth = [0, 0, 0, 0]`
- `recovered = [0, a, a, a]`.

All four signed residuals and the anchor-relative translated residuals are exactly representable binary64 values. For the two-level sample `[0, a, a, a]`, the sample mean is `3a/4`, the sum of squared centered residuals is `3a²/4`, and therefore

`SE(mean) = sqrt((3a²/4) / (4 × 3)) = |a| / 4`.

The correctly represented result is `0x3fcf_ffff_ffff_ffff`. The predecessor exact-translated second-moment path normalizes, squares, forms the second-moment numerator, divides, and takes a square root; for this payload it returns adjacent lower `0x3fcf_ffff_ffff_fffe`. This is a one-ULP decision-quality defect despite every translated residual being exact.

Public RED `4386d9ace83cd54aa129067ddc589b1a628147a2` adds `crates/validation_core/tests/bias_standard_error_singleton_level_rounding_contract.rs` and fixes the expected represented result across the original ordering, a permutation, and a sign mirror.

## Causal repair

Repair `79ad03fae4364d6c364915a062eb0fc8615eaa43` remains inside `exact_translated_residual_standard_error` after the existing exact anchor-relative translation proof. If the translated sample has exactly two represented levels and either level occurs once, the sample standard error simplifies for any supported `n` to `|level_gap| / n`; the implementation evaluates that identity directly and retains fail-closed behavior when a nonzero represented gap would divide below binary64 range.

The repair intentionally does not claim globally correctly rounded `n > 2` standard errors. Samples with three or more represented levels, two-level samples without a singleton, and cases that fail the exact translation admission continue through the existing bounded second-moment or fallback paths and require their own represented-input counterexample before any broader change.

## Alternatives considered

Applying arbitrary-precision arithmetic to every Validation metric was rejected because it widens the production arithmetic contract far beyond the demonstrated defect and adds a new runtime dependency. Special-casing only the four-row payload was rejected because the algebraic identity is determined by the singleton/two-level structure rather than by `n = 4`. Leaving the generic square/root path unchanged was rejected because the public contract can reproduce a deterministic one-ULP error from exact represented residuals.

## Traceability

- Bounded context: Validation Evidence.
- Production API: `validation_core::bias_standard_error`.
- Production module: `crates/validation_core/src/bias.rs`.
- Public regression: `crates/validation_core/tests/bias_standard_error_singleton_level_rounding_contract.rs`.
- RED: `4386d9ace83cd54aa129067ddc589b1a628147a2`.
- Causal repair: `79ad03fae4364d6c364915a062eb0fc8615eaa43`.
- CHANGELOG: `017ad11ad974219a5a0e1cf91c1ecf55c44524c2`.
- Landing vehicle: PR #488.

IEEE 754 binary floating-point semantics remain the numerical representation authority. Known-truth recovery metrics remain performance measures rather than LLM judgments; simulation acceptance must continue to report the estimand, bias/dispersion behavior, and Monte Carlo uncertainty.

## References

Institute of Electrical and Electronics Engineers. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). IEEE. https://doi.org/10.1109/IEEESTD.2019.8766229

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
