# ADR 0075 — Role-contradiction refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-02
**Supersedes:** None; complements ADR 0003 (customer/competitor cannot occupy the same group; roles are not entity classes) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0074 (subevent-containment), ADR 0073 (inferred-status), ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses a customer/competitor pair in one group and
refuses to treat a contextual role as a permanent entity class, via
`role_contradiction::ContextualRole`, `refuse_contradictory_roles`, and
`refuse_role_as_entity_class`. Operators still cannot request that census
as a digest-bound analysis-run output.

Subevent-containment (#478 / ADR 0074) binds parent/child interval
containment. Inferred-status (#473) binds observed-versus-inferred
evidence. Episode-membership (#461 / ADR 0072) binds membership windows.
Relation-absence (#460 / ADR 0071) binds observation status.
Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.
Copied-text (#427 / ADR 0065) binds unique-content/stopword vocabulary.
Copy-identity (#416 / ADR 0058) binds template-copy identity.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `role_contradiction_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `RoleContradictionAssignment` rows with closed
  `ContextualRole` values, group identity, and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction, comparing cutoffs as typed `KnowledgeCutoff::instant()`
  values rather than strings;
- excludes assignments whose availability is later than the knowledge cutoff;
- invokes `refuse_role_as_entity_class` and `refuse_contradictory_roles`
  without reimplementing the customer/partner/competitor vocabulary;
- requires a mixed census of at least one customer, one partner, and one
  competitor after cutoff exclusion, with at least one contradictory pair
  and one compatible pair;
- emits a canonical SHA-256-digested `tepp.role_contradiction.v1` artifact
  with per-role counts, matching entity-class refusal counts, contradictory
  and compatible pair counts, and inference status
  `customer_competitor_cannot_share_group_role_is_not_entity_class`;
- applies `MAX_EVIDENCE_UNITS` to both execution admission and artifact
  validation;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU backends,
  or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate subevent-containment (#478 / ADR 0074) — rejected because that
   profile binds parent/child interval containment, not commercial roles.
2. Duplicate inferred-status (#473) — rejected because that profile binds
   observed-versus-inferred evidence, not customer/competitor overlap.
3. Duplicate episode-membership (#461 / ADR 0072) — rejected because that
   profile binds membership windows, not role contradiction.
4. Duplicate relation-absence (#460 / ADR 0071) — rejected because that
   profile binds `ObservationStatus`, not contextual roles.
5. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`, not customer/competitor overlap.
6. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
7. Bind the existing role-contradiction refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe role-contradiction refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, subevent-containment, inferred-status, episode-membership,
relation-absence, outcome-order, membership-target, location-membership,
membership-posterior ICC, copied-text, copy-identity, citation-edge,
corpus-background, method-effect estimation, or topic birth/split/merge.
Snapshot / profile / cutoff mismatch, empty or single-class corpora,
duplicate assignment identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
customer / partner / competitor corpora, cutoff exclusion, equivalent
RFC 3339 cutoff instants, empty/single-class/duplicate refusal, snapshot /
profile / cutoff mismatch, oversize, compact census claims above
`MAX_EVIDENCE_UNITS`, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `role_contradiction_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps customer
and competitor from sharing a group, keeps roles out of the entity-class
vocabulary, and keeps `identity_recovery_rate` off inspect payloads.
