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

RED `c5ec42e40307f3645c18b0d73114b73e01745a20` fixes this contract through the public `mean_bias` and `bias_standard_error` APIs. Causal repair `7499042f7451b2e3d5e9f83843aeea82c4f5ff06` validates each signed residual, normalizes by the largest residual magnitude, uses deterministic compensated summation, and restores scale only after dividing by the replication count. Exact cancellation is canonical `+0.0`; a represented non-zero normalized mean that becomes `0.0` only when scaled back fails closed rather than being reported as zero bias.

Review of that repair exposed a second avoidable intermediate: bias SE still squared unscaled deviations and accumulated those raw squares. A final SEM can be representable even when the raw sum of squared deviations or the intermediate sample variance is not. Public RED `7de0ef90944925ae7b232a8280f5bf9096df6502` uses signed residuals `[1e154, -1e154, 0]`: the raw square sum overflows, while the intended sample SEM is finite at approximately `1e154 / sqrt(3)`. Causal repair `cad231620679d8f912bded36c654446032b45e57` scales finite deviations before squaring and forms the SEM directly, avoiding unnecessary materialization of an overflowing variance. If subtraction from the finite bias mean itself overflows, the reference falls back to a scale-normalized deviation calculation. Oracle/edge refinements `8a6cc346d0b058340285c4172bc14c42c0cdbfa5` and `28d96c2315c58db5336292e000c9f6132cff2621` retain a one-ULP-tolerant public oracle while covering constant extreme bias, `f64::MAX` opposite residuals, direct-deviation overflow, exact cancellation, and non-zero mean underflow.

These are Validation Evidence implementation repairs, not changes to the bias estimand, estimator target, or longitudinal domain semantics, so they do not require a new PRD target or ADR. Morris, White, and Crowther's ADEMP guidance treats bias and uncertainty as explicitly defined simulation performance measures against known truth; avoidable intermediate overflow must not silently redefine whether those measures exist.

## 2026-09-03 RMSE arithmetic hardening

The same review was applied independently to generic RMSE rather than copying the bias estimator. Protected-main `root_mean_square_error` squared each finite absolute residual before normalization. A residual of `f64::MAX` is itself representable, and two identical such residuals have a representable RMSE of exactly `f64::MAX`, but the predecessor rejected the result because `f64::MAX²` overflowed. The inverse boundary was also unsound: a minimum-subnormal residual can square to zero before averaging, allowing a representable non-zero RMSE to be reported as exact perfect recovery.

Public RED `dd41ff5323cbc6aa3f2da8a0fb6af540e38c582e` requires constant `f64::MAX` residuals to produce RMSE `f64::MAX` and exact-zero RMSE SE. Boundary RED refinement `f4e19991bfe7b83cdce767a7214193c9e53e0b2b` requires one minimum-subnormal residual among two observations to preserve the representable minimum-subnormal RMSE, while one such residual among four observations fails closed because the positive real RMSE lies at the binary64 half-ULP boundary and would otherwise become false zero.

Causal repair `6b182107376b9b7dec66570d21a1ea6b002266f3` normalizes absolute residuals by their largest finite magnitude before squaring, deterministically accumulates normalized squares, and restores scale only after taking the square root. RMSE SE uses the same normalized squared-residual domain for its sample variance and applies the residual scale once at the end. This preserves representable extreme RMSE and RMSE SE without materializing avoidably overflowing raw squares or squared deviations. Exact all-zero residuals remain exact zero; a non-zero normalized RMSE or RMSE SE that scales below binary64 range fails closed instead of becoming perfect recovery.

This is Validation Evidence numerical execution policy for TEPP's existing recovery metric, not a new psychometric estimator or a Longitudinal Modeling primitive. IEEE/ISO/IEC 60559-2020 remains the published active international adoption of IEEE 754-2019 as of 2026-09-03; IEEE P754 is an active revision project, not a replacement published standard.

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

IEEE Computer Society. (2020). *IEEE/ISO/IEC 60559-2020: ISO/IEC/IEEE international standard—Floating-point arithmetic*. IEEE Standards Association. https://standards.ieee.org/ieee/60559/10226/

## Verification

- unit oracle tests for every metric, including empty/unequal/non-finite inputs, inverted intervals, extreme/subnormal RMSE, single-replication MC, and SE-aware accept/reject;
- foundation recovery study unit test with known loadings, intervals, temporal order, edges, and report serialization;
- `crates/validation_core/tests/bias_overflow_safe_mean_contract.rs` exercises both the representable `f64::MAX` constant-bias case and a representable SEM whose predecessor raw square sum overflows through public APIs;
- `crates/validation_core/tests/rmse_overflow_safe_contract.rs` exercises representable `f64::MAX` RMSE, exact-zero RMSE SE for constant extreme residuals, representable minimum-subnormal RMSE, and false-perfect underflow refusal through public APIs;
- exact cancellation and non-zero-bias underflow are distinct contracts: cancellation remains zero, while an unrepresentable positive mean fails closed;
- exact all-zero recovery and non-zero RMSE underflow are distinct contracts: exact zero remains zero, while a positive real RMSE that would round to false zero fails closed;
- exact-head hosted workspace line and branch coverage gates remain required before the Draft repair can be promoted.
