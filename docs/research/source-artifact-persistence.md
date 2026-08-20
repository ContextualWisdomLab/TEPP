# Source-artifact persistence (doctoring)

## Scope

`source_artifact` already exists on the foundation schema as an append-only
identity distinct from `document_record`. This slice adds the fail-closed
insert and primary-key lookup contract so an artifact cannot be persisted
with a non-canonical digest, a negative size, or a hostile media-type or
object-store label. ADR 0013 also requires idempotent writes: a retry of the
same immutable identity must succeed, and a same-id payload change must fail
closed (Jensen & Snodgrass, 1999). The artifact identity remains independent of the content
digest: identical bytes may be acquired in different tenant or provenance
contexts (National Institute of Standards and Technology, 2015).

This does not add a new migration number. Append-only triggers already live
in `0004`. Bytes themselves stay in the evidence crate or a protected object
store; this contract binds the identity, digest, declared size, and clocks.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model*.
World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

National Institute of Standards and Technology. (2015). *Secure Hash Standard
(SHS)* (FIPS PUB 180-4). https://doi.org/10.6028/NIST.FIPS.180-4

Identity and digest must stay separate. Collapsing the primary key onto
`content_sha256` would erase distinct acquisition events (Moreau & Missier,
2013). Availability and system time remain explicit so later cutoff
eligibility can exclude artifacts that were not yet available (Jensen &
Snodgrass, 1999).

## Verification

- contract tests reject short/uppercase digests, negative size, empty and
  hostile media types, oversized labels, and empty/hostile object refs;
- zero-byte artifacts and `NULL` object refs render;
- recording-session coverage for insert and primary-key lookup;
- live PostgreSQL CI inserts a valid artifact, retries the same identity,
  looks it up by identity, refuses a same-id digest change, and refuses a
  negative size when `TEPP_LIVE_POSTGRES=1`.
