# ADR 0053 — Loopback export retrieval GET path

**Decision status:** Accepted
**Implementation maturity:** active-PR
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0009, ADR 0011, and ADR 0018 for the operator-visible jump from a purpose-bound export authorization to a durable export identity. Does not supersede ADR 0014 claim-promotion authority. ADR 0026–0052 remain on live GAP-003A engine-library, terminal-wire DTO, GET-by-id, lifecycle-POST, cancel, loopback-CLI, collection-GET, retry, stored-request, retry-lineage, lookup, retry-parent, wait-CLI, and analysis-run profile slices.
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

`POST /v1/exports` on `NaruonLiveService` authorizes a purpose-bound export and
returns a decision without minting a retrievable identity. JSON-LD and GraphML
envelopes exist as library contracts. Operators who hold a 200 authorization
therefore cannot address that export later. Returning RMSE, bias, coverage,
SE-gate, source text, or `tepp.scientific_acceptance.v1` on the retrieval body
would treat authorization identity as measurement evidence. GET-by-id (#359)
is status/terminal by `run_id` on another stack and remains 400 here. GAP-010
Figma/export is a visual-analytics workflow and is not this HTTP identity.

## Decision

`AnalysisRunLiveService` (`tepp-loopback`) serves naruon-only export routes on
loopback:

- `POST /v1/exports` mints an opaque `export_id`, stores a metric-free
  retrieval receipt in memory, and returns HTTP 200 with that receipt.
- `GET /v1/exports/{export_id}` returns the same metric-free receipt.
- The payload is `export_id`, `artifact_id`, `decision_code`, `purpose`, and
  `idempotency_key`.
- `tepp.scientific_acceptance.v1`, RMSE, bias, coverage, SE-gate, report,
  `terminal_result`, `tenant_workspace_id`, `principal_id`,
  `includes_source_text`, and source bodies never appear.
- LineageWeave is refused. Naruon owns the current purpose-bound export
  adapter. `NaruonLiveService` stays POST-only.
- Empty GET bodies only. Query strings, collection GET `/v1/exports`,
  GET-by-id analysis-run paths, POST `/v1/exports/{export_id}`, and nonempty
  GET bodies fail closed.
- The identity travels in the path. The NARUON GET exchange does not send an
  `idempotency-key` header or credentials.
- Unknown identities fail closed. Persistence remains GAP-003B.

## Non-goals

- Production TLS, public bind, or durable export storage.
- Leiden community detection, Driver p.16 std-family restoration, or
  Figma/export work (GAP-010).
- Promoting an ADR 0014 scientific claim from HTTP success.
- Duplicating GET `/v1/analysis-runs/{run_id}`, POST running/terminal, POST
  cancel, GET collection, POST retry, GET stored-request, GET retry-lineage,
  lookup GET, retry-parent GET, wait CLI, or adding GET to `NaruonLiveService`.
- Returning JSON-LD or GraphML envelopes on this identity route.

## Alternatives considered

1. **Ask operators to re-POST authorization** — rejected because a 200
   decision is not an addressable identity and conflicting retries fail closed.
2. **Return `tepp.scientific_acceptance.v1` or JSON-LD on succeeded retrieval**
   — rejected because retrieval bodies must stay metric-free and GAP-010
   visual/export envelopes remain later work.
3. **Add GET to `NaruonLiveService`** — rejected because that listener stays
   POST-only except existing Naruon-only analysis-run inspects on other stacks.
4. **Metric-free export retrieval GET on `AnalysisRunLiveService`** — accepted.

## Consequences

- Operators can resolve a 200 export authorization to a durable `export_id`
  without scanning artifacts.
- Retrieval pages cannot be mistaken for a succeeded scientific-acceptance
  result or a JSON-LD/GraphML envelope.
- GET-by-id analysis-run status may later return a digest-bound artifact
  without changing these retrieval gates.

## Failure and recovery

Unknown identities, extra path segments, collection GET, query strings,
nonempty GET bodies, metric keys, LineageWeave, unpublished consumers,
consumer mismatch, and non-loopback hosts return a redacted `400` envelope.
Oversized identities return `413`. Credential headers remain `403`. The
in-memory registry is not durable; a restart requires re-POSTing the original
metric-free authorization. Callers must not fabricate a succeeded
scientific-acceptance artifact from a retrieval payload.

## Security, privacy, scientific-integrity, and governance impact

- No credential headers cross the consumer boundary.
- Export retrieval remains loopback-only, size-bounded, naruon-scoped, and
  content-redacting.
- HTTP `200` on a retrieval payload is not measurement evidence and is not
  release evidence.
- Tenant, principal, and source-text flags stay off the retrieval body so
  retrieval cannot become a PII oracle.

## Compatibility and migration

`NaruonLiveService` POST `/v1/exports` still returns an authorization
decision without an `export_id`. Create POST, temporal-context, and
project-history paths are unchanged. GET-by-id remains refused on this slice.
Production adapters may replace loopback while preserving metric-free
retrieval fields and the artifact refusal.

## Verification

Falsifiable evidence:

- GET retrieval JSON has no RMSE/bias/coverage/SE-gate/scientific-acceptance/
  `terminal_result`/`tenant_workspace_id`/`principal_id`/`includes_source_text`
  keys;
- POST then GET of the minted `export_id` returns the matching `artifact_id`;
- GET does not leak another consumer's export;
- collection GET, GET-by-id, query strings, nonempty bodies, LineageWeave,
  unknown identities, and POST `/v1/exports/{export_id}` fail closed;
- `NaruonLiveService` still refuses GET;
- Clippy `-D warnings`, `tepp_api` tests, rustdoc, and exact-head review
  remain required.

## Rollback and supersession

Rollback removes export retrieval GET dispatch and the `AnalysisRunLiveService`
export POST mint; `NaruonLiveService` POST authorization remains valid. A
superseding ADR is required to persist the registry, bind a public address,
emit scientific-acceptance or JSON-LD on retrieval, add GET to
`NaruonLiveService`, or treat HTTP success as an ADR 0014 claim.

## Related authority

- ADR 0009 owns purpose-bound disclosure without blanket masking.
- ADR 0011 owns standalone/CWL MSA service authority.
- ADR 0018 owns consumer-scoped ingress and metric-free receipts.
- ADR 0027 owns GET-by-id status (live on another PR).
