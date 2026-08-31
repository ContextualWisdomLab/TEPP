# ADR 0064 — Provenance-is-not-transition refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0002/0003 (citation and retrospective edges are not state transitions) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0063 (lineage-criterion fitting), ADR 0062 (corpus-background), ADR 0061 (modality-source), ADR 0060 (prompt-source), ADR 0059 (style-source), ADR 0058 (copy-identity), or ADR 0057 (simulation method-effect census).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat citation, translation, revision,
and retrospective-report edges as forward state transitions via
`citation_edge::refuse_provenance_as_transition`. Operators still cannot
request that refusal census as a digest-bound analysis-run output.
Lineage-criterion fitting (#423 / ADR 0063) binds TDT link-criterion
posteriors and does not replace `citation_edge`. Corpus-background
refusals (#422 / ADR 0062) bind unique-content/stopword vocabulary and
do not replace provenance-is-not-transition.

`edge_kind_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `citation_edge_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `CitationEdgeDocument` rows with closed
  `ProvenanceKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_provenance_as_transition` without reimplementing the
  provenance vocabulary;
- requires at least two documents and at least two distinct provenance
  kinds so the census is not a single-kind dump;
- emits a canonical SHA-256-digested `tepp.citation_edge.v1` artifact with
  per-kind counts, matching refusal counts, and inference status
  `provenance_is_not_a_state_transition`;
- does not emit `edge_kind_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate lineage-criterion fitting (#423) — rejected because that
   profile binds TDT link-criterion posteriors and does not bind
   `citation_edge`.
2. Duplicate corpus-background refusals (#422) — rejected because that
   profile binds unique-content/stopword vocabulary, not
   provenance-is-not-transition.
3. Put `edge_kind_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
4. Bind the existing citation-edge refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe provenance-is-not-transition refusals
as a digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, lineage-criterion fitting, corpus-background, modality-source,
prompt-source, style-source, copy-identity, method-effect estimation, or
topic birth/split/merge. Snapshot/profile/cutoff mismatch, empty or
single-kind corpora, and duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed provenance
corpora, empty/single-kind/duplicate refusal, snapshot / profile /
cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `citation_edge_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
provenance-is-not-transition distinct from unique-content/stopword
refusals and from `edge_kind_recovery_rate` inspect metrics.
