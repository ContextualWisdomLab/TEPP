# Interpretation-run GET-by-id (doctoring)

## Scope

`GET /v1/interpretation-runs/{idempotency_key}` is the operator-visible
retrieval of one accepted hypothetical interpretation-run identity on
`OrchestratorLiveService` / `tepp-orchestrator-loopback`. HTTP method, path,
and header semantics follow current HTTP semantics (Fielding, Nottingham, &
Reschke, 2022). Fail-closed refusal of unpublished consumers, nonempty GET
bodies, present `idempotency-key`, extra extra-segments, pagination headers,
review/Copilot/GitHub credential flags, and scientific-authority promotion is
repository contract authority (ADR 0071; ADR 0069; ADR 0010; ADR 0011;
ADR 0014), not an RFC inference rule.

Retrieval JSON is metric-free. `claim_status` remains `hypothetical`.
`scientific_authority` remains false. `tepp.scientific_acceptance.v1` never
appears. A 200 retrieval body is not a completed psychometric result,
calibrated score, theta estimate, uncertainty statement, causal inference, or
scientific claim.

## Authority

### External standards (HTTP only)

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics*
(RFC 9110). IETF. https://doi.org/10.17487/RFC9110

RFC 9110 §9.3.1 describes GET as a method for retrieving the target resource.
TEPP maps that retrieval onto a bounded, hypothetical interpretation-run
identity. The RFC does not define psychometric acceptance, RMSE, causality,
or claim promotion.

### Internal contract evidence

- `docs/adr/0071-interpretation-run-retrieval-get.md` — this retrieval
- `docs/adr/0069-interpretation-run-collection-get.md` — collection GET
- `docs/adr/0064-interpretation-run-cli.md` — create CLI
- `docs/adr/0010-adaptive-llm-orchestration.md` — mode vocabulary and
  scientific-authority separation
- `docs/adr/0011-standalone-modular-msa-boundary.md` — modular HTTP boundary
- `docs/adr/0014-scientific-claim-promotion-and-release-evidence.md` — HTTP
  200 is not a scientific claim
- `crates/orchestrator_live/tests/interpretation_run_retrieval_http_contract.rs`
  — fail-closed GET-by-id proofs
- `crates/orchestrator_live/tests/live_http_contract.rs` — loopback GET-by-id
  proofs

## Verification

- `GET /v1/interpretation-runs/{idempotency_key}` of an accepted
  contextual-orchestrator run returns `hypothetical` with
  `scientific_authority` false and without RMSE/bias/coverage/SE-gate keys,
  `evidence_span_ids`, `causal_score`, or `tepp.scientific_acceptance.v1`;
- extra extra-segments, naruon or LineageWeave, nonempty body, present
  `idempotency-key`, pagination headers, missing identity, slash/NUL keys fail
  closed.

## Non-claims

This slice does not implement a retrieval CLI, analysis-run GET-by-id, export
GET, project-history GET-by-id, persistence, production TLS, Leiden consensus,
GAP-010 Figma/export, provider execution, causal inference, or an ADR 0014
scientific claim-promotion package.
