### Fixed

- Validation Evidence `bias_standard_error` now preserves the exact translated two-level identity `SE(mean) = |level_gap| / n` when either residual level occurs once. This prevents the general square/second-moment/square-root path from moving an exactly represented four-observation singleton/repeated-level result by one binary64 ULP while retaining the existing fail-closed behavior for an unrepresentable nonzero standard error.
