# Export idempotency-key lookup CLI doctoring

`tepp-export-lookup lookup` mints ADR 0093's typed Naruon GET onto spawned `tepp-loopback` TCP. The CLI exists so an operator can resolve a purpose-bound export receipt without writing raw HTTP. HTTP framing follows RFC 9110 (Fielding, Nottingham, & Reschke, 2022); the consumer, privacy and scientific-authority boundaries are TEPP contracts.

The CLI accepts the same opaque idempotency-key domain as the create and HTTP contracts. A slash-containing key is not parsed as CLI routing syntax: it is passed to the typed exchange and percent-encoded into a single HTTP path segment. The literal value `by-idempotency` also remains valid key data after the route prefix. NUL and oversized values remain invalid. This prevents a valid create receipt from becoming operationally unresolvable because a later adapter invented a narrower key grammar.

Public binds, `localhost`, non-HTTPS origins, unpublished consumers, LineageWeave, credential-shaped flags, nonempty stdin, malformed framing and metric-bearing responses fail closed. Success stdout is the metric-free `ExportIdempotencyLookup`; tenant/principal/source-text and `tepp.scientific_acceptance.v1` are not emitted. `NaruonLiveService` remains POST-only.

ADR 0099's stored-request-by-idempotency convenience route remains quarantined and is not activated by this CLI. Resolving an export identity does not authorize disclosure of its original authorization request.

Exact-head regressions exercise POST→CLI lookup for ordinary, slash-containing and route-prefix-looking keys, plus loopback, credential, body, response-framing and scientific-field refusals. Persistence and public-service deployment remain outside this adapter slice.
