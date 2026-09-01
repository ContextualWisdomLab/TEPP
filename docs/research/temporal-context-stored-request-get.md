# Temporal-context stored-request GET (doctoring)

`GET /v1/temporal-context/{idempotency_key}/request` returns one accepted
LineageWeave create request on `tepp-loopback`. HTTP semantics follow RFC 9110
(Fielding, Nottingham, & Reschke, 2022). Fail-closed naruon, extra segments,
slash/NUL, leftover bodies, credential flags, and scientific-authority
promotion are repository contract (ADR 0091; ADR 0014).

`inference_status` on the stored projection remains `temporal_association_only`.
`tepp.scientific_acceptance.v1` never appears. HTTP 200 is not a scientific
claim.

Does not re-open cancel lineages, collection GET, GAP-010 Figma/export,
persistence, Leiden, or an ADR 0014 claim-promotion package.
