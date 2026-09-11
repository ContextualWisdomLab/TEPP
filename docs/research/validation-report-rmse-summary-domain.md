# Validation report RMSE-summary domain invariant

## Finding

`MonteCarloSummary` is intentionally metric-neutral: its mean and percentile endpoints may be negative when it summarizes a signed metric such as bias. `ValidationReport::monte_carlo_rmse`, however, gives that same wire shape a narrower scientific meaning. Every RMSE replication is nonnegative, so a negative Monte Carlo RMSE mean or percentile endpoint is finite but scientifically impossible evidence.

A second support constraint follows from the same typed meaning. Let retained RMSE replications be `x_i >= 0`, let `n` be the replication count, and let `m` be their sample mean. For a fixed nonnegative sum `sum(x_i) = n m`, the largest possible squared deviation occurs when one replication carries the whole sum and the remaining `n - 1` values are zero. Therefore

`SD <= sqrt(n) * m`

for the sample standard deviation, and because the stored Monte Carlo standard error is `SD / sqrt(n)`,

`SE(mean) <= m`.

The bound is attainable: for four replications `[0, 0, 0, 4]`, `mean = 1`, sample `SD = 2`, and `SE = 1`. A generic summary such as `n = 4`, `mean = 1`, `SD = 3`, `SE = 1.5` is internally coherent as a sign-neutral carrier because `SE = SD / sqrt(n)`, but it cannot have been produced by four nonnegative RMSE replications.

Before these repairs, `ValidationReport::validate()` delegated the nested object only to the generic `MonteCarloSummary::validate()` contract and then checked the RMSE-specific sign domain. A caller could therefore construct or deserialize a report whose field was explicitly named `monte_carlo_rmse` while carrying negative RMSE evidence or a positive mean/spread combination outside nonnegative sample support, and the report could pass canonical JSON and human-summary projection.

## Decision

Keep `MonteCarloSummary` sign-neutral because it is reusable for signed recovery metrics. Enforce the narrower domain only at the `ValidationReport::monte_carlo_rmse` ownership boundary:

- `mean >= 0`;
- `percentile_lower >= 0`;
- `percentile_upper >= 0`;
- exact-zero RMSE mean requires zero spread, zero SE, and zero percentile support;
- positive RMSE mean requires `SE(mean) <= mean`, with the same small binary64 relative tolerance used at the point-RMSE support boundary;
- all existing generic Monte Carlo count, finiteness, uncertainty, percentile-order, and zero-spread support invariants remain mandatory.

Checking `SE / mean` at the typed boundary avoids duplicating the generic `SD / sqrt(n)` coherence calculation while still enforcing the nonnegative-support theorem. If the ratio overflows or is otherwise non-finite, the durable artifact fails closed. The boundary case remains admissible within the explicit floating-point tolerance.

This is an artifact-admission invariant, not a new estimator and not reusable static psychometric arithmetic. The change therefore remains in TEPP `validation_core`; it does not move arithmetic into or copy source from `fast-mlsirm`.

## RED -> repair trace

- Public RED `3cd6e41ddeffbb41e0a6179a65bc3dd9b60f41d8`: direct validation, canonical JSON, human projection, and JSON ingress must reject negative values when a generic Monte Carlo summary is embedded specifically as RMSE evidence.
- Causal sign-domain repair `0090259d01ee00ad0de35ba0c4c9cb7a37c0b13c`: `ValidationReport::validate()` first applies the generic summary validator and then enforces the nonnegative RMSE-specific mean/percentile domain.
- Generic zero-spread support later made an older zero-mean fixture self-contradictory; test repair `e201a2f46953152255345a7b3abc35f05ea9e33c` aligns the fixture with the stronger upstream summary contract rather than weakening that contract.
- Public RED `43a7dec1dbc848435ca099aea80db46c8cbd97e5`: a generic-valid `n=4, mean=1, SD=3, SE=1.5` summary must fail when embedded as RMSE evidence, while the attainable `[0,0,0,4]` boundary summary (`mean=1, SD=2, SE=1`) must remain valid.
- Causal nonnegative-support repair `2f78954eb21a08c316d0d6b70f659685fb283a0b`: positive Monte Carlo RMSE summaries fail when `standard_error / mean > 1 + 64*EPSILON`; zero mean retains the exact-perfect-recovery rule.
- Changelog trace `4ce54959a5c70ccae8212a6494d346ceef0ff35f`.

Owned module/API/test:

- `crates/validation_core/src/report.rs`
- `validation_core::ValidationReport::validate`
- `validation_core::ValidationReport::to_json`
- `validation_core::ValidationReport::to_human_summary`
- serde ingress/egress for `ValidationReport`
- `crates/validation_core/tests/validation_report_rmse_summary_domain_contract.rs`
- `crates/validation_core/tests/validation_report_rmse_summary_nonnegative_support_contract.rs`
- `crates/validation_core/tests/validation_report_zero_mean_rmse_summary_contract.rs`

## Methodological basis

Morris, White, and Crowther (2019) treat simulation performance measures as explicitly defined quantities tied to their estimands and recommend coding and execution checks. RMSE is the square root of a mean squared error and therefore has a nonnegative range; negative Monte Carlo RMSE evidence or a mean/spread combination that no nonnegative replication sample can realize is not an alternative convention but an artifact-domain violation.

The 2014 *Standards for Educational and Psychological Testing* remain the current published AERA/APA/NCME edition while revision is underway. TEPP uses that validity framework to require coherent interpretation and reporting of evidence, rather than accepting a payload solely because each scalar is machine-representable.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
