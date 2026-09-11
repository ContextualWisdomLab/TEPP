### Fixed

- `MonteCarloSummary` now rejects empirical nearest-rank percentile endpoints that cannot coexist with the recorded sample mean, sample standard deviation, and replication count. Because the canonical sample SD is the square root of the represented-mean squared-deviation sum divided by `n - 1`, every retained endpoint must satisfy `|x - mean| <= SD * sqrt(n - 1)`. Admission evaluates that support on a shared scale so full-range signed summaries do not overflow validation-only arithmetic, while adjacent-binary64 samples whose represented mean rounds remain admissible.
