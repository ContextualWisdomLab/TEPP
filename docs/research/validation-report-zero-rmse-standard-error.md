# Point RMSE and RMSE standard-error coherence

## Problem

`ValidationReport` validated `rmse` and `rmse_standard_error` independently as finite, nonnegative numbers. That admitted a durable artifact with exact-zero RMSE and a positive RMSE standard error even though the canonical `validation_core::rmse_standard_error` producer returns exact zero whenever every residual is exactly zero.

Within TEPP's declared RMSE contract, `RMSE = sqrt(mean(r_i^2))`. For finite represented residuals, exact-zero RMSE implies every `r_i` is exactly zero. The delta-method RMSE standard error implemented by `validation_core` is therefore also exactly zero. A report containing `rmse = 0` and `rmse_standard_error > 0` contradicts the metric definition even though each field is individually representable.

## Decision

Keep the RMSE arithmetic and standard-error estimator unchanged. Enforce the joint invariant only at `ValidationReport` admission: when `rmse == 0.0`, `rmse_standard_error` must also equal numerical zero. IEEE `-0.0` and `+0.0` remain one zero-valued scientific state.

This is Validation Evidence artifact coherence, not a reusable psychometric estimator. No fast-mlsirm source or mutable sibling dependency is introduced.

## Alternatives rejected

- Make every positive RMSE require positive standard error: rejected because equal nonzero residual magnitudes can produce positive RMSE with exactly zero squared-residual spread and therefore zero RMSE standard error.
- Change `rmse_standard_error` arithmetic: rejected because the producer already returns zero for exact perfect recovery; the defect was at durable report admission.
- Infer hidden residuals from the report: rejected because the report intentionally stores only summary evidence.

## Traceability

- Public RED: `f7b018c5f10d11c7cc21a3430242e37c2d7a1056`, `crates/validation_core/tests/validation_report_zero_rmse_standard_error_contract.rs`.
- Causal repair: `4c5999186141dafd8d6293d5da66ab8e19693f5c`, `ValidationReport::validate` in `crates/validation_core/src/report.rs`.
- Release note: `6ae5b254669868162428fc5c85538e9f5052ac6c`, `CHANGELOG.d/validation-report-zero-rmse-standard-error.md`.
- Producer contract: `root_mean_square_error` and `rmse_standard_error` in `crates/validation_core/src/rmse.rs`.
- Owner: TEPP Validation Evidence.

## Methodological basis

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086. ADEMP requires performance measures and their Monte Carlo or sampling uncertainty to be defined coherently; a stored point metric and uncertainty field that cannot jointly arise from the declared estimator should fail admission rather than become durable evidence.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. The 2014 edition remains the current published edition while revision is underway; this repair strengthens internal consistency of validation evidence and does not substitute arithmetic checks for substantive validity arguments.
