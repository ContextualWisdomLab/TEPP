# Temporal-context collection CLI (doctoring)

## Scope

`tepp-temporal-contexts list` is the operator-visible loopback CLI that mints a
typed LineageWeave `GET /v1/temporal-context` onto spawned `tepp-loopback`
TCP. HTTP method, path, and header semantics follow current HTTP semantics
(Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of unpublished
consumers, nonempty leftover stdin, `localhost`, `http` origin, credential
flags, public bind, and scientific-authority promotion is repository contract
authority (ADR 0082; ADR 0081; ADR 0014), not an RFC inference rule.

Stdout is metric-free with `inference_status=temporal_association_only`. Event
labels, actor lists, timeline events, evidence text, findings, and
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

- `docs/adr/0082-temporal-context-collection-cli.md` — this CLI
- `docs/adr/0081-temporal-context-collection-get.md` — collection GET
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/temporal_context_collection_cli_contract.rs` —
  fail-closed CLI proofs

## Verification

- `tepp-temporal-contexts list` of accepted LineageWeave identities returns
  metric-free rows without RMSE/bias/coverage/SE-gate keys, event labels,
  actor lists, evidence text, findings, causal scores, or
  `tepp.scientific_acceptance.v1`;
- naruon, nonempty leftover stdin, `localhost`, `http` origin, and public bind
  fail closed;
- `NaruonLiveService` still refuses GET.

## Non-claims

This slice does not implement GAP-010 Figma/export, temporal-context POST CLI,
project-history collection CLI, persistence, production TLS, Leiden consensus,
provider execution, causal inference, or an ADR 0014 scientific
claim-promotion package.
