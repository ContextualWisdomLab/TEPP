# ADR 0074 — Subevent-containment refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-02
**Supersedes:** None; complements ADR 0003 (a subevent interval cannot escape the parent event-time interval) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0073 (inferred-status), ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy). This is subevent-versus-parent containment, not episode-membership-window containment.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses a half-open subevent interval that starts
before or ends after its parent, via `subevent_containment::EventInterval`
and `refuse_escaped_subevent`. Operators still cannot request that census
as a digest-bound analysis-run output.

Episode-membership (#461 / ADR 0072) binds membership windows to episode
intervals. Inferred-status (#473 / ADR 0073) binds inferred-versus-observed
refusals. Relation-absence (#460 / ADR 0071) binds observation status.
Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.

`containment_recovery_rate` and `identity_recovery_rate` stay library-side.
This slice does not put a `scientific_acceptance` metric on inspect
payloads. Episode-window containment remains `episode_membership`.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `subevent_containment_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `SubeventContainmentAssignment` rows with
  half-open `EventInterval` parent/child bounds and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- excludes assignments whose availability is later than the knowledge
  cutoff;
- invokes `refuse_escaped_subevent` without reimplementing the
  containment vocabulary;
- requires a mixed census of at least one contained and one escaped
  assignment after cutoff exclusion;
- emits a canonical SHA-256-digested `tepp.subevent_containment.v1` artifact
  with contained/escaped counts, matching escape-refusal counts, and
  inference status `subevent_interval_cannot_escape_parent_interval`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate episode-membership (#461 / ADR 0072) — rejected because that
   profile binds membership windows to episode intervals, not
   subevent-versus-parent containment.
2. Duplicate inferred-status (#473 / ADR 0073) — rejected because that
   profile binds inferred-versus-observed status.
3. Duplicate relation-absence (#460 / ADR 0071) — rejected because that
   profile binds observation status.
4. Duplicate outcome-order (#458 / ADR 0070) — rejected because that
   profile binds IPO event-time order.
5. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`.
6. Duplicate location-membership (#430 / ADR 0066) — rejected because
   that profile binds `location_membership` LocationKind refusals.
7. Put `containment_recovery_rate` or `identity_recovery_rate` on the
   operator artifact — rejected because inspect payloads stay metric-free
   and `tepp.scientific_acceptance.v1` never appears.
8. Bind the existing subevent-containment refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe subevent-containment refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, episode-membership, inferred-status, relation-absence,
outcome-order, membership-target, location-membership,
membership-posterior ICC, copied-text, copy-identity, citation-edge,
method-effect estimation, or topic birth/split/merge. Snapshot / profile
/ cutoff mismatch, empty or single-class corpora, duplicate assignment
identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
contained/escaped corpora, cutoff exclusion, empty/single-class/duplicate
refusal, snapshot / profile / cutoff mismatch, oversize, compact
`MAX_EVIDENCE_UNITS + 1` artifact refusal, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Exact-head hosted Rust Foundation CI, Documentation Quality, Security
Scan, and SAST Semgrep on this branch head are required landing evidence.
Predecessor-head checks do not transfer.

## Rollback and supersession

Rollback removes the `subevent_containment_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps subevent
intervals inside the parent interval, keeps this distinct from
episode-membership-window containment, and keeps recovery rates off
inspect payloads.
