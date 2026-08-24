# ADR 0002 — Six-clock temporal semantics and leakage prevention

**Decision status:** Accepted
**Implementation maturity:** partial — typed clocks, interval algebra, event/assertion/cutoff/availability identity, complete document clocks, revision ordering, and provenance-vs-transition gates are implemented-main; downstream graph/split enforcement remains accepted-target.
**Date:** 2026-08-05
**Supersedes:** None. ADR 0013 owns persistence/split representation; ADR 0016 owns event-intelligence reasoning above these temporal primitives.

## Context

A single document date cannot represent when an event happened, when a claim was asserted, when a file was authored, when the system observed it, when an analyst could actually use it, and which evidence was legally/scientifically available to a historical model. Collapsing these clocks creates retrospective leakage and invalid transition reasoning.

TEPP also needs uncertain, open, overlapping, and irregular intervals. A total order or one timestamp per entity would fabricate precision that the source does not contain.

## Decision

TEPP stores event/valid time, assertion time, document time, system time, availability time, and model knowledge cutoff as different typed values. Uncertain and open intervals retain boundary semantics, source precision, and provenance. Interval topology follows Allen's thirteen elementary relations (Allen, 1983). Event and time-marking vocabulary is aligned with ISO-TimeML (International Organization for Standardization, 2012). Outward instant/interval relation names may map to OWL-Time (Hobbs & Pan, 2017). These sources define the temporal algebra and annotation contract; they do not by themselves claim a complete TimeML corpus annotator or unrestricted global satisfiability.

Historical analysis includes evidence only when its governed availability interval is fully eligible for the analysis cutoff. The practical invariant is `available_time <= knowledge_cutoff`; uncertain availability that can extend beyond the cutoff fails closed unless a versioned policy explicitly defines a conservative admissible interpretation.

Forward transition, state-change, and input→process→outcome edges require a valid event-time partial order. Citation, revision, translation, support, contradiction, summary, and retrospective-reporting edges may point to the past but cannot become reverse transitions. Derived interval relations retain source evidence and the reasoner claim boundary.

Superseded PRs #5 and #6 retain earlier clock and Allen/path-consistency work on a discarded lineage. They are historical audit evidence only. The protected-main implementation is the merged PR #8 temporal foundation plus the merged PR #9 replay of Task 4 interval algebra; those pull requests are lineage, not a living unmerged product. Path consistency is not documented as unrestricted global satisfiability.

## Non-goals

- do not force all events into a total order;
- do not infer causality from temporal precedence alone;
- do not use document/event time as a substitute for evidence availability;
- do not coerce unknown or interval-valued time to a false exact instant.

## Alternatives considered

1. **Single `document_date`/`event_date` field** — rejected because it collapses evidence availability, historical replay, and event chronology.
2. **Use ingestion/system time as the only operational clock** — rejected because late-imported historical evidence would appear to happen late while still describing earlier events.
3. **Six typed clocks plus interval/partial-order semantics** — accepted.

## Consequences

A later document about an earlier event cannot leak into an earlier model. Irregular observations, delayed reporting, revisions, retrospective evidence, overlapping episodes, and uncertain dates remain representable. Model evaluation uses rolling-origin and relation-aware splits. Event/psychometric models can distinguish real change from reporting delay or document revision.

## Failure and recovery

Unknown, malformed, contradictory, or policy-ineligible temporal evidence is not silently repaired. The operation returns unresolved/contradictory/ineligible state, preserves original evidence, and may be re-evaluated only with corrected metadata, later evidence, or a superseding temporal policy. Recovery never changes a historical cutoff merely to make a run succeed.

## Security, privacy, and governance impact

Availability/system-time metadata can reveal business workflow timing and is protected under the same purpose/tenant controls as source evidence. Time fields cannot be supplied by an untrusted LLM as authoritative without validated provenance. Scientific/release claims follow ADR 0014.

## Compatibility and migration

Wire/database/API contracts retain clock type, interval boundaries, precision, and provenance explicitly. A change in the meaning of any clock or historical eligibility rule is a breaking scientific decision requiring a superseding ADR and PRD update. Persistence details are governed by ADR 0013.

## Verification

Property and integration tests cover nominal clock separation, interval algebra, uncertain/open boundaries, timezone/DST normalization, contradiction detection, transition cycles, historical snapshots, delayed availability, cutoff-crossing uncertainty, rolling-origin/relation-aware partitioning, and synthetic truth with known event and document processes.

Protected-main `temporal_core` must continue to pass exact-head repository, line/branch coverage, rustdoc, dependency/security, and current-review gates. Historical checks from superseded PRs #5 and #6 do not transfer as current-head merge evidence even when later PRs preserved their TDD implementation lineage.

## Rollback and supersession

Rollback selects the previous temporal contract/version and recomputes dependent snapshots/artifacts; it never reinterprets already-published evidence silently. Supersede only with a contract that explicitly migrates all six clock meanings and preserves or deliberately changes leakage semantics with new validation evidence.

## References

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Hobbs, J. R., & Pan, F. (2017). *Time ontology in OWL* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

International Organization for Standardization. (2012). *Language resource management—Semantic annotation framework (SemAF)—Part 1: Time and events (SemAF-Time, ISO-TimeML)* (ISO 24617-1:2012).
