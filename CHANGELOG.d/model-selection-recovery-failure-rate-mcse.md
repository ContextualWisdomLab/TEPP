### Scientific recovery failure-rate Monte Carlo error

- `SelectedKRecoverySummary` now reports the Monte Carlo standard error of the empirical failed-replication rate over every attempted replication, `sqrt(p_hat * (1 - p_hat) / R)`.
- Failure-rate uncertainty is unconditional on fit/selection success, while candidate-`K` bias and RMSE remain conditional on successful replications with their existing Monte Carlo standard errors.
- Exact zero failure rate retains exact zero failure-rate Monte Carlo error. This is finite-replication simulation evidence only; it does not claim a confidence interval, rolling-origin leakage safety, or held-out predictive validation. (#680, #682)
