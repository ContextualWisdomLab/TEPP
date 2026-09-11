## Fixed

- Validation Evidence now rejects `monte_carlo_rmse` summaries whose sampling spread cannot arise from nonnegative RMSE replications. For retained RMSE values `x_i >= 0` with sample mean `m`, the sample standard deviation satisfies `SD <= sqrt(n) * m`, so the standard error of the replication mean cannot exceed `m`. The generic `MonteCarloSummary` remains sign-neutral for metrics such as bias; this support check is applied only when the summary occupies the RMSE-specific report slot.
