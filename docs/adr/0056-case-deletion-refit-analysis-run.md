# ADR 0056 — Exhaustive case-deletion refit as an analysis-run output profile

**Decision status:** Proposed
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Last repaired:** 2026-09-14
**Supersedes:** None; complements ADR 0012 (producer-owned case-deletion influence) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already runs the same scientific fitter on the complete corpus and on every actual `D \ {i}` corpus inside `analysis_engine::fit_exhaustive_case_deletion`. That runner owns scientific refitting, but it intentionally does not own Analysis Run snapshot provenance or evidence-availability admission.

The original profile branch incorrectly described itself as cutoff-safe while passing every supplied `CaseDeletionDocument` directly into exhaustive fitting. It had no per-document snapshot or `AvailableTime`, compared request and executor cutoffs as RFC 3339 text, and placed the domain inference claim in terminal `validation_status`. It also admitted an unbounded number of exhaustive deletions even though the runner retains `n(n-1)` deleted-corpus document identities plus one full posterior and one posterior per deletion. Those are application-boundary defects, not reasons to copy the fitter.

Reweighting, a fixed posterior, or a diagonal approximation must not replace an actual deleted-data fit.

## Decision

Add the `case_deletion_refit_v1` Analysis Run output profile to `analysis_engine` as a historical-admission adapter over the existing exhaustive fitter.

The adapter:

- keeps `CaseDeletionDocument` as the scientific fitter contract and carries immutable snapshot IDs plus `AvailableTime` in aligned application-layer slices;
- rejects provenance-length mismatch and cross-snapshot evidence before scientific fitting;
- compares parsed `KnowledgeCutoff::instant()` values, so equivalent RFC 3339 spellings of one instant bind identically;
- excludes same-snapshot evidence with `AvailableTime > KnowledgeCutoff` before duplicate/scientific admission, preserving historical replay invariance;
- keeps duplicate identities among evidence actually visible at the cutoff fail-closed through the existing runner;
- bounds raw candidate evidence by `MAX_EVIDENCE_UNITS` before adapter allocation;
- bounds exhaustive materialization by at most `256 * 255 = 65,280` retained document identities, which implies at most 256 admitted documents and 257 fitter invocations including the full fit;
- rejects an oversized admitted census before calling the fitter;
- invokes `fit_exhaustive_case_deletion` without reimplementing leave-one-out fitting;
- emits a canonical SHA-256-digested `tepp.case_deletion_refit.v1` artifact with admitted document count, deletion-refit count, independent seed-domain count, full-fit seed domain, and inference status `exhaustive_actual_deletion_not_reweighting_approx`;
- reports terminal provider validation separately as `validated`;
- keeps raw posteriors with the scientific fitter rather than copying them onto the operator artifact;
- keeps the 256 KiB `from_json` cap as an untrusted-input boundary while proving the maximal valid canonical artifact is smaller than that cap;
- refuses reuse of `composed_fitted_lineage_v1`, `fitted_candidate_k_v1`, `pareto_candidate_k_v1`, and `trsl_topic_lineage_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends, or emit topic birth/split/merge.

Historical replay invariant: within the raw operational input bound, adding same-snapshot evidence that becomes available only after the requested cutoff cannot change the earlier artifact or terminal result. Cross-snapshot evidence is a provenance violation and is rejected rather than censored.

## Alternatives considered

1. Put snapshot and availability fields into reusable `CaseDeletionDocument` — rejected because those fields belong to Analysis Run historical admission, not the scientific fitter's reusable document contract.
2. Treat all supplied documents as already cutoff-admitted — rejected because the public profile would then make future evidence capable of changing earlier results.
3. Use RFC 3339 string equality — rejected because two legal spellings can identify the same instant.
4. Reuse the general `MAX_EVIDENCE_UNITS = 100_000` bound as the exhaustive-refit budget — rejected because the actual runner retains a quadratic `n(n-1)` identity population and fitter-owned posteriors; the exhaustive path needs its own materially smaller bound.
5. Stream or discard deleted-corpus identities to admit larger corpora — deferred. That is a scientific-runner representation change and requires its own evidence before changing this profile's resource limit.
6. Copy raw posteriors onto the operator artifact — rejected because the fitter owns posterior meaning and the Analysis Run artifact remains bounded.

## Consequences

Operators can request historically reproducible exhaustive actual case deletion as a digest-bound terminal result without making future evidence visible to an earlier cutoff. The profile now has an explicit resource denominator tied to the runner's quadratic retained-identity representation. Larger corpora fail before any scientific fit instead of monopolizing a worker.

The 256-document ceiling is an application-path safety contract for the current representation, not a claim that case-deletion science is intrinsically limited to 256 documents. Raising it requires changing or re-proving the retained representation and fitter/posterior resource envelope.

This branch is still unmerged. `Proposed` remains the correct ADR status until protected-main integration and release gates are satisfied.

## Repair evidence

- RED `bc877e152fc9ad239de88f7050fd95854cdefab6` demonstrates that equivalent cutoff spellings, terminal validation/domain-claim separation, and an oversized 257-document census were not enforced on the predecessor source.
- Causal source repair `eff6eafcec5d644d3414332bd6ce750c344652bd` adds explicit snapshot/availability provenance, instant-based cutoff binding, historical censoring before scientific admission, the quadratic retained-identity resource budget, terminal `validated`, and bounded canonical serialization.
- Runtime dependency repair `6d0fb9081334e30dbe062aa11bc594eed801f3e7` promotes canonical `corpus_split::cutoff_eligible` from dev-only to production dependency.
- Contract migration `4a85e41519bf806334d2d4f41cde9938d9cd8d6c` proves historical replay invariance, cross-snapshot/misaligned-provenance refusal, visible-duplicate refusal, equivalent cutoff instants, and zero fitter calls for a 257-document census.

## Verification

Required exact-head evidence includes at least:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p analysis_engine --no-deps
python3 scripts/validate_documentation.py
python3 scripts/check_docstrings.py
```

Repository merge/release acceptance additionally requires current-head owned-production line and branch coverage, the organization security/CodeQL/documentation workflows, resolved valid review findings, and qualifying independent review. Predecessor receipts do not transfer after a head change.

## Rollback and supersession

Rollback removes the `case_deletion_refit_v1` profile without changing the protected-main scientific fitter. No persisted schema migration is introduced. Supersede only with an ADR that preserves actual deleted-data fits, historical cutoff/provenance semantics, explicit resource admission, and the distinction from reweighting, fixed posteriors, and Bayesian sampling.
