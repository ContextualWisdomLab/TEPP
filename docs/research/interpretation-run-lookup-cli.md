# Interpretation-run lookup CLI (doctoring)

`tepp-interpretation-run-lookup lookup` mints
`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}` onto spawned
`tepp-orchestrator-loopback` TCP. HTTP semantics follow RFC 9110 (Fielding,
Nottingham, & Reschke, 2022). Fail-closed unpublished consumers, leftover
stdin, public bind, `localhost`, `http` origin, credential flags, reserved
prefix, and scientific-authority promotion are repository contract (ADR 0096;
ADR 0014).

`claim_status` remains `hypothetical`. `scientific_authority` remains false.
`tepp.scientific_acceptance.v1` never appears. CLI success is not a scientific
claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Dual identity of GET-by-id
(`idempotency_key`) versus this lookup (`interpretation_run_id`). Analog of
export idempotency-key lookup CLI; not a duplicate of GET-by-id or of lookup
GET (#467).
