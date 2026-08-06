# ADR 0008: Immutable evidence identities, digests, exact spans, and wire records

- **Status:** Accepted
- **Date:** 2026-08-05
- **Decision owners:** Contextual Wisdom Lab
- **Supersedes:** None

## Context

Every later temporal, event, multilingual, and psychometric claim must be
traceable to source evidence without depending on a mutable caller buffer,
ambiguous text offsets, an identifier that changes when content is rehashed, or
a wire payload that can bypass domain validation. UTF-8 byte coordinates alone
are unsafe for human-facing text selection because a byte offset can fall inside
a multibyte code point. Character-only offsets do not preserve the byte-exact
source location required for audit, hashing, and interchange. PDF or
page-derived evidence additionally needs validated layout coordinates.

A content digest is evidence that bytes differ or agree under the selected hash
algorithm; it is not a substitute for a stable record identity, authorization,
authenticity, a signature, or provenance. The domain layer must therefore keep
identity, content verification, source ownership, location, and interchange as
separate contracts. External JSON must not deserialize directly into private
domain fields or silently accept unknown fields, unsupported versions, altered
content, or coordinates that no longer match the reconstructed document.

## Decision

1. Generate independent record identifiers as RFC 9562 `UUIDv7` values.
2. Hash immutable source bytes and UTF-8 document text with `SHA-256` and expose
   the digest as a canonical lower-case 64-character hexadecimal value.
3. Copy caller-provided bytes and text into owned immutable storage before a
   record is accepted. Later mutation of the caller's buffer cannot change the
   accepted record.
4. Reject empty artifacts and documents and enforce explicit byte limits before
   allocating immutable records.
5. Represent a text span with all of the following:
   - owning document identifier;
   - inclusive byte start and exclusive byte end;
   - inclusive Unicode-scalar start and exclusive Unicode-scalar end; and
   - optional validated page and layout coordinates.
6. Validate byte bounds, UTF-8 boundaries, scalar bounds, and exact agreement
   between byte and Unicode-scalar coordinates. Empty, reversed, mismatched,
   mid-code-point, out-of-bounds, and cross-document spans fail closed.
7. Use one-based positive page numbers, finite positive page dimensions, finite
   nonnegative offsets, positive rectangle dimensions, and in-page bounds.
8. Keep domain fields private. Interchange uses explicit internal DTOs carrying
   a required `schema_version`; the current accepted version is `1`.
9. Reject malformed JSON, missing or unknown fields, unsupported versions,
   malformed identifiers and digests, invalid byte values, and unknown nested
   page-layout fields with stable content-redacting errors.
10. Reconstruct every wire record through validated domain constructors. A
    declared digest is recomputed from the supplied bytes or UTF-8 text, and a
    mismatch fails with `ContentDigestMismatch` before a record is accepted.
11. Reapply configured content limits during wire reconstruction and revalidate
    exact byte/scalar coordinates and page geometry against the supplied owning
    document. Serialization never exposes internal caches such as scalar
    lengths or storage representation.
12. Treat `SHA-256` equality as content-verification evidence only. Signed
    provenance, tenant authorization, source acquisition metadata, and chain of
    custody remain separate later contracts.

## Consequences

- Evidence records remain stable even when identical content is ingested into
  distinct provenance contexts.
- Callers can verify content without receiving mutable access to accepted bytes
  or text.
- Exact spans can round-trip across Unicode text and page-oriented evidence
  without silently snapping to nearby boundaries.
- Strict versioned JSON can cross process or service boundaries without making
  serde layout or private Rust fields the public domain model.
- Wire reconstruction detects content substitution, stale document ownership,
  unknown extensions, unsupported versions, and hostile coordinate changes.
- Scalar offsets are not grapheme-cluster or word boundaries. User-facing text
  segmentation remains a separate locale- and language-profile concern.
- The current slice does not yet provide source acquisition metadata,
  signatures, database migrations, JSON Schema publication, W3C PROV
  serialization, or cryptographic chain-of-custody evidence.

## Validation

- RFC 9562 vectors and generated `UUIDv7` identifiers are tested.
- Known `SHA-256` vectors, canonical hexadecimal round trips, malformed digests,
  caller-buffer mutation, empty input, and byte limits are tested.
- Hostile multibyte Unicode text verifies byte and scalar counts.
- Exact-span tests cover valid selections, empty and reversed ranges,
  out-of-bounds coordinates, both UTF-8 boundaries, scalar mismatches, and
  cross-document use.
- Page tests cover invalid page numbers, nonfinite and nonpositive dimensions,
  invalid rectangle components, and horizontal and vertical overflow.
- Versioned wire tests cover identity-preserving round trips, unknown fields,
  unsupported versions, malformed JSON, malformed identifiers and digests,
  digest/content mismatch, content limits, invalid byte values, nested field
  rejection, cross-document spans, invalid UTF-8 boundaries, and out-of-page
  geometry.
- Generated multilingual and decomposed-Unicode cases enumerate every valid
  nonempty code-point-aligned span and prove exact JSON round trips; invalid
  scalar and mid-code-point coordinates fail closed.
- Production line and branch coverage remain exact 100% merge gates.

## References

Bray, T. (Ed.). (2017). *The JavaScript Object Notation (JSON) data interchange
format* (RFC 8259). RFC Editor. https://doi.org/10.17487/RFC8259

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

The Unicode Consortium. (2025). *Unicode Standard Annex #29: Unicode text
segmentation* (Revision 47).
https://www.unicode.org/reports/tr29/tr29-47.html

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629).
RFC Editor. https://doi.org/10.17487/RFC3629

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*.
World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV
ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/
