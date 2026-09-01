# ADR 0073 — Inferred-status refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0003 (inferred relations cannot be promoted to observed evidence or transitions) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy). This is observed-versus-inferred promotion refusal, not unobserved-is-not-negative and not membership-window containment.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat an inferred relation as observed
evidence or as a state transition, via `inferred_status::EvidenceStatus`,
`refuse_inferred_as_observed`, and `refuse_inferred_as_transition`.
Operators still cannot request that census as a digest-bound analysis-run
output.

Episode-membership (#461 / ADR 0072) binds membership-window containment.
Relation-absence (#460 / ADR 0071) binds `unobserved` as not-negative
with a three-status vocabulary that already treats `inferred` as
presence. Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. `unobserved` and
`no_relationship` are not wire statuses here.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `inferred_status_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `InferredStatusEvidence` rows with closed
  `EvidenceStatus` values and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- excludes rows whose availability is later than the knowledge cutoff;
- invokes `refuse_inferred_as_observed` and
  `refuse_inferred_as_transition` without reimplementing the
  `observed` / `inferred` vocabulary;
- requires a mixed census of at least one `observed` and one `inferred`
  after cutoff exclusion;
- emits a canonical SHA-256-digested `tepp.inferred_status.v1` artifact
  with per-status counts, matching inferred refusal counts, and
  inference status `inferred_is_not_observed_and_not_transition`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate episode-membership (#461 / ADR 0072) — rejected because that
   profile binds membership-window containment, not inferred promotion.
2. Duplicate relation-absence (#460 / ADR 0071) — rejected because that
   profile binds `unobserved` as not-negative and already treats
   `inferred` as presence.
3. Duplicate outcome-order (#458 / ADR 0070) — rejected because that
   profile binds IPO event-time order.
4. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`.
5. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
6. Bind the existing inferred-status refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe inferred-status refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, relation-absence, episode-membership, outcome-order,
membership-target, location-membership, membership-posterior ICC,
copied-text, copy-identity, citation-edge, subevent containment,
method-effect estimation, or topic birth/split/merge. Snapshot /
profile / cutoff mismatch, empty or single-class corpora, duplicate
evidence identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
`observed` / `inferred` corpora, cutoff exclusion,
empty/single-class/duplicate refusal, snapshot / profile / cutoff
mismatch, oversize, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `inferred_status_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps inferred
relations out of observed evidence and out of transitions, keeps this
distinct from relation-absence unobserved-is-not-negative, and keeps
`identity_recovery_rate` off inspect payloads.
