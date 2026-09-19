# Export idempotency-key lookup HTTP doctoring

`GET /v1/exports/by-idempotency/{idempotency_key}` resolves a purpose-bound export authorization key to the metric-free server-assigned export identity on `AnalysisRunLiveService`. HTTP method/path/framing semantics follow RFC 9110 (Fielding, Nottingham, & Reschke, 2022); the product-specific authorization, privacy and scientific-authority rules are TEPP contracts.

The important compatibility invariant is that lookup does not narrow the idempotency-key domain already admitted by export authorization. Keys are opaque request identity. A key containing `/` is percent-encoded into one route segment, with segmentation performed before percent decoding. A key whose literal value is `by-idempotency` remains data at `/v1/exports/by-idempotency/by-idempotency`. Raw extra path segments, NUL and oversized values still fail closed. This avoids accepting an export and later making its receipt impossible to resolve.

The returned `ExportIdempotencyLookup` contains only `export_id`, `decision_code` and the exact decoded `idempotency_key`. Tenant/workspace, principal, source-text and scientific metric/acceptance fields are refused recursively. Zero and ambiguous matches fail closed rather than becoming a tenant-count oracle. LineageWeave is outside this Naruon-owned adapter and `NaruonLiveService` stays POST-only.

The related stored-request-by-idempotency route is a different disclosure boundary. ADR 0099 quarantines that convenience path because the first version had no authenticated tenant/principal binding. Successful metric-free identity lookup is therefore not authorization to retrieve the original request.

The listener remains loopback HTTP/1.1 with bounded framing and deadlines; it is not a production public TLS service. Persistence remains GAP-003B. Exact-head tests cover slash and route-prefix-looking keys through POST→lookup round trips as well as fail-closed privacy and framing cases.
