# Adaptive orchestration router and comparable-budget ablation

## Scope

This note doctors the `tepp_api` governed router that implements the first executable slice of ADR 0010 without a database migration and without live model I/O:

1. `route_orchestration` selects versioned modes `direct`, `verify`, `committee`, `conductor`, or `abstain` from CPU `f64` unit-interval risk, ambiguity, and evidence-sufficiency scores plus an explicit token budget;
2. the plan records workflow stage count, recursion depth, decomposition, TEPP-owned access lists, and role-specific reasoning effort;
3. documents cannot change policy, access lists, or credentials (`DocumentControlAttempt`);
4. blinded model review that failed a deterministic scientific gate abstains — LLM preference cannot rescue a statistically rejected candidate;
5. `record_budget_ablation` requires a `direct` baseline and the same task kind, policy version, and access list before it reports whether the compared budget is within a 10 percent relative band;
6. `bind_contextual_orchestrator` emits a credential-free execution binding, accepts only a canonical lowercase `sha256:` evidence-manifest digest, and refuses abstention.

Live NVIDIA NIM HTTP, learned conductor calibration, and production-quality claims remain accepted-target.

## Authoritative sources

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2026). TRINITY: An evolved LLM coordinator. In *International Conference on Learning Representations (ICLR 2026)*. https://arxiv.org/abs/2512.04695

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2026). Learning to orchestrate agents in natural language with the Conductor. In *International Conference on Learning Representations (ICLR 2026)*. https://arxiv.org/abs/2512.04388

Tang, Y., Cetin, E., Xu, J., Sun, Q., Nielsen, S., Richard, V., Goda, H., Tymchenko, I., Nguyen, N., Lee, H., Ashiga, M., Kotyan, S., Kuroki, S., & Clanuwat, T. (2026). *Sakana Fugu technical report* [Preprint]. arXiv. https://arxiv.org/abs/2606.21228

## Application

TRINITY motivates lightweight model/role delegation rather than a fixed deep graph (Xu et al., 2026). Conductor motivates recording topology, instructions, and recursive test-time scaling as explicit variables (Nielsen et al., 2026). Fugu frames production orchestration as a query-adaptive scaffold behind a model-compatible boundary (Tang et al., 2026). TEPP therefore treats the router as a deterministic policy object: deeper modes must justify themselves against a direct baseline at a comparable budget, and LLM output remains an untrusted proposal under deterministic statistical gates. These citations are experimental motivation, not authority to replace TEPP estimands.

## Verification

- low-risk span classification and schema conversion route `direct`;
- material risk, adversarial verification, and low-ambiguity concept/narrative work route `verify`;
- high-ambiguity concept alignment and gated blinded review route `committee`;
- high-complexity narrative synthesis routes `conductor` when the budget allows and steps down otherwise;
- insufficient evidence, failed scientific gates, and sub-minimum budgets abstain;
- document-controlled policy, access, or credentials are denied;
- non-unit scores and unknown policy versions fail closed;
- ablation rejects a non-direct baseline or a comparison that changes task kind, policy version, or access list, and reports incomparable zero or wide-band budgets;
- orchestrator bindings omit credentials, reject raw source and malformed evidence-manifest values, and refuse abstention.
