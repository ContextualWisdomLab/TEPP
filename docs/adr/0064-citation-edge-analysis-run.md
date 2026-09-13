# ADR 0064 — Provenance-is-not-transition refusals as an analysis-run output profile

**Decision status:** Proposed
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

The initial branch called the profile cutoff-safe while
`CitationEdgeDocument` carried neither immutable source snapshot provenance
nor `AvailableTime`. It also compared RFC 3339 cutoff strings rather than
the represented temporal instant, admitted every supplied row into duplicate
and domain checks, had no raw `MAX_EVIDENCE_UNITS` guard before identity-set
allocation, counted the raw slice rather than the cutoff-admitted census,
and reused the domain inference claim as terminal provider validation state.
Those properties allow future evidence or harmless timestamp spelling to
change a historical result and therefore are not acceptable cutoff semantics.

`edge_kind_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `citation_edge_v1` analysis-run output profile to
`analysis_engine` as Proposed implementation lineage. The executor:

- requires every `CitationEdgeDocument` to carry a bounded document identity,
  immutable source `snapshot_id`, closed `ProvenanceKind`, and explicit
  `AvailableTime`;
- compares request and executor cutoffs as parsed `KnowledgeCutoff::instant()`
  values, not RFC 3339 text;
- rejects cross-snapshot evidence before aggregation;
- excludes same-snapshot evidence with `AvailableTime > knowledge_cutoff`
  before duplicate-identity or provenance-domain admission, so unavailable
  future rows cannot perturb a historical replay;
- preserves fail-closed duplicate detection for evidence actually visible at
  the cutoff;
- rejects raw input larger than `MAX_EVIDENCE_UNITS` before identity-set
  allocation and derives `document_count` from admitted identities;
- invokes `refuse_provenance_as_transition` without reimplementing the
  provenance vocabulary and accepts only the canonical
  `ProvenanceIsNotTransition` refusal; unexpected provider success or another
  present/future provider error fails closed through a directly tested guard;
- requires at least two admitted documents and at least two distinct
  provenance kinds so the census is not a single-kind dump;
- emits a SHA-256-digested `tepp.citation_edge.v1` artifact with per-kind
  counts, matching refusal counts, and inference status
  `provenance_is_not_a_state_transition`;
- keeps the 256 KiB cap on untrusted `from_json` input, while a maximal-valid
  output proof over the 256-byte identifier bounds, strict at-most-nine-digit
  RFC 3339 fraction/offset syntax, and bounded census demonstrates canonical
  output cannot reach that cap; the redundant post-validation egress branch is
  therefore absent;
- keeps terminal provider validation state separate as `validated`;
- does not emit `edge_kind_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

Historical replay invariant: adding a same-snapshot row that is unavailable
at the requested cutoff cannot change duplicate admission, kind counts,
artifact identity, or terminal result. Cross-snapshot evidence is not censored
as historical data; it is a provenance violation and fails closed.

## Alternatives considered

1. Duplicate lineage-criterion fitting (#423) — rejected because that
   profile binds TDT link-criterion posteriors and does not bind
   `citation_edge`.
2. Duplicate corpus-background refusals (#422) — rejected because that
   profile binds unique-content/stopword vocabulary, not
   provenance-is-not-transition.
3. Treat every supplied row as cutoff-admitted — rejected because the caller
   cannot prove historical availability without explicit row provenance and a
   future row could alter an earlier result.
4. Compare RFC 3339 strings — rejected because different legal spellings can
   denote the same temporal instant.
5. Reject all post-cutoff rows — rejected for same-snapshot historical
   evidence because the Analysis Run contract censors unavailable future rows;
   cross-snapshot rows still fail closed as provenance violations.
6. Keep an output-side 256 KiB branch after validating bounded fields —
   rejected after the maximal-valid wire proof showed the branch cannot be
   reached; the independent untrusted-input cap remains.
7. Put `edge_kind_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
8. Bind the existing citation-edge refusals to ADR 0022's analysis-run
   profile — selected, subject to this Proposed implementation reaching
   protected-main acceptance.

## Consequences

Operators can eventually request provenance-is-not-transition refusals whose
historical census is explicitly bound to immutable snapshot and availability
provenance. Future-unavailable rows cannot affect an earlier run; visible
duplicates still fail closed. The adapter also fails closed if the non-exhaustive
`CitationEdgeError` provider contract drifts away from the exact refusal this
profile claims. The artifact does not claim MCMC, GPU parity,
lineage-criterion fitting, corpus-background, modality-source, prompt-source,
style-source, copy-identity, method-effect estimation, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, cross-snapshot evidence,
oversized raw corpora, empty or single-kind admitted corpora, and duplicate
visible identities fail closed.

Because this decision is not implemented on protected `main`, this ADR remains
`Proposed`. The shared ADR index must not claim `Accepted` authority for this
branch-local implementation.

## Verification

The branch preserves explicit RED → repair evidence:

- `74d6ea13c211e3218bb4f61654f53a5b44badeb7` adds contracts proving that
  equivalent RFC 3339 spellings of one instant bind identically and terminal
  validation state is distinct from the domain inference claim;
- `3aac0968af752dee7b12ce542b8c73c3e1c7f036` adds explicit snapshot and
  availability provenance, parsed-instant cutoff binding, cutoff-before-
  duplicate admission, raw population bounding, admitted-count semantics and
  provider/domain status separation;
- `e05a38ecd52f60aee8dcfaa039862c7526d35570` exercises historical replay
  invariance to a future duplicate identity, cross-snapshot refusal and the raw
  population bound;
- `742b65bc0dc4d8c745cf3bc1f7b924abd1de7b80` promotes canonical
  `corpus_split::cutoff_eligible` to a production dependency and removes
  duplicate development-only dependency declarations;
- `b780479fa9bcb9624e53dc79521899708163a05c` moves the non-exhaustive
  provider-result fallback behind a directly testable fail-closed guard and
  adds the bounded-output proof before removing only the unreachable egress
  limit branch;
- `f9e5a17f52e97efd0d2b564d31e8af4bdf8d93db` strengthens that proof with the
  maximum permitted identifier lengths and maximum strict RFC 3339 timestamp
  spelling.

Run on the exact surviving head:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Hosted exact-head line/branch coverage, security, CodeQL, documentation and
independent review remain required; predecessor receipts do not transfer.

## Rollback and supersession

Rollback removes the `citation_edge_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that preserves explicit
snapshot/availability provenance, instant-based cutoff semantics, historical
replay invariance, fail-closed provider-drift handling,
provenance-is-not-transition distinctness from unique-content/stopword
refusals, and the metric-free inspect boundary.
