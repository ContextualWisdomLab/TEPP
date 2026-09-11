### Fixed

- `validation_core::wilson_coverage_interval` now preserves a representable all-covered Wilson miss mass when an exactly representable sample count and small positive `z²` make the direct `n / (n + z²)` denominator round back to `n`. The repair is boundary-local: it subtracts the algebraically equivalent `z² / (n + z²)` only when the direct lower endpoint has spuriously collapsed to exact `1.0`, while smaller uncertainty that is genuinely below binary64 resolution remains `1.0`.
