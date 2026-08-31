# ADR 0033 — Longitudinal CWC composition as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers Enders and Tofighi (2007) cluster-mean-centered
within/between OLS and the CWC contextual effect inside `psychometric_core`.
Operators still cannot request that composition as a digest-bound analysis-run
output. Recovery primitives alone are not the ESEM/DSEM engine (GAP-006 / #169).
A second Driver p.16 `std`-family restore would not close this operator-visible
gap.

## Decision

Add the `longitudinal_cwc_v1` analysis-run output profile to `analysis_engine`.
The executor:

- consumes already-mapped clustered predictor/outcome coordinates plus
  `available_time`;
- excludes rows whose availability is later than the request `knowledge_cutoff`;
- invokes `recover_cluster_mean_within_between_slopes` without reimplementing
  CWC;
- invokes `claim_causal_effect` so temporal precedence cannot promote the
  slopes to a causal estimand;
- emits a canonical SHA-256-digested `tepp.longitudinal_cwc.v1` artifact with
  row/cluster counts, excluded-after-cutoff count, within/between/contextual
  slopes, and inference status `composed_cwc_slopes_not_causal`;
- does not invent an ESEM/DSEM sampler, persist rows, or claim strong
  invariance or Rubin pooling.

This is two-level OLS composition, not DSEM, not RI-CLPM, and not a
random-effects sampler.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind composition to
   an analysis run.
2. Put CWC execution into `tepp_api` — rejected because transport contracts and
   scientific composition would become one service boundary.
3. Bind the existing `psychometric_core` CWC recovery to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe within/between/contextual slopes as a
digest-bound terminal result. The artifact is not a causal effect, not an ESEM
fit, and not implemented-main until exact-head Checks and two independent
approvals land.

## Verification

```text
cargo fmt -p analysis_engine -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Known-truth noiseless CWC recovers within `0.5`, between `2.0`, contextual
`1.5`. Cutoff exclusion, snapshot/profile mismatch, empty eligibility, and
single-cluster remainder fail closed.

## Rollback and supersession

Rollback removes the `longitudinal_cwc_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps CWC distinct
from between-cluster effects and from causal identification.
