# Monte Carlo summary uncertainty coherence

## Problem

`MonteCarloSummary` is durable Validation Evidence. Its `standard_error` field is the standard error of the replication mean, while `standard_deviation` is the sample SD over the retained replications. Before this repair, a payload could remain finite and ordered yet claim impossible uncertainty: exact-zero SE with nonzero replication spread, SE at least as large as SD with more than one replication, or nonzero sample spread/SE for the canonical singleton summary.

That is an artifact-admission defect, not a new estimator. `summarize_replications` already computes `SE = SD / sqrt(n)` and fails closed when a nonzero SD projects to exact-zero SE. The missing boundary was validation of externally constructed or deserialized summaries.

## Evidence and decision

Morris, White, and Crowther (2019) treat simulation studies as empirical experiments and require Monte Carlo standard errors to quantify uncertainty from a finite number of simulation repetitions. Their analysis separates the empirical spread of replication-level estimates from Monte Carlo uncertainty of derived performance measures. For a sample mean over independent replications, finite `n > 1` implies a positive SEM strictly smaller than a positive sample SD; a positive SD cannot legitimately become exact-zero SEM at a finite replication count.

TEPP therefore keeps `MonteCarloSummary` sign-neutral so it can summarize signed metrics such as bias, but tightens uncertainty-domain admission:

- `replication_count` must remain positive;
- SD and SE must remain finite and nonnegative;
- nonzero SD with exact-zero SE is rejected;
- for `n > 1`, nonzero SE must be strictly smaller than nonzero SD;
- the canonical `n = 1` summary has zero SD and zero SE;
- percentile ordering remains a separate generic invariant.

The repair deliberately does not require serialized clients to reproduce TEPP's exact binary64 `SD / sqrt(n)` bit pattern. The canonical producer still computes that value; admission rejects scientifically impossible uncertainty while avoiding a cross-language bit-for-bit serialization requirement.

## Traceability

- Bounded context: Validation Evidence.
- Production module: `crates/validation_core/src/monte_carlo.rs`.
- Public contract: `MonteCarloSummary::validate`, serde ingress/egress through the existing `MonteCarloSummary` implementations.
- Regression: `crates/validation_core/tests/monte_carlo_standard_error_coherence_contract.rs`.
- Isolating RED: `e2d0c057d39b7786dbd96528d4e259775c6c2e01` demonstrates that `n = 4`, `SD = 0.5`, `SE = 0.5` was still admitted even though a finite multi-replication SEM must be strictly smaller than a positive SD.
- Causal repair: `0e973b566ec969d0fea8b7403bf09602cdebd4a3` changes the multi-replication admission boundary from `SE > SD` to `SE >= SD` while preserving the earlier false-zero and singleton guards.
- Changelog: `CHANGELOG.d/validation-monte-carlo-summary-uncertainty-coherence.md`.

This remains TEPP-owned artifact validation. It does not duplicate reusable static psychometric arithmetic from `fast-mlsirm`, introduce LLM authority, or change the scientific definition of RMSE, bias, coverage, or any longitudinal estimand.

## Reference

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
