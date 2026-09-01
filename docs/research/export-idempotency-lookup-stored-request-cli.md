# Export lookup stored-request CLI quarantine (doctoring)

`tepp-export-lookup-request get` is quarantine-parity of ADR 0099. It mints
no executable `GET /v1/exports/by-idempotency/{idempotency_key}/request`
disclosure onto spawned `tepp-loopback` TCP. The typed exchange builder
returns `authorization_denied` after origin/key validation. HTTP semantics
follow RFC 9110 (Fielding, Nottingham, & Reschke, 2022).

An idempotency key is replay identity, not authorization to disclose another
tenant's stored create. `tenant_workspace_id` and `principal_id` never appear
on CLI stdout. `tepp.scientific_acceptance.v1` never appears. CLI failure is
not a scientific claim. `NaruonLiveService` stays POST-only. LineageWeave is
refused.

Does not re-open cancel lineages, GAP-010 Figma/export, persistence, Leiden,
or an ADR 0014 claim-promotion package. Does not weaken ADR 0099. Does not
duplicate `tepp-export-lookup` (#466) or `{export_id}/request` CLI (#459).
