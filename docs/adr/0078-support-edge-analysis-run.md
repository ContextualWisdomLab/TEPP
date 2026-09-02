# ADR 0078 — Support-edge refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-02
**Supersedes:** None; complements ADR 0002 (citation, revision, translation, and retrospective-reporting edges may point to the past but never become reverse state transitions), ADR 0003 (support, contradiction, summary, and `outcome_of` are not state transitions), and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0077 (summarizes-edge), ADR 0076 (retrospective-edge), ADR 0075 (role-contradiction), ADR 0074 (subevent-containment), ADR 0073 (inferred-status), ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat support, contradiction, summary,
and `outcome_of` as forward state transitions, via
`support_edge::EvidenceKind` and `refuse_evidence_as_transition`.
Operators still cannot request that mixed-kind census as a digest-bound
analysis-run output.

Summarizes-edge (#484 / ADR 0077) binds summary-versus-source identity
in the `summarizes_edge` crate. Retrospective-edge (#483 / ADR 0076)
binds later reports about earlier events. Role-contradiction (#482 /
ADR 0075) binds customer/competitor group overlap. Subevent-containment
(#478 / ADR 0074) binds parent/child interval containment.
Inferred-status (#473) binds observed-versus-inferred evidence.
Episode-membership (#461 / ADR 0072) binds membership windows.
Relation-absence (#460 / ADR 0071) binds observation status.
Outcome-order (#458 / ADR 0070) binds IPO event-time order.
Membership-target (#434 / ADR 0069) binds `MembershipTargetKind`.
Location-membership (#430 / ADR 0066) binds geographic/market assignment.
Copied-text (#427 / ADR 0065) binds unique-content/stopword vocabulary.
Copy-identity (#416 / ADR 0058) binds template-copy identity.

`edge_kind_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `support_edge_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `SupportEdgeAssignment` rows with closed
  `EvidenceKind` values and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction, comparing cutoffs as typed `KnowledgeCutoff::instant()`
  values rather than strings;
- excludes assignments whose availability is later than the knowledge cutoff
  before duplicate-identity checks;
- invokes `refuse_evidence_as_transition` without reimplementing the
  evidential vocabulary;
- requires a mixed census of all four evidential kinds after cutoff
  exclusion, with matching transition-refusal counts;
- emits a canonical SHA-256-digested `tepp.support_edge.v1` artifact
  with per-kind counts, matching refusal counts, and inference status
  `evidence_is_not_transition`;
- applies `MAX_EVIDENCE_UNITS` to both execution admission and artifact
  validation;
- does not emit `edge_kind_recovery_rate`, invent MCMC, select GPU backends,
  or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate summarizes-edge (#484 / ADR 0077) — rejected because that
   profile binds summary-versus-source identity in `summarizes_edge`, not
   the four-kind evidential-versus-transition gate.
2. Duplicate retrospective-edge (#483 / ADR 0076) — rejected because that
   profile binds later-report identity, not evidential kinds.
3. Duplicate role-contradiction (#482 / ADR 0075) — rejected because that
   profile binds customer/competitor overlap, not evidential kinds.
4. Duplicate outcome-order (#458 / ADR 0070) — rejected because that
   profile binds IPO event-time order, not evidential-versus-transition.
5. Put `edge_kind_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
6. Bind the existing support-edge refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe support-edge refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, summarizes-edge, retrospective-edge, role-contradiction,
subevent-containment, inferred-status, episode-membership,
relation-absence, outcome-order, membership-target, location-membership,
membership-posterior ICC, copied-text, copy-identity, citation-edge,
corpus-background, method-effect estimation, or topic birth/split/merge.
Snapshot / profile / cutoff mismatch, empty or incomplete mixed-kind
corpora, duplicate assignment identities, and oversized corpora fail
closed.

## Verification

The PR includes Rust unit and integration tests for mixed four-kind
corpora, cutoff exclusion, equivalent RFC 3339 cutoff instants,
empty/incomplete/duplicate refusal, snapshot / profile / cutoff
mismatch, oversize, compact census claims above `MAX_EVIDENCE_UNITS`,
and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `support_edge_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps support,
contradiction, summary, and `outcome_of` out of the transition vocabulary
and keeps `edge_kind_recovery_rate` off inspect payloads.
