# Point RMSE and RMSE standard-error coherence

## Problem

`ValidationReport` originally validated `rmse` and `rmse_standard_error` independently as finite, nonnegative numbers. The first repair closed the exact-perfect case: a durable artifact with exact-zero RMSE and a positive RMSE standard error contradicted the canonical `validation_core::rmse_standard_error` producer.

A broader support invariant follows from the same producer. Let `x_i = r_i^2 >= 0`, `m = mean(x) = RMSE^2`, and let `s_x` be the sample standard deviation with denominator `n - 1`. For fixed nonnegative sample mean, the maximum sample variance occurs when one observation carries all mass (`x_1 = n m`) and the remaining observations are zero. Then `s_x = sqrt(n) m`. The crate's delta-method definition

`SE(RMSE) = s_x / (2 * RMSE * sqrt(n))`

therefore satisfies `SE(RMSE) <= RMSE / 2` for every admissible finite squared-residual sample. The boundary is attained, for example, by two residual magnitudes `[0, 1]`. A report such as `rmse = 0.2`, `rmse_standard_error = 0.11` is individually finite and nonnegative but cannot arise from the declared producer.

## Decision

Keep the RMSE arithmetic and standard-error estimator unchanged. Enforce joint support only at `ValidationReport` admission:

- exact-zero RMSE still requires numerical-zero RMSE standard error;
- positive RMSE requires `rmse_standard_error / rmse <= 0.5 + 64 * EPSILON` so a represented boundary result is not rejected by cross-operation binary64 rounding;
- positive RMSE with zero standard error remains valid when squared residuals are exactly constant.

IEEE `-0.0` and `+0.0` remain one zero-valued scientific state. The relative check also fails closed if a positive standard error divided by a tiny positive RMSE is not representable.

This is Validation Evidence artifact coherence, not a reusable psychometric estimator. No fast-mlsirm source or mutable sibling dependency is introduced.

## Alternatives rejected

- Make every positive RMSE require positive standard error: rejected because equal nonzero residual magnitudes produce positive RMSE with exactly zero squared-residual spread.
- Store an arbitrary uncertainty field beside RMSE: rejected because the field is explicitly named `rmse_standard_error` and the crate exposes one canonical squared-residual delta-method producer for it.
- Require bit-exact `SE <= RMSE / 2` with no tolerance: rejected because the mathematical boundary is produced through separate binary64 operations and a valid boundary artifact must survive harmless last-bit differences.
- Infer hidden residuals from the report: rejected because the report intentionally stores summary evidence rather than raw residual vectors.

## Traceability

- Exact-zero public RED: `f7b018c5f10d11c7cc21a3430242e37c2d7a1056`, `crates/validation_core/tests/validation_report_zero_rmse_standard_error_contract.rs`.
- Exact-zero causal repair: `4c5999186141dafd8d6293d5da66ab8e19693f5c`, `ValidationReport::validate` in `crates/validation_core/src/report.rs`.
- Exact-zero release note: `6ae5b254669868162428fc5c85538e9f5052ac6c`, `CHANGELOG.d/validation-report-zero-rmse-standard-error.md`.
- Positive-RMSE support RED: `a2aca5b077665433aad0e5531d53360b599b64b3`, `crates/validation_core/tests/validation_report_rmse_standard_error_upper_bound_contract.rs`.
- Positive-RMSE causal repair: `32f094029732e2333b35ad3a11521c4c3d956798`, `ValidationReport::validate` in `crates/validation_core/src/report.rs`.
- Positive-RMSE release note: `7b61c107d1bc6391651ce2341e6ad27c2cada17a`, `CHANGELOG.d/validation-report-rmse-standard-error-support.md`.
- Producer contract: `root_mean_square_error` and `rmse_standard_error` in `crates/validation_core/src/rmse.rs`.
- Owner: TEPP Validation Evidence.

## Methodological basis

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086. ADEMP requires performance measures and their Monte Carlo or sampling uncertainty to be defined coherently; a stored point metric and uncertainty field that cannot jointly arise from the declared estimator should fail admission rather than become durable evidence.

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. The 2014 edition remains the current published edition while revision is underway. AERA's Joint Committee is revising that edition; as of 31 August 2026 AERA also publishes a current Task Force roster for the Standards work. This repair strengthens internal consistency of validation evidence and does not substitute arithmetic checks for substantive validity arguments.
