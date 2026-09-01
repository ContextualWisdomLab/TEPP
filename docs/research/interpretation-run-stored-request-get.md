# Interpretation-run stored-request GET (doctoring)

`GET /v1/interpretation-runs/{idempotency_key}/request` returns one accepted
contextual-orchestrator create request on `tepp-orchestrator-loopback`. HTTP
semantics follow RFC 9110 (Fielding, Nottingham, & Reschke, 2022). Fail-closed
unpublished consumers, extra segments, slash/NUL, credential flags, and
scientific-authority promotion are repository contract (ADR 0085; ADR 0014).

`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. HTTP 200 is not a scientific claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package.
