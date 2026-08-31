# ADR 0060 — Prompt-boilerplate refusals as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (prompt boilerplate is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0059 (style-source refusals), ADR 0058 (copy-identity), ADR 0057 (simulation method-effect census), or ADR 0052–0056.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat prompt boilerplate as unique
latent content or as stopword deletion via
`prompt_source::refuse_prompt_as_unique_content` and
`refuse_prompt_as_stopword_deletion`. Operators still cannot request that
refusal census as a digest-bound analysis-run output. Style-source
refusals (#418 / ADR 0059) bind `StyleKind` and do not replace
`prompt_source`.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `prompt_source_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-validated `PromptSourceDocument` rows with closed
  `PromptKind` values;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `refuse_prompt_as_unique_content` and
  `refuse_prompt_as_stopword_deletion` without reimplementing the
  prompt/content vocabulary;
- emits a canonical SHA-256-digested `tepp.prompt_source.v1` artifact with
  unique-content/prompt-boilerplate counts, matching refusal counts, and
  inference status `prompt_boilerplate_is_not_unique_content_not_stopword_deletion`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU
  backends, or emit topic birth/split/merge events.

## Alternatives considered

1. Duplicate style-source refusals (#418) — rejected because that
   profile binds `StyleKind` StyleResidue/UniqueContent and does not
   bind `prompt_source`.
2. Put `identity_recovery_rate` on the operator artifact — rejected
   because inspect payloads stay metric-free and
   `tepp.scientific_acceptance.v1` never appears.
3. Bind the existing prompt-source refusals to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe prompt-boilerplate refusals as a
digest-bound terminal result. The artifact does not claim MCMC, GPU
parity, style-source, copy-identity, method-effect estimation, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, empty or
single-kind corpora, and duplicate document identities fail closed.

## Verification

The PR includes Rust unit and integration tests for mixed unique/prompt
corpora, empty/unique-only/prompt-only/duplicate refusal, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `prompt_source_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps
prompt boilerplate distinct from style-source refusals and from
`identity_recovery_rate` inspect metrics.
