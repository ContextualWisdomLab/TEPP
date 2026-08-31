# Project-history GET-by-id (doctoring)

## Scope

`GET /v1/project-histories/{idempotency_key}` is the operator-visible retrieval
of one accepted cutoff-safe project-history projection on
`AnalysisRunLiveService` / `tepp-loopback`. HTTP method, path, and header
semantics follow current HTTP semantics (Fielding, Nottingham, & Reschke,
2022). Fail-closed refusal of unpublished consumers, nonempty GET bodies,
review/Copilot/GitHub credential flags, and scientific-authority promotion is
repository contract authority (ADR 0066; ADR 0028; ADR 0021; ADR 0011;
ADR 0014), not an RFC inference rule.

The response is the stored `ProjectHistoryProjection`. `inference_status`
remains `temporal_association_only`. `tepp.scientific_acceptance.v1` never
appears. A 200 retrieval is not a completed temporal model, calibrated score,
theta estimate, uncertainty statement, causal inference, or scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto one bounded, cutoff-safe project-history
projection. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0066-project-history-retrieval-get.md` — this retrieval
- `docs/adr/0028-project-history-collection-get.md` — collection GET
- `docs/adr/0021-lineageweave-project-history-boundary.md` — POST boundary
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/tepp_api/tests/project_history_retrieval_http_contract.rs` —
  fail-closed retrieval proofs

## Verification

- `GET /v1/project-histories/{idempotency_key}` of an accepted LineageWeave
  projection returns `temporal_association_only` without RMSE/bias/coverage/
  SE-gate keys, `causal_score`, or `tepp.scientific_acceptance.v1`;
- collection GET remains metric-free identities;
- naruon consumer, nonempty body, extra segments, and unknown keys fail closed.

## Non-claims

This slice does not implement collection CLI, retrieval CLI, GET-by-id for
analysis runs, persistence, production TLS, Leiden consensus, or an ADR 0014
scientific claim-promotion package.
