# Export idempotency-key lookup CLI (doctoring)

## Scope

Operators who receive a 200 purpose-bound export authorization still cannot
mint `GET /v1/exports/by-idempotency/{idempotency_key}` without writing raw
HTTP. `tepp-export-lookup lookup` is the first published binary that mints
that typed naruon exchange onto spawned `tepp-loopback` TCP. HTTP method,
path, `Host`, and `Transfer-Encoding` semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of table-access
URLs, review/Copilot/NIM/proxy credential headers, metric keys, LineageWeave
on this naruon-owned adapter, reserved prefix-as-key, leftover stdin, and
non-loopback binds is repository contract authority, not an RFC inference
rule.

The live listener is loopback HTTP/1.1 with an installed read/write deadline.
It is not a production TLS/`$PORT` service. Persistence remains GAP-003B.
JSON-LD/GraphML envelopes, Figma views, and GAP-010 visual export workflows
remain later work. `NaruonLiveService` stays POST-only.

## Internal contract evidence

- ADR 0094 owns this lookup CLI.
- ADR 0093 owns lookup GET.
- ADR 0054 owns retrieval GET-by-id.
- ADR 0009 owns purpose-bound disclosure without blanket masking.
- ADR 0011 owns the standalone/CWL MSA boundary.
- `docs/API_CONTRACT.md` names `GET /v1/exports/by-idempotency/{idempotency_key}`
  as the target lookup shape.

## Non-goals

lookup GET (#465), GET-by-id (#411), retrieval CLI (#417), collection GET/CLI
(#443/#444), stored-request GET/CLI (#457/#459), export-authorize CLI (#410),
analysis-run lookup CLI (#401), cancel lineages (closed), Leiden, Driver p.16
std-family restoration, Figma/export (GAP-010), and Compose persistence
(GAP-003B).
