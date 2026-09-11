# Bias standard error under pairwise subtraction roundoff

## Problem

`validation_core::bias_standard_error` previously formed each signed recovery residual as one binary64 `recovered - truth` value before computing dispersion. That is acceptable only when the represented-input subtraction is exact enough for the requested uncertainty statistic. Two distinct represented-input residuals can round to the same binary64 subtraction result, producing a false zero standard error even though the sampling uncertainty is representable.

The public RED in `crates/validation_core/tests/bias_standard_error_pairwise_subtraction_roundoff_contract.rs` fixes the smallest useful boundary:

- `truth = [2^-54, 2^-55]`
- `recovered = [1, 1]`
- exact represented-input residuals are `r1 = 1 - 2^-54` and `r2 = 1 - 2^-55`
- both pairwise binary64 subtractions round to `1.0`
- `r1 - r2 = -2^-55`
- for two observations, `SE(mean) = |r1 - r2| / 2 = 2^-56`

The predecessor therefore returned `0.0` even though `2^-56` is exactly representable. The sign-mirrored payload must return the same positive uncertainty. A companion equality control uses identical represented-input residuals and remains exact zero.

## Constraints

The repair must preserve the existing scientific contract that an individual signed residual which is itself unrepresentable is rejected. It must not make the two-observation identity a second estimator definition for larger samples, duplicate reusable static psychometric arithmetic from `fast-mlsirm`, or introduce arbitrary-precision production dependencies merely to repair one bounded Validation Evidence edge.

## Alternatives considered

Using the already rounded residual vector and adding an epsilon was rejected because it invents uncertainty without recovering represented input mass. Replacing all `bias_standard_error` arithmetic with a new exact superaccumulator was rejected for this change because the demonstrated defect is specifically the two-observation subtraction-roundoff boundary and a broader rewrite would exceed the causal evidence. Returning `InvalidInput` whenever pairwise subtraction roundoff is detected was also rejected: the RED has a finite, exactly representable scientific answer, so fail-closed rejection would discard valid Validation Evidence rather than preserve it.

## Decision

After the existing finite-residual admission gate, `n = 2` cases with nonzero error-free subtraction roundoff use the exact two-observation identity

`SE(mean) = |r1 - r2| / 2`.

The difference is evaluated from the represented inputs as `[recovered[0], -truth[0], -recovered[1], truth[1]]` through `deterministic_representable_sum_over_count(..., 2)`. This reuses the canonical cancellation-safe expanded-sum boundary already owned by `validation_core`, preserves the existing unrepresentable-residual refusal policy, and avoids making either rounded pairwise residual authoritative for dispersion.

Cases with more than two observations, and two-observation cases whose pairwise residual subtractions are exact, retain the predecessor path. This change therefore does not claim globally correctly rounded sample standard errors.

## Risk and follow-up

The remaining risk is the `n > 2` case: represented-input subtraction roundoff can still alter dispersion while the present repair intentionally leaves that path unchanged. A follow-up change requires a concrete public counterexample with a materially different representable standard error and a bounded arithmetic strategy that does not regress full-range overflow handling. The current RED is not evidence for a blanket higher-order rewrite.

## TRACEABILITY

- RED: `c9c55ea568d27e33d6e522b02e6431ec4c6983d2`
- causal repair: `f6d7da9681022e0df7a60f444e02894605201cd4`
- CHANGELOG: `ab7730ca5246598dd916ff264e5bdf97ecf72754`
- production module: `crates/validation_core/src/bias.rs`
- public contract: `crates/validation_core/tests/bias_standard_error_pairwise_subtraction_roundoff_contract.rs`
- bounded context: Validation Evidence
- owner boundary: reusable static psychometric estimators remain in `fast-mlsirm`; no mutable sibling source is consumed.

## References

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
