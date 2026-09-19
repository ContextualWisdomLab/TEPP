# Interpretation-run lookup stored-request CLI (doctoring)

`tepp-interpretation-run-lookup-request get` mints
`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request` onto
spawned `tepp-orchestrator-loopback` TCP. HTTP semantics follow RFC 9110
(Fielding, Nottingham, & Reschke, 2022). Fail-closed unpublished consumers,
leftover stdin, public bind, `localhost`, `http` origin, reserved prefix,
slash/NUL, credential flags, cancel extra-segment, and scientific-authority
promotion are repository contract (ADR 0098; ADR 0014).

`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. CLI success is not a scientific claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Dual identity of stored-request CLI
(`idempotency_key`) versus this lookup (`interpretation_run_id`). Not a
duplicate of lookup stored-request GET (#469) or of `{key}/request` CLI (#454).
