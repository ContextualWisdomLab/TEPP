# TEPP LLM Orchestration and Test-Time Compute Contract

**Status:** Partial — `tepp_api::route_orchestration` is the governed selector; live provider execution is not yet shipped.
**Last reviewed:** 2026-08-13

## 1. Purpose

TEPP uses LLMs only for bounded semantic unitization, candidate-model review, evidence-grounded interpretation, and independent claim verification. Statistical estimation, temporal eligibility, event-relation validity, measurement invariance, numerical acceptance, and release authority remain deterministic/Rust or governed human authority.

The product must allocate test-time compute adaptively rather than assume that either one frontier model or a large fixed multi-agent graph is always best.

## 2. Research basis

The orchestration design is informed by three 2026 Sakana AI research lines:

- **TRINITY** ([ORCH-TRINITY-2026](research/standards-and-literature.md#orch-trinity-2026)): a lightweight coordinator selects a model and Thinker/Worker/Verifier role over multiple turns, showing learned adaptive delegation under budget constraints.
- **Conductor** ([ORCH-CONDUCTOR-2026](research/standards-and-literature.md#orch-conductor-2026)): a learned coordinator generates communication topology and focused natural-language instructions for a heterogeneous model pool, including recursive test-time scaling.
- **Sakana Fugu** ([ORCH-FUGU-2026](research/standards-and-literature.md#orch-fugu-2026)): production-oriented query-adaptive scaffolds combining learned orchestration approaches and exposing the orchestration system behind a model-compatible API.

These results motivate experiments; they do not prove that deeper orchestration is always superior for TEPP workloads.

## 3. Orchestration modes

| Mode | Typical TEPP use | Default compute |
|---|---|---|
| direct | simple span classification, deterministic-schema fill, low-ambiguity label | one model call |
| verify | interpretation or classification with material unsupported-claim risk | producer + independent verifier |
| committee | K/model interpretation with scientific ambiguity | blinded parallel raters + adjudication |
| conductor | complex evidence synthesis or multi-stage semantic reasoning | adaptive roles/topology under explicit budget |
| abstain | provider/evidence/validation insufficient | no forced answer |

`tepp_api::route_orchestration` is the governed selector. It chooses the cheapest mode expected to satisfy the quality/risk profile, but latency is not the primary objective. Quality, evidence support, calibration, disagreement, controllability, and reproducibility dominate. The returned plan is a proposal: `scientific_authority_code` remains `deterministic_statistical_gates`.

## 4. Explicit experimental variables

Every orchestration benchmark records and can ablate:

- model/provider pool;
- direct versus multi-agent topology;
- workflow stage count;
- worker count;
- task decomposition granularity;
- recursion depth;
- allowed tool/access list;
- role assignment;
- per-role reasoning effort;
- total token/call/compute budget;
- verification/adjudication policy;
- stopping rule;
- provider failure/fallback behavior.

Comparisons must use approximately comparable budgets or report the budget difference explicitly.

## 5. Role-specific effort

Suggested target policy:

- semantic span extraction/classification: low-to-medium reasoning;
- concept merge/alignment review: medium reasoning;
- blinded K/model-selection review: high reasoning with independent judges;
- final narrative synthesis: medium/high reasoning;
- verifier/adversarial evidence check: high reasoning with source-only access;
- routine formatting or schema conversion: minimal reasoning.

The policy is empirically calibrated and versioned rather than hard-coded as a permanent truth.

## 6. Evidence and trust boundary

LLM calls receive only the minimum evidence bundle needed for the assigned role. Documents are untrusted observations and cannot alter orchestration policy, tools, credentials, model pool, access lists, or scientific gates.

Each call records:

```text
provider_id
model_id
model_revision_or_endpoint
prompt_template_hash
system_policy_hash
role_code
reasoning_effort_code
workflow_run_id
workflow_step_id
parent_step_ids
access_profile_id
evidence_manifest_hash
input_digest
output_digest
usage_record
duration_record
verdict_status
```

Raw credentials are never model-visible. Model outputs are proposals, not source facts.

## 7. Quality metrics

At minimum evaluate:

- task accuracy/F1 against human gold where available;
- unsupported-claim rate;
- evidence citation/span precision and recall;
- calibration/Brier score for confidence-bearing tasks;
- rater/inter-model agreement and disagreement resolution;
- language-specific performance;
- prompt-injection success rate;
- abstention quality;
- token/call/compute cost;
- provider/model failure resilience;
- repeated-run variance.

For model-selection review, statistical predictive/recovery/stability/invariance gates run before LLM judgment. LLM preference cannot rescue a statistically rejected candidate.

## 8. contextual-orchestrator boundary

`contextual-orchestrator` is the preferred CWL provider-neutral integration when available. TEPP owns the evidence bundle, statistical/model-selection policy, scientific acceptance, artifact provenance, and allowed role/access configuration. The orchestrator owns provider routing/orchestration execution within the supplied policy. Neither service reads the other's application database directly.

`orchestrator_live` exposes a loopback-only `POST /v1/interpretation-runs` listener that records the selected mode and budget. Listener output is hypothetical and cannot become scientific authority. Production TLS and provider execution remain outside this crate.

## 9. Development and live-test credentials

Live model tests use GitHub Secret `NVIDIA_NIM_API_KEY` with the minimum runtime mapping required by the selected adapter. `COPILOT_GITHUB_TOKEN` is prohibited. Existing independent review-agent credentials are separate and must not be renamed, copied, or repurposed for TEPP model execution.

## 10. Acceptance and fallback

A workflow fails closed or abstains when required evidence is missing, provider results violate schema, model disagreement exceeds policy, injection tests trigger, or verifier support is insufficient. Provider outage may route to an allowed alternative, return deferred/unresolved state, or use deterministic fallback where scientifically valid. It never silently changes the estimand or fabricates semantic evidence.

## 11. Required ablation before production claim

Before claiming an orchestration mode materially improves TEPP, compare at least:

1. strongest approved single-model direct baseline;
2. direct + verifier;
3. fixed role-based multi-agent workflow;
4. adaptive/learned-conductor-style workflow where available;
5. at least two reasoning-effort/budget settings.

Report uncertainty and failure modes, not only the best benchmark score.
