# ADR 0034 — Rubin loading uncertainty as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Last reviewed:** 2026-09-14
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main owns two deliberately different `psychometric_core` contracts:
`recover_loading_point_estimate_mean` computes a scaled, compensated mean of
posterior-draw OLS loading point estimates, while
`combine_draw_level_ols_loadings` computes Rubin (1996) `Q̄`, `Ū`, `B`, and
`T = Ū + (1 + 1/m)B` for complete-data OLS loadings. The latter's ordinary
loading accumulation is not a substitute for the former's robust point
estimate under large cancellation.

Operators still cannot request the joint result as a historical, digest-bound
Analysis Run output. The original branch also left several application
contracts weaker than the profile name implied: row availability lacked an
immutable snapshot identity, request/executor cutoffs were compared as RFC
3339 text, draw/matrix materialization was unbounded, artifact counts could
claim unreachable executions, serialized `T` was not checked against its
components, and terminal provider validation reused the scientific inference
label.

## Decision

Add `rubin_loading_uncertainty_v1` to `analysis_engine` as an application
composition over the protected-main scientific owners. The executor:

- requires every `RubinLoadingObservation` to carry the requested immutable
  `snapshot_id` and typed `AvailableTime`;
- rejects cross-snapshot observations and excludes same-snapshot observations
  with `AvailableTime > KnowledgeCutoff` before matrix/scientific admission;
- parses request cutoffs and compares temporal instants, while persisted
  artifacts retain one canonical RFC 3339 cutoff representation;
- invokes `recover_loading_point_estimate_mean` for the robust point estimate
  and `combine_draw_level_ols_loadings` independently for Rubin `Q̄/Ū/B/T`;
- limits the current application representation to at most 256 complete-data
  draws and 1,000,000 admitted observation-by-draw cells before transposition.
  These are resource envelopes, not psychometric validity recommendations;
- bounds total raw observation population with `MAX_EVIDENCE_UNITS` and makes
  imported artifact counts obey the same reachable envelope;
- validates imported `T` by recomputing the exact binary64 expression used by
  the scientific owner. Canonical JSON round-tripping preserves the component
  values, so exact equality is the chosen wire-integrity policy rather than a
  tolerance that could admit a different scientific result;
- applies the 256 KiB artifact envelope to both untrusted `from_json` and
  canonical `to_json`, with a maximal-valid escaping proof for the output
  direction;
- propagates artifact/digest errors instead of asserting that accepted request
  identifiers make serialization infallible;
- emits terminal `AnalysisResultSummary.validation_status = "validated"` and
  keeps `rubin_combined_ols_loadings_not_mislevy_pv` solely as the artifact's
  scientific inference boundary.

This remains draw-level OLS combination. It is not person-level plausible-value
pooling, an ESEM/DSEM sampler, CWC, persistence, or a causal estimator.

## Historical replay invariant

For a fixed requested snapshot and knowledge cutoff, adding evidence that only
becomes available after that cutoff must not change the earlier admitted
factor-score/draw matrix or its scientific result. Such rows may change only
the excluded-after-cutoff count. A row from another immutable snapshot is not
historical censoring; it is a provenance violation and fails closed even when
its availability is later than the cutoff.

## Alternatives considered

1. Use `combine_draw_level_ols_loadings.mean_loading` for both fields — rejected
   because it bypasses the protected robust point-estimate contract and can
   differ under large cancellation.
2. Compare cutoff strings — rejected because legal RFC 3339 representations of
   one instant must not alter historical execution.
3. Drop late rows without snapshot provenance — rejected because unrelated
   snapshot data could be silently attributed to the requested run.
4. Accept any finite nonnegative serialized `T` — rejected because a digest-
   bound uncertainty artifact must be internally consistent with its own
   components and draw count.
5. Leave draws unbounded and rely on `MAX_EVIDENCE_UNITS` — rejected because
   matrix materialization is a separate multiplicative resource dimension.
6. Put Rubin arithmetic into `analysis_engine` — rejected because reusable
   scientific arithmetic remains `psychometric_core`-owned.

## Scientific acceptance boundary

Known-truth/noiseless fixtures and edge contracts are regression evidence, not
commercial scientific acceptance. Issue #503 owns repeated true-loading
recovery, bias/RMSE with Monte Carlo uncertainty, an explicitly justified
interval construction and empirical coverage, attempted/recovered/failed
denominators, design sensitivity, and leakage-safe historical evaluation.
Neither LLM judgment nor synthetic unit fixtures may close that gap.

Primary authority for the current combining rule remains:

Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the
American Statistical Association, 91*(434), 473–489.
https://doi.org/10.1080/01621459.1996.10476908

Repository research authority is `docs/research/rubin-total-variance.md`.

## Consequences

The profile has a narrower, auditable temporal and resource boundary, and its
artifact can no longer claim a Rubin total inconsistent with its serialized
components. Consumers can distinguish provider validation from the scientific
claim boundary and can distinguish the robust point estimate from Rubin `Q̄`.
The profile remains Draft/Proposed and not implemented-main while #503 and the
normal exact-head merge gates remain unresolved.

## Verification

Required exact-head verification includes:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Regression contracts cover equivalent cutoff instants, future-evidence replay,
cross-snapshot refusal, robust-point versus naive-mean cancellation, exact and
exceeded draw/resource bounds, inconsistent Rubin totals, artifact count
bounds, and terminal provider/domain-status separation.

## Rollback and supersession

Rollback removes the `rubin_loading_uncertainty_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that preserves the
scientific owner split, temporal provenance, resource admission, and the
Rubin-versus-Mislevy claim boundary.
