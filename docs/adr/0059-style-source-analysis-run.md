# ADR 0059 — House-voice style refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (style residue is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0058 (copy-identity refusals), ADR 0057 (simulation method-effect census), ADR 0056 (case-deletion), ADR 0055 (composed fitted-K+lineage), ADR 0054 (export GET), ADR 0053 (Pareto candidate-`K`), or ADR 0052 (joint posterior Laplace draws).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat house-voice style residue as unique
latent content or as stopword deletion via
`style_source::refuse_style_as_unique_content` and
`refuse_style_as_stopword_deletion`. Operators still cannot request that
refusal census as a digest-bound analysis-run output. Copy-identity
refusals (#416 / ADR 0058) bind `CopyKind` and do not replace
`style_source`.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `style_source_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `StyleSourceDocument` rows with closed
  `StyleKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_style_as_unique_content` and
  `refuse_style_as_stopword_deletion` without reimplementing the
  style/content vocabulary;
- emits a canonical SHA-256-digested `tepp.style_source.v1` artifact with
  unique-content/style-residue counts, matching refusal counts, and
  inference status `style_residue_is_not_unique_content_not_stopword_deletion`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate copy-identity refusals (#416) — rejected because that
   profile binds `CopyKind` TemplateCopy/SourceDocument and does not
   bind `style_source`.
2. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
3. Bind the existing style-source refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe house-voice style refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, copy-identity, method-effect estimation, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, empty or
single-kind corpora, and duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed unique/style
corpora, empty/unique-only/style-only/duplicate refusal, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `style_source_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
house-voice style residue distinct from copy-identity refusals and from
`identity_recovery_rate` inspect metrics.
