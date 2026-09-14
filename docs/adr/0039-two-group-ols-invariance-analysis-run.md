# ADR 0039 — Two-group OLS invariance as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already classifies two-group OLS invariance
(`configural` / `metric` / `strong` / `strict`) and recovers a
strong/strict-gated latent-mean difference `(ȳ_c − ȳ_r) / λ` inside
`psychometric_core`. Operators still cannot request that joint wiring as a
digest-bound analysis-run output. Recovery primitives alone are not the
ESEM/DSEM engine (GAP-006 / #169). A second Driver p.16 `std`-family restore,
another CWC bind, another Rubin/plausible-value bind, GAP-169 composition, or
another GAP-003A HTTP slice would not close this operator-visible gap.

The library helpers are explicit: metric/weak invariance licenses shared
*metric* meaning and does not license latent-mean comparison. `#84` wire names
are `configural` / `metric` / `scalar`; local `strict` has no `#84` wire name.
This is two-group OLS, not MGCFA.

## Decision

Add the `two_group_ols_invariance_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-mapped factor scores, indicators, an admitted indicator
  kind, and `available_time` for a reference group and a comparison group;
- excludes observations whose availability is later than the request
  `knowledge_cutoff`;
- jointly invokes `classify_two_group_ols_invariance` and
  `recover_strong_gated_latent_mean_difference` without reimplementing either
  helper, using the library's hardcoded `1e-9` OLS tolerances;
- emits a canonical SHA-256-digested `tepp.two_group_ols_invariance.v1`
  artifact with per-group observation counts, excluded-after-cutoff counts,
  indicator kind, local status, `#84` wire name (`scalar` or `null`),
  `licenses_latent_mean_comparison`, the gated latent-mean difference, OLS
  intercepts/loadings/residuals, and inference status
  `two_group_ols_invariance_not_mgcfa`;
- fails closed with `PsychometricError::StrongInvarianceRequired` when the
  classified status is configural or metric;
- does not invent an MGCFA sampler, persist rows, treat metric as a mean
  license, or claim implemented-main.

This is two-group OLS invariance evidence, not MGCFA, not CWC, not Rubin `T`,
and not a random-effects sampler.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind invariance to
   an analysis run.
2. Duplicate the GAP-006 CWC, Rubin, or GAP-169 composition analysis-run
   binds — rejected because those estimands are already occupied by live PRs.
3. Emit metric-only classification as a succeeded terminal result — rejected
   because metric does not license latent-mean comparison; the profile fails
   closed instead of leaking a mean.
4. Bind the existing `psychometric_core` classify/recover helpers to ADR
   0022's analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe two-group OLS invariance and a
strong/strict-gated latent-mean difference as a digest-bound terminal result.
Metric-only series fail closed. The artifact is not MGCFA, not an ESEM fit,
and not implemented-main until exact-head Checks and two independent
approvals land.

0026–0038 remain on other live PRs (GAP-003A HTTP stack, TDT/CHRONOS, CWC,
Rubin, GAP-169 composition). This decision uses 0039.

## Verification

```text
cargo fmt -p analysis_engine -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Known-truth reference series `([-1, 0, 1], intercept 0.5, loading 1.2)` versus
comparison `([1, 2, 3], 0.5, 1.2)` recovers difference `2.0` and classifies
`strict`. Metric-only intercept `1.5` returns `StrongInvarianceRequired`.
Two-observation series cap at `strong` (`#84` `scalar`) and recover `1.0`.
Cutoff exclusion, snapshot/profile mismatch, empty eligibility, raw
proportions, and singular loadings fail closed.

## Rollback and supersession

Rollback removes the `two_group_ols_invariance_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps metric
from licensing latent-mean comparison and keeps two-group OLS distinct from
MGCFA.
