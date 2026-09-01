# Project-history GET-by-id CLI (doctoring)

## Scope

`tepp-project-history-get get` is the operator-visible client of loopback
`GET /v1/project-histories/{idempotency_key}` on `AnalysisRunLiveService` /
`tepp-loopback`. HTTP method, path, and header semantics follow current HTTP
semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed refusal of
unpublished consumers, nonempty GET bodies, review/Copilot/GitHub credential
flags, and scientific-authority promotion is repository contract authority
(ADR 0067; ADR 0066; ADR 0028; ADR 0021; ADR 0011; ADR 0014), not an RFC
inference rule.

Stdout is the stored `ProjectHistoryProjection`. `inference_status` remains
`temporal_association_only`. `tepp.scientific_acceptance.v1` never appears.
Process exit 0 is not a completed temporal model, calibrated score, theta
estimate, uncertainty statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto one bounded, cutoff-safe project-history
projection. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0067-project-history-retrieval-cli.md` — this CLI
- `docs/adr/0066-project-history-retrieval-get.md` — GET-by-id HTTP
- `docs/adr/0028-project-history-collection-get.md` — collection GET
- `docs/adr/0021-lineageweave-project-history-boundary.md` — POST boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — process
  exit 0 is not a scientific claim
- `crates/tepp_api/tests/project_history_retrieval_cli_contract.rs` —
  fail-closed CLI proofs

## Verification

- `tepp-project-history-get get` of an accepted LineageWeave projection
  returns `temporal_association_only` without RMSE/bias/coverage/SE-gate
  keys, `causal_score`, or `tepp.scientific_acceptance.v1`;
- naruon consumer, nonempty stdin, pagination flags, public bind,
  `localhost`, and credential flags fail closed;
- `NaruonLiveService` refuses the composed GET.

## Non-claims

This slice does not implement collection CLI, POST CLI, persistence,
production TLS, Leiden consensus, or an ADR 0014 scientific claim-promotion
package.
