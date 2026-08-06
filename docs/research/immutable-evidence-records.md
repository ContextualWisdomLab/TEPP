# Immutable evidence records: standards and implementation boundaries

## Scope

This doctoring note records the standards basis for TEPP's first executable
evidence-domain slice. It supports immutable source bytes, stable identities,
`SHA-256` content digests, UTF-8 document records, exact byte and Unicode-scalar
spans, and optional page-layout coordinates. It does not claim source
attestation, digital signatures, W3C PROV interchange, database persistence, or
a complete chain of custody.

## Cryptographic content digests

FIPS PUB 180-4 specifies the Secure Hash Standard, including `SHA-256`. TEPP
uses `SHA-256` to detect whether candidate bytes agree with the bytes accepted
into an immutable source or document record. The digest is deliberately not the
record identifier: identical bytes may be acquired in different provenance,
tenant, document, or event contexts and must remain separately addressable.

NIST has announced a decision to revise FIPS 180-4. The implementation therefore
names the algorithm explicitly, pins its Rust dependency, stores canonical
algorithm output rather than an implementation-specific object, and leaves
algorithm agility to a future versioned digest contract. No current code claims
that a bare digest proves authorship, authorization, or authenticity.

## UTF-8 and exact coordinates

RFC 3629 defines UTF-8 as a transformation format for Unicode. A valid TEPP span
must begin and end at UTF-8 code-point boundaries. It also stores Unicode-scalar
coordinates and verifies that they agree exactly with the selected byte range.
This dual representation prevents a caller from presenting a visually plausible
character range that points to different source bytes.

Unicode Standard Annex #29 defines default grapheme-cluster, word, and sentence
boundaries and explicitly permits tailoring. TEPP's current scalar coordinates
are lower-level evidence locations, not user-perceived grapheme, word, or
sentence segmentation. Language-aware semantic segmentation remains a later
validated module and must not be inferred from scalar offsets alone.

## Provenance boundary

W3C PROV-DM and PROV-O distinguish entities, activities, agents, derivations,
and attribution. TEPP's separate record identifier, content digest, source link,
and exact span are prerequisites for that model but are not themselves a full
PROV graph. Later persistence and export work must retain this separation and
add acquisition activities, responsible agents, derivation relations, and
versioned serialization without changing accepted evidence bytes.

## Engineering implications

- Caller-owned buffers are copied before acceptance.
- Artifact and document size limits are checked before immutable allocation.
- Stable `UUIDv7` identity and `SHA-256` content verification are independent.
- Page geometry accepts only finite, positive dimensions and bounded rectangles.
- Error messages are stable and redact content.
- Domain fields remain private; external APIs will use explicit versioned DTOs.
- Exact 100% production line and branch coverage is required because every
  validation branch is part of the evidence trust boundary.

## References

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

National Institute of Standards and Technology. (2023, March 7). *Decision to
revise FIPS 180-4, Secure Hash Standard*. NIST Computer Security Resource Center.
https://csrc.nist.gov/News/2023/decision-to-revise-fips-180-4

The Unicode Consortium. (2025). *Unicode Standard Annex #29: Unicode text
segmentation* (Revision 47).
https://www.unicode.org/reports/tr29/tr29-47.html

Yergeau, F. (2003). *UTF-8, a transformation format of ISO 10646* (RFC 3629).
RFC Editor. https://doi.org/10.17487/RFC3629

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*.
World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV
ontology*. World Wide Web Consortium. https://www.w3.org/TR/prov-o/
