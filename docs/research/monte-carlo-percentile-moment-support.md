# Monte Carlo percentile moment support

## Decision

`MonteCarloSummary` is a reusable Validation Evidence carrier for scalar Monte Carlo metrics. Its empirical percentile endpoints are produced by `summarize_replications` with an inclusive nearest-rank rule, so each endpoint is one of the retained observations rather than an extrapolated quantile estimate.

Let retained values be `x_1, ..., x_n`, represented binary64 mean be `m`, and the canonical sample standard deviation with denominator `n - 1` be `s`. The producer computes deviations `d_i = x_i - m` and therefore records

`(n - 1) s^2 = sum_i d_i^2`.

Every retained observation contributes one nonnegative squared deviation to that sum, so for each `j`,

`d_j^2 <= (n - 1) s^2`,

and therefore

`|x_j - m| <= s * sqrt(n - 1)`.

Every inclusive nearest-rank percentile endpoint is a retained `x_j`, so `percentile_lower` and `percentile_upper` must satisfy the same support bound. For the public fixture `n=4`, `mean=1`, `SD=0.5`, the support radius is `0.5 * sqrt(3) < 1`; an endpoint of `2.0` is therefore impossible even though the mean, SD, SE, and endpoint are individually finite.

A stronger textbook bound based on zero-sum deviations was considered and rejected for artifact admission. TEPP deliberately stores a represented binary64 mean, and the mathematical sample mean can lie between adjacent binary64 values. The edge fixture `[1.0, next_up(1.0)]` exercises that projection: validation must use the squared-deviation identity actually implemented by the producer rather than assume that deviations from the represented mean sum to exact real zero.

The admission comparison normalizes `mean`, endpoint, and `SD` by a shared finite scale before subtraction and multiplication. This prevents opposite-sign full-range binary64 values from overflowing merely because Validation Evidence is being checked. A small relative binary64 tolerance is allowed at the support boundary; the rule does not require cross-language bit-for-bit equality.

This support law is generic. It does not impose RMSE nonnegativity on `MonteCarloSummary`; signed summaries such as bias remain valid when their retained empirical endpoints fit the represented moments.

## RED, review, and causal repair

- Public RED `40acb4f6f51dd9d7074c652fb6448eaa942b95ac` adds `monte_carlo_percentile_moment_support_contract.rs`. The predecessor accepted `n=4`, `mean=1`, `SD=0.5`, `SE=0.25`, `percentile_lower=0.75`, `percentile_upper=2.0` even though the recorded squared-deviation budget cannot contain that endpoint.
- Initial repair `2798e4f92dbb30019e2b1288e59d09564ae73a70` exposed an over-strong zero-sum assumption during immediate self-review. It was not treated as completion evidence.
- Causal correction `c7151b498ccbd562e7945a12a53c55472d93acac` bases admission on the producer's represented-mean squared-deviation identity, `|x - mean| <= SD * sqrt(n - 1)`.
- Edge reinforcement `dbef285b6348cf691bbb72c25350912a5463e11e` proves that adjacent-binary64 samples remain admissible when the represented mean rounds.
- Changelog correction `c38a320c730875919a8bff58de42e2db859c248d` records the final durable-evidence contract.

This is TEPP Validation Evidence artifact admission. It does not introduce or relocate a psychometric estimator, change Longitudinal Modeling composition, or consume mutable fast-mlsirm source.

## Methodological trace

Simulation evidence is useful only when reported performance measures and Monte Carlo uncertainty are interpretable as summaries of realizable retained replications. Rejecting moment-incompatible empirical endpoints keeps the durable artifact aligned with the sample it claims to summarize rather than relying on a downstream renderer or LLM to infer scientific plausibility.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
