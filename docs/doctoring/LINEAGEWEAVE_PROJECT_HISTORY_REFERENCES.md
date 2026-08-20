# LineageWeave project-history contract references

This doctoring record documents the authorities used by TEPP's versioned LineageWeave project-history projection. The projection validates and orders explicitly supplied evidence. It does not infer a missing event, identify a hidden actor, estimate theta, calculate confidence, or promote temporal order to causation.

## Contract decisions

| Authority | TEPP decision |
|---|---|
| ISO 8601-1:2019 and RFC 3339 | Parse event, availability, and knowledge-cutoff timestamps as absolute clocks and reject malformed or future-leaking evidence. |
| W3C Time Ontology in OWL and Allen interval algebra | Represent temporal relations separately from causal or psychometric authority. The response contract exposes `temporal_association_only`. |
| W3C PROV-O / PROV-DM | Preserve source identities and evidence references; findings may cite only event and post identities contained in the submitted authorized bundle. |
| RFC 8259 | Use strict versioned JSON DTOs with unknown-field rejection and bounded collections. |
| RFC 9110 | Publish an explicit POST resource path, media type, idempotency key, and fail-closed error behavior. |
| Allen (1983) | Apply deterministic qualitative temporal ordering without claiming that succession establishes cause. |

## Invariants

1. `available_at` must not exceed the request `knowledge_cutoff`.
2. Event identities must be unique and the focus event must belong to the request.
3. The response must preserve every submitted event and its evidence fields.
4. Participant count must equal the distinct opaque actor identities present in the supplied events.
5. Findings must cite only supplied event IDs and source-post IDs.
6. LineageWeave and Naruon use consumer-scoped idempotency namespaces.
7. No caller credential or cross-service database access is part of the project-history contract.
8. Loopback HTTP is a local modular boundary; a non-loopback deployment requires HTTPS/TLS at the service edge.

## APA 7th references

Allen, J. F. (1983). Maintaining knowledge about temporal intervals. *Communications of the ACM, 26*(11), 832–843. https://doi.org/10.1145/182.358434

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange format* (RFC 8259). Internet Engineering Task Force. https://doi.org/10.17487/RFC8259

Cox, S., & Little, C. (Eds.). (2017). *Time ontology in OWL*. World Wide Web Consortium. https://www.w3.org/TR/owl-time/

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

International Organization for Standardization. (2019). *Date and time—Representations for information interchange—Part 1: Basic rules* (ISO Standard No. 8601-1:2019). https://www.iso.org/standard/70907.html

Klyne, G., & Newman, C. (2002). *Date and time on the Internet: Timestamps* (RFC 3339). Internet Engineering Task Force. https://doi.org/10.17487/RFC3339

Moreau, L., & Missier, P. (Eds.). (2013a). *PROV-DM: The PROV data model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

Moreau, L., & Missier, P. (Eds.). (2013b). *PROV-O: The PROV ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/
