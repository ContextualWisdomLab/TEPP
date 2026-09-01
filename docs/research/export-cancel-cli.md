# Export cancel CLI (doctoring)

## Scope

`tepp-export-cancel cancel` is the operator-visible loopback CLI that mints a
typed naruon `POST /v1/exports/{export_id}/cancel` onto spawned
`tepp-loopback` TCP. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal
of unpublished consumers, nonempty leftover stdin, present
`idempotency-key`, slash/NUL identities, review/Copilot/GitHub credential
flags, public bind, `localhost`, `http` origin, and scientific-authority
promotion is repository contract authority (ADR 0078; ADR 0077; ADR 0014),
not an RFC inference rule.

Stdout is metric-free with `cancelled=true`. Tenant, principal, source text,
and `tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a
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

- `docs/adr/0078-export-cancel-cli.md` — this CLI
- `docs/adr/0077-export-cancel-http.md` — cancel HTTP
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/export_cancel_cli_contract.rs` — fail-closed CLI
  proofs

## Verification

- `tepp-export-cancel cancel` of an authorized naruon export returns
  metric-free `cancelled=true` without RMSE/bias/coverage/SE-gate keys,
  tenant, principal, source text, or `tepp.scientific_acceptance.v1`;
- LineageWeave, nonempty leftover stdin, present `idempotency-key`, slash/NUL
  identities, public bind, `localhost`, and `http` origin fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not implement GAP-010 Figma/export, analysis-run cancel CLI,
interpretation-run cancel CLI, persistence, production TLS, Leiden
consensus, provider execution, causal inference, or an ADR 0014 scientific
claim-promotion package.
