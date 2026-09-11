### Fixed

- `validation_core::bias_standard_error` now evaluates the exact two-observation identity `SE(mean) = |r₁-r₂| / 2` for exact represented residuals as well as subtraction-roundoff cases. This prevents a rounded two-point residual mean from changing the dispersion geometry before the standard error is formed.
- The public contract covers the adjacent-binary64 pair `[1, next_down(1)]`, its sign mirror, and an equal-residual zero-uncertainty control.
