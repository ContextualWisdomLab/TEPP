# ADR 0079 — Prediction-contradiction refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-03
**Supersedes:** None; complements ADR 0002 (six-clock eligibility and Allen algebra), ADR 0016 (TDT/CHRONOS prediction stays hypothetical until coverage), and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0078 (support-edge), ADR 0077 (summarizes-edge), ADR 0076 (retrospective-edge), ADR 0075 (role-contradiction), ADR 0074 (subevent-containment), ADR 0073 (inferred-status), ADR 0072 (episode-membership), ADR 0071 (relation-absence), ADR 0070 (outcome-order), ADR 0069 (membership-target), ADR 0066 (location-membership), ADR 0065 (copied-text residue), ADR 0064 (provenance-is-not-transition / citation-edge), or ADR 0058 (copy-identity / template-copy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to promote unmatched predicted mass to observed
fact, via `prediction_contradiction::refuse_promotion`. Coverage requires Allen
`during`, `starts`, `finishes`, or `equals`. Partial overlap, `meets` /
`met_by`, and Allen `before` / `after` stay hypothetical.
`refuse_contradiction_or_adjacency` is not promotion authority.
Operators still cannot request that mixed-kind census as a digest-bound
analysis-run output.

Support-edge (#485 / ADR 0078) binds evidential kinds that never become
transitions. Summarizes-edge (#484 / ADR 0077) binds summary-versus-source
identity. Retrospective-edge (#483 / ADR 0076) binds later reports about
earlier events. Role-contradiction (#482 / ADR 0075) binds customer/competitor
group overlap. Citation-edge (#426 / ADR 0064) binds provenance-is-not-transition.

`contradiction_agreement_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `prediction_contradiction_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `PredictionContradictionAssignment` rows with
  predicted and observed closed event-time intervals and availability time;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction, comparing cutoffs as typed `KnowledgeCutoff::instant()`
  values rather than strings;
- excludes assignments whose availability is later than the knowledge cutoff
  before duplicate-identity checks;
- invokes `refuse_promotion` without reimplementing Allen classification;
- requires a mixed census of covered, partial-overlap, adjacent, and
  contradictory pairs after cutoff exclusion, with matching promotion-refusal
  counts;
- emits a canonical SHA-256-digested `tepp.prediction_contradiction.v1`
  artifact with per-kind counts, matching refusal counts, and inference status
  `unmatched_prediction_is_not_observed`;
- applies `MAX_EVIDENCE_UNITS` to both execution admission and artifact
  validation;
- does not emit `contradiction_agreement_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate support-edge (#485 / ADR 0078) — rejected because that profile
   binds evidential-versus-transition kinds, not predicted-versus-observed
   promotion.
2. Duplicate citation-edge (#426 / ADR 0064) — rejected because that profile
   binds provenance identity, not promotion coverage.
3. Duplicate role-contradiction (#482 / ADR 0075) — rejected because that
   profile binds customer/competitor overlap, not predicted intervals.
4. Bind `refuse_contradiction_or_adjacency` as promotion authority — rejected
   because partial overlap still leaves unmatched predicted mass.
5. Put `contradiction_agreement_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and `tepp.scientific_acceptance.v1`
   never appears.
6. Bind the existing prediction-contradiction refusals to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe prediction-contradiction refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, support-edge, summarizes-edge, retrospective-edge, role-contradiction,
subevent-containment, inferred-status, episode-membership, relation-absence,
outcome-order, membership-target, location-membership, membership-posterior
ICC, copied-text, copy-identity, citation-edge, corpus-background,
method-effect estimation, or topic birth/split/merge. Snapshot / profile /
cutoff mismatch, empty or incomplete mixed-kind corpora, duplicate assignment
identities, and oversized corpora fail closed.

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

Rollback removes the `prediction_contradiction_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
unmatched predicted mass hypothetical and keeps
`contradiction_agreement_rate` off inspect payloads.
