# Interpretation-run collection GET (doctoring)

## Scope

`GET /v1/interpretation-runs` is the operator-visible collection of accepted
hypothetical interpretation runs on `OrchestratorLiveService` /
`tepp-orchestrator-loopback`. HTTP method, path, and header semantics follow
current HTTP semantics (Fielding, Nottingham, & Reschke, 2022). Fail-closed
refusal of unpublished consumers, nonempty GET bodies, present
`idempotency-key`, extra path segments, review/Copilot/GitHub credential
flags, and scientific-authority promotion is repository contract authority
(ADR 0069; ADR 0010; ADR 0011; ADR 0014), not an RFC inference rule.

Collection JSON is metric-free. `claim_status` remains `hypothetical`.
`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. A 200 collection page is not a completed psychometric result,
calibrated score, theta estimate, uncertainty statement, causal inference, or
scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto a bounded, hypothetical interpretation-run
collection. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0069-interpretation-run-collection-get.md` — this collection
- `docs/adr/0064-interpretation-run-cli.md` — create CLI
- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/orchestrator_live/tests/interpretation_run_collection_http_contract.rs`
  — fail-closed collection proofs
- `crates/orchestrator_live/tests/live_http_contract.rs` — loopback GET proofs
- `docs/adr/0070-interpretation-run-collection-cli.md` — collection CLI

## Verification

- `GET /v1/interpretation-runs` of accepted contextual-orchestrator runs
  returns `hypothetical` rows without RMSE/bias/coverage/SE-gate keys,
  `evidence_span_ids`, `causal_score`, or `tepp.scientific_acceptance.v1`;
- GET extra segments, naruon or LineageWeave consumer, nonempty body, present
  `idempotency-key`, and unknown verbs fail closed.

## Non-claims

This slice does not implement interpretation-run collection CLI, GET-by-id,
export CLI, analysis-run collection GET, project-history collection GET, wait
CLI, lookup CLI, persistence, production TLS, Leiden consensus, GAP-010
Figma/export, causal inference, or an ADR 0014 scientific claim-promotion
package.
