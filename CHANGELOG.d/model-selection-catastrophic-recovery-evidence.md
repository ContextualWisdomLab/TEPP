### Scientific validation

- `SelectedKRecoverySummary` now remains constructible when zero or one replication succeeds, so catastrophic and near-catastrophic fitting/selection failure cannot disappear behind `InsufficientRecoveryReplications`. Failure counts/rates always retain the full non-empty attempted denominator; conditional bias/RMSE are `None` with zero successes, and conditional Monte Carlo standard errors are `None` until at least two successes exist. No NaN or zero sentinel is fabricated. See #696.
