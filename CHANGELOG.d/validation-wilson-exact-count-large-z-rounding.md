### Fixed

- `validation_core::wilson_coverage_interval` now preserves the exactly represented finite sample-count contribution when an all-covered Wilson endpoint has an exactly representable `n` but large finite `z²` completely absorbs `n` in the rounded denominator `n + z²`. The exact-count path recovers the addition residual and applies a fused quotient-residual correction. A later partial-denominator repair extends the same compensated denominator mechanism to ordinary inexact `n + z²` sums; the near-one complementary repair and the inexact-`u64` path remain separate boundary contracts.
