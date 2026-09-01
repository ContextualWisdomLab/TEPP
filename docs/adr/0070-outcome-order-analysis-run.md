# ADR 0070 — Input-process-outcome order refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0002 (forward IPO transitions never move backward in event time; `outcome_of` is provenance) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0069 (membership-target), ADR 0068 (topic-context posterior), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses reverse or contemporaneous event-time rank on
`input_to` and `process_to`, and refuses to treat `outcome_of` as a state
transition, via `outcome_order::OutcomeKind`, `refuse_reverse_ipo_order`, and
`refuse_outcome_of_as_transition`. Operators still cannot request that IPO
census as a digest-bound analysis-run output.

Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.
Copied-text (#427 / ADR 0065) binds unique-content/stopword vocabulary.
Copy-identity (#416 / ADR 0058) binds template-copy identity.
Provenance-is-not-transition (#426 / ADR 0064) binds citation-edge, not
`outcome_of`.

`kind_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `outcome_order_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `OutcomeOrderEdge` rows with closed
  `OutcomeKind` values, opaque event-time ranks, and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- excludes edges whose availability is later than the knowledge cutoff;
- invokes `refuse_reverse_ipo_order` and
  `refuse_outcome_of_as_transition` without reimplementing the
  `input_to` / `process_to` / `outcome_of` vocabulary;
- requires a mixed census of at least one `input_to`, one `process_to`,
  and one `outcome_of` after cutoff exclusion;
- emits a canonical SHA-256-digested `tepp.outcome_order.v1` artifact
  with per-kind counts, matching `outcome_of` refusal counts, and
  inference status `input_process_forward_outcome_of_is_not_transition`;
- does not emit `kind_recovery_rate`, invent MCMC, select GPU backends,
  or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate membership-target (#434 / ADR 0069) — rejected because that
   profile binds `MembershipTargetKind`, not IPO event-time order.
2. Duplicate location-membership (#430 / ADR 0066) — rejected because
   that profile binds `location_membership` LocationKind refusals.
3. Duplicate copied-text (#427) or copy-identity (#416) — rejected
   because those profiles bind residue/template identity, not IPO kinds.
4. Duplicate citation-edge (#426 / ADR 0064) — rejected because that
   profile binds citation provenance, not `outcome_of`.
5. Put `kind_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
6. Bind the existing outcome-order refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe IPO-order refusals as a digest-bound
terminal result. The artifact does not claim MCMC, GPU parity,
membership-target, location-membership, membership-posterior ICC,
copied-text, copy-identity, citation-edge, corpus-background,
method-effect estimation, or topic birth/split/merge. Snapshot / profile
/ cutoff mismatch, empty or single-class corpora, reverse or uncertain
IPO order, duplicate edge identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
`input_to` / `process_to` / `outcome_of` corpora, cutoff exclusion,
empty/single-class/duplicate/reverse/uncertain refusal, snapshot /
profile / cutoff mismatch, oversize, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `outcome_order_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps `input_to`
and `process_to` strictly forward in event-time rank, keeps `outcome_of`
out of the transition vocabulary, and keeps `kind_recovery_rate` off
inspect payloads.
