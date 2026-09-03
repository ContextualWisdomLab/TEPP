### Fixed

- `validation_core::wilson_coverage_interval` now evaluates the all-covered Wilson lower endpoint as `n / (n + z²)` instead of subtracting nearly equal `O(z²)` center and margin terms, preserving a positive binary64-representable lower bound for large finite critical values rather than collapsing it to exact zero.
