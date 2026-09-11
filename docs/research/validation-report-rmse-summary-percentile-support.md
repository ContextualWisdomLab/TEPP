# Monte Carlo RMSE percentile support

## Decision

`ValidationReport::monte_carlo_rmse` is a typed Validation Evidence slot for retained RMSE replications, so every underlying replication is nonnegative even though the reusable `MonteCarloSummary` carrier must remain sign-neutral for other metrics such as bias.

For retained values `x_i >= 0`, replication count `n`, and represented mean `m`, the finite sample sum is `sum(x_i) = n*m` in the mathematical estimand. Since no nonnegative member can exceed the sum, every retained value satisfies `x_i <= n*m`. `summarize_replications` uses inclusive nearest-rank endpoints selected from the sorted retained values rather than an extrapolating quantile model, so every stored empirical percentile endpoint must satisfy the same support bound.

The artifact-admission check evaluates the upper endpoint as

`percentile_upper / mean <= replication_count`

for positive mean, with a small binary64 relative tolerance. This form avoids requiring the finite product `n*mean` to be representable merely to validate an otherwise finite endpoint. Exact-zero mean remains governed by the stronger perfect-recovery invariant that spread, standard error, and percentile support are all numeric zero. The bound is attainable: `[0, 0, 0, 4]` has `n=4`, `mean=1`, and maximum/100th-percentile endpoint `4 = n*mean`.

## RED and causal repair

- RED `84a200ee451247c8f75fcce322aa7fc558f38c43` adds `validation_report_rmse_summary_percentile_support_contract.rs`. A generic-valid summary with `n=4`, `mean=1`, coherent `SD=0.5` / `SE=0.25`, and empirical upper endpoint `5` is impossible for nonnegative replications but predecessor report admission accepted it.
- Causal repair `04c9cdd4b89d37145853316bb419943735830c79` adds the typed percentile-support admission rule without narrowing `MonteCarloSummary` globally.
- Changelog trace `00b9278a2c1bc1b912ed7c945391afe71f56bf55` records the buyer-visible durable-evidence correction.

This is TEPP Validation Evidence artifact admission and projection. It does not introduce a psychometric estimator, change Longitudinal Modeling composition, or copy reusable arithmetic from fast-mlsirm.

## Methodological trace

Monte Carlo performance summaries need their reported measures and Monte Carlo uncertainty to remain coherent with the simulated estimand and retained replications. The typed support check operationalizes that requirement at TEPP's durable-artifact boundary rather than asking an LLM or downstream report renderer to infer whether a finite payload is scientifically possible.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
