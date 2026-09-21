### Fixed

- Validation Evidence now rejects `monte_carlo_rmse` summaries whose empirical nearest-rank percentile endpoint exceeds the finite support available to `n` nonnegative RMSE replications with the recorded mean. The typed report boundary evaluates the support as `percentile_upper / mean <= replication_count` with a small binary64 tolerance, preserving the attainable `[0, ..., 0, n*mean]` boundary without materializing an overflow-prone sample sum. Generic `MonteCarloSummary` remains sign-neutral for metrics such as bias.
