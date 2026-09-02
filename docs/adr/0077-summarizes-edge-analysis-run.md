# ADR 0077 — Summarizes-edge refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-02
**Supersedes:** None; complements ADR 0002 (citation, revision, translation, and retrospective-reporting edges may point to the past but never become reverse state transitions), ADR 0003 (a summary is not a state transition and not the source document), and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0076 (retrospective-edge), ADR 0075 (role-contradiction), ADR 0074 (subevent-containment), ADR 0073 (inferred-status), ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat a summary as a forward state
transition or as the source document identity, via
`summarizes_edge::SummarizesKind`,
`refuse_summary_as_transition`, and
`refuse_summary_as_source_identity`. Operators still cannot request
that census as a digest-bound analysis-run output.

Retrospective-edge (#483 / ADR 0076) binds later reports about earlier
events. Role-contradiction (#482 / ADR 0075) binds customer/competitor
group overlap. Subevent-containment (#478 / ADR 0074) binds parent/child
interval containment. Inferred-status (#473) binds observed-versus-inferred
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

Add the `summarizes_edge_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `SummarizesEdgeAssignment` rows with closed
  `SummarizesKind` values and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction, comparing cutoffs as typed `KnowledgeCutoff::instant()`
  values rather than strings;
- excludes assignments whose availability is later than the knowledge cutoff;
- invokes `refuse_summary_as_transition` and
  `refuse_summary_as_source_identity` without reimplementing the
  summary/source vocabulary;
- requires a mixed census of at least one summary and one source document
  after cutoff exclusion, with matching transition and source-identity
  refusal counts and a matching compatible-source count;
- emits a canonical SHA-256-digested `tepp.summarizes_edge.v1` artifact
  with per-kind counts, matching refusal counts, compatible-source counts,
  and inference status
  `summary_is_not_transition_and_not_source_identity`;
- applies `MAX_EVIDENCE_UNITS` to both execution admission and artifact
  validation;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU backends,
  or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate retrospective-edge (#483 / ADR 0076) — rejected because that
   profile binds later-report identity, not summary-versus-source identity.
2. Duplicate role-contradiction (#482 / ADR 0075) — rejected because that
   profile binds customer/competitor overlap, not summary identity.
3. Duplicate copy-identity (#416 / ADR 0058) — rejected because that
   profile binds template-copy identity, not summary-versus-source.
4. Duplicate citation-edge (#426 / ADR 0064) — rejected because that
   profile binds provenance-is-not-transition, not summary identity.
5. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
6. Bind the existing summarizes-edge refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe summarizes-edge refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, retrospective-edge, role-contradiction, subevent-containment,
inferred-status, episode-membership, relation-absence, outcome-order,
membership-target, location-membership, membership-posterior ICC,
copied-text, copy-identity, citation-edge, corpus-background,
method-effect estimation, or topic birth/split/merge. Snapshot / profile /
cutoff mismatch, empty or single-kind corpora, duplicate assignment
identities, and oversized corpora fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed
summary / source-document corpora, cutoff exclusion, equivalent RFC 3339
cutoff instants, empty/single-kind/duplicate refusal, snapshot / profile /
cutoff mismatch, oversize, compact census claims above
`MAX_EVIDENCE_UNITS`, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `summarizes_edge_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps summaries
out of the transition and source-document vocabularies and keeps
`identity_recovery_rate` off inspect payloads.
