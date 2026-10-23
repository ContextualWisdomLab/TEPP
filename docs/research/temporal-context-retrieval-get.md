# Temporal-context GET-by-id (doctoring)

## Scope

`GET /v1/temporal-context/{idempotency_key}` is the operator-visible loopback
retrieval of one accepted LineageWeave temporal-context identity on
`AnalysisRunLiveService` / `tepp-loopback`. HTTP method, path, and header
semantics follow current HTTP semantics (Fielding, Nottingham, & Reschke,
2022). Fail-closed refusal of unpublished consumers, collection path, extra
segments, slash/NUL identities, nonempty leftover bodies, credential flags,
public bind, and scientific-authority promotion is repository contract
authority (ADR 0083; ADR 0002; ADR 0014), not an RFC inference rule.

The retrieval is metric-free with `inference_status=temporal_association_only`.
Event labels, actor lists, timeline events, evidence text, findings, and
`tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a completed
psychometric result, calibrated score, theta estimate, uncertainty statement,
causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

### Internal contract evidence

- `docs/adr/0083-temporal-context-retrieval-get.md` — this GET-by-id
- `docs/adr/0002-six-clock-temporal-semantics.md` — cutoff-safe association
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/temporal_context_retrieval_http_contract.rs`

## Verification

- `GET /v1/temporal-context/{idempotency_key}` of an accepted identity returns
  a metric-free row without RMSE, event labels, actor lists, or
  `tepp.scientific_acceptance.v1`;
- naruon, collection path, extra segments, slash/NUL, and missing keys fail
  closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not re-open collection GET (#449), cancel lineages, GAP-010
Figma/export, persistence, production TLS, Leiden consensus, causal inference,
or an ADR 0014 scientific claim-promotion package.
