# ADR 0002: Six-Clock Temporal Semantics and Leakage Prevention

**Status:** Accepted  
**Date:** 2026-08-05  
**Last updated:** 2026-08-06

## Context

A single document date cannot distinguish when an event occurred, when a claim was asserted, when a document was created, when TEPP observed it, when an analyst could use it, and which evidence an historical model run was permitted to know. Conflating those clocks creates future-information leakage and makes delayed reporting, revisions, and retrospective evidence impossible to represent correctly.

The executable foundation must also preserve uncertainty without treating an unknown time as an infinitely permissive interval. Wire records need a stable lexical profile and must reconstruct through the same domain validation as direct Rust construction.

## Decision

TEPP represents the following clocks as distinct sealed nominal Rust types over one absolute nanosecond-resolution instant:

- `EventTime`;
- `AssertionTime`;
- `DocumentTime`;
- `SystemTime`;
- `AvailableTime`; and
- `KnowledgeCutoff`.

Task 3 accepts a deliberately strict RFC 3339 profile: four-digit dates, the uppercase `T` separator, explicit seconds, an optional one-to-nine-digit fractional second, and either uppercase `Z` or an exact `±HH:MM` offset. Values normalize deterministically to UTC. Leap seconds, local timestamps without offsets, shortened offsets, bracketed time-zone names, RFC 9557 suffixes, and other lossy or ambiguous forms fail closed in wire version `1`.

`TemporalInterval<T>` retains one nominal clock and represents:

- exact single instants;
- bounded intervals;
- lower-open or upper-open intervals; and
- explicitly unknown intervals.

Boundaries are included, excluded, or unbounded. Precision is explicit from nanosecond through year, with `unknown` reserved for explicitly unknown intervals. Reversed, empty, semantically inconsistent, and unknown-precision known intervals fail closed. An unknown interval does not claim that every candidate instant is contained.

Clock and interval wire records use a strict schema version `1`, reject unknown JSON fields, bind the nominal clock type, and reconstruct through timestamp and interval validation. Generated Draft 2020-12 JSON Schemas include the same strict timestamp lexical pattern used by the runtime parser.

At the platform level, historical inclusion remains governed by:

\[
\operatorname{available\_time}(d) \leq \operatorname{knowledge\_cutoff}.
\]

Forward transition edges will require a valid event-time partial order. Citation, revision, translation, support, contradiction, summary, and retrospective-reporting edges may point to the past but must not become reverse state transitions.

## Implemented scope

Task 3 implements the six nominal clocks, strict absolute-instant parsing and UTC normalization, typed intervals and uncertainty, stable redacting errors, versioned JSON wire records, and matching JSON Schema output in `temporal_core`.

## Deferred scope

The following are not claimed by Task 3 and require later independently reviewed slices:

- Allen-style interval relations, converse and composition tables;
- transitive closure, contradiction proofs, and bounded reasoner resources;
- event ontology and forward-transition graph validation;
- persistence and bitemporal database constraints;
- historical snapshot and leakage-safe corpus-split enforcement; and
- synthetic event/document process recovery.

## Consequences

Compile-time nominal types prevent accidental substitution of one clock for another. Strict lexical parsing avoids silently accepting timestamps whose interpretation depends on locale, implicit time zones, or unsupported extensions. Explicit unknown values remain distinguishable from open-ended evidence. Runtime records and published schemas now describe the same accepted timestamp language.

Later persistence, relation, split, and simulation layers must depend inward on these validated values rather than redefining time semantics independently.

## Verification

Task 3 tests cover:

- nominal parity across all six clocks;
- strict syntax and invalid calendar semantics;
- UTC, offset, nanosecond, and daylight-saving normalization cases;
- exact, bounded, open-ended, excluded-boundary, and unknown interval behavior;
- reversed, empty, unknown-precision, and certainty-invalid intervals;
- strict JSON round trips and hostile wire payloads;
- clock, version, unknown-field, malformed-boundary, and timestamp rejection;
- Draft 2020-12 schema shape and runtime/schema lexical-profile parity; and
- stable content-redacting errors.

Interval algebra, graph closure, leakage snapshots, and synthetic truth are verified only when their deferred tasks are implemented; they are not evidence for this task.
