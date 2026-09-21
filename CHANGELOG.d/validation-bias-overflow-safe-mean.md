### Fixed

- `validation_core::mean_bias` now computes finite signed residual means without allowing either raw-sum overflow or largest-scale normalization to erase a representable low-order bias after extreme mixed-sign cancellation. Opposite signs cancel at represented magnitude first; the remaining one-sign mass is power-of-two normalized and divided by the original recovery denominator before scale restoration.
- `validation_core::bias_standard_error` reuses the same stable bias mean and scales deviations before squaring, forming the SEM directly rather than materializing an avoidably overflowing raw square sum or sample variance. Constant extreme finite bias retains its exact zero standard error, and representable non-constant extreme cases remain measurable.
- Exact cancellation remains zero; a mathematically non-zero mean or standard error that would become false zero only because it falls below binary64 range fails closed rather than being reported as perfect recovery.
