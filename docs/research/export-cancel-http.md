# Export cancel HTTP (doctoring)

## Scope

`POST /v1/exports/{export_id}/cancel` on `AnalysisRunLiveService` /
`tepp-loopback` removes one authorized naruon export identity. HTTP method,
path, and header semantics follow current HTTP semantics (Fielding,
Nottingham, & Reschke, 2022). Fail-closed refusal of unpublished consumers,
nonempty leftover body, present `idempotency-key`, extra path segments,
slash/NUL identities, credential headers, public bind, and
scientific-authority promotion is repository contract authority (ADR 0077;
ADR 0075; ADR 0014), not an RFC inference rule.

The receipt is metric-free with `cancelled=true`. Tenant, principal, source
text, and `tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a
completed psychometric result, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing according to the
resource's own semantics. TEPP maps that processing onto in-memory removal of
one metric-free export identity. The RFC does not define psychometric
acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0077-export-cancel-http.md` — this cancel route
- `docs/adr/0075-export-collection-get.md` — collection GET
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/export_cancel_http_contract.rs` — fail-closed proofs

## Verification

- cancel of an authorized naruon export returns metric-free `cancelled=true`
  without RMSE/bias/coverage/SE-gate keys, tenant, principal, source text, or
  `tepp.scientific_acceptance.v1`;
- subsequent GET-by-id and collection GET omit the identity;
- LineageWeave, nonempty leftover body, present `idempotency-key`, extra path
  segments, slash/NUL identities, and `http` origin fail closed;
- `NaruonLiveService` still refuses GET and does not serve this cancel path
  as a 200.

## Non-claims

This slice does not implement GAP-010 Figma/export, analysis-run cancel,
interpretation-run cancel, persistence, production TLS, Leiden consensus,
provider execution, causal inference, or an ADR 0014 scientific
claim-promotion package.
