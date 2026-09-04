# Monte Carlo percentile moment support

## Decision

`MonteCarloSummary` is a reusable Validation Evidence carrier for scalar Monte Carlo metrics. Its empirical percentile endpoints are produced by `summarize_replications` with an inclusive nearest-rank rule, so each endpoint is one of the retained observations rather than an extrapolated quantile estimate.

Let retained values be `x_1, ..., x_n`, represented sample mean be `m`, and sample standard deviation with denominator `n - 1` be `s`. For one retained observation define `d_j = x_j - m`. The remaining deviations sum to `-d_j`. By the Cauchy–Schwarz inequality,

`sum_{i != j} d_i^2 >= d_j^2 / (n - 1)`.

Therefore

`(n - 1) s^2 = sum_i d_i^2 >= d_j^2 * n / (n - 1)`,

which gives the finite-sample support bound

`|x_j - m| <= s * (n - 1) / sqrt(n)`.

Every inclusive nearest-rank percentile endpoint is a retained `x_j`, so the same bound is required of `percentile_lower` and `percentile_upper`. The bound is attainable: `[0.75, 0.75, 0.75, 1.75]` has `n=4`, `mean=1`, sample `SD=0.5`, and endpoint deviation `0.75 = 0.5 * 3 / 2`.

The admission comparison normalizes `mean`, endpoint, and `SD` by a shared finite scale before subtraction and multiplication. This prevents opposite-sign full-range binary64 values from overflowing merely because Validation Evidence is being checked. A small relative binary64 tolerance is allowed at the support boundary; the rule does not require cross-language bit-for-bit equality.

This support law is generic. It does not impose RMSE nonnegativity on `MonteCarloSummary`; signed summaries such as bias remain valid when their retained empirical endpoints fit the represented moments.

## RED and causal repair

- Public RED `40acb4f6f51dd9d7074c652fb6448eaa942b95ac` adds `monte_carlo_percentile_moment_support_contract.rs`. The predecessor accepted `n=4`, `mean=1`, `SD=0.5`, `SE=0.25`, `percentile_lower=0.75`, `percentile_upper=2.0` even though no retained sample with those moments can contain `2.0`. The attainable `1.75` boundary and a valid signed summary are preserved in the same contract.
- Causal repair `2798e4f92dbb30019e2b1288e59d09564ae73a70` enforces the finite-sample endpoint support inside `MonteCarloSummary::validate` using scale-normalized arithmetic.
- Changelog trace `e6c5b3d98491ffedd104fce7db677e04acc3b3f1` records the durable-evidence correction.

This is TEPP Validation Evidence artifact admission. It does not introduce or relocate a psychometric estimator, change Longitudinal Modeling composition, or consume mutable fast-mlsirm source.

## Methodological trace

Simulation evidence is useful only when reported performance measures and Monte Carlo uncertainty are interpretable as summaries of realizable retained replications. Rejecting moment-incompatible empirical endpoints keeps the durable artifact aligned with the sample it claims to summarize rather than relying on a downstream renderer or LLM to infer scientific plausibility.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
