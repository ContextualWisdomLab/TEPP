# ADR 0061 — Non-lexical modality refusals as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (non-lexical modality is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0060 (prompt-source), ADR 0059 (style-source), ADR 0058 (copy-identity), or ADR 0057 (simulation method-effect census).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat non-lexical modality as unique
latent content or as stopword deletion via
`modality_source::refuse_modality_as_unique_content` and
`refuse_modality_as_stopword_deletion`. Operators still cannot request that
refusal census as a digest-bound analysis-run output. Prompt-source
refusals (#419 / ADR 0060) bind `PromptKind` and do not replace
`modality_source`.

The original branch shape was not historically safe enough to support that
claim. `ModalitySourceDocument` carried neither immutable snapshot provenance
nor `AvailableTime`, raw census size was not bounded before identity-set
allocation, request and executor cutoffs were compared as RFC 3339 text rather
than instants, and terminal `validation_status` reused the domain inference
label. Those are application-boundary defects rather than changes to the
`modality_source` domain vocabulary.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads.

GPU kernels, MCMC, and topic birth/split/merge remain later GAP-004 work
and are not this slice.

## Decision

Add the `modality_source_v1` analysis-run output profile to
`analysis_engine`. The proposed executor:

- accepts `ModalitySourceDocument` rows with explicit immutable `snapshot_id`,
  `AvailableTime`, and closed `ModalityKind` values;
- compares request and executor knowledge cutoffs by parsed instant, so
  equivalent RFC 3339 spellings bind to the same historical instant;
- rejects cross-snapshot evidence before aggregation;
- excludes same-snapshot evidence with `AvailableTime > knowledge_cutoff`
  before duplicate-identity and domain admission, so post-cutoff evidence
  cannot perturb a historical replay;
- continues to reject duplicate identities among evidence that is actually
  visible at the cutoff;
- rejects raw censuses above `MAX_EVIDENCE_UNITS` before constructing the
  identity set;
- invokes `refuse_modality_as_unique_content` and
  `refuse_modality_as_stopword_deletion` without reimplementing the
  modality/content vocabulary;
- emits a canonical SHA-256-digested `tepp.modality_source.v1` artifact with
  admitted unique-content/non-lexical-modality counts, matching refusal
  counts, and inference status
  `non_lexical_modality_is_not_unique_content_not_stopword_deletion`;
- reports terminal provider validation state as `validated`, separate from the
  domain inference label;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU backends, or
  emit topic birth/split/merge events.

The artifact `document_count` is the cutoff-admitted identity count, not raw
input length. The raw admission bound remains operational and is checked before
historical censoring.

## Alternatives considered

1. Duplicate prompt-source refusals (#419) — rejected because that profile
   binds `PromptKind` PromptBoilerplate/UniqueContent and does not bind
   `modality_source`.
2. Treat post-cutoff evidence as a hard failure — rejected for same-snapshot
   evidence because historical replay must be invariant to later-arriving rows;
   cutoff censoring is the established Analysis Run behavior. Cross-snapshot
   evidence remains a hard admission failure because it violates snapshot
   provenance rather than time eligibility.
3. Compare RFC 3339 cutoff strings verbatim — rejected because different legal
   offsets can denote the same instant.
4. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1` never
   appears.
5. Bind the existing modality-source refusals to ADR 0022's analysis-run
   profile — selected, subject to protected-main landing and current-head
   evidence.

## Consequences

A historical replay is invariant to same-snapshot rows that became available
after the requested cutoff. Snapshot provenance cannot be silently reassigned,
and visible duplicate identities still fail closed. Memory growth is bounded by
raw admission before `BTreeSet` allocation. Provider validation status no
longer masquerades as a domain inference claim.

The artifact does not claim MCMC, GPU parity, prompt-source, style-source,
copy-identity, method-effect estimation, or topic birth/split/merge. Because
this decision is implemented only on an unmerged Draft branch, it remains
`Proposed`; protected main is the authority for acceptance.

## Verification

The PR includes Rust unit and integration tests for mixed unique/modality
corpora, equivalent cutoff spellings, cutoff-equality admission, post-cutoff
historical replay invariance, cross-snapshot refusal, raw census bounds,
empty/unique-only/modality-only/visible-duplicate refusal,
snapshot/profile/cutoff mismatch, terminal validation-state separation, and
artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

No predecessor check or review receipt transfers across a head change.

## Rollback and supersession

Rollback removes the `modality_source_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps non-lexical
modality distinct from prompt-source refusals and from
`identity_recovery_rate` inspect metrics, and that preserves explicit snapshot
and availability provenance for historical replay.
