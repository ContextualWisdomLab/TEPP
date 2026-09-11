### Fixed

- `validation_core::summarize_replications` now preserves representable Monte Carlo mean, sample standard deviation, and standard error across full-range finite replication values without relying on an overflowing Welford squared-deviation intermediate.
- Monte Carlo sampling uncertainty is scaled before squaring; a mathematically nonzero standard deviation or standard error that would become exact zero only because it falls below binary64 range fails closed instead of being reported as no simulation uncertainty.
