# Monte Carlo summary uncertainty coherence

## Problem

`MonteCarloSummary` is durable Validation Evidence. Its `standard_error` field is the standard error of the retained-replication mean, while `standard_deviation` is the sample SD over those replications. The canonical producer therefore defines `SE = SD / sqrt(n)`. Earlier admission checks rejected several impossible cases but still allowed materially understated or overstated positive SE values whenever they were merely finite, positive, and smaller than SD. For example, `n = 4`, `SD = 0.5`, `SE = 0.2` passed even though the represented summary contract implies `SE = 0.25`.

That is an artifact-admission defect, not a new estimator. `summarize_replications` already computes the canonical relation and fails closed when a nonzero SD projects to exact-zero SE. The missing boundary was coherence validation for externally constructed or deserialized summaries.

## Evidence and decision

Morris, White, and Crowther (2019) treat simulation studies as empirical experiments and require Monte Carlo standard errors to quantify uncertainty from a finite number of simulation repetitions. For the mean of independent replication-level values, the Monte Carlo standard error is the empirical replication SD divided by the square root of the number of replications. The durable summary therefore cannot admit an arbitrary positive SE independently of its represented SD and replication count.

TEPP keeps `MonteCarloSummary` sign-neutral so it can summarize signed metrics such as bias, while tightening uncertainty coherence:

- `replication_count` must remain positive;
- SD and SE must remain finite and nonnegative;
- zero SD requires exact-zero SE;
- positive SD requires `n > 1` and a positive representable SE;
- positive SE must agree with `SD / sqrt(n)` within `64 * f64::EPSILON` relative error;
- the tolerance is deliberately wider than bit-for-bit equality so adjacent correctly rounded binary64 results from independent implementations remain admissible, while materially understated or overstated uncertainty is rejected;
- the canonical `n = 1` summary has zero SD and zero SE;
- percentile ordering remains a separate generic invariant.

The relative comparison is scale-free. If canonical `SD / sqrt(n)` itself becomes exact zero while SD is nonzero, the payload fails closed instead of presenting zero Monte Carlo uncertainty.

## Traceability

- Bounded context: Validation Evidence.
- Production module: `crates/validation_core/src/monte_carlo.rs`.
- Public contract: `MonteCarloSummary::validate`, serde ingress/egress through the existing `MonteCarloSummary` implementations.
- Regression: `crates/validation_core/tests/monte_carlo_standard_error_coherence_contract.rs`.
- Earlier isolating RED: `e2d0c057d39b7786dbd96528d4e259775c6c2e01` demonstrated that a multi-replication SE equal to SD was admitted.
- Earlier repair: `0e973b566ec969d0fea8b7403bf09602cdebd4a3` rejected exact-zero SE with positive spread and multi-replication `SE >= SD` without asserting the canonical relationship.
- Canonical-coherence RED: `0a4c242fbd1c5b2f35e71a1ca1665ca9f7861338` demonstrates that `n = 4`, `SD = 0.5`, `SE = 0.2` was still admitted and that zero spread could still carry positive SE.
- Causal repair: `9b53076a53032441623ff487a006ec6d20812030` validates the represented `SD / sqrt(n)` relationship and zero-spread contract.
- Cross-language tolerance regression: `141246cfef7c44b327f7bfbeb22bc51279c6b9f0` admits the adjacent binary64 value to the canonical SE so admission is not a bit-pattern equality gate.
- Changelog: `CHANGELOG.d/validation-monte-carlo-summary-uncertainty-coherence.md`.

This remains TEPP-owned artifact validation. It does not duplicate reusable static psychometric arithmetic from `fast-mlsirm`, introduce LLM authority, or change the scientific definition of RMSE, bias, coverage, or any longitudinal estimand.

## Reference

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
