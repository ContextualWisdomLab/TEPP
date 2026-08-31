# ADR 0034 — Rubin loading uncertainty as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already averages posterior-draw OLS loadings and combines those
loadings with Rubin (1996) total variance `T = Ū + (1 + 1/m) B` inside
`psychometric_core`. Operators still cannot request that joint uncertainty
wiring as a digest-bound analysis-run output. Recovery primitives alone are
not the ESEM/DSEM engine (GAP-006 / #169). A second Driver p.16 `std`-family
restore, another CWC bind, or another GAP-003A HTTP slice would not close this
operator-visible gap.

The library helpers are explicit: the draw-mean is not Rubin pooling, and
Rubin `T` on complete-data OLS loadings is not Mislevy person-level
plausible-value draws.

## Decision

Add the `rubin_loading_uncertainty_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-mapped factor scores, complete-data indicator draws, an
  admitted indicator kind, and `available_time`;
- excludes observations whose availability is later than the request
  `knowledge_cutoff`;
- jointly invokes `recover_loading_point_estimate_mean` and
  `combine_draw_level_ols_loadings` without reimplementing either helper;
- emits a canonical SHA-256-digested `tepp.rubin_loading_uncertainty.v1`
  artifact with observation/draw counts, excluded-after-cutoff count,
  indicator kind, point-estimate mean, Rubin `Q̄`/`Ū`/`B`/`T`, and inference
  status `rubin_combined_ols_loadings_not_mislevy_pv`;
- does not invent an ESEM/DSEM sampler, persist rows, treat the draws as
  Mislevy person-level plausible values, or claim strong invariance.

This is draw-level OLS combination, not multiple imputation of persons, not
CWC, and not a random-effects sampler.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind uncertainty
   to an analysis run.
2. Duplicate the GAP-006 CWC analysis-run bind — rejected because CWC slopes
   are a different estimand already occupied by a live PR.
3. Put Rubin combination into `tepp_api` — rejected because transport
   contracts and scientific combination would become one service boundary.
4. Bind the existing `psychometric_core` draw-mean and Rubin `T` helpers to
   ADR 0022's analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe joint point-estimate and Rubin-`T` loading
uncertainty as a digest-bound terminal result. The artifact is not Mislevy
person-level plausible values, not an ESEM fit, and not implemented-main
until exact-head Checks and two independent approvals land.

## Verification

```text
cargo fmt -p analysis_engine -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Known-truth noiseless draws recover mean loading `0.8` with strictly positive
between-draw variance and Rubin `T = Ū + (1 + 1/m) B`. Cutoff exclusion,
snapshot/profile mismatch, empty eligibility, a single draw, raw proportions,
and unequal draw lengths fail closed.

## Rollback and supersession

Rollback removes the `rubin_loading_uncertainty_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps Rubin
`T` on complete-data OLS loadings distinct from Mislevy person-level
plausible values.
