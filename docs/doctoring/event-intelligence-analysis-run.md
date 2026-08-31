# TDT/CHRONOS analysis-run bind — GAP-007 operator slice

**Review date:** 2026-08-31
**Active slice:** `analysis_engine` output profile `tdt_chronos_workflow_v1`
for issue #170 / GAP-007

Protected main already composes admitted TDT detection artifacts and CHRONOS
schema/forecast hypotheses in `event_core` (`compose_event_intelligence`).
Operators still could not request that workflow as a cutoff-safe analysis run
with a digest-bound terminal result.

## Bounded closure

`execute_event_intelligence_run` binds the existing composition to one
analysis-run profile. It:

- excludes mentions unavailable at the request knowledge cutoff;
- drops TDT links that cite an excluded mention;
- keeps first-story and track streams aligned to remaining mentions;
- records envelope layer `tdt_detection` and hypothesis layer
  `chronos_prediction`;
- records inference status `composed_workflow_not_instance_or_transition`;
- invokes `refuse_composition_as_instance` and
  `refuse_composition_as_transition`;
- emits canonical `tepp.tdt_chronos_workflow.v1` JSON and its SHA-256 digest
  on the succeeded terminal result.

This slice does not extract mentions, persist rows, promote an event instance,
or create a state transition. HTTP status/lifecycle and Compose persistence
remain separate live PRs.

## Evidence boundary

The current implementation is active-PR evidence only. Exact-head checks,
independent review, and protected merge are required before the capability is
promoted to implemented-main.
