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

This denominator contract does not by itself define a practical scientific threshold. Thresholds or SE-aware promotion rules must be fixed from the intended estimand/design before the acceptance run rather than tuned to observed CI-scale replications.

## Prospective nominal-95% interval calibration design

Issue #725 records the first versioned larger-run design before TEPP executes it. `validation_core::CoverageCalibrationDesign::tepp_nominal_95_v1()` is the source representation of that decision.

- Design identity: `tepp.coverage.nominal95.v1`.
- Independent sampling unit: one generated DGP replication after #722 collapses the declared rolling-origin window coverage values within that DGP. Document, ALR-coordinate, and expanding-window interval indicators are not Monte Carlo replications.
- Nominal interval coverage: `0.95`.
- Practical psychometric simulation band: `[0.91, 0.98]`. This follows Muthén and Muthén's simulation convention; it is a project-specific acceptance criterion, not a theorem that every 95% interval procedure must satisfy this exact band in every DGP.
- Predeclared attempted DGP count: `10_000` independent seeds.
- Monte Carlo precision requirement: successful-DGP coverage-mean standard error `<= 0.005`.

The 10,000-attempt design is a precision decision, not a result-dependent sample extension. A DGP-level coverage scalar is bounded in `[0,1]`; therefore its variance is at most `1/4`, so 10,000 independent successful DGP values give a worst-case standard error of the mean no larger than `0.5 / sqrt(10_000) = 0.005`. When numerical failures reduce the successful sample, the actual standard error reported by the #716/#717 Monte Carlo owner remains authoritative and the attempted/success/failure denominator from #724 remains attached.

`validation_core::assess_coverage_calibration(...)` requires the evidence's attempted DGP count to match the versioned design exactly. It evaluates the conditional successful-DGP coverage mean against the declared practical band and the owner-reported Monte Carlo standard error against the precision target. It does **not** define or conceal an acceptable numerical-failure rate. A passing conditional calibration assessment is therefore not by itself convergence/robustness acceptance, scientific claim promotion, or release authority.

The ordinary CI contract stays intentionally small. It proves composition and denominator semantics but does not execute 10,000 DGPs or claim acceptance. The expensive acceptance run must consume the versioned design unchanged and persist exact design/seed/replay evidence. Any later extension must receive a new design identity and disjoint seed range rather than silently modifying `v1` after outcomes are visible.

## Authoritative sources

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

Muthén, L. K., & Muthén, B. O. (2002). How to use a Monte Carlo study to decide on sample size and determine power. *Structural Equation Modeling: A Multidisciplinary Journal, 9*(4), 599–620. https://doi.org/10.1207/S15328007SEM0904_8

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Verification

Seeded determinism, temporal order, membership multiplicity, parent linkage, digest integrity, recovery-denominator accounting, prospective design identity, exact attempted-DGP count, practical coverage-band evaluation, and Monte Carlo precision accounting are fail-closed. Workspace line and branch coverage gates must remain complete.