# ADR 0002: Six-Clock Temporal Semantics and Leakage Prevention

**Status:** Accepted  
**Date:** 2026-08-05

## Decision

TEPP stores event/valid time, assertion time, document time, system time, availability time, and model knowledge cutoff as different typed values. Uncertain and open intervals retain precision and provenance. Historical analysis includes evidence only when `available_time <= knowledge_cutoff`.

Forward transition edges require a valid event-time partial order. Citation, revision, translation, support, contradiction, summary, and retrospective-reporting edges may point to the past but cannot become reverse transitions. Derived interval relations retain their source evidence.

## Consequences

A document written later about an earlier event cannot leak into an earlier model. Irregular observations, delayed reporting, revisions, and retrospective evidence remain representable. Model evaluation uses rolling-origin and relation-aware splits.

## Verification

Property and integration tests cover interval algebra, uncertain boundaries, timezone/DST normalization, contradiction detection, transition cycles, historical snapshots, delayed availability, and synthetic truth with known event and document processes.
