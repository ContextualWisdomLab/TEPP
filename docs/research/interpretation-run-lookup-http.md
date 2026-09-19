# Interpretation-run lookup GET (doctoring)

`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}` returns one
accepted contextual-orchestrator metric-free identity on
`tepp-orchestrator-loopback`. HTTP semantics follow RFC 9110 (Fielding,
Nottingham, & Reschke, 2022). Fail-closed unpublished consumers, extra
segments, slash/NUL, reserved prefix, zero or ambiguous matches, credential
flags, and scientific-authority promotion are repository contract (ADR 0095;
ADR 0014).

`claim_status` remains `hypothetical`. `scientific_authority` remains false.
`tepp.scientific_acceptance.v1` never appears. HTTP 200 is not a scientific
claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Dual identity of GET-by-id
(`idempotency_key`) versus this lookup (`interpretation_run_id`). Analog of
export idempotency-key lookup; not a duplicate of GET-by-id.
