# Export stored-request GET (doctoring)

`GET /v1/exports/{export_id}/request` returns one accepted naruon
export-authorization request on `tepp-loopback`. HTTP semantics follow RFC 9110
(Fielding, Nottingham, & Reschke, 2022). Fail-closed LineageWeave, extra
segments, slash/NUL, leftover bodies, credential flags, and scientific-authority
promotion are repository contract (ADR 0089; ADR 0014).

`tepp.scientific_acceptance.v1` never appears. HTTP 200 is not a scientific
claim. `NaruonLiveService` stays POST-only.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package.
