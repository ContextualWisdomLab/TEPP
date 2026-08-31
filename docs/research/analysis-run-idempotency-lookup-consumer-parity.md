# Analysis-run idempotency-lookup consumer parity (doctoring)

## Scope

`LineageWeave` and the Naruon compatibility listener must be able to resolve
a metric-free analysis-run identity from an idempotency key without inventing
a second DTO. HTTP method, path, and `Host` semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
non-loopback binds, table-access hosts, review/Copilot/GitHub credential
headers, and scientific-authority promotion is repository contract authority
(ADR 0011; ADR 0018; ADR 0037; ADR 0047), not an RFC inference rule.

This slice does not serve GET status, running/terminal POST, collection GET,
retry POST, retry-parent, retry-lineage, or persistence. `NaruonLiveService`
already keys idempotency replay, so accepted creates return a real `run_id`.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource's
current state. TEPP maps that retrieval onto a bounded, consumer-scoped
resolution of an idempotency key. The RFC does not define psychometric
acceptance, RMSE, or claim promotion.

### Internal contract evidence

- `docs/adr/0047-analysis-run-idempotency-lookup-consumer-parity.md`
- `docs/adr/0037-analysis-run-idempotency-lookup-get.md`
- `docs/adr/0018-consumer-scoped-analysis-run-ingress.md`
- `docs/adr/0011-standalone-modular-msa-boundary.md`

## Claim boundary

HTTP `200` lookup is not a completed temporal model, calibrated score, theta
estimate, uncertainty statement, or scientific claim.
`tepp.scientific_acceptance.v1` never appears.
