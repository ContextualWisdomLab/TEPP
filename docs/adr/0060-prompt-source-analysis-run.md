# ADR 0060 — Prompt-boilerplate refusals as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (prompt boilerplate is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0059 (style-source refusals), ADR 0058 (copy-identity), ADR 0057 (simulation method-effect census), or ADR 0052–0056.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat prompt boilerplate as unique latent
content or as stopword deletion via `prompt_source::refuse_prompt_as_unique_content`
and `refuse_prompt_as_stopword_deletion`. Operators still cannot request that
refusal census as a digest-bound analysis-run output. Style-source refusals
(#418 / ADR 0059) bind `StyleKind` and do not replace `prompt_source`.

The original branch shape was not historically safe enough to support its
cutoff-safe claim. `PromptSourceDocument` carried neither immutable snapshot
provenance nor `AvailableTime`; raw census size was not bounded before identity
allocation; request and executor cutoffs were compared as RFC 3339 text rather
than instants; and terminal `validation_status` reused the domain inference
label. These are Analysis Run application-boundary defects, not changes to the
`prompt_source` domain vocabulary.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. GPU kernels, MCMC, and topic
birth/split/merge remain later GAP-004 work.

## Decision

Add the proposed `prompt_source_v1` analysis-run output profile to
`analysis_engine`. The executor:

- accepts `PromptSourceDocument` rows with explicit immutable `snapshot_id`,
  `AvailableTime`, and closed `PromptKind` values;
- compares request and executor knowledge cutoffs by parsed instant, so
  equivalent RFC 3339 spellings bind to the same historical instant;
- rejects cross-snapshot evidence before aggregation;
- excludes same-snapshot evidence with `AvailableTime > knowledge_cutoff`
  before duplicate-identity and domain admission;
- continues to reject duplicate identities among evidence visible at the
  cutoff;
- rejects raw censuses above `MAX_EVIDENCE_UNITS` before constructing the
  identity set;
- invokes `refuse_prompt_as_unique_content` and
  `refuse_prompt_as_stopword_deletion` without reimplementing prompt/content
  vocabulary;
- emits a SHA-256-digested `tepp.prompt_source.v1` artifact with admitted
  unique-content/prompt-boilerplate counts, matching refusal counts, and
  inference status `prompt_boilerplate_is_not_unique_content_not_stopword_deletion`;
- reports provider validation state separately as `validated`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU backends, or
  emit topic birth/split/merge events.

The artifact `document_count` is the cutoff-admitted identity count rather than
raw input length. The raw admission bound remains operational and is checked
before historical censoring.

## Alternatives considered

1. Duplicate style-source refusals (#418) — rejected because that profile binds
   `StyleKind` StyleResidue/UniqueContent and does not bind `prompt_source`.
2. Treat same-snapshot post-cutoff evidence as a hard failure — rejected because
   historical replay must remain invariant to later-arriving evidence. Such rows
   are censored before identity/domain admission; cross-snapshot evidence remains
   a hard provenance failure.
3. Compare RFC 3339 cutoff strings verbatim — rejected because different legal
   offsets can denote the same instant.
4. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1` never
   appears.
5. Bind the existing prompt-source refusals to ADR 0022's analysis-run profile —
   selected, subject to protected-main landing and current-head evidence.

## Consequences

A historical replay is invariant to same-snapshot rows that became available
after the requested cutoff. Snapshot provenance cannot be silently reassigned,
and visible duplicate identities still fail closed. Memory growth is bounded by
raw admission before `BTreeSet` allocation. Provider validation state no longer
masquerades as a domain inference claim.

The artifact does not claim MCMC, GPU parity, style-source, copy-identity,
method-effect estimation, or topic birth/split/merge. Because this decision is
implemented only on an unmerged Draft branch, it remains `Proposed`.

## Verification

The PR includes Rust unit and integration tests for mixed unique/prompt corpora,
equivalent cutoff spellings, cutoff-equality admission, post-cutoff historical
replay invariance, cross-snapshot refusal, raw census bounds,
empty/unique-only/prompt-only/visible-duplicate refusal,
snapshot/profile/cutoff mismatch, provider validation-state separation, and
artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

No predecessor check or review receipt transfers across a head change.

## Rollback and supersession

Rollback removes the `prompt_source_v1` profile. No persisted schema migration
is introduced. Supersede only with an ADR that keeps prompt boilerplate distinct
from style-source refusals and from `identity_recovery_rate` inspect metrics,
and preserves explicit snapshot and availability provenance for historical
replay.
