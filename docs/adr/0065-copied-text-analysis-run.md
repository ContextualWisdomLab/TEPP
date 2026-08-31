# ADR 0065 — Copied-text residue refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (copied-text residue is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0064 (citation-edge provenance-is-not-transition), ADR 0063 (lineage-criterion fitting), ADR 0062 (corpus-background), ADR 0061 (modality-source), ADR 0060 (prompt-source), ADR 0059 (style-source), ADR 0058 (copy-identity / template-copy), or ADR 0057 (simulation method-effect census).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat copied-text residue as unique
latent content or as stopword deletion via
`copied_text::refuse_copied_text_as_unique_content` and
`refuse_copied_text_as_stopword_deletion`. Operators still cannot
request that refusal census as a digest-bound analysis-run output.
Copy-identity refusals (#416 / ADR 0058) bind `CopyKind` template-copy
identity and do not replace `copied_text`. Citation-edge refusals
(#426 / ADR 0064) bind provenance-is-not-transition and do not replace
copied-text residue.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `copied_text_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `CopiedTextDocument` rows with closed
  `CopiedKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_copied_text_as_unique_content` and
  `refuse_copied_text_as_stopword_deletion` without reimplementing the
  copied/unique vocabulary;
- emits a canonical SHA-256-digested `tepp.copied_text.v1` artifact with
  unique-content/copied-text counts, matching refusal counts, and
  inference status `copied_text_is_not_unique_content_not_stopword_deletion`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate copy-identity refusals (#416) — rejected because that
   profile binds `CopyKind` template-copy identity and does not bind
   `copied_text`.
2. Duplicate citation-edge refusals (#426) — rejected because that
   profile binds provenance-is-not-transition, not copied-text residue.
3. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
4. Bind the existing copied-text refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe copied-text residue refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, template-copy identity, citation-edge, corpus-background,
modality-source, prompt-source, style-source, method-effect estimation,
or topic birth/split/merge. Snapshot/profile/cutoff mismatch, empty or
single-kind corpora, and duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed unique/copied
corpora, empty/unique-only/copied-only/duplicate refusal, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `copied_text_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
copied-text residue distinct from template-copy identity and from
`identity_recovery_rate` inspect metrics.
