### Fixed

- `MonteCarloSummary` now rejects empirical nearest-rank percentile endpoints that cannot coexist with the recorded sample mean, sample standard deviation, and replication count. For any retained observation, `|x - mean| <= SD * (n - 1) / sqrt(n)`; admission evaluates that finite-sample support on a shared scale so full-range signed summaries do not overflow validation-only arithmetic.
