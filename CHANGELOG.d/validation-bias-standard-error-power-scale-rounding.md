### Fixed

- `validation_core::bias_standard_error` now keeps exact translated residual geometry on a power-of-two normalization scale before evaluating the translated second moment.
- This prevents an exactly represented gap such as `5 * 2^-52` from being converted into `rounded(1/3) * gap`, which can move the final `SE(mean)` by one ULP when the non-power scale is restored.
- The public contract covers the sign-mirrored three-observation boundary; cases that cannot prove exact translated residual deltas still retain the predecessor bounded fallback.
