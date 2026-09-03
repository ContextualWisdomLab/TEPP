### Fixed

- `validation_core::wilson_coverage_interval` now evaluates the all-covered Wilson lower endpoint as `n / (n + z²)` instead of subtracting nearly equal `O(z²)` center and margin terms, preserving a positive binary64-representable lower bound for large finite critical values rather than collapsing it to exact zero.
- Strict-interior coverage now falls back to an algebraically rationalized, `z²`-normalized lower-root expression when the generic `center - margin` evaluation collapses a positive representable Wilson lower endpoint to exact zero at extreme finite critical values.
- All-uncovered and strict-interior coverage now use the same rationalized positive-lower calculation on the complementary uncovered proportion when the generic `center + margin` path falsely rounds a Wilson upper endpoint to exact `1.0`. Ordinary upper endpoints remain on the direct path when no false-one collapse occurs.
