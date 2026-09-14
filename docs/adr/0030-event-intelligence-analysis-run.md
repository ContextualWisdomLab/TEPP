# ADR 0030 — TDT/CHRONOS composition as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on the active product branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0016 event-intelligence layers and ADR 0022 analysis-run execution.
**Figma File ID:** N/A — this increment changes a Rust analysis crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already admits already-extracted TDT detection artifacts and
CHRONOS schema/forecast hypotheses into one versioned workflow
(`compose_event_intelligence` in `event_core`, ADR 0016). Operators still cannot
request that composed workflow as an analysis-run output profile. Without an
engine bind, the composition remains a library call: it is not cutoff-filtered
against the accepted run, not digest-bound as a terminal result, and not
explicitly refused as an event instance or state transition at the analysis-run
boundary.

This slice does not invent a new extractor, persist the composition, or promote
detection/prediction into a forward transition.

## Decision

Add the `tdt_chronos_workflow_v1` analysis-run output profile to
`analysis_engine`. For that profile the engine:

- requires `model_contract_version` `event_intelligence_workflow_v1` and the
  request snapshot/cutoff to match the execution arguments;
- consumes already-extracted mentions, links, first-story labels, tracks,
  schema slots, and forecasts through `EventIntelligenceRunInput`;
- excludes mentions whose `available_time` is later than the request
  `knowledge_cutoff`, drops links that cite an excluded mention, and keeps
  first-story/track streams index-aligned to remaining mentions;
- calls `compose_event_intelligence` and the explicit
  `refuse_composition_as_instance` / `refuse_composition_as_transition`
  refusals;
- emits a canonical SHA-256-digested `tepp.tdt_chronos_workflow.v1` artifact
  with bounded counts, TDT envelope layer `tdt_detection`, CHRONOS hypothesis
  layer `chronos_prediction`, and inference status
  `composed_workflow_not_instance_or_transition`;
- fails closed when no mention remains eligible, when first-story or track
  streams are not length-aligned, when snapshot/profile/cutoff diverge, or when
  the artifact claim boundary is tampered.

The engine does not extract mentions from source text, does not write
persistence rows, and does not create an event instance or state transition.

## Alternatives considered

1. Treat library composition on main as product-complete — rejected because
   operators still cannot request the workflow as an analysis run.
2. Invent a new extractor inside `analysis_engine` — rejected because ADR 0016
   already owns extraction/admission; this slice only binds admitted artifacts.
3. Persist the composition or promote it to an instance/transition — rejected
   because persistence is GAP-003B and promotion remains an independent
   authority.
4. Bind the existing composition through the analysis-run profile — accepted.

## Consequences

Consumers can request a reproducible TDT/CHRONOS workflow and receive a
digest-bound terminal result that preserves detection-versus-prediction layers.
Source text never enters the artifact. Later HTTP, persistence, and export
slices must consume this schema rather than re-compose a second authority.

## Verification

The stacked PR includes Rust unit and integration tests for canonical artifact
round-trip, tamper refusal, known-truth composition counts, cutoff exclusion of
a delayed revised mention, snapshot/profile/cutoff mismatch, stream
misalignment, empty eligibility, and receipt identity. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
```

The supporting research citations remain Allan (2002), Li et al. (2021), and
Anagnostopoulos, Batsakis, and Petrakis (2013) as recorded under ADR 0016.

## Rollback and supersession

Rollback removes the `tdt_chronos_workflow_v1` profile and the
`event_intelligence_artifact` module while preserving `event_core` composition
and the readiness/topic-lineage executors. No persisted schema migration is
introduced. Supersession requires a new ADR if the profile changes cutoff
semantics, claim-boundary copy, or promotion authority.

## References

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Anagnostopoulos, E., Batsakis, S., & Petrakis, E. G. M. (2013). CHRONOS: A reasoning engine for qualitative temporal information in OWL. *Procedia Computer Science, 22*, 70–77. https://doi.org/10.1016/j.procs.2013.09.082

Li, M., Li, S., Wang, Z., Huang, L., Cho, K., Ji, H., Han, J., & Voss, C. (2021). The future is not one-dimensional: Complex event schema induction by graph modeling for event prediction. In *Proceedings of the 2021 Conference on Empirical Methods in Natural Language Processing* (pp. 5203–5215). Association for Computational Linguistics. https://doi.org/10.18653/v1/2021.emnlp-main.422
