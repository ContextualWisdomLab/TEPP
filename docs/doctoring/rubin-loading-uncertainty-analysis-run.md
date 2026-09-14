# Rubin loading-uncertainty analysis-run bind

**Review date:** 2026-09-14
**Active slice:** GAP-006 / issue #169 operator-visible composition
**Scientific acceptance owner:** #503
**Implementation maturity:** active-PR — not implemented-main

Protected main owns the reusable numerical contracts. `recover_loading_point_estimate_mean` performs the robust point-estimate aggregation, while `combine_draw_level_ols_loadings` owns Rubin (1996) `Q̄`, `Ū`, `B`, and `T`. `validation_core` owns reusable bias, RMSE, coverage, and Monte Carlo summary metrics. This Analysis Run profile composes those owners without copying their arithmetic paths.

The review found that the predecessor branch was materially weaker than its cutoff-safe claim. It lacked per-row immutable snapshot provenance, compared cutoffs as timestamp text, admitted an unbounded draw dimension, accepted artifact counts outside the executable population, did not verify serialized Rubin `T`, used the Rubin mean as the supposedly robust point estimate, asserted artifact hashing could not fail, and wrote the scientific inference label into provider `validation_status`.

## Repair lineage

- RED `152cb3913d0920850d9a53e13cb6b4edfff26fb7` makes equivalent RFC 3339 cutoff instants, provider/domain status separation, robust point-estimate cancellation, inconsistent Rubin totals, count bounds, and the draw ceiling executable contracts.
- Causal repair `dd46f8ebac0e1bc110648152453dc9fce87bef23` adds per-observation immutable `snapshot_id`, instant-based cutoff binding, cross-snapshot refusal, a 256-draw / 1,000,000 admitted-cell application resource envelope, reachable artifact count validation, exact binary64 Rubin-total integrity, symmetric 256 KiB artifact wire admission, protected-main robust point-estimate invocation, error propagation, and terminal `validation_status = "validated"`.
- Contract migration `8f0e743348290d65d15e048216cccb1f5953f2af` updates the integration surface to explicit snapshot provenance and adds historical replay, cross-snapshot, resource-matrix, point-estimate, cutoff-instant, and terminal-status regressions.
- ADR `a4a0ab0120c9222418f8042af68149e57d8cfe38` returns ADR 0034 from premature `Accepted` branch authority to `Proposed` and records the scientific/resource alternatives.
- Scientific acceptance begins at `aa39f6d28d697d91014ab45daffa06f7c3182615` and is hardened on the current scientific-evidence child by consuming protected-main `validation_core`, centering the `T` interval diagnostic on Rubin `Q̄`, and keeping the robust point estimate as the separate recovery target. `docs/research/rubin-loading-uncertainty-scientific-acceptance.md` records the predeclared design, exact seeds, attempted/recovered/failed denominators, bias/RMSE and Monte Carlo uncertainty, Rubin `T` behavior, interval diagnostic boundary, and checked numerical evidence.

The historical replay contract is explicit: same-snapshot rows with `AvailableTime > KnowledgeCutoff` do not enter the earlier scientific matrix. Cross-snapshot rows are provenance violations and fail closed rather than being censored. Persisted artifacts use canonical cutoff text, while request/executor binding is by temporal instant.

The 256-draw and 1,000,000 matrix-cell ceilings are application resource limits for the current representation. They are not psychometric recommendations and must not be cited as scientific sample-size or imputation-count guidance.

## Scientific acceptance evidence

`crates/analysis_engine/tests/rubin_loading_scientific_acceptance.rs` owns the executable repeated-sampling gate. Eight 512-replication scenarios vary observation count, draw count, true loading, and residual scale while exercising positive between-draw uncertainty. Every reported metric keeps the attempted denominator; a refused execution increments failure rather than disappearing from recovery statistics.

The point-estimate gate reports bias, RMSE, and their Monte Carlo uncertainty against known loading truth through `validation_core`. The Rubin uncertainty gate records mean `Ū`, `B`, and `T`. The 95% interval check is explicitly a large-sample normal diagnostic `Q̄ ± 1.959963984540054 * sqrt(T)`, not a claim that the profile implements Rubin small-sample degrees of freedom. Its empirical coverage is accepted only within the predeclared design and Monte Carlo envelope recorded in the research note.

A separate 512-replication rolling-origin gate supplies 64 rows while making only the first 48 available at the early cutoff. The early full-input execution must be bit-identical in point estimate, Rubin mean, and `T` to a prefix-only execution, while reporting 16 future rows excluded. The later cutoff admits all 64 rows. This is the profile-level leakage-safe historical evidence required by #503.

This acceptance design is single-level because that is the current profile contract. It does not collapse an existing multilevel, cross-classified, or multiple-membership design. If those structures enter the profile later, this evidence must be extended before the corresponding scientific claim survives.

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908

Repository formula authority remains `docs/research/rubin-total-variance.md`; acceptance evidence is `docs/research/rubin-loading-uncertainty-scientific-acceptance.md` plus its exact-head Rust test.

Exact-head required workflows, owned-production line/branch coverage, resolved current findings, a qualifying independent approval, and conflict-resolving successor inheritance remain required before Ready/release claims. Scientific acceptance is not considered passing until the exact-head acceptance test and required CI gates are GREEN; predecessor receipts do not transfer.
