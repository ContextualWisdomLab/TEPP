# Interpretation-run stored-request CLI (doctoring)

`tepp-interpretation-run-request get` mints
`GET /v1/interpretation-runs/{idempotency_key}/request` onto spawned
`tepp-orchestrator-loopback` TCP. HTTP semantics follow RFC 9110 (Fielding,
Nottingham, & Reschke, 2022). Fail-closed unpublished consumers, leftover
stdin, public bind, `localhost`, `http` origin, credential flags, and
scientific-authority promotion are repository contract (ADR 0086; ADR 0014).

`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. CLI success is not a scientific claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package.
