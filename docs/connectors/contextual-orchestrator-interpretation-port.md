# contextual-orchestrator interpretation port for TEPP

**Status:** Accepted-target modular integration contract  
**Last reviewed:** 2026-08-13

## Boundary

TEPP may call a provider-neutral interpretation/orchestration port for semantic unitization, blinded model review, and evidence-bounded interpretation. `contextual-orchestrator` does **not** own:

- TEPP statistical truth or recovery metrics;
- source evidence identities or digests;
- model registry / scientific acceptance gates;
- merge, release, or independent review authority (ADR 0010; ADR 0015).

LLM/provider settings are execution policy only. Deterministic scientific gates remain authoritative (AGENTS.md §11).

## Allowed orchestration modes

TEPP may allocate test-time computation between:

1. direct model routing with bounded reasoning effort;
2. deeper multi-agent workflows with recorded workflow depth, decomposition, access lists, recursion, role-specific reasoning effort, verification/adjudication, and comparable-budget ablations.

These allocations are guided by Fugu, Conductor, and TRINITY research cited in `docs/research/standards-and-literature.md` and `docs/LLM_ORCHESTRATION.md`.

## Credential separation

- Live LLM tests and product-dev loops use `NVIDIA_NIM_API_KEY` only.
- `COPILOT_GITHUB_TOKEN` is prohibited.
- Existing independent review-agent credentials must not be repurposed for product development or interpretation traffic.

## Wire interchange

`tepp_api::orchestrator_interpretation_exchange` builds a credential-free `POST https://<host>/v1/interpretation-runs`. The host must be a DNS name; `postgres`, `jdbc`, `sql`, and `tables` hosts are refused. `refuse_repository_write_secret` accepts only `NVIDIA_NIM_API_KEY` as a model-credential name. `refuse_orchestrator_as_scientific_acceptance` always denies treating orchestrator output as statistical truth.

## Failure modes

- missing provider key → fail closed without fallback to repository-write tokens;
- untrusted model output → never becomes source of scientific acceptance;
- resource exhaustion → explicit bounded failure/defer, not silent degradation.

## Authority sources

Sakana AI. (2026). *Fugu technical report* (as cited in TEPP research register).

Additional Conductor / TRINITY and ISO/NIST orchestration governance mappings remain in `docs/research/standards-and-literature.md` and `docs/LLM_ORCHESTRATION.md`.
