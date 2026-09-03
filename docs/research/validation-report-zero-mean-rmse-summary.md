# Zero-mean Monte Carlo RMSE summary admission

## Problem

`MonteCarloSummary` is deliberately sign-neutral because the same carrier can summarize signed quantities such as bias. `ValidationReport::monte_carlo_rmse` is narrower: every retained replication is an RMSE and is therefore nonnegative. The report boundary already rejected negative means and percentile endpoints, but it still admitted an exact-zero Monte Carlo RMSE mean together with positive spread, positive standard error, or positive empirical percentile endpoints.

For a finite set of nonnegative represented RMSE replications, an arithmetic mean of exactly zero implies that every retained replication is exactly zero. The sample standard deviation, standard error of the mean, and every empirical percentile are consequently zero. Treating a zero mean with positive uncertainty/support as valid durable evidence would describe perfect average recovery and non-perfect replications at the same time.

## Decision

Keep `MonteCarloSummary` generic and sign-neutral. Enforce the stronger invariant only when the summary occupies `ValidationReport::monte_carlo_rmse`: if `mean == 0.0`, `standard_deviation`, `standard_error`, `percentile_lower`, and `percentile_upper` must all equal numerical zero. IEEE signed zero is one zero-valued scientific result, so `-0.0` is accepted wherever numerical equality to zero holds.

The change also repairs the pre-existing report round-trip fixture to use the generic summary contract's canonical Monte Carlo standard error, `SD / sqrt(n)`, rather than the approximate literal `0.003` for `SD = 0.01` and `n = 10`.

## Alternatives rejected

- Reject zero-mean/positive-spread summaries in `MonteCarloSummary` globally: rejected because a generic signed metric can have zero mean with positive spread.
- Require the Monte Carlo RMSE mean to lie between stored percentile endpoints: rejected because the stored percentile levels are not part of the payload, and skewed nonnegative samples can legitimately have means outside a selected percentile interval.
- Reconstruct hidden replications from summary fields: impossible and outside the evidence contract.

## Traceability

- Public RED: `a17dfe1bd9d845356b757ec98e930b5d927eab8f`, `crates/validation_core/tests/validation_report_zero_mean_rmse_summary_contract.rs`.
- Causal repair: `d17d803415333f79bf6291efed9206a630979adf`, `ValidationReport::validate` in `crates/validation_core/src/report.rs`.
- Release note: `42be85748c4e4136c54fd1f918dfa89533376451`, `CHANGELOG.d/validation-report-zero-mean-rmse-summary.md`.
- Owner: TEPP Validation Evidence. No reusable static psychometric estimator or mutable sibling implementation is introduced.

## Methodological basis

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086. The ADEMP framework separates estimands and performance measures and treats Monte Carlo uncertainty as uncertainty of an explicitly defined performance measure. That supports rejecting artifacts whose stored moments cannot jointly represent the declared RMSE performance measure.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. The 2014 edition remains the current published edition; this repair concerns the integrity and interpretability of validation evidence rather than replacing substantive validity arguments with an arithmetic gate.
