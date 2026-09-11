### Fixed

- `validation_core::bias_standard_error` now preserves two-observation sampling uncertainty when distinct represented-input residuals both round to the same binary64 `recovered - truth` value. For `n = 2`, subtraction-roundoff cases evaluate the exact represented-input identity `SE = |r₁ - r₂| / 2` through the existing cancellation-safe expanded-sum boundary, while still rejecting unrepresentable individual residuals and preserving exact-equality zero uncertainty.
