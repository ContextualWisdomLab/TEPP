### Fixed

- `validation_core::root_mean_square_error` now normalizes finite absolute residuals before squaring, so representable extreme RMSE values such as constant `f64::MAX` residuals are no longer rejected only because an intermediate square overflows.
- Minimum-subnormal recovery error is preserved when the final RMSE remains representable. A mathematically non-zero RMSE that would round to exact zero at the final binary64 boundary fails closed rather than being reported as perfect recovery.
- `validation_core::rmse_standard_error` computes squared-residual variation in the same normalized domain and restores the residual scale only once, preserving exact zero uncertainty for constant extreme residuals and finite uncertainty when raw squared deviations would overflow.
