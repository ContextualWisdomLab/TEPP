## Validation: recovery-metric attempted denominator

- Added `MonteCarloRecoveryMetricSummary` and `summarize_recovery_metric_replications(...)` so scalar recovery evidence retains attempted, successful, and failed replication counts plus failure rate and Bernoulli Monte Carlo standard error alongside the conditional successful-sample Monte Carlo summary.
- Catastrophic/all-failed experiments keep a valid failure denominator and expose no fabricated scalar metric. Zero attempts, more successful measurements than attempts, non-finite measurements, and invalid percentile bounds fail closed.
- The simulator-backed rolling-origin topic RMSE and mean-absolute parameter-bias contract now consumes this owner, while typed structural experiment errors still abort before any summary is minted.
