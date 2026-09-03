# ADR 0072 — Episode-membership refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0003 (episode membership cannot escape the episode event-time interval) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy). This is membership-window containment, not subevent-versus-parent containment.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses a membership window that starts before or
ends after its episode, via `episode_membership::EventWindow` and
`refuse_membership_outside_episode`. Operators still cannot request that
census as a digest-bound analysis-run output.

Relation-absence (#460 / ADR 0071) binds observation status.
Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. Subevent parent-window
containment remains `subevent_containment`.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `episode_membership_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `EpisodeMembershipAssignment` rows with
  closed `EventWindow` membership/episode bounds and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- excludes assignments whose availability is later than the knowledge
  cutoff;
- invokes `refuse_membership_outside_episode` without reimplementing the
  containment vocabulary;
- requires a mixed census of at least one contained and one escaped
  assignment after cutoff exclusion;
- emits a canonical SHA-256-digested `tepp.episode_membership.v1` artifact
  with contained/escaped counts, matching escape-refusal counts, and
  inference status `membership_window_cannot_escape_episode_interval`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate relation-absence (#460 / ADR 0071) — rejected because that
   profile binds observation status, not episode-window containment.
2. Duplicate outcome-order (#458 / ADR 0070) — rejected because that
   profile binds IPO event-time order.
3. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`.
4. Duplicate location-membership (#430 / ADR 0066) — rejected because
   that profile binds `location_membership` LocationKind refusals.
5. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
6. Bind the existing episode-membership refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe episode-membership refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, relation-absence, outcome-order, membership-target,
location-membership, membership-posterior ICC, copied-text,
copy-identity, citation-edge, subevent containment, method-effect
estimation, or topic birth/split/merge. Snapshot / profile / cutoff
mismatch, empty or single-class corpora, duplicate assignment
identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
contained/escaped corpora, cutoff exclusion, empty/single-class/duplicate
refusal, snapshot / profile / cutoff mismatch, oversize, and artifact
tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `episode_membership_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps membership
windows inside the episode interval, keeps this distinct from
subevent-versus-parent containment, and keeps `identity_recovery_rate`
off inspect payloads.
