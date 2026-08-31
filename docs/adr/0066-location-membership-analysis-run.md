# ADR 0066 — Location-membership refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0003 (location is a time-varying market membership, not entity identity and not a language channel) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0065 (copied-text residue), ADR 0064 (citation-edge provenance-is-not-transition), ADR 0063 (lineage-criterion fitting), ADR 0062 (corpus-background), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat location membership as permanent
entity identity or as a language channel via
`location_membership::refuse_location_as_entity_identity` and
`refuse_location_as_language_channel`. Operators still cannot request
that refusal census as a digest-bound analysis-run output.
Membership-posterior ICC (#398) binds a psychometric ICC estimator and
does not replace `location_membership`. Copied-text refusals (#427 /
ADR 0065) bind unique-content/stopword vocabulary and do not replace
location-versus-entity identity.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `location_membership_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `LocationMembershipDocument` rows with
  closed `LocationKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_location_as_entity_identity` and
  `refuse_location_as_language_channel` without reimplementing the
  location/entity/language vocabulary;
- requires at least one location membership and at least one
  non-location treatment so the census is mixed;
- emits a canonical SHA-256-digested `tepp.location_membership.v1`
  artifact with per-kind counts, matching refusal counts, and inference
  status `location_is_not_entity_identity_not_language_channel`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate membership-posterior ICC (#398) — rejected because that
   profile binds a psychometric ICC estimator and does not bind
   `location_membership`.
2. Duplicate copied-text refusals (#427) — rejected because that
   profile binds unique-content/stopword vocabulary, not
   location-versus-entity identity.
3. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
4. Bind the existing location-membership refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe location-membership refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, membership-posterior ICC, copied-text, citation-edge,
corpus-background, method-effect estimation, or topic birth/split/merge.
Snapshot/profile/cutoff mismatch, empty or single-kind corpora, and
duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
location/entity/language corpora, empty/single-kind/duplicate refusal,
snapshot / profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `location_membership_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
location membership distinct from entity identity, language channels,
and `identity_recovery_rate` inspect metrics.
