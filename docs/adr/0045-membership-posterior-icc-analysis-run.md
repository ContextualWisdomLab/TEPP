# ADR 0045 — Nested ICC of posterior coordinates under classified membership

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0003 (multiple membership), ADR 0005 (posterior coordinates), and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already classifies nested, cross-classified, and multiple-membership
designs inside `membership_core`, recovers a nested ANOVA ICC only for nested
designs, and computes Kish ESS of membership weights. `psychometric_core`
already averages finite posterior-draw point estimates without Rubin pooling.
Operators still cannot request that composition as a digest-bound analysis-run
output. Recovery primitives alone are not an MMMC sampler and are not the
ESEM/DSEM engine (GAP-006 / #169). A second Driver p.16 `std`-family restore,
Leiden consensus, GAP-003A scientific-acceptance wiring, Compose persistence,
CWC slopes, Rubin loading uncertainty, longitudinal ESEM/DSEM collapse gating,
two-group OLS invariance, or irregular event-time log-rate would not close this
operator-visible gap.

Collapsing multiple membership into a nested ICC is the atomistic fallacy.
Rubin `T` is a different estimand already bound on a live analysis-run profile.

## Decision

Add the `membership_posterior_icc_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes posterior-draw observations bound to a membership assignment and
  `available_time`;
- excludes observations whose availability is later than the request
  `knowledge_cutoff`;
- forms point estimates through `posterior_draw_point_estimate_mean` and does
  not invoke Rubin pooling;
- inserts admitted assignments into `MembershipNetwork` and classifies the
  design at a caller-supplied event-time instant without collapsing structure;
- invokes `nested_intraclass_correlation` only for `MembershipDesign::Nested`;
- refuses nested ICC for multiple-membership and cross-classified designs while
  still emitting Kish ESS of admitted membership weights;
- emits a canonical SHA-256-digested `tepp.membership_posterior_icc.v1` artifact
  with design, eligible counts, optional nested ICC, Kish ESS, and an inference
  status that names the claim boundary;
- does not invent an MMMC sampler, persist rows, or claim ESEM/DSEM.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind membership
   design to an analysis run.
2. Bind Kish-weighted CWC (#312) — rejected because that psychometric expose is
   already a live draft and is a different estimand.
3. Reuse the ESEM/DSEM membership-design collapse gate (#376) — rejected because
   that profile treats design as an ESEM/DSEM admission gate and does not emit
   nested ICC or Kish ESS of posterior-draw membership weights.
4. Bind Rubin loading uncertainty (#374) — rejected because Rubin `T` is not
   the posterior-mean point estimate used here.
5. Put membership ICC into `tepp_api` — rejected because transport contracts
   and scientific composition would become one service boundary.
6. Bind existing `membership_core` nested ICC plus Kish ESS and
   `psychometric_core` posterior means to ADR 0022's analysis-run profile —
   accepted.

## Consequences

Operators can request cutoff-safe nested ICC of posterior means when membership
is nested, and still receive Kish ESS when membership is multiple or
cross-classified, without collapsing those designs. The engine does not expose
source text or identity mappings. Nested ICC remains undefined for MMMC; the
artifact records the refusal instead of substituting a nested number.

Cutoff exclusion, snapshot/profile mismatch, empty eligibility, invalid draws,
duplicate assignments, and oversize corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for nested ANOVA recovery of
posterior means, cutoff exclusion, multiple-membership and cross-classified
refusal of nested ICC with Kish ESS, snapshot/profile mismatch, empty
eligibility, non-finite draws, oversize corpora, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

Exact-head Checks and two independent approvals are required before any
implemented-main claim.

## Rollback and supersession

Rollback removes the `membership_posterior_icc_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps nested ICC
distinct from multiple-membership/cross-classified designs and keeps posterior
means distinct from Rubin pooling.
