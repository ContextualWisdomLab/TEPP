# ADR 0100 — Quarantine-parity export lookup stored-request CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR security quarantine
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0099. Does not re-open cancel lineages.
Does not supersede ADR 0014. Unique versus protected main; 0026–0099 occupied
including #466=0093+0094+0099.
**Figma File ID:** N/A — this increment changes a Rust CLI binary and has no
user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0099 quarantines
`GET /v1/exports/by-idempotency/{idempotency_key}/request` because a
consumer-only lookup can disclose another tenant's stored authorization
request (`tenant_workspace_id`, `principal_id`) when the idempotency key is
unique in the naruon namespace. Operators still had no published binary that
mints that reserved route onto spawned `tepp-loopback` TCP. Reusing
`tepp-export-lookup` would collide with identity lookup. A disclosure CLI
would weaken the ADR 0099 fail-closed quarantine.

## Decision

Publish `tepp-export-lookup-request get` as quarantine-parity of ADR 0099:

- `from_args` admits loopback host, `https` origin, naruon consumer, and a
  syntactically valid key. Empty stdin is admitted.
- Compose calls `naruon_export_idempotency_lookup_stored_request_exchange`,
  which returns `authorization_denied` after origin/key validation. The CLI
  never serializes a stored authorization request and never prints
  `tenant_workspace_id` or `principal_id`.
- Public bind, `localhost`, `http` origin, unpublished consumer,
  LineageWeave, credential flags, reserved `by-idempotency` as a key,
  slash/NUL, and leftover stdin fail closed before the quarantine result.
- `NaruonLiveService` stays POST-only. CLI failure is not an ADR 0014 claim.

Reactivation of a disclosure CLI requires the same versioned
tenant-and-principal binding as ADR 0099.

## Non-goals

- Weakening ADR 0099 or treating an idempotency key as a bearer credential.
- Production TLS, public bind, or durable export storage.
- Project-history by-idempotency lookup (duplicates GET-by-id).
- Temporal-context stored-request GET (already #464).
- Re-opening cancel lineages, Leiden, persistence, or GAP-010.

## Alternatives considered

1. Disclosure CLI that prints the stored create — rejected; weakens ADR 0099.
2. Reuse `tepp-export-lookup` — rejected; ADR 0094.
3. Project-history by-idempotency lookup — rejected; GET-by-id already keys
   by `idempotency_key`.
4. Quarantine-parity dedicated binary — accepted.

## Consequences

Operators who try the reserved extra-segment from a published binary receive
the same authorization denial as the typed exchange. No stored create is
disclosed.

## Failure and recovery

Invalid hosts, origins, consumers, keys, leftover stdin, and the quarantined
valid path fail closed. Credential headers remain `authorization_denied`.

## Verification

- Valid `tepp-export-lookup-request get` returns `authorization_denied` and
  never prints tenant/principal/artifact identities from a stored create;
- LineageWeave, public bind, `localhost`, `http` origin, leftover stdin,
  reserved prefix, and slash fail closed before disclosure;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the published binary; ADR 0099 quarantine remains. A
superseding ADR is required to disclose stored creates from a client key.

## Related authority

ADR 0099, ADR 0094, ADR 0014, RFC 9110 (Fielding, Nottingham, & Reschke, 2022).
