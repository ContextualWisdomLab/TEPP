# Temporal-context collection GET (doctoring)

## Scope

`GET /v1/temporal-context` is the operator-visible loopback collection that
enumerates accepted LineageWeave temporal-context identities on
`AnalysisRunLiveService` / `tepp-loopback`. HTTP method, path, and header
semantics follow current HTTP semantics (Fielding, Nottingham, & Reschke,
2022). Fail-closed refusal of unpublished consumers, nonempty leftover bodies,
present `idempotency-key` on GET, slash/NUL identities, credential flags,
public bind, and scientific-authority promotion is repository contract
authority (ADR 0081; ADR 0002; ADR 0014), not an RFC inference rule.

Collection rows are metric-free with `inference_status=temporal_association_only`.
Event labels, actor lists, timeline events, evidence text, findings, and
`tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a completed
psychometric result, calibrated score, theta estimate, uncertainty statement,
causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a current
representation. TEPP maps that retrieval onto an in-memory page of metric-free
temporal-context identities. The RFC does not define psychometric acceptance,
RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0081-temporal-context-collection-get.md` — this GET
- `docs/adr/0002-six-clock-temporal-semantics.md` — cutoff-safe association
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/temporal_context_collection_http_contract.rs` —
  fail-closed collection proofs

## Verification

- `GET /v1/temporal-context` of accepted LineageWeave identities returns
  metric-free rows without RMSE/bias/coverage/SE-gate keys, event labels,
  actor lists, evidence text, findings, causal scores, or
  `tepp.scientific_acceptance.v1`;
- naruon, nonempty leftover body, GET `idempotency-key`, slash/NUL identities,
  and public bind fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not implement GAP-010 Figma/export, temporal-context CLI,
project-history collection GET, persistence, production TLS, Leiden consensus,
provider execution, causal inference, or an ADR 0014 scientific
claim-promotion package.
