# ADR 0036 — Longitudinal ESEM/DSEM engine composition as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0005 (posterior-aware ESEM/DSEM interpretation) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already recovers construct-class gates, posterior-draw point
estimates, strong/strict-gated latent means, event-time lag clocks, nested ICC
refusal for non-nested membership, and within/between component identity.
Operators still cannot request that composition as a digest-bound analysis-run
output. Recovery primitives and Driver p.16 `std`-family restores are not the
ESEM/DSEM engine (GAP-169 / #169).

## Decision

Add the `longitudinal_esem_dsem_composition_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-mapped posterior draws plus an explicit measurement design
  and `available_time`;
- excludes observations whose availability is later than the request
  `knowledge_cutoff`;
- refuses point estimates (`InsufficientDraws` for fewer than two draws);
- invokes `interpret_as_reflective`, `compare_latent_means`,
  `refuse_between_as_within_change`, nested-ICC membership refusal, and
  event-time lag admission without inventing an ESEM/DSEM sampler;
- invokes `claim_causal_effect` so temporal precedence cannot promote the
  composition to a causal estimand;
- refuses OLS recovery offered as DSEM;
- emits a canonical SHA-256-digested
  `tepp.longitudinal_esem_dsem_composition.v1` artifact with observation/draw
  counts, excluded-after-cutoff count, posterior-draw mean, construct class,
  preserved membership design, within component, event-time clock, strong or
  strict invariance, and inference status `composed_engine_not_estimator`;
- does not persist rows, collapse cross-classified or multiple-membership
  designs, or claim implemented-main.

Cross-classified membership without collapse is recorded as `cross_classified`.
This is engine composition, not an estimator, not CWC slopes, and not a
`std`-family restore.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind composition to
   an analysis run.
2. Put ESEM/DSEM composition into `tepp_api` — rejected because transport
   contracts and scientific composition would become one service boundary.
3. Bind recovered psychometric, longitudinal, and membership gates to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe longitudinal ESEM/DSEM engine composition as a
digest-bound terminal result. The artifact is not an ESEM fit, not a DSEM
sampler, not a causal effect, and not implemented-main until exact-head Checks
and two independent approvals land.

## Verification

```text
cargo fmt -p analysis_engine -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Known-truth equal draws recover posterior-draw mean `0.5`. Cutoff exclusion,
snapshot/profile mismatch, empty eligibility, point estimates, formative or
network reinterpretation, metric-only means, between-as-within, non-event
clocks, collapsed non-nested membership, OLS-as-DSEM, and causal-from-precedence
fail closed. Cross-classified without collapse succeeds.

## Rollback and supersession

Rollback removes the `longitudinal_esem_dsem_composition_v1` profile. No
persisted schema migration is introduced. Supersede only with an ADR that keeps
engine composition distinct from a fitted ESEM/DSEM estimator and from causal
identification.
