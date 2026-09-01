# ADR 0099 — Quarantine unscoped export idempotency-key stored-request lookup

**Decision status:** Accepted  
**Implementation maturity:** active-PR security quarantine  
**Date:** 2026-09-01  
**Supersedes:** the initial active-route interpretation of this same ADR; complements ADR 0093 and ADR 0089. Does not supersede ADR 0014.  
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.  
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0093 publishes `GET /v1/exports/by-idempotency/{idempotency_key}` as a
metric-free identity lookup. A follow-on implementation added
`GET /v1/exports/by-idempotency/{idempotency_key}/request` to return the stored
export-authorization request directly.

Exact-head review found that the first implementation scoped lookup only by
`tepp-consumer: naruon`. The underlying export registry is keyed by consumer,
tenant/workspace, and idempotency key, but the GET searched the whole Naruon
consumer namespace. A caller that knew another tenant's otherwise unique
idempotency key could therefore receive that tenant's original authorization
request, including `tenant_workspace_id` and `principal_id`. The route had no
request field or trusted header that could prove the caller's tenant and
principal scope.

This is an authorization-boundary defect, not a documentation-only issue. An
idempotency key is replay identity; it is not authorization to disclose the
stored create request.

## Decision

The stored-request-by-idempotency route is quarantined fail closed until the
Analysis Run API has an explicit tenant-and-principal authorization binding.

- The live dispatcher may still recognize the reserved route so it cannot fall
  through to a different GET interpretation, but serialization of a stored
  authorization request is rejected because tenant/principal identity is
  forbidden on this unscoped response path.
- The public Naruon exchange builder validates origin and key syntax, then
  returns `authorization_denied` rather than minting a request that the service
  cannot authorize correctly.
- Raw and percent-decoded `/` in the idempotency key are rejected. Proxy/path
  normalization must not change the identity interpreted by the loopback
  dispatcher.
- `tenant_workspace_id` and `principal_id` are now explicit forbidden response
  keys for this quarantined lookup, in addition to scientific metric and
  terminal-result keys.
- Lookup GET from ADR 0093 remains metric-free and separate. Stored-request GET
  by server-issued `export_id` remains a different adapter contract and is not
  authorized by this ADR.
- `NaruonLiveService` stays POST-only. LineageWeave remains refused on this
  Naruon-owned adapter. HTTP failure/success is never ADR 0014 scientific
  evidence.

Reactivation requires a versioned contract that binds the request to the
already-authorized tenant/workspace and principal (or an equivalent stronger
authorization context), proves cross-tenant and cross-principal denial, and
passes exact-head security/coverage/review gates. A consumer-only check or
knowledge of an idempotency key is insufficient.

## Non-goals

- Inventing a new authentication scheme inside this repair.
- Treating an idempotency key as a bearer credential.
- Weakening the metric-free export identity lookup from ADR 0093.
- Production TLS, public bind, or durable export storage.
- Promoting an ADR 0014 scientific claim from transport state.
- Re-opening cancel lineages, persistence, Leiden, or GAP-010 UI work.

## Alternatives considered

1. Keep the route because idempotency keys are expected to be opaque — rejected;
   opacity is not an authorization boundary.
2. Return the original request after checking only `tepp-consumer: naruon` —
   rejected because all Naruon tenants share that consumer code.
3. Add ad-hoc tenant/principal headers in this repair — rejected until those
   values have a defined authenticated authority and versioned admission
   contract; trusting caller-supplied scope would only move the defect.
4. Quarantine the route while preserving deterministic parsing and evidence —
   accepted.

## Consequences

The convenience one-hop stored-request lookup is temporarily unavailable, but
no cross-tenant request identity can be disclosed through this path. Operators
can continue to use the metric-free idempotency lookup and other independently
authorized export surfaces. The feature can return only after its authorization
context is explicit and testable.

## Failure and recovery

A syntactically valid stored-request-by-idempotency client request fails closed
with authorization denial. Direct loopback attempts cannot emit the stored
request because the response guard rejects tenant/principal identity. Unknown
keys, extra path segments, raw or percent-decoded slash, NUL, reserved prefix,
nonempty body, POST, LineageWeave, unpublished consumers, credential headers,
and non-loopback hosts remain fail closed.

Recovery requires RED tests proving that same idempotency keys across different
tenants and principals cannot cross-read, followed by a versioned authorization
binding and exact-head GREEN security/coverage evidence.

## Verification

- a valid-looking client exchange is denied while scope binding is absent;
- a posted export followed by unscoped stored-request-by-idempotency GET returns
  a redacted error and does not echo tenant, principal, or artifact identity;
- serialized authorization requests carrying `tenant_workspace_id` or
  `principal_id` are rejected on this response boundary;
- `%2F` and raw slash in an idempotency key are rejected consistently;
- LineageWeave, unknown keys, cancel extra-segments, metric payloads and
  malformed origins remain fail closed;
- exact-head branch/line coverage, clippy, rustdoc, security workflows and
  independent review remain required before any surviving landing vehicle may
  advance.

## Rollback and supersession

Do not roll back to the unscoped active route. A future superseding decision may
reactivate this resource only with an authenticated tenant/principal (or
stronger equivalent) scope contract and regression evidence. Repository-wide
ADR identity normalization remains tracked separately; this file preserves the
existing 0099 lineage rather than minting another operation-specific ADR.

## Related authority

ADR 0093, ADR 0089, ADR 0054, ADR 0014, RFC 9110 (Fielding, Nottingham, &
Reschke, 2022).
