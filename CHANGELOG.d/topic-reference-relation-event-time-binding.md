# Scientific relation event-time binding

- `ReferenceTopicInput` now requires each numerically admitted observed transition edge's source and target intervals to contain the corresponding modeled `EventTime` coordinates.
- Contradictory observed relation timing fails closed before the TRSL-TM relation penalty, generalized-Gauss-Newton uncertainty, or lineage output can consume that edge; inferred edges remain non-authoritative under #673.
- This is fit-admission consistency, not source/event-time authentication. Evidence provenance and owner-authenticated event-time binding remain tracked by #658 and #671.
