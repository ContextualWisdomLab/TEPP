# TEPP LLM Orchestration and Test-Time Compute Contract

**Status:** Partial — TEPP owns semantic-task policy/evidence contracts; provider routing requires a released `contextual-orchestrator` contract and is not yet deployable from the current repository state.  
**Last reviewed:** 2026-09-02

## 1. Purpose

TEPP uses LLMs only for bounded semantic unitization, candidate-model review, evidence-grounded interpretation, and independent claim verification. Statistical estimation, temporal eligibility, event-relation validity, measurement invariance, numerical acceptance, and release authority remain deterministic/Rust or governed human authority.

The product may allocate test-time compute adaptively rather than assume that either one frontier model or a large fixed multi-agent graph is always best. TEPP decides the semantic task, evidence/access policy, scientific risk, and admissible orchestration mode; provider/model/group routing is a `contextual-orchestrator` responsibility.

## 2. Research basis

The orchestration design is informed by three 2026 Sakana AI research lines:

- **TRINITY** ([ORCH-TRINITY-2026](research/standards-and-literature.md#orch-trinity-2026)): a lightweight coordinator selects a model and Thinker/Worker/Verifier role over multiple turns, showing learned adaptive delegation under budget constraints.
- **Conductor** ([ORCH-CONDUCTOR-2026](research/standards-and-literature.md#orch-conductor-2026)): a learned coordinator generates communication topology and focused natural-language instructions for a heterogeneous model pool, including recursive test-time scaling.
- **Sakana Fugu** ([ORCH-FUGU-2026](research/standards-and-literature.md#orch-fugu-2026)): production-oriented query-adaptive scaffolds combining learned orchestration approaches and exposing the orchestration system behind a model-compatible API.

These results motivate experiments; they do not prove that deeper orchestration is always superior for TEPP workloads.

## 3. Orchestration modes

| Mode | Typical TEPP use | TEPP policy request |
|---|---|---|
| direct | simple span classification, deterministic-schema fill, low-ambiguity label | one governed semantic call |
| verify | interpretation or classification with material unsupported-claim risk | producer + independent verifier |
| committee | K/model interpretation with scientific ambiguity | blinded parallel raters + adjudication |
| conductor | complex evidence synthesis or multi-stage semantic reasoning | adaptive roles/topology under explicit budget |
| abstain | evidence/contract/capability insufficient | no forced answer |

`tepp_api::route_orchestration` governs only the TEPP-side task/mode proposal. It does not select a provider, provider group, concrete model, or paid fallback. Provider-neutral routing and execution are delegated through a released `contextual-orchestrator` API/client/schema. Quality, evidence support, calibration, disagreement, controllability, and reproducibility dominate; `scientific_authority_code` remains `deterministic_statistical_gates`.

## 4. Explicit experimental variables

Every orchestration benchmark records and can ablate:

- the released orchestrator contract/version and routing receipt;
- orchestrator-selected provider/model identity as observed provenance, not TEPP routing configuration;
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
- failure/abstention outcome.

Comparisons must use approximately comparable budgets or report the budget difference explicitly. Fugu/Conductor/TRINITY experiments are ablations over a released orchestration boundary; they do not authorize branch-local provider routing.

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

LLM calls receive only the minimum evidence bundle needed for the assigned role. Documents are untrusted observations and cannot alter orchestration policy, tools, credentials, routing, access lists, or scientific gates.

Each call records:

```text
orchestrator_contract_version
orchestrator_receipt_id
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

Provider/model fields are returned provenance. They are not TEPP provider-selection inputs. Raw provider credentials are never TEPP model-visible or required by TEPP semantic clients. Model outputs are proposals, not source facts.

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
- orchestrator failure resilience;
- repeated-run variance.

For model-selection review, statistical predictive/recovery/stability/invariance gates run before LLM judgment. LLM preference cannot rescue a statistically rejected candidate.

## 8. contextual-orchestrator boundary

`contextual-orchestrator` is the canonical CWL owner of provider discovery, key auto-discovery, provider/model/group routing, paid/free policy, request-family adaptation, fallback, streaming/tool-call lifecycle, and provider execution. TEPP owns evidence bundles, semantic-task policy, statistical/model-selection policy, scientific acceptance, artifact provenance, and allowed role/access configuration. Neither service reads the other's application database directly.

Production TEPP integration consumes only a **released, versioned** contextual-orchestrator API/client/schema through an ACL with immutable artifact identity and provenance. A mutable protected-main commit, open PR head, or checksum-pinned source snapshot without an immutable release is candidate evidence, not production dependency authority. At the 2026-09-02 review, contextual-orchestrator has no GitHub release, so production semantic execution through this boundary remains fail-closed until a compatible release exists and is verified/adopted.

`orchestrator_live` may expose a local adapter/listener for contract tests and hypothetical planning, but it is not a second provider router and cannot become scientific authority. Production provider execution stays behind the released contextual-orchestrator boundary.

## 9. Development and live-test credentials

TEPP semantic clients use only the contextual-orchestrator gateway credential appropriate to the released contract. Model-backed GitHub Actions request `orchestrator/free`; they do not receive or select provider credentials, providers, models, provider groups, or paid fallbacks. `COPILOT_GITHUB_TOKEN` is prohibited. Independent review-agent credentials remain separate and cannot be renamed, copied, or repurposed for model execution.

If the released orchestrator cannot expose a required capability, TEPP fails closed and the missing contract/capability is repaired at the contextual-orchestrator owner before consumer adoption. TEPP does not compensate by importing a provider SDK or secret.

## 10. Acceptance, timeout, and failure semantics

A workflow fails closed or abstains when required evidence is missing, the released orchestration contract is unavailable, results violate schema, model disagreement exceeds policy, injection tests trigger, or verifier support is insufficient. Provider outage/routing recovery is owned by contextual-orchestrator; TEPP receives the governed result or a typed unresolved/failure outcome and never silently chooses a replacement provider or paid route.

Reasoning, streaming, and tool-call work is not terminated merely because an arbitrary elapsed-time default expires. User cancellation, provider-declared termination, and explicit administrative timeout are separate typed outcomes. Long-running OpenCode/Strix/Noema-style work must remain possible when the released contract supports it.

No LLM path may silently change an estimand, perform numerical scientific acceptance, authoritatively activate a candidate, or fabricate semantic evidence.

## 11. Required ablation before production claim

Before claiming an orchestration mode materially improves TEPP, compare at least:

1. strongest approved direct baseline exposed by the released orchestrator;
2. direct + verifier;
3. fixed role-based multi-agent workflow;
4. adaptive/learned-conductor-style workflow where available;
5. at least two reasoning-effort/budget settings.

Report uncertainty, routing receipts, contract version, budget, and failure modes, not only the best benchmark score.
