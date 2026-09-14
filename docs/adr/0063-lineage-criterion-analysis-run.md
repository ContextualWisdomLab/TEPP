# ADR 0063 — Independent TDT link-criterion fitting as an analysis-run output profile

**Decision status:** Proposed  
**Implementation maturity:** active-PR — composed on this branch; not implemented-main  
**Date:** 2026-08-31  
**Supersedes:** None; complements ADR 0023 (lineage-criterion anchor) and ADR 0022 (cutoff-safe analysis-run execution).  
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.  
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already fits independently observed TDT link-criterion Jeffreys posteriors through `analysis_engine::fit_lineage_criterion_posteriors`. That fitter owns the criterion arithmetic. Event-time draws are producer evidence; record order and document timestamps are not substitutes for event time.

The original branch composition exposed four boundary defects before it could truthfully call itself cutoff-safe: pair observations carried no availability provenance, equivalent RFC 3339 spellings were compared as text rather than instants, malformed event-time draws reached the fitter, and pair/draw cardinality was not bounded before posterior materialization. It also copied the domain inference claim into `AnalysisResultSummary.validation_status`, which is provider-authored validation state.

Method-effect labels, template-copy refusals, house-voice refusals, prompt-boilerplate refusals, non-lexical modality refusals, exhaustive case-deletion, composed fitted-lineage, Pareto candidate-`K`, and topic activity remain different profiles. Full Bayesian sampling, GPU execution, and topic birth/split/merge are outside this slice.

## Constraints

- `fit_lineage_criterion_posteriors` remains the scientific-arithmetic owner; this adapter must not duplicate the Jeffreys estimator.
- Event time and evidence availability are different clocks. An event-time draw never proves that its observation was available at the run cutoff.
- Historical replay must ignore same-snapshot observations unavailable until after the requested cutoff before duplicate, event-time, or scientific admission can affect the result. The engine-wide raw-input cardinality bound remains an operational admission limit.
- Cross-snapshot provenance is a hard error rather than historical censoring.
- The terminal validation status describes provider validation. The domain claim `independent_tdt_criterion_not_date_from_record_order` remains on the artifact.
- Resource admission must happen before expensive posterior materialization.

## Decision

Add the `lineage_criterion_v1` analysis-run output profile to `analysis_engine` as a thin application/Validation adapter around the protected-main fitter.

The profile binds every offered `LineageCriterionObservation` to explicit immutable snapshot identity and pair-level `AvailableTime`. The availability value means when the complete pair observation became usable as evidence; producers must not report it earlier than either endpoint's availability. The adapter checks provenance-slice alignment, rejects cross-snapshot rows, and uses `corpus_split::cutoff_eligible` before validating or fitting same-snapshot observations. Future-unavailable rows therefore cannot create duplicate-pair failures, malformed-event-time failures, pair counts, or scientific results for an earlier cutoff.

Request cutoff binding uses parsed `KnowledgeCutoff::instant()` equality, so equivalent RFC 3339 offsets denote the same cutoff. Admitted predecessor and successor event-time draws must parse as `EventTime`; availability is never inferred from those draws.

Raw pair population is bounded by `analysis_engine::MAX_EVIDENCE_UNITS`. The admitted pair-count × draw-count materialization budget is bounded to 1,000,000 draw values, matching the workspace's existing bounded posterior-draw scale rather than allowing unbounded CPU/memory work. Fewer than two requested draws fail before fitting.

The emitted `tepp.lineage_criterion.v1` artifact contains only run/snapshot/cutoff identity, admitted pair count, draw count, and the fixed domain inference claim. Raw pair identities and posterior values remain with the scientific fitter. `AnalysisResultSummary.validation_status` is exactly `validated` on success.

The 256 KiB `from_json` cap remains an untrusted-input admission boundary. Valid artifact identifiers and counts are bounded tightly enough that canonical output cannot approach that limit; a maximal valid artifact test exercises worst-case JSON escaping rather than retaining an unreachable post-validation egress branch.

## Alternatives considered

1. Keep textual cutoff equality — rejected because RFC 3339 allows different representations of the same instant.
2. Treat event-time draws as evidence availability — rejected because event time and availability are distinct clocks and this would introduce temporal leakage.
3. Validate all rows before applying the cutoff — rejected because future evidence could change a historical replay through duplicate, malformed-time, or fitter refusal paths.
4. Copy or reimplement Jeffreys fitting in the adapter — rejected because `fit_lineage_criterion_posteriors` is the canonical scientific owner.
5. Leave pair/draw work unbounded and rely on allocator failure — rejected because the API accepts externally supplied cardinalities and the estimator materializes posterior draws.
6. Put the domain inference claim in terminal `validation_status` — rejected because that field is provider-authored validation state, not a scientific conclusion.

## Evidence and effects

RED `f475a312dde77aa3c316b5249f3eb2b0b036835b` exposes textual cutoff equality and malformed event-time acceptance. Repair `a33ee6807ab843b11d085a28b365fd08b7948810` binds cutoff equality by instant, validates event-time syntax, and separates terminal validation from the domain claim.

RED `8f4523cf33f75c99a7f3d827745f5580d5139b69` adds explicit provenance, historical replay, cross-snapshot, alignment, and resource-budget contracts. Repair `e6de3aad0ded19eeefeb67a7f9dc1795025d3b1b` admits same-snapshot evidence by availability before scientific validation, bounds posterior materialization, and proves the bounded artifact wire shape. `55edf20de86ac42540c294ad333d20d8cc4d045e` promotes the canonical cutoff helper to a runtime dependency. `1591d7a998659969161b5819eae1cb90844d049c` corrects the integration assertion against the optional terminal summary without changing the production contract.

These commits are active-PR evidence, not protected-main acceptance. Current-head checks and qualifying review must be regenerated after every source change.

## Verification

The branch carries unit and integration contracts for equivalent cutoff spellings, explicit snapshot/availability provenance, future-unavailable replay invariance, cross-snapshot and provenance-alignment refusal, malformed visible event times, raw/resource limits, scientific fitter refusal, digest binding, artifact tamper resistance, and terminal/domain-claim separation.

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Risks and follow-up

The application adapter currently receives pair-level availability provenance adjacent to estimator-owned observations. Upstream producers remain responsible for deriving that availability no earlier than both evidence endpoints; a future versioned producer contract may replace the parallel transport with a dedicated pair-evidence value object without changing the scientific fitter.

ADR 0063 remains Proposed until this composition is inherited by the surviving Analysis Run vehicle, exact-head quality/security gates and coverage pass, and the change reaches protected main. The shared ADR index must not mark this decision Accepted while the implementation is branch-only.

## Rollback and supersession

Rollback removes the `lineage_criterion_v1` profile without changing the protected-main scientific fitter or persisted schema. Any superseding ADR must preserve the separation among event time, availability, knowledge cutoff, provider validation, and scientific inference, and must not infer dates from record order or promote CHRONOS predictions to observed facts.
