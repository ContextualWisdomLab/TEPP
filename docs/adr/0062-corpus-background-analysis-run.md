# ADR 0062 — Corpus-background refusals as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (corpus-background wording is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0061 (modality-source), ADR 0060 (prompt-source), ADR 0059 (style-source), ADR 0058 (copy-identity), or ADR 0057 (simulation method-effect census).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat corpus-background wording as unique
latent content or as stopword deletion via
`corpus_background::refuse_corpus_background_as_unique_content` and
`refuse_corpus_background_as_stopword_deletion`. Operators still cannot
request that refusal census as a digest-bound analysis-run output.
Modality-source refusals (#421 / ADR 0061) bind `ModalityKind` and do not
replace `corpus_background`.

The original branch implementation called this profile cutoff-safe while
`CorpusBackgroundDocument` carried no `AvailableTime`. It also compared the
request cutoff to a canonical RFC 3339 rendering as text. That made equivalent
representations of the same instant fail and left the executor unable to
exclude evidence that did not exist at the historical cutoff. A future row
could therefore enter duplicate/domain admission and change an earlier replay.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. The terminal
`AnalysisResultSummary.validation_status` is provider-authored lifecycle
validation state and must not be reused for the domain inference claim.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `corpus_background_v1` analysis-run output profile to
`analysis_engine`. The executor:

- requires every `CorpusBackgroundDocument` to carry explicit `AvailableTime`;
- parses request and execution cutoffs as `KnowledgeCutoff` values and compares
  their instants rather than RFC 3339 spelling;
- excludes rows with `AvailableTime > knowledge_cutoff` before duplicate
  identity and domain admission, so future evidence cannot perturb a historical
  census;
- preserves duplicate refusal among evidence actually available at the cutoff;
- preserves the raw `MAX_EVIDENCE_UNITS` operational admission bound and
  derives `document_count` from cutoff-admitted identities;
- invokes `refuse_corpus_background_as_unique_content` and
  `refuse_corpus_background_as_stopword_deletion` without reimplementing
  the background/content vocabulary;
- emits a canonical SHA-256-digested `tepp.corpus_background.v1` artifact
  with unique-content/corpus-background counts, matching refusal counts,
  and inference status
  `corpus_background_is_not_unique_content_not_stopword_deletion`;
- emits terminal `validation_status = "validated"` separately from that domain
  inference status;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Keep the branch-local constructor without availability provenance — rejected
   because a historical cutoff cannot be enforced from document identity and
   kind alone.
2. Compare RFC 3339 strings — rejected because equal instants can have distinct
   valid textual representations.
3. Reject every future-unavailable row — rejected because mere future evidence
   existence must not change an earlier scientific replay; it is excluded from
   the historical census instead.
4. Duplicate modality-source refusals (#421) — rejected because that profile
   binds `ModalityKind` and does not bind `corpus_background`.
5. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1` never
   appears.

## Evidence and repair lineage

- RED `dabe8a60cf5603b0be6ee7860d26bea0d1cb7e49` requires equivalent RFC 3339
  cutoff spellings to bind to the same instant and separates terminal
  validation status from the domain inference claim.
- Repair `34336a883c70634cb1641cad38bc8873fab10bc9` performs typed cutoff comparison
  and emits `validation_status = "validated"`.
- RED `c9ca5d4a0b217b9ae4c28a065e7c4c8be00f7a35` requires explicit availability
  provenance and proves that a future-unavailable row reusing a visible
  identity cannot change the historical replay.
- Repair `cf7da47260f3d731a15ae09be8d0f343db8a529d` adds `AvailableTime`, applies the
  cutoff before duplicate/domain admission, preserves the raw population bound,
  and derives artifact counts from admitted identities.
- `ebb867f680e1aa993cfe86c4af54c6623e357c7f` migrates the existing execution
  fixtures to explicit availability provenance.
- `b30da70030b85b335465c2cccd558436607109fd` promotes `corpus_split` from a
  test-only dependency to the runtime dependency required by the production
  cutoff predicate.

These commits are branch evidence only. `Proposed` remains appropriate until
this profile is consolidated into the surviving Analysis Run landing vehicle,
its exact surviving head passes protected checks and review, and protected main
contains the implementation.

## Consequences

Operators can eventually request leakage-safe corpus-background refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU parity,
modality-source, prompt-source, style-source, copy-identity, method-effect
estimation, or topic birth/split/merge. Snapshot/profile/cutoff mismatch, empty
or single-kind admitted corpora, duplicate admitted document identities, and
raw population overflow fail closed.

The constructor change intentionally has no compatibility path that invents an
availability timestamp. Callers must provide provenance explicitly.

## Verification

The PR includes Rust unit and integration tests for mixed unique/background
corpora, historical replay under future-unavailable duplicate identities,
equivalent RFC 3339 cutoff spellings, terminal validation-status separation,
empty/unique-only/background-only/duplicate refusal, snapshot/profile/cutoff
mismatch, population bounds, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `corpus_background_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps explicit
availability provenance, instant-based cutoff semantics, historical replay
invariance, corpus-background wording distinct from modality-source refusals,
and `identity_recovery_rate` outside inspect payloads.
