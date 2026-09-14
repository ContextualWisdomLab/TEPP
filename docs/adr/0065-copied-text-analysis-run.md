# ADR 0065 — Copied-text residue refusals as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0004/0012 (copied-text residue is not unique content) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0064 (citation-edge provenance-is-not-transition), ADR 0063 (lineage-criterion fitting), ADR 0062 (corpus-background), ADR 0061 (modality-source), ADR 0060 (prompt-source), ADR 0059 (style-source), ADR 0058 (copy-identity / template-copy), or ADR 0057 (simulation method-effect census).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already refuses to treat copied-text residue as unique latent content or as stopword deletion via `copied_text::refuse_copied_text_as_unique_content` and `refuse_copied_text_as_stopword_deletion`. Operators still cannot request that refusal census as a digest-bound analysis-run output.

The first branch implementation called this profile cutoff-safe while `CopiedTextDocument` carried only a document identity and `CopiedKind`. It therefore could not prove when a row became available or which immutable snapshot supplied it. A future row could participate in duplicate detection and counts for an earlier historical replay, and an unrelated snapshot could be attributed to the requested snapshot. The executor also compared RFC 3339 cutoff strings rather than temporal instants and allocated duplicate-tracking state before enforcing the shared analysis population bound.

`identity_recovery_rate` stays library-side. This profile does not put a `scientific_acceptance` metric on inspect payloads. GPU kernels, MCMC, and topic birth/split/merge remain outside this decision.

## Decision

Add the `copied_text_v1` analysis-run output profile to `analysis_engine` as an active PR implementation. The executor:

- requires every `CopiedTextDocument` to carry an immutable `snapshot_id` and explicit `AvailableTime` rather than inventing provenance in a compatibility constructor;
- binds request and executor cutoffs by parsed `KnowledgeCutoff::instant()` equality, so equivalent RFC 3339 spellings of one instant are equivalent;
- rejects cross-snapshot rows as provenance violations;
- excludes same-snapshot rows whose `AvailableTime` is later than the knowledge cutoff before duplicate or domain admission, so future evidence cannot change an earlier historical replay;
- preserves fail-closed duplicate detection among rows actually visible at the cutoff;
- applies `MAX_EVIDENCE_UNITS` to the raw document slice before allocating duplicate-tracking state, and derives `document_count` from cutoff-admitted identities;
- invokes the existing copied-text domain refusals without copying their vocabulary and accepts only the expected provider results; unexpected provider success or another present/future error fails closed;
- emits a canonical SHA-256-digested `tepp.copied_text.v1` artifact with unique-content/copied-text counts, matching refusal counts, and inference status `copied_text_is_not_unique_content_not_stopword_deletion`;
- keeps terminal provider validation state as `validated`, separate from the domain inference claim;
- retains the 256 KiB untrusted-input cap. Bounded identifiers, timestamps, and census counts make valid canonical output smaller than that cap, so no second post-validation output-size branch is needed.

## Evidence and alternatives

RED commit `1d2c67f4b41f7788329e21ee4b1ef91bd3c30039` adds contracts for equivalent cutoff instants, explicit availability/snapshot provenance, future-duplicate replay invariance, cross-snapshot refusal, provider/domain status separation, and exact-limit versus limit-plus-one population admission. The causal repair is the ordinary forward successor on this branch; predecessor checks do not transfer.

Alternatives considered:

1. Keep document provenance implicit in the run request — rejected because a run-level snapshot/cutoff cannot prove each supplied row came from that snapshot or was available at that cutoff.
2. Reject all post-cutoff rows — rejected for historical replay because adding evidence that did not yet exist must not turn an earlier valid run into a failure. Same-snapshot future rows are censored before duplicate/domain admission; cross-snapshot rows remain admission errors.
3. Compare RFC 3339 strings — rejected because multiple legal textual representations can denote one instant.
4. Duplicate copy-identity or citation-edge vocabulary — rejected because those profiles own different domain semantics. This adapter consumes the protected-main `copied_text` contract.
5. Put `identity_recovery_rate` on the operator artifact — rejected because inspect payloads remain metric-free and numerical/scientific acceptance is a separate boundary.

## Consequences

A historical copied-text census is invariant to same-snapshot evidence that becomes available after its knowledge cutoff. Visible duplicates still fail closed, cross-snapshot evidence cannot be silently relabelled, and raw input is bounded before identity allocation. Provider validation and domain inference remain separate claims.

The implementation remains branch-local and therefore this ADR stays `Proposed`. Consolidation into the surviving Analysis Run vehicle must preserve the source, tests, refusal semantics, temporal/provenance invariants, doctoring, and traceability before this branch can be treated as superseded.

## Verification

Run on the unchanged surviving head:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

Exact-head required workflows, owned production line/branch coverage, current review findings, and qualifying independent approval remain separate merge gates.

## Rollback and supersession

Rollback removes the `copied_text_v1` profile. No persisted schema migration is introduced. Supersede only after a verified successor inherits the explicit snapshot/availability provenance, instant-based cutoff binding, historical replay invariant, raw population bound, fail-closed provider contract, and copied-text/domain distinction. Simple Close without verified inheritance is not supersession.
