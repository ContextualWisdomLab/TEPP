# ADR 0008 — Immutable evidence identities, digests, exact spans, and wire records

**Decision status:** Accepted  
**Implementation maturity:** implemented-main — inbound size/depth/identity/provenance refusal for untrusted documents, records, checkpoints, and LLM outputs is `payload_bound` on the active PR  
**Date:** 2026-08-05  
**Decision owners:** Contextual Wisdom Lab  
**Supersedes:** None. ADR 0013 owns future persistence, reproducibility-manifest, and relation-aware split authority.

## Context

Every later temporal, event, multilingual, and psychometric claim must be traceable to source evidence without depending on a mutable caller buffer, ambiguous text offsets, an identifier that changes when content is rehashed, or a wire payload that can bypass domain validation. UTF-8 byte coordinates alone can point inside a multibyte code point, while character-only offsets do not preserve the byte-exact source location required for audit and interchange. Page-derived evidence additionally needs validated geometry.

A content digest proves only byte equality/difference under the selected algorithm. It is not stable record identity, authorization, authenticity, signature, or provenance. Identity, content verification, source ownership, exact location, and interchange must therefore remain separate contracts.

## Decision

1. Generate independent record identifiers as RFC 9562 `UUIDv7` values.
2. Hash immutable source bytes and UTF-8 document text with SHA-256 and expose a canonical lowercase hexadecimal digest.
3. Copy caller-provided bytes/text into owned immutable storage before acceptance.
4. Enforce explicit bounded content limits before allocating accepted records.
5. Represent a text span with owning document identity, half-open UTF-8 byte coordinates, matching half-open Unicode-scalar coordinates, and optional validated page/layout geometry.
6. Reject empty/reversed/out-of-bounds/mismatched/mid-code-point/cross-document spans and invalid/non-finite/out-of-page geometry.
7. Keep domain fields private. Public interchange uses explicit versioned DTOs; unknown fields and unsupported versions fail closed.
8. Reconstruct every wire record through validated domain constructors and recompute content digests rather than trusting declared hashes.
9. Reapply configured content limits and ownership/coordinate/geometry validation during reconstruction.
10. Treat SHA-256 as content-verification evidence only; provenance, authorization, acquisition metadata, signatures, and chain of custody are separate authorities.

## Non-goals

- do not treat scalar offsets as grapheme/word/sentence boundaries;
- do not use digest equality as proof of provenance or authorization;
- do not allow serde/private struct layout to become the public wire contract;
- do not define the database/run-manifest model here; ADR 0013 owns that layer.

## Alternatives considered

1. **Content hash as record primary identity** — rejected because identical bytes can legitimately exist in different provenance/authorization contexts.
2. **Only byte offsets or only character offsets** — rejected because neither alone satisfies both byte-exact audit and Unicode-safe human location.
3. **Direct deserialization into private domain structs** — rejected because it can bypass invariants and couples wire compatibility to implementation layout.
4. **Independent opaque identity + canonical digest + dual coordinates + strict versioned reconstruction** — accepted.

## Consequences

Evidence records remain stable even when identical content is ingested into distinct provenance contexts. Exact spans round-trip across Unicode/page evidence. Strict wire reconstruction detects substitution, stale ownership, unknown extensions, unsupported versions, and hostile coordinate changes. Locale/language segmentation remains a separate concern under ADR 0004.

## Failure and recovery

Malformed JSON, unsupported versions, invalid identifiers/digests/bytes, content-digest mismatch, configured-limit overflow, invalid UTF-8 boundaries, scalar disagreement, cross-document ownership, or bad page geometry fails closed with content-redacting errors. Recovery requires corrected authorized evidence or a versioned migration; it never mutates an accepted record in place to make reconstruction succeed.

## Security, privacy, and governance impact

Immutable evidence and exact spans are necessary for audit but can expose sensitive source context. Authorization and PII use follow ADR 0009. Ordinary logs should carry opaque IDs/digests and bounded diagnostics rather than raw source text. Digest equality does not grant access.

## Compatibility and migration

Wire records carry explicit schema versions. A new version must preserve or intentionally migrate identity/content/location semantics and keep old artifacts reconstructable or explicitly unsupported with migration tooling. Persistence/run-manifest binding is defined by ADR 0013.

## Verification

Tests cover RFC 9562 identifiers, SHA-256 vectors and mutation detection, owned-buffer immutability, hostile multibyte Unicode, exact byte/scalar span enumeration, page boundaries, stable redacted errors, strict round trips, unknown-field/version rejection, digest/content mismatch, configured limits, invalid byte values, nested field rejection, cross-document spans, invalid UTF-8 boundaries, and generated multilingual/decomposed-Unicode cases. Production line/branch coverage remains exact 100%.

## Rollback and supersession

Rollback uses the previous supported wire/domain version without rewriting immutable records. Supersede only with a decision that preserves independent record identity, canonical content verification, exact source-location semantics, strict reconstruction, and provenance/authorization separation.

## References

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange format* (RFC 8259). RFC Editor. https://doi.org/10.17487/RFC8259

Davis, K., Peabody, B., & Leach, P. (2024). *Universally unique identifier (UUID)* (RFC 9562). RFC Editor. https://doi.org/10.17487/RFC9562

National Institute of Standards and Technology. (2015). *Secure Hash Standard (SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629). RFC Editor. https://doi.org/10.17487/RFC3629

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*. World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/
