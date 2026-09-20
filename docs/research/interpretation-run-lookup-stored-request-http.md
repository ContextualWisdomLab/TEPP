# Interpretation-run lookup stored-request GET (doctoring)

`GET /v1/interpretation-runs/by-run-id/{interpretation_run_id}/request` returns
the stored contextual-orchestrator create request on
`tepp-orchestrator-loopback`. HTTP semantics follow RFC 9110 (Fielding,
Nottingham, & Reschke, 2022). Fail-closed unpublished consumers, extra
segments, slash/NUL, reserved prefix, zero or ambiguous matches, credential
flags, cancel extra-segment, and scientific-authority promotion are repository
contract (ADR 0097; ADR 0014).

`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. HTTP 200 is not a scientific claim.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Dual identity of stored-request GET
(`idempotency_key`) versus this lookup (`interpretation_run_id`). Not a
duplicate of lookup GET (#467) or of `{key}/request` (#453).
