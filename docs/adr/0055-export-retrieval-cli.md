# ADR 0055 — Loopback export retrieval CLI

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0054 (export retrieval GET) for the operator-visible client. Does not reuse ADR 0026 (export-authorize CLI / engine-library stacks), ADR 0053 (Pareto profile), or ADR 0054. Does not supersede ADR 0014 claim-promotion authority.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

ADR 0054 owns `GET /v1/exports/{export_id}` on `AnalysisRunLiveService` and the
typed `naruon_export_retrieval_exchange`. Operators still have to hand-roll
HTTP/1.1 to retrieve a minted export identity on spawned `tepp-loopback`.
`tepp-exports authorize` (#410) POSTs authorization on `NaruonLiveService` and
does not retrieve. Wait CLI (#406), lookup CLI (#401), retry-parent CLI
(#400), and temporal-context CLI (#414) are different verbs or different
stacks. `tepp_api` owns export retrieval; the CLI belongs here.

## Decision

Publish `tepp-export-get`:

- `tepp-export-get get` mints `naruon_export_retrieval_exchange` and renders
  through `loopback_http1_from_export_retrieval_exchange`.
- `--origin` stays the published HTTPS origin; only `--host` is the loopback
  bind address printed by `tepp-loopback`.
- Empty stdin is required; nonempty GET bodies fail closed.
- Success stdout is a metric-free `200 OK` identity (`export_id`,
  `artifact_id`, `decision_code`, `purpose`, `idempotency_key`).
- Public bind hosts, `localhost`, unpublished consumers, LineageWeave,
  credential-shaped flags, and non-`https` origins fail closed.
- The GET exchange does not send an `idempotency-key` header.
- Persistence remains GAP-003B.
- This slice does not add GET to `NaruonLiveService`. LineageWeave remains
  refused on this naruon-owned adapter.

## Non-goals

- Production TLS, public bind, or durable export storage.
- Leiden community detection, Driver p.16 restoration, or Figma/export work.
- Promoting an ADR 0014 scientific claim from HTTP success.
- Export-authorize CLI (`tepp-exports`), export retrieval GET HTTP, GET-by-id,
  wait CLI, or adding GET to `NaruonLiveService`.

## Alternatives considered

1. **Keep hand-rolled export-retrieval HTTP in each operator script** —
   rejected because GAP-003A is operator-visible and authorize already has a
   CLI on another stack.
2. **Add `get` to `tepp-exports` on the authorize-CLI stack** — rejected
   because that stack does not include retrieval GET (#411) and targets
   `NaruonLiveService`.
3. **Open LineageWeave or add GET to `NaruonLiveService`** — rejected; Naruon
   owns the current purpose-bound export adapter (ADR 0011/0018/0054).
4. **Metric-free naruon export-retrieval CLI on `tepp-loopback`** — accepted.

## Consequences

- Operators can retrieve a minted export identity without embedding the
  library or writing HTTP/1.1.
- HTTP 200 on retrieval is not release evidence.

## Failure and recovery

Non-loopback hosts, `localhost`, non-`https` origins, unpublished consumers,
LineageWeave, metric keys, empty identities, nonempty GET bodies, and unknown
capabilities return a fail-closed API error. The in-memory registry is not
durable; a restart requires re-POSTing the original metric-free
authorization.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Export retrieval remains loopback-served, size-bounded, naruon-scoped, and
  content-redacting.
- Retrieval receipts stay metric-free. Tenant, principal, and source-text
  flags stay off stdout.

## Compatibility and migration

Export retrieval GET and the naruon GET exchange are unchanged. Production
adapters may replace loopback while preserving metric-free retrieval fields.
`tepp-exports authorize` remains the POST client on `NaruonLiveService`.

## Verification

Falsifiable evidence:

- naruon export-retrieval CLI is HTTPS GET `/v1/exports/{export_id}` without
  credentials, RMSE keys, or an `idempotency-key` header;
- LineageWeave, public bind, `localhost`, `http://` origins, and nonempty
  bodies fail closed;
- POST mint then typed GET CLI stdout matches the minted `artifact_id` and
  never prints `tepp.scientific_acceptance.v1`;
- `NaruonLiveService` still refuses the composed GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes the export-retrieval CLI; export retrieval GET and the
naruon GET exchange remain valid. A superseding ADR is required to persist
retrieval, bind a public address, emit scientific-acceptance or JSON-LD on
retrieval, add GET to `NaruonLiveService`, or treat HTTP success as an
ADR 0014 claim.

## Related authority

- ADR 0054 owns export retrieval GET.
- ADR 0009 owns purpose-bound disclosure without blanket masking.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0011 owns standalone/modular HTTP boundaries.
- ADR 0014 owns scientific claim promotion.
