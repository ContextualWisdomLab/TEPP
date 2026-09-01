# Export idempotency-key stored-request lookup security doctoring

The first `GET /v1/exports/by-idempotency/{idempotency_key}/request`
implementation searched the whole Naruon consumer namespace and returned the
stored export-authorization request. The registry itself is tenant-aware, but
the GET carried no authenticated tenant/workspace or principal scope. Knowledge
of a unique idempotency key could therefore disclose another tenant's
`tenant_workspace_id`, `principal_id`, and artifact request metadata.

ADR 0099 now quarantines the route. A syntactically valid client exchange is
denied until a versioned tenant-and-principal authorization context exists, and
the live response guard rejects serialized tenant/principal identity. Raw and
percent-decoded slash are both refused so intermediaries cannot normalize one
opaque key into a different path interpretation. The metric-free identity lookup
from ADR 0093 remains separate and does not gain stored-request authority.

The security rule is deliberately stronger than key opacity: an idempotency key
identifies a replay domain; it is not a bearer authorization credential. A
future reactivation must prove cross-tenant and cross-principal isolation with
same-key regression cases and exact-head coverage/security evidence. Merely
adding caller-controlled scope headers without an authenticated authority would
not repair the boundary.

HTTP semantics remain aligned with RFC 9110 (Fielding, Nottingham, & Reschke,
2022). `tepp.scientific_acceptance.v1` is unrelated to this transport repair and
never appears. `NaruonLiveService` remains POST-only and LineageWeave remains
outside this Naruon-owned adapter.
