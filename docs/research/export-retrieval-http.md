# Export retrieval HTTP (doctoring)

## Scope

Operators who receive a 200 purpose-bound export authorization still cannot
address that export later. `POST /v1/exports` on `NaruonLiveService` returns a
decision without minting an `export_id`. `GET /v1/exports/{export_id}` on
`AnalysisRunLiveService` is the first executable retrieval route. HTTP method,
path, `Host`, and `Transfer-Encoding` semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of table-access
URLs, review/Copilot/NIM/proxy credential headers, metric keys, LineageWeave
on this naruon-owned adapter, and non-loopback binds is repository contract
authority, not an RFC inference rule.

The live listener is loopback HTTP/1.1 with an installed read/write deadline.
It is not a production TLS/`$PORT` service. Persistence remains GAP-003B.
JSON-LD/GraphML envelopes, Figma views, and GAP-010 visual export workflows
remain later work.

## Internal contract evidence

- ADR 0054 owns this retrieval GET.
- ADR 0009 owns purpose-bound disclosure without blanket masking.
- ADR 0011 owns the standalone/CWL MSA boundary.
- `docs/API_CONTRACT.md` names `GET /v1/exports/{export_id}` as the target
  retrieval shape.
- `docs/connectors/naruon-artifact-consumer.md` records naruon as the current
  purpose-bound export adapter.

## Non-goals

GET-by-id status (#359), lifecycle POST, cancel, collection GET, retry,
stored-request GET, retry-lineage GET, lookup GET, retry-parent GET, wait CLI,
Leiden, Driver p.16 std-family restoration, Figma/export (GAP-010), and
Compose persistence (GAP-003B).
