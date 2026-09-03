### Fixed

- `validation_core::mean_bias` now computes finite signed residual means with deterministic scale-normalized compensated summation, so a representable extreme bias such as two `f64::MAX` residuals is no longer rejected solely because the raw sum overflows.
- `validation_core::bias_standard_error` reuses the same stable bias mean and scales deviations before squaring, forming the SEM directly rather than materializing an avoidably overflowing raw square sum or sample variance. Constant extreme finite bias retains its exact zero standard error, and representable non-constant extreme cases remain measurable.
- Exact cancellation remains zero; a mathematically non-zero mean or standard error that would become false zero only because it falls below binary64 range fails closed rather than being reported as perfect recovery.
