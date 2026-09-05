# ADR 0040 — Irregular event-time log-rate as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers Voelkle, Oud, Davidov, and Schmidt (2012) exact
scalar local log-rates and maps discrete lags across unequal event intervals
inside `psychometric_core`. Operators still cannot request that composition as a
digest-bound analysis-run output. Recovery primitives alone are not the
ESEM/DSEM engine (GAP-006 / #169). A second Driver p.16 `std`-family restore,
Leiden consensus, GAP-003A scientific-acceptance wiring, or Compose persistence
would not close this operator-visible gap.

Discrete autoregressive coefficients from unequal intervals are not one
coefficient. The licensed path is `a = ln(φ_src) / Δt_src` then
`φ_ref = exp(a Δt_ref)`. Pooling those discrete lags fails closed.

## Decision

Add the `irregular_event_time_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-mapped event-time scores plus `available_time`;
- excludes occasions whose availability is later than the request
  `knowledge_cutoff`;
- invokes `recover_event_series_mean_log_rate` without reimplementing the
  exponential map;
- maps the first eligible discrete lag onto a caller-supplied reference
  interval through `map_discrete_lag_across_event_intervals`;
- invokes `refuse_pooled_discrete_lag_across_unequal_intervals` when the caller
  asks to pool unequal intervals;
- invokes `claim_causal_effect` so temporal precedence cannot promote the
  log-rate to a causal estimand;
- emits a canonical SHA-256-digested `tepp.irregular_event_time.v1` artifact
  with occasion/interval counts, excluded-after-cutoff count, mean log-rate,
  mapped reference lag, and inference status
  `composed_interval_mapped_lags_not_dsem`;
- does not invent a DSEM sampler, persist rows, or claim strong invariance.

This is scalar event-time composition, not DSEM, not RI-CLPM, and not a
random-effects sampler.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind composition to
   an analysis run.
2. Bind CWC-then-irregular residual log-rate — rejected because that psychometric
   expose is already a live draft (#327) and is a different estimand.
3. Put interval mapping into `tepp_api` — rejected because transport contracts
   and scientific composition would become one service boundary.
4. Bind the existing `psychometric_core` irregular event-time recovery to
   ADR 0022's analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe irregular event-time log-rates as a
digest-bound terminal result. The artifact is not a causal effect, not a DSEM
fit, and not implemented-main until exact-head Checks and two independent
approvals land. Security and privacy are unchanged: scores are identity-free
mapped coordinates. Standalone/MSA boundaries stay with `analysis_engine`; no
cross-service table access is introduced.

## Verification

```text
cargo fmt -p analysis_engine -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Known-truth noiseless scores `1.0`, `0.5`, `0.25` at event times `0`, `1`, `2`
recover mean log-rate `ln(0.5)` and mapped reference lag `0.25` at `Δt_ref = 2`.
Cutoff exclusion, snapshot/profile mismatch, pooled unequal intervals, non-event
clocks, empty eligibility, and oversize corpora fail closed.

## Rollback and supersession

Rollback removes the `irregular_event_time_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps per-interval
log-rate mapping distinct from pooled discrete lags and from DSEM.
