# Project-history cancel CLI (doctoring)

## Scope

`tepp-project-history-cancel cancel` is the operator-visible loopback CLI
that mints a typed LineageWeave
`POST /v1/project-histories/{idempotency_key}/cancel` onto spawned
`tepp-loopback` TCP. HTTP method, path, and header semantics follow current
HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal
of unpublished consumers, nonempty leftover stdin, `localhost`, `http`
origin, slash/NUL identities, credential flags, public bind, and
scientific-authority promotion is repository contract authority (ADR 0080;
ADR 0079; ADR 0014), not an RFC inference rule.

Stdout is metric-free with `cancelled=true` and
`inference_status=temporal_association_only`. Evidence text, findings, actor
lists, and `tepp.scientific_acceptance.v1` never appear. HTTP 200 is not a
completed psychometric result, calibrated score, theta estimate, uncertainty
statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.3 describes POST as a method for processing according to the
resource's own semantics. TEPP maps that processing onto in-memory removal of
one metric-free project-history identity. The RFC does not define
psychometric acceptance, RMSE, causality, or claim promotion.

### Internal contract evidence

- `docs/adr/0080-project-history-cancel-cli.md` — this CLI
- `docs/adr/0079-project-history-cancel-http.md` — cancel HTTP
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/project_history_cancel_cli_contract.rs` — fail-closed
  CLI proofs

## Verification

- `tepp-project-history-cancel cancel` of an accepted LineageWeave
  project-history returns metric-free `cancelled=true` without
  RMSE/bias/coverage/SE-gate keys, evidence text, findings, causal scores, or
  `tepp.scientific_acceptance.v1`;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, slash/NUL
  identities, and public bind fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not implement GAP-010 Figma/export, analysis-run cancel CLI,
interpretation-run cancel CLI, export cancel CLI, persistence, production
TLS, Leiden consensus, provider execution, causal inference, or an ADR 0014
scientific claim-promotion package.
