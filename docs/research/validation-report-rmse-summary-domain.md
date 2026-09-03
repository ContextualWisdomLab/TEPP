# Validation report RMSE-summary domain invariant

## Finding

`MonteCarloSummary` is intentionally metric-neutral: its mean and percentile endpoints may be negative when it summarizes a signed metric such as bias. `ValidationReport::monte_carlo_rmse`, however, gives that same wire shape a narrower scientific meaning. Every RMSE replication is nonnegative, so a negative Monte Carlo RMSE mean or percentile endpoint is finite but scientifically impossible evidence.

Before this repair, `ValidationReport::validate()` delegated the nested object only to the generic `MonteCarloSummary::validate()` contract. A caller could therefore construct or deserialize a report whose field was explicitly named `monte_carlo_rmse` while carrying negative RMSE evidence, and the report could pass canonical JSON and human-summary projection.

## Decision

Keep `MonteCarloSummary` sign-neutral because it is reusable for signed recovery metrics. Enforce the narrower nonnegative domain only at the `ValidationReport::monte_carlo_rmse` ownership boundary:

- `mean >= 0`;
- `percentile_lower >= 0`;
- `percentile_upper >= 0`;
- all existing generic Monte Carlo count, finiteness, uncertainty, and percentile-order invariants remain mandatory.

This is an artifact-admission invariant, not a new estimator and not reusable static psychometric arithmetic. The change therefore remains in TEPP `validation_core`; it does not move arithmetic into or copy source from `fast-mlsirm`.

## RED -> repair trace

- Public RED `3cd6e41ddeffbb41e0a6179a65bc3dd9b60f41d8`: direct validation, canonical JSON, human projection, and JSON ingress must reject negative values when a generic Monte Carlo summary is embedded specifically as RMSE evidence.
- Causal repair `0090259d01ee00ad0de35ba0c4c9cb7a37c0b13c`: `ValidationReport::validate()` first applies the generic summary validator and then enforces the nonnegative RMSE-specific mean/percentile domain.
- Changelog trace `d2631d1b0047ba8dbf78058272ea6c00d9b0c9a3`.

Owned module/API/test:

- `crates/validation_core/src/report.rs`
- `validation_core::ValidationReport::validate`
- `validation_core::ValidationReport::to_json`
- `validation_core::ValidationReport::to_human_summary`
- serde ingress/egress for `ValidationReport`
- `crates/validation_core/tests/validation_report_rmse_summary_domain_contract.rs`

## Methodological basis

Morris, White, and Crowther (2019) treat simulation performance measures as explicitly defined quantities tied to their estimands and recommend coding and execution checks. RMSE is the square root of a mean squared error and therefore has a nonnegative range; a negative Monte Carlo summary carried under an RMSE-specific field is not an alternative convention but a domain violation.

The 2014 *Standards for Educational and Psychological Testing* remain the current published AERA/APA/NCME edition while revision is underway. TEPP uses that validity framework to require coherent interpretation and reporting of evidence, rather than accepting a payload solely because each scalar is machine-representable.

## References

American Educational Research Association, American Psychological Association, & National Council on Measurement in Education. (2014). *Standards for educational and psychological testing*. American Educational Research Association. https://www.testingstandards.net/open-access-files.html

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086
