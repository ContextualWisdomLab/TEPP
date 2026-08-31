# ADR 0062 — Corpus-background refusals as an analysis-run output profile

**Decision status:** Accepted
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

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `corpus_background_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `CorpusBackgroundDocument` rows with closed
  `CorpusBackgroundKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_corpus_background_as_unique_content` and
  `refuse_corpus_background_as_stopword_deletion` without reimplementing
  the background/content vocabulary;
- emits a canonical SHA-256-digested `tepp.corpus_background.v1` artifact
  with unique-content/corpus-background counts, matching refusal counts,
  and inference status
  `corpus_background_is_not_unique_content_not_stopword_deletion`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate modality-source refusals (#421) — rejected because that
   profile binds `ModalityKind` NonLexicalModality/UniqueContent and does
   not bind `corpus_background`.
2. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
3. Bind the existing corpus-background refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe corpus-background refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, modality-source, prompt-source, style-source, copy-identity,
method-effect estimation, or topic birth/split/merge.
Snapshot/profile/cutoff mismatch, empty or single-kind corpora, and
duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed unique/background
corpora, empty/unique-only/background-only/duplicate refusal, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `corpus_background_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
corpus-background wording distinct from modality-source refusals and from
`identity_recovery_rate` inspect metrics.
