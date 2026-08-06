# Task 3 Temporal Wire Foundations

## Purpose

This doctoring note traces the executable Task 3 temporal contract to authoritative standards and separates implemented behavior from later temporal reasoning claims. References use APA 7th style.

Task 3 is a storage-independent domain slice. It defines nominal clocks, absolute instants, uncertain intervals, JSON interchange, and JSON Schema output. It does not implement interval algebra, graph closure, event ontology, bitemporal persistence, or historical leakage enforcement.

## Implemented temporal contract

### Six nominal clocks

TEPP distinguishes:

- event or valid time;
- assertion time;
- document time;
- system observation time;
- evidence availability time; and
- model knowledge cutoff.

The six Rust types share an absolute instant representation but are not assignment-compatible. The distinction is a TEPP domain decision used to prevent semantic substitution; ISO 8601 and RFC 3339 define timestamp representations, not these six business meanings.

### Wire-version 1 timestamp profile

Wire version `1` accepts only:

```text
YYYY-MM-DDTHH:MM:SS[.1-9 fractional digits](Z|±HH:MM)
```

The parser then applies calendar and offset validation and normalizes the accepted instant to UTC. The profile is intentionally narrower than every syntax that a general date-time library or a future standard extension might understand.

The following fail closed:

- timestamps without an explicit offset;
- spaces or lowercase separators;
- dates without explicit seconds;
- shortened or second-resolution numeric offsets;
- leap-second values;
- malformed calendar or clock values;
- bracketed time-zone annotations; and
- RFC 9557 suffixes carrying additional time-zone or calendar information.

RFC 9557 updates RFC 3339 with additional information. TEPP does not silently accept those suffixes in wire version `1`; a future adoption requires an ADR, a new compatibility decision, and a wire-version review.

### Temporal intervals

`TemporalInterval<T>` represents exact, bounded, one-sided open-ended, and explicitly unknown temporal evidence. It retains:

- one nominal clock type;
- included, excluded, or unbounded boundaries;
- precision from nanosecond through year; and
- exact, bounded, or unknown certainty.

An explicitly unknown interval is not treated as an interval containing every instant. Reversed boundaries, equal boundaries presented as bounded rather than exact, two-unbounded known intervals, and known intervals carrying unknown precision fail closed.

ISO 8601-2 defines broader extension syntax for uncertain, approximate, unspecified, set-valued, and extended interval representations. Task 3 does not claim that syntax. TEPP currently models its bounded uncertainty through typed domain values and its own versioned JSON records.

### JSON wire records and schemas

Clock and interval JSON records:

- require schema version `1`;
- require a clock discriminator;
- reject unknown fields;
- reconstruct through domain validation;
- distinguish malformed JSON, unsupported versions, clock mismatch, invalid timestamps, invalid precision, invalid certainty, invalid order, and empty intervals through stable redacting errors; and
- publish Draft 2020-12 JSON Schemas.

The generated schemas carry the same strict timestamp regular-expression profile used before runtime calendar validation. `format: date-time` remains an interoperability annotation; the explicit `pattern` records TEPP's narrower lexical contract.

JSON Schema alone does not prove calendar validity, UTC normalization, interval order, exact-boundary equality, or certainty consistency. Those semantic invariants remain runtime domain validation.

## Standards-to-code traceability

| Source | Contract used in Task 3 | Deliberate limit |
|---|---|---|
| ISO 8601-1:2019 | Gregorian date and 24-hour time representation, UTC and numeric offset framing | TEPP adopts a narrower wire profile and does not claim the complete standard |
| ISO 8601-2:2019 | informs the distinction between exact and uncertain temporal information | extension syntax is not accepted in wire version `1` |
| RFC 3339 | Internet timestamp baseline with explicit date, time, seconds, and offset | TEPP rejects leap seconds and other forms outside its strict profile |
| RFC 9557 | records the current extension path for timestamp annotations | bracketed annotations and suffixes are rejected in wire version `1` |
| JSON Schema Draft 2020-12 | object shape, required fields, constants, enums, patterns, and unknown-field control | semantic temporal validation remains in Rust |
| OWL-Time | future vocabulary for instants, intervals, and qualitative temporal relations | Allen-style relation reasoning is deferred to Task 4 |

At the implementation date, ISO 8601-1:2019 remained the published international standard, while a second-edition committee draft was under development. TEPP therefore binds wire version `1` to the published 2019 edition and requires an explicit compatibility review rather than following a draft automatically.

## Security and failure behavior

Temporal input can be attacker-controlled. Task 3 therefore:

- applies bounded ASCII syntax checks before calendar parsing;
- does not infer a missing offset from the host locale;
- does not echo rejected timestamp content in errors;
- rejects unknown JSON fields and malformed boundary shapes;
- binds wire records to the expected nominal clock; and
- reconstructs intervals instead of trusting serialized certainty or order declarations.

RFC 9557 identifies inconsistent interpretation and data-format vulnerabilities as timestamp risks. TEPP's versioned, narrow, fail-closed profile reduces parser differentials until the richer syntax is explicitly designed and tested.

## Verification mapping

The executable test suite covers:

| Evidence | Representative verification |
|---|---|
| lexical profile | separator, digit, fraction, offset, ASCII, leap-second, and suffix rejection |
| calendar semantics | invalid dates and times rejected after lexical acceptance |
| normalization | equivalent offsets, nanosecond ordering, and daylight-saving offset examples |
| nominal typing | every clock exposes the same contract without losing its clock identity |
| interval semantics | exact, bounded, included, excluded, open-ended, reversed, empty, and unknown cases |
| wire reconstruction | malformed JSON, unknown fields, versions, clock mismatches, boundary shapes, precision, and certainty |
| schema parity | Draft 2020-12 marker, constants, required fields, enums, and strict timestamp patterns |
| trust boundary | stable content-redacting errors and no hidden error source |

Line and branch coverage are merge gates, but coverage is not treated as proof that deferred relation algebra, persistence, leakage prevention, or event modeling exists.

## Deferred research boundary

Task 4 must separately doctor and implement:

- Allen's interval relations;
- converse and composition tables;
- path consistency and bounded closure;
- contradiction witnesses and proof provenance;
- resource limits against adversarial graphs; and
- property-based relation tests.

OWL-Time can provide the outward vocabulary, while the executable reasoner requires a primary algorithmic source and independent correctness tests. No Task 3 result is cited as evidence that this reasoner already exists.

## References

Hobbs, J. R., & Pan, F. (2017). *Time ontology in OWL* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Hutton, B., Andrews, H., Wright, A., & Dennis, G. (2022). *JSON Schema: A media type for describing JSON documents* (Draft 2020-12). JSON Schema. https://json-schema.org/draft/2020-12/json-schema-core.html

International Organization for Standardization. (2019a). *Date and time—Representations for information interchange—Part 1: Basic rules* (ISO Standard No. 8601-1:2019). https://www.iso.org/standard/70907.html

International Organization for Standardization. (2019b). *Date and time—Representations for information interchange—Part 2: Extensions* (ISO Standard No. 8601-2:2019). https://www.iso.org/standard/70908.html

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). RFC Editor. https://doi.org/10.17487/RFC3339

Sharma, U., & Bormann, C. (2024). *Date and time on the Internet: Timestamps with additional information* (RFC 9557). RFC Editor. https://doi.org/10.17487/RFC9557

Wright, A., Hutton, B., Andrews, H., & Dennis, G. (2022). *JSON Schema validation: A vocabulary for structural validation of JSON* (Draft 2020-12). JSON Schema. https://json-schema.org/draft/2020-12/json-schema-validation.html
