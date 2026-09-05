# Temporal-context retrieval CLI (doctoring)

## Scope

`tepp-temporal-context-get get` is the operator-visible loopback CLI that mints
a typed LineageWeave `GET /v1/temporal-context/{idempotency_key}` onto spawned
`tepp-loopback` TCP. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
unpublished consumers, nonempty leftover stdin, `localhost`, `http` origin,
slash/NUL identities, credential flags, public bind, and scientific-authority
promotion is repository contract authority (ADR 0084; ADR 0083; ADR 0014), not
an RFC inference rule.

Stdout is metric-free with `inference_status=temporal_association_only`. Event
labels, actor lists, timeline events, evidence text, findings, and
`tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a completed
psychometric result, calibrated score, theta estimate, uncertainty statement,
causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0084-temporal-context-retrieval-cli.md` — this CLI
- `docs/adr/0083-temporal-context-retrieval-get.md` — GET-by-id HTTP
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/temporal_context_retrieval_cli_contract.rs`

## Verification

- `tepp-temporal-context-get get` of an accepted identity returns a metric-free
  row without RMSE, event labels, actor lists, or
  `tepp.scientific_acceptance.v1`;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, slash/NUL, and
  public bind fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not re-open collection GET (#449), cancel lineages, GAP-010
Figma/export, persistence, production TLS, Leiden consensus, causal inference,
or an ADR 0014 scientific claim-promotion package.
