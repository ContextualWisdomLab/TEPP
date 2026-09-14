# Rubin loading-uncertainty analysis-run bind

**Review date:** 2026-09-14
**Active slice:** GAP-006 / issue #169 operator-visible composition
**Scientific acceptance owner:** #503
**Implementation maturity:** active-PR — not implemented-main

Protected main owns the reusable numerical contracts. `recover_loading_point_estimate_mean` performs the robust point-estimate aggregation, while `combine_draw_level_ols_loadings` owns Rubin (1996) `Q̄`, `Ū`, `B`, and `T`. This Analysis Run profile composes those owners without copying either arithmetic path.

The review found that the predecessor branch was materially weaker than its cutoff-safe claim. It lacked per-row immutable snapshot provenance, compared cutoffs as timestamp text, admitted an unbounded draw dimension, accepted artifact counts outside the executable population, did not verify serialized Rubin `T`, used the Rubin mean as the supposedly robust point estimate, asserted artifact hashing could not fail, and wrote the scientific inference label into provider `validation_status`.

## Repair lineage

- RED `152cb3913d0920850d9a53e13cb6b4edfff26fb7` makes equivalent RFC 3339 cutoff instants, provider/domain status separation, robust point-estimate cancellation, inconsistent Rubin totals, count bounds, and the draw ceiling executable contracts.
- Causal repair `dd46f8ebac0e1bc110648152453dc9fce87bef23` adds per-observation immutable `snapshot_id`, instant-based cutoff binding, cross-snapshot refusal, a 256-draw / 1,000,000 admitted-cell application resource envelope, reachable artifact count validation, exact binary64 Rubin-total integrity, symmetric 256 KiB artifact wire admission, protected-main robust point-estimate invocation, error propagation, and terminal `validation_status = "validated"`.
- Contract migration `8f0e743348290d65d15e048216cccb1f5953f2af` updates the integration surface to explicit snapshot provenance and adds historical replay, cross-snapshot, resource-matrix, point-estimate, cutoff-instant, and terminal-status regressions.
- ADR `a4a0ab0120c9222418f8042af68149e57d8cfe38` returns ADR 0034 from premature `Accepted` branch authority to `Proposed` and records the scientific/resource alternatives.

The historical replay contract is explicit: same-snapshot rows with `AvailableTime > KnowledgeCutoff` do not enter the earlier scientific matrix. Cross-snapshot rows are provenance violations and fail closed rather than being censored. Persisted artifacts use canonical cutoff text, while request/executor binding is by temporal instant.

The 256-draw and 1,000,000 matrix-cell ceilings are application resource limits for the current representation. They are not psychometric recommendations and must not be cited as scientific sample-size or imputation-count guidance.

## Evidence boundary

The current deterministic fixtures verify contracts and known identities. They do not establish profile-level recovery or interval coverage. Issue #503 requires repeated true-loading recovery, bias/RMSE with Monte Carlo uncertainty, an explicitly justified interval construction before any coverage claim, empirical coverage with Monte Carlo uncertainty, attempted/recovered/failed denominators, design sensitivity, and leakage-safe historical evaluation.

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908

Repository authority: `docs/research/rubin-total-variance.md`.

Exact-head required workflows, owned-production line/branch coverage, resolved current findings, a qualifying independent approval, conflict-resolving successor inheritance, and #503 or equivalent checked-in scientific evidence remain required before Ready/release claims. Predecessor receipts do not transfer.
