# ADR 0059 — House-voice style refusals as an analysis-run output profile

**Decision status:** Proposed
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
refusal census as a digest-bound analysis-run output. Copy-identity refusals
(#416 / ADR 0058) bind `CopyKind` and do not replace `style_source`.

The original branch did not justify a cutoff-safe claim: `StyleSourceDocument`
carried neither immutable snapshot provenance nor `AvailableTime`; raw census
size was unbounded before identity allocation; cutoff equality used RFC 3339
text instead of temporal instant; and terminal provider validation state reused
the domain inference label. These are Analysis Run application-boundary defects,
not changes to `style_source` domain truth.

`identity_recovery_rate` stays library-side. This slice does not put a
`scientific_acceptance` metric on inspect payloads. GPU kernels, MCMC, and topic
birth/split/merge remain later GAP-004 work.

## Decision

Add the proposed `style_source_v1` analysis-run output profile to
`analysis_engine`. The executor:

- accepts `StyleSourceDocument` rows with explicit immutable `snapshot_id`,
  `AvailableTime`, and closed `StyleKind` values;
- compares request and executor knowledge cutoffs by parsed instant;
- rejects cross-snapshot evidence before aggregation;
- excludes same-snapshot evidence with `AvailableTime > knowledge_cutoff`
  before duplicate-identity and domain admission;
- continues to reject duplicate identities among evidence visible at the
  cutoff;
- rejects raw censuses above `MAX_EVIDENCE_UNITS` before constructing the
  identity set;
- invokes `refuse_style_as_unique_content` and
  `refuse_style_as_stopword_deletion` without reimplementing style/content
  vocabulary;
- emits a SHA-256-digested `tepp.style_source.v1` artifact with admitted
  unique-content/style-residue counts, matching refusal counts, and inference
  status `style_residue_is_not_unique_content_not_stopword_deletion`;
- reports terminal provider validation state separately as `validated`;
- does not emit `identity_recovery_rate`, invent MCMC, select GPU backends, or
  emit topic birth/split/merge events.

The artifact `document_count` is the cutoff-admitted identity count, not raw
input length. The raw admission bound is checked before historical censoring.

## Alternatives considered

1. Duplicate copy-identity refusals (#416) — rejected because that profile binds
   `CopyKind` TemplateCopy/SourceDocument and does not bind `style_source`.
2. Treat same-snapshot post-cutoff evidence as a hard failure — rejected because
   historical replay must remain invariant to later-arriving evidence. Such rows
   are censored before identity/domain admission; cross-snapshot evidence remains
   a hard provenance failure.
3. Compare RFC 3339 cutoff strings verbatim — rejected because different legal
   offsets can denote the same instant.
4. Put `identity_recovery_rate` on the operator artifact — rejected because
   inspect payloads stay metric-free and `tepp.scientific_acceptance.v1` never
   appears.
5. Bind existing style-source refusals to ADR 0022's analysis-run profile —
   selected, subject to protected-main landing and current-head evidence.

## Consequences

A historical replay is invariant to same-snapshot rows that became available
after the requested cutoff. Snapshot provenance cannot be silently reassigned,
visible duplicate identities remain fail closed, and raw admission bounds memory
before identity-set allocation. Provider validation state no longer masquerades
as a domain inference claim.

The artifact does not claim MCMC, GPU parity, copy-identity, method-effect
estimation, or topic birth/split/merge. Because the implementation exists only
on an unmerged Draft branch, this decision remains `Proposed`.

## Verification

The PR includes Rust unit and integration tests for mixed unique/style corpora,
equivalent cutoff spellings, cutoff-equality admission, post-cutoff historical
replay invariance, cross-snapshot refusal, raw census bounds,
empty/unique-only/style-only/visible-duplicate refusal,
snapshot/profile/cutoff mismatch, provider validation-state separation, both
refusal-count invariants, count overflow, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

No predecessor check or review receipt transfers across a head change.

## Rollback and supersession

Rollback removes the `style_source_v1` profile. No persisted schema migration
is introduced. Supersede only with an ADR that keeps house-voice style residue
distinct from copy-identity refusals and `identity_recovery_rate` inspect
metrics, while preserving explicit snapshot and availability provenance for
historical replay.
