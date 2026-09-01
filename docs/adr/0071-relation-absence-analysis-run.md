# ADR 0071 — Relation-absence refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0003 (unobserved pairs are not negative edges) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat an unobserved relation pair as
evidence of no relationship, via `relation_absence::ObservationStatus`
and `refuse_absence_as_negative`. Operators still cannot request that
census as a digest-bound analysis-run output.

Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.
Copied-text (#427 / ADR 0065) binds unique-content/stopword vocabulary.
Copy-identity (#416 / ADR 0058) binds template-copy identity.
Provenance-is-not-transition (#426 / ADR 0064) binds citation-edge.

`status_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. `no_relationship`
is not a wire status.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `relation_absence_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `RelationAbsencePair` rows with closed
  `ObservationStatus` values and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- excludes pairs whose availability is later than the knowledge cutoff;
- invokes `refuse_absence_as_negative` without reimplementing the
  `observed` / `inferred` / `unobserved` vocabulary;
- requires a mixed census of at least one `observed`, one `inferred`,
  and one `unobserved` after cutoff exclusion;
- emits a canonical SHA-256-digested `tepp.relation_absence.v1` artifact
  with per-status counts, matching unobserved refusal counts, and
  inference status `unobserved_is_not_negative_observed_inferred_are_presence`;
- does not emit `status_recovery_rate`, invent MCMC, select GPU backends,
  or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate outcome-order (#458 / ADR 0070) — rejected because that
   profile binds IPO event-time order, not observation status.
2. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`, not absence-is-not-negative.
3. Duplicate location-membership (#430 / ADR 0066) — rejected because
   that profile binds `location_membership` LocationKind refusals.
4. Duplicate copied-text (#427) or copy-identity (#416) — rejected
   because those profiles bind residue/template identity, not observation
   status.
5. Put `status_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
6. Bind the existing relation-absence refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe relation-absence refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, outcome-order, membership-target, location-membership,
membership-posterior ICC, copied-text, copy-identity, citation-edge,
corpus-background, method-effect estimation, or topic birth/split/merge.
Snapshot / profile / cutoff mismatch, empty or single-class corpora,
duplicate pair identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
`observed` / `inferred` / `unobserved` corpora, cutoff exclusion,
empty/single-class/duplicate refusal, snapshot / profile / cutoff
mismatch, oversize, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `relation_absence_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
`unobserved` out of the negative-edge vocabulary, keeps `observed` and
`inferred` as presence, and keeps `status_recovery_rate` off inspect
payloads.
