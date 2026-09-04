### Fixed

- `validation_core::wilson_coverage_interval` now preserves the exactly represented finite sample-count contribution when an all-covered Wilson endpoint has an exactly representable `n` but large finite `z²` completely absorbs `n` in the rounded denominator `n + z²`. The exact-count path recovers the addition residual and applies a fused quotient-residual correction only on that absorption boundary; ordinary direct evaluation, the near-one complementary repair, and the inexact-`u64` path remain unchanged.
