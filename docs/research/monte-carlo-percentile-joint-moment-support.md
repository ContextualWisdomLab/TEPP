# Joint empirical-percentile moment support

## Scientific contract

`MonteCarloSummary` stores a represented binary64 sample mean `m`, sample standard deviation `s` with denominator `n - 1`, and inclusive nearest-rank percentile endpoints selected from the retained replications. The generic carrier is intentionally sign-neutral because it is used for signed metrics such as bias as well as nonnegative metrics such as RMSE.

For the producer's represented-mean deviations `d_i = x_i - m`, the stored spread satisfies

`(n - 1) s^2 = sum_i d_i^2`.

The existing endpoint-support contract correctly requires each retained percentile endpoint to satisfy `|d| <= s * sqrt(n - 1)`. That condition is necessary but not sufficient when the lower and upper endpoints are numerically distinct. Distinct endpoint values must come from distinct retained observations, so both squared deviations consume the same finite sample budget:

`(percentile_lower - m)^2 + (percentile_upper - m)^2 <= (n - 1) s^2`.

If the two endpoint values are numerically equal, they may designate the same retained observation or duplicate observations with the same represented value, so admission conservatively counts that value once rather than inventing a second observation.

The attainable fixture `[0.75, 0.75, 0.75, 1.75]` has `n = 4`, represented mean `1.0`, sample SD `0.5`, SE `0.25`, and distinct endpoints `0.75` and `1.75`. Its joint endpoint contribution is `0.25^2 + 0.75^2 = 0.625`, within the total deviation budget `3 * 0.5^2 = 0.75`.

By contrast, a summary with the same `n`, mean, SD, and SE but endpoints `0.25` and `1.75` passes the predecessor's separate endpoint checks because each absolute deviation is `0.75 < 0.5 * sqrt(3)`. It is nevertheless impossible: the two observed endpoints alone require `0.75^2 + 0.75^2 = 1.125`, exceeding the entire recorded deviation budget `0.75` before any other retained replication is considered.

## Numerical implementation

The admission check avoids raw full-range subtraction and squaring. It divides mean, both endpoints, and SD by one shared finite magnitude, combines the two normalized deviations with binary64 `hypot`, and compares that norm with the normalized `SD * sqrt(n - 1)` support using the same small relative tolerance as the individual endpoint contract. This preserves the represented-mean semantics already adopted by the predecessor repair and does not assume that deviations from the rounded binary64 mean sum to exact real zero.

## RED, causal repair, and traceability

- Public RED `c4a13826144708582a73ec4458967bb103287338` adds `crates/validation_core/tests/monte_carlo_percentile_joint_moment_support_contract.rs`. It preserves an attainable joint-support fixture, rejects the individually-valid-but-jointly-impossible `(0.25, 1.75)` endpoints, verifies JSON egress/ingress fail closed, and preserves equal-endpoint admission without double-counting one represented value.
- Causal production repair `cb3f80a2ff3439d238d1bd6e674ef789d25f36c7` extends `MonteCarloSummary::validate` with the distinct-endpoint joint deviation budget while retaining the predecessor's represented-mean individual endpoint rule.
- Changelog trace `f727450d7a68d253254d3f0a8ae8d305a08137eb` records the durable-evidence repair.

This is Validation Evidence artifact admission in TEPP. It does not create a new psychometric estimator, does not change Longitudinal Modeling composition, and does not relocate reusable static psychometric arithmetic from fast-mlsirm.

## Methodological reference

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

Morris et al. distinguish estimands, performance measures, and Monte Carlo uncertainty and recommend considering performance measures jointly. TEPP's validation boundary therefore treats a summary as a coherent evidence artifact rather than validating each reported scalar in isolation.
