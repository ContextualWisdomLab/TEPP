# Task 11 — Recovery metrics and Monte Carlo acceptance

## Scope

Task 11 delivers pure CPU `f64` recovery and calibration metrics in `validation_core` for TEPP scientific acceptance under AGENTS.md §9 and ADR 0014:

1. parameter absolute residuals and tolerance match counts;
2. root-mean-square error (RMSE) and delta-method RMSE standard error;
3. mean signed bias and bias standard error;
4. empirical interval coverage with Wilson score bounds;
5. undirected relation-edge precision and recall;
6. pairwise temporal-order accuracy;
7. Monte Carlo replication summaries with nearest-rank percentiles;
8. standard-error-aware acceptance gates (`|estimate − target| ≤ k · SE`);
9. machine-readable JSON and human-readable recovery reports.

These metrics are deterministic reference implementations. They do not replace estimator production paths; they quantify recovery of known synthetic truth.

## Authoritative sources

Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. *Journal of the American Statistical Association, 22*(158), 209–212. https://doi.org/10.1080/01621459.1927.10502953

Casella, G., & Berger, R. L. (2002). *Statistical inference* (2nd ed.). Duxbury.

Efron, B., & Tibshirani, R. J. (1993). *An introduction to the bootstrap*. Chapman & Hall/CRC. https://doi.org/10.1007/978-1-4899-4541-9

Morris, M. D. (1991). Factorial sampling plans for preliminary computational experiments. *Technometrics, 33*(2), 161–174. https://doi.org/10.1080/00401706.1991.10484804

Manning, C. D., Raghavan, P., & Schütze, H. (2008). *Introduction to information retrieval*. Cambridge University Press.

## Formula notes

- **RMSE** = √(mean((recovered − truth)²)); SE uses the delta-method form sd(r²)/(2 · RMSE · √n) with sample SD of squared residuals.
- **Bias** = mean(recovered − truth); SE is the ordinary SEM of the signed differences.
- **Coverage** is the closed-interval hit rate; Wilson bounds use the normal critical value `z` (for example 1.96).
- **Edge precision/recall** operate on normalized undirected edge identities.
- **Temporal-order accuracy** scores pairwise sign agreement, treating exact ties as a distinct class.
- **Monte Carlo** percentiles use inclusive nearest-rank on sorted finite replications.

## Verification

- unit oracle tests for every metric, including empty/unequal/non-finite inputs, inverted intervals, overflow RMSE, single-replication MC, and SE-aware accept/reject;
- foundation recovery study unit test with known loadings, intervals, temporal order, edges, and report serialization;
- workspace line and branch coverage gates must remain complete for production modules.
