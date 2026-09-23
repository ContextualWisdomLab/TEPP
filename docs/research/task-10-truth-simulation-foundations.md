# Task 10 — Realistic temporal/event truth simulation foundations

## Scope

`tepp_simulation` generates deterministic known-truth corpora that:

1. separate latent event occurrence from document creation and availability;
2. attach multilevel memberships and method-effect variants (revision, translation, template-copy);
3. inject controlled missingness and relation false-negative/false-positive noise;
4. emit a digest-bound truth manifest for recovery studies.

## Simulation-study analysis contract

Morris, White, and Crowther's ADEMP framing treats simulation studies as empirical experiments: aims, data-generating mechanisms, estimands, methods, and performance measures are declared before interpreting results, and Monte Carlo error is reported for estimated performance. TEPP applies that separation directly to recovery evidence.

`model_selection::selected_k_recovery_summary_from_results(...)` owns the typed selected-`K` replication outcome and preserves owner-admitted numerical failures in the attempted denominator while structural experiment invalidity fails closed. Continuous recovery measures such as topic-content RMSE or bias use `validation_core::summarize_recovery_metric_replications(...)`: the successful scalar distribution is reported conditionally, but attempted count, failed count, failure rate, and Bernoulli Monte Carlo standard error remain attached to the same evidence object. An all-failed experiment therefore has a valid failure denominator and no fabricated scalar metric.

This denominator contract does not define a practical scientific threshold. Thresholds or SE-aware promotion rules must be fixed from the intended estimand/design before the acceptance run rather than tuned to observed CI-scale replications.

## Authoritative sources

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Verification

Seeded determinism, temporal order, membership multiplicity, parent linkage, digest integrity, and recovery-denominator accounting are fail-closed. Workspace line and branch coverage gates must remain complete.
