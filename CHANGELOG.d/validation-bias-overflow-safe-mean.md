### Fixed

- `validation_core::mean_bias` now computes finite signed residual means with deterministic scale-normalized compensated summation, so a representable extreme bias such as two `f64::MAX` residuals is no longer rejected solely because the raw sum overflows.
- `validation_core::bias_standard_error` reuses the same stable bias mean, allowing constant extreme finite bias to retain its exact zero standard error. Exact cancellation remains zero; a mathematically non-zero bias that falls below binary64 range fails closed rather than being reported as zero.
