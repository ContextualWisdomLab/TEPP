# Export idempotency-key lookup stored-request GET (doctoring)

`GET /v1/exports/by-idempotency/{idempotency_key}/request` returns the stored
naruon export-authorization request on `tepp-loopback`. HTTP semantics follow
RFC 9110 (Fielding, Nottingham, & Reschke, 2022). Fail-closed unpublished
consumers, extra segments, slash/NUL, reserved prefix, zero or ambiguous
matches, credential flags, cancel extra-segment, and scientific-authority
promotion are repository contract (ADR 0099; ADR 0014).

`tepp.scientific_acceptance.v1` never appears. HTTP 200 is not a scientific
claim. `NaruonLiveService` stays POST-only. LineageWeave is refused.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Dual identity of stored-request GET
(`export_id`) versus this lookup (`idempotency_key`). Not a duplicate of
lookup GET (#466) or of `{export_id}/request` (#459).
