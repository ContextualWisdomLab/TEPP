# ADR 0008: Immutable evidence identities, digests, and exact spans

- **Status:** Accepted
- **Date:** 2026-08-05
- **Decision owners:** Contextual Wisdom Lab
- **Supersedes:** None

## Context

Every later temporal, event, multilingual, and psychometric claim must be
traceable to source evidence without depending on a mutable caller buffer,
ambiguous text offsets, or an identifier that changes when content is rehashed.
UTF-8 byte coordinates alone are unsafe for human-facing text selection because
a byte offset can fall inside a multibyte code point. Character-only offsets do
not preserve the byte-exact source location required for audit, hashing, and
interchange. PDF or page-derived evidence additionally needs validated layout
coordinates.

A content digest is evidence that bytes differ or agree under the selected hash
algorithm; it is not a substitute for a stable record identity, authorization,
authenticity, a signature, or provenance. The domain layer must therefore keep
identity, content verification, source ownership, and location as separate
contracts.

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
8. Keep domain fields private. Persistence and external serialization will use
   explicit versioned DTOs rather than making storage or wire formats the domain
   model by accident.
9. Treat `SHA-256` equality as content-verification evidence only. Signed
   provenance, tenant authorization, source acquisition metadata, and chain of
   custody remain separate later contracts.

## Consequences

- Evidence records remain stable even when identical content is ingested into
  distinct provenance contexts.
- Callers can verify content without receiving mutable access to accepted bytes
  or text.
- Exact spans can round-trip across Unicode text and page-oriented evidence
  without silently snapping to nearby boundaries.
- Scalar offsets are not grapheme-cluster or word boundaries. User-facing text
  segmentation remains a separate locale- and language-profile concern.
- The current slice does not yet provide source acquisition metadata,
  signatures, JSON DTOs, database migrations, W3C PROV serialization, or
  cryptographic chain-of-custody evidence.

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
- Production line and branch coverage remain exact 100% merge gates.

## References

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
