# ADR 0016 — TDT, CHRONOS, and Event Ontology intelligence boundary

**Decision status:** Accepted  
**Implementation maturity:** active-PR — TDT topic detection (new-topic FAR/miss, cluster pair precision/recall, and topic-cluster-versus-instance refusal) lives in existing `event_core`; remaining TDT detection/tracking and CHRONOS schema/temporal layers remain accepted-target  

**Date:** 2026-08-12  
**Supersedes:** None; complements ADR 0002 temporal semantics and ADR 0003 event ontology/membership.

## Context

TEPP uses event intelligence for segmentation, linking, first-story detection, event tracking, schema instantiation, prediction, and temporal-consistency reasoning. These functions have different epistemic meanings. If they are collapsed into one opaque “event model,” a predicted event can be confused with an observed event, a topic link with a causal edge, or path consistency with proof of a complete real-world chronology.

## Decision

TEPP separates three event-intelligence layers:

1. **Event Ontology layer** owns versioned event instances, mentions, roles/arguments, subevents, evidence spans, places, products/outcomes, and provenance. Observed mentions are fallible evidence; event instances are modeled objects.
2. **TDT-style detection/tracking layer** owns story/event segmentation, link detection, new/first-story detection, topic/event detection, and longitudinal tracking. Its outputs are probabilistic measurement/detection evidence and require calibration and false-positive/false-negative evaluation.
3. **CHRONOS-style reasoning layer** has two explicit subcontracts: semantic/neural event-schema extraction/prediction and symbolic/qualitative temporal-consistency reasoning. A predicted event/schema completion remains hypothetical until supported by later evidence. Temporal reasoning validates consistency/partial order under its stated algebra and resource bounds; it does not prove unrestricted global satisfiability unless a later implementation explicitly does so.

Transition edges admitted to the state/input-process-outcome graph remain governed by ADR 0002/0003 and cannot be created merely because TDT/CHRONOS predicts or links two events. Retrospective evidence and schema predictions remain provenance/hypothesis edges until independently promoted.

## Alternatives considered

1. **Single end-to-end event graph with no evidence-state distinction** — rejected because observation, inference, prediction, and transition authority become conflated.
2. **Use topic clusters as event identity** — rejected because topical similarity is not sufficient evidence for event identity or temporal relation.
3. **Layered ontology + detection/tracking + schema prediction + symbolic consistency** — accepted.

## Consequences

- every event/relation carries an evidence/inference/prediction status and provenance;
- event tracking can be evaluated as a measurement process rather than treated as ground truth;
- temporal-consistency failures can reject a proposed schema/transition without rewriting source evidence;
- TDT/CHRONOS outputs can feed psychometric and longitudinal models only through versioned, uncertainty-bearing contracts;
- event prediction is never silently converted to historical fact.

## Failure and recovery

Low-confidence, contradictory, temporally inconsistent, out-of-budget, or unsupported event intelligence returns unresolved/abstained/hypothetical status. Recovery may re-run detection or reasoning with a later model/evidence set, but the original evidence and prior decision artifact remain immutable for audit.

## Security, privacy, and governance impact

Event links can reveal sensitive relationships even when direct identifiers are absent. Access, export, retention, and provider disclosure follow ADR 0009. Untrusted document text cannot alter ontology schema, tool authority, or temporal-reasoner policy.

## Compatibility and migration

Event-intelligence APIs distinguish observed evidence, inferred relations, predictions, and promoted transition edges. Schema/model-version changes require migration/compatibility notes. A consumer such as naruon receives the status and uncertainty fields and may not flatten predictions into facts.

## Verification

Required evidence includes segmentation accuracy, link precision/recall, first-story false-alarm/miss rates, tracking stability/calibration, event mention/relation recovery, schema-slot accuracy, prediction calibration, temporal contradiction detection, path-consistency laws/limits, lineage/provenance integrity, and realistic multilingual/time-delayed cases. Scientific claims must remain within the tested task and data regime.

## Rollback and supersession

Rollback selects the last validated event-intelligence model/reasoner version and preserves previously issued artifacts with their version/status. Supersede only with an ADR that maintains explicit separation of observed evidence, inferred event identity/relation, prediction, temporal consistency, and transition authority.
