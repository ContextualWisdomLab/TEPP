# ADR 0010 — Adaptive LLM orchestration and test-time compute

**Decision status:** Accepted
**Implementation maturity:** partial — `tepp_api` governed router, comparable-budget ablation record, credential-free contextual-orchestrator binding are implemented-main; evidence-bounded `interpretation_gateway` is on the active PR and is not implemented-main until exact-head checks, review, and protected-main integration complete; live NIM execution, learned conductor calibration, and production ablation evidence remain accepted-target
**Date:** 2026-08-10
**Supersedes:** The LLM orchestration-selection/ablation clauses previously co-located in ADR 0006. ADR 0006 remains authoritative for GPU/VRAM and model-credential separation; ADR 0015 governs autonomous repository-write/review/merge authority.

## Context

TEPP uses LLMs for bounded semantic unitization, blinded model review, evidence-grounded interpretation, and claim verification. The workload ranges from low-ambiguity schema-constrained classification to complex multilingual evidence synthesis. A fixed single-model policy wastes reasoning budget on simple cases, while a fixed deep multi-agent graph can add cost, disagreement, and failure surface without improving scientific validity.

Recent [TRINITY (ORCH-TRINITY-2026)](../research/standards-and-literature.md#orch-trinity-2026), [Conductor (ORCH-CONDUCTOR-2026)](../research/standards-and-literature.md#orch-conductor-2026), and [Fugu (ORCH-FUGU-2026)](../research/standards-and-literature.md#orch-fugu-2026) work motivates adaptive delegation, role assignment, communication topology, and recursive/test-time compute scaling. These results are evidence for experimentation, not authority to replace TEPP's deterministic/statistical gates.

## Decision

TEPP supports versioned orchestration modes `direct`, `verify`, `committee`, `conductor`, and `abstain`. A governed router chooses a mode using task risk, ambiguity, evidence sufficiency, and an explicit compute budget. Workflow stage count, decomposition, recursion depth, access lists, model/provider pool, role assignment, role-specific reasoning effort, verification policy, stopping rule, and total budget are first-class experimental/configuration variables.

Statistical estimation, temporal eligibility, event-relation validity, measurement invariance, numerical acceptance, and release authority remain outside LLM authority. LLM output is always an untrusted proposal tied to evidence and a reproducibility record.

## Alternatives considered

1. **Always use one strongest model** — simple, but cannot exploit cheap/direct work or independent verification and provides a weak research basis for test-time scaling.
2. **Always use a fixed multi-agent workflow** — rejected because unnecessary depth can increase cost and correlated failure.
3. **Adaptive, evidence-bounded orchestration with comparable-budget ablation** — accepted.

## Consequences

- Direct single-model performance is a required baseline for every material orchestration claim.
- Multi-agent/deeper routing must demonstrate measurable benefit at reported/comparable budgets.
- Role-specific reasoning effort and topology are versioned and observable.
- Provider/model/prompt/policy/evidence/access-list identities are bound into run evidence.
- High disagreement or insufficient evidence may cause abstention or scientific/human escalation.
- `contextual-orchestrator` is the preferred CWL execution integration when available, but TEPP retains scientific and evidence authority.

## Security and privacy

Documents cannot change orchestration policy, allowed tools, provider credentials, access lists, or role authority. Calls use evidence-minimized payloads and raw secrets are never exposed to models. Live scientific/model tests use `NVIDIA_NIM_API_KEY`; the separate hourly proposal gateway exception is governed by ADR 0017 and uses its own all-provider bootstrap. `COPILOT_GITHUB_TOKEN` is prohibited. Review-agent identities and credentials remain separate.

## Compatibility and migration

Orchestration policies, model pools, role definitions, prompts, access lists, and budget/stopping rules are versioned. A provider/model replacement does not silently inherit prior calibration or quality claims. Integration with `contextual-orchestrator` remains optional under ADR 0011 and cannot change TEPP estimands.

## Verification

Before production claims, compare at least a direct baseline, direct+verifier, fixed role-based workflow, and adaptive conductor-style workflow where available. Vary at least two reasoning-effort/budget settings and report accuracy/F1, unsupported-claim rate, evidence support, calibration, disagreement, abstention quality, language slices, injection resilience, repeated-run variance, tokens/calls/cost, and provider failure behavior.

## Rollback

Every accepted orchestrated path has a bounded fallback: approved direct model, deterministic function, deferred/unresolved state, or explicit abstention. A rollback never changes the underlying estimand or promotes an unsupported interpretation.

## Supersession

Supersede when primary evidence and TEPP validation justify a materially different test-time-compute allocation mechanism while retaining deterministic scientific authority and reproducible evidence.
