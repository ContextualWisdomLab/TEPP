# Export collection GET (doctoring)

## Scope

`GET /v1/exports` is the operator-visible enumeration of authorized
purpose-bound export identities on `AnalysisRunLiveService` /
`tepp-loopback`. HTTP method, path, and header semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
unpublished consumers, nonempty GET bodies, present `idempotency-key`, extra
path segments, review/Copilot/GitHub credential flags, and
scientific-authority promotion is repository contract authority (ADR 0075;
ADR 0054; ADR 0014), not an RFC inference rule.

Collection JSON is metric-free. Tenant, principal, source text, and
`tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a completed
psychometric result, calibrated score, theta estimate, uncertainty statement,
causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving a current
representation of the target resource. TEPP maps that retrieval onto a
bounded, in-memory page of metric-free export identities. The RFC does not
define psychometric acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0075-export-collection-get.md` — this collection
- `docs/adr/0054-export-retrieval-get.md` — GET-by-id
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/export_collection_http_contract.rs` — fail-closed
  collection proofs

## Verification

- `GET /v1/exports` of authorized naruon exports returns metric-free
  identities without RMSE/bias/coverage/SE-gate keys, tenant, principal,
  source text, or `tepp.scientific_acceptance.v1`;
- LineageWeave, nonempty body, present `idempotency-key`, extra path
  segments, slash/NUL cursors fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not implement a collection CLI, GAP-010 Figma/export,
analysis-run collection GET, interpretation-run collection GET, persistence,
production TLS, Leiden consensus, provider execution, causal inference, or an
ADR 0014 scientific claim-promotion package.
