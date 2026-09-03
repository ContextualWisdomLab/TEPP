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

## 2026-09-03 bias arithmetic hardening

Fresh Validation Evidence review found that the protected-main implementation formed mean signed bias by adding raw finite residuals before dividing by `n`. That makes the numerical procedure stricter than the estimand: two finite residuals equal to `f64::MAX` have a representable mean of `f64::MAX`, but their raw sum overflows. The same raw-sum dependency also made the standard error of a constant extreme bias fail even though its sampling variance is exactly zero.

RED `c5ec42e40307f3645c18b0d73114b73e01745a20` fixes this contract through the public `mean_bias` and `bias_standard_error` APIs. Causal repair `7499042f7451b2e3d5e9f83843aeea82c4f5ff06` validates each signed residual, normalizes by the largest residual magnitude, uses deterministic compensated summation, and restores scale only after dividing by the replication count. Exact cancellation is canonical `+0.0`; a represented non-zero normalized mean that becomes `0.0` only when scaled back fails closed rather than being reported as zero bias. The bias-SE sample variance remains fail-closed when a squared deviation is itself outside binary64 range; that is a separate numerical contract rather than part of this repair.

This is a Validation Evidence implementation repair, not a change to the bias estimand, estimator target, or longitudinal domain semantics, so it does not require a new PRD target or ADR. Morris, White, and Crowther's ADEMP guidance treats bias as an explicitly defined simulation performance measure against known truth; implementation overflow must not silently redefine when that measure exists. IEEE/ISO/IEC 60559-2020, the active international adoption of IEEE 754-2019 as of 2026-09-03, supplies the floating-point execution model. IEEE has an active P754 revision project approved in 2024, but that project is not substituted for the published active standard.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

IEEE Computer Society. (2020). *IEEE/ISO/IEC 60559-2020: ISO/IEC/IEEE international standard—Floating-point arithmetic*. IEEE Standards Association. https://standards.ieee.org/ieee/60559/10226/

## Verification

- unit oracle tests for every metric, including empty/unequal/non-finite inputs, inverted intervals, overflow RMSE, single-replication MC, and SE-aware accept/reject;
- foundation recovery study unit test with known loadings, intervals, temporal order, edges, and report serialization;
- `crates/validation_core/tests/bias_overflow_safe_mean_contract.rs` exercises the representable `f64::MAX` constant-bias case through the public API;
- exact cancellation and non-zero-bias underflow are distinct contracts: cancellation remains zero, while an unrepresentable positive mean fails closed;
- workspace line and branch coverage gates must remain complete for production modules.
