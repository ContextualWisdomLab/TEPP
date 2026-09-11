### Fixed

- Reject `ValidationReport::monte_carlo_rmse` payloads whose exact-zero mean is paired with nonzero spread, standard error, or empirical percentile endpoints; for nonnegative RMSE replications, a zero mean is exact perfect recovery in every retained replication. The report fixture now also uses the canonical `SD / sqrt(n)` Monte Carlo standard error required by the generic summary contract.
