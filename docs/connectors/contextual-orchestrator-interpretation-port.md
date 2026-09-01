# contextual-orchestrator interpretation port for TEPP

**Status:** Partial modular integration contract — `tepp_api::bind_contextual_orchestrator` is the credential-free binding and `service_tls::authorize_orchestrator_live_port` gates production live ports; the `orchestrator_live` loopback HTTP/1.1 interpretation listener is on the active PR; production TLS remains accepted-target.  
**Last reviewed:** 2026-08-16

## Boundary

TEPP may call a provider-neutral interpretation/orchestration port for semantic unitization, blinded model review, and evidence-bounded interpretation. `contextual-orchestrator` does **not** own:

- TEPP statistical truth or recovery metrics;
- source evidence identities or digests;
- model registry / scientific acceptance gates;
- merge, release, or independent review authority (ADR 0010; ADR 0015).

LLM/provider settings are execution policy only. Deterministic scientific gates remain authoritative (AGENTS.md §11). A production live bind uses `service_tls::authorize_orchestrator_live_port` and cannot be loopback plaintext. This document does not claim a deployed TLS listener.

`orchestrator_live::OrchestratorLiveService` binds loopback TCP and serves
`POST /v1/interpretation-runs` plus `GET /v1/interpretation-runs`. Accepted
output is always hypothetical and never scientific authority. Collection GET
returns metric-free identities only. Non-loopback binds, table-access hosts, and
review/Copilot/GitHub credential headers fail closed. The listener does not
call a model provider.

## Allowed orchestration modes

`tepp_api::route_orchestration` selects `direct`, `verify`, `committee`, `conductor`, or `abstain` from TEPP-owned risk, ambiguity, evidence-sufficiency, and token-budget scores. `tepp_api::bind_contextual_orchestrator` may be called only for a non-abstaining plan and never includes credentials or raw source. TEPP may allocate test-time computation between:

1. direct model routing with bounded reasoning effort;
2. deeper multi-agent workflows with recorded workflow depth, decomposition, access lists, recursion, role-specific reasoning effort, verification/adjudication, and comparable-budget ablations.

These allocations are guided by Fugu, Conductor, and TRINITY research cited in `docs/research/standards-and-literature.md` and `docs/LLM_ORCHESTRATION.md`. Documents cannot change policy, access lists, or credentials.

## Credential separation

- Live LLM tests and product-dev loops use `NVIDIA_NIM_API_KEY` only.
- `COPILOT_GITHUB_TOKEN` is prohibited.
- Existing independent review-agent credentials must not be repurposed for product development or interpretation traffic.

## Failure modes

- missing provider key → fail closed without fallback to repository-write tokens;
- untrusted model output → never becomes source of scientific acceptance;
- resource exhaustion → explicit bounded failure/defer, not silent degradation.

## Authority sources

Sakana AI. (2026). *Fugu technical report* (as cited in TEPP research register).

Additional Conductor / TRINITY and ISO/NIST orchestration governance mappings remain in `docs/research/standards-and-literature.md` and `docs/LLM_ORCHESTRATION.md`.
