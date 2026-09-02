# ADR 0017 — Hourly contextual-orchestrator gateway for autonomous proposals

**Decision status:** Accepted
**Implementation maturity:** active-PR — workflow wiring and bootstrap contract require exact-head Checks and protected-main integration
**Date:** 2026-08-20
**Last clarified:** 2026-09-02
**Supersedes:** None; this ADR clarifies the hourly proposal credential and cost-admission route without changing scientific or reviewer authority
**Related ADRs:** ADR 0010 (LLM test-time compute), ADR 0011 (modular MSA), ADR 0015 (autonomous development authority)

## Context

The hourly product-development workflow previously configured OpenCode directly against one NVIDIA NIM endpoint. That bypassed the repository's contextual-orchestrator integration and model-discovery contract and put a provider key in the model-agent process. The scheduler must remain a proposal producer: it may not merge, release, deploy, approve, or alter reviewer-agent credentials.

A later audit found a second boundary defect. The bootstrap discovered all providers and then ranked every general-chat candidate by cheapest cost. The checksum-pinned contextual-orchestrator selector treats an unpriced row as zero for its generic cost ordering, and a nonzero route may still be among the cheapest. Cheapest therefore does not prove free. The hourly workflow is required to remain inside the organizational `orchestrator/free` cost boundary, so price-unknown or paid routes must not enter its agent pool.

## Decision

The hourly proposal runner starts a pinned, ephemeral `ContextualWisdomLab/contextual-orchestrator` loopback gateway. At bootstrap it registers `BYTEZ_API_KEY`, `NVIDIA_NIM_API_KEY`, `NVIDIA_NIM_API_KEY_SUB`, `OPENROUTER_API_KEY`, and `OPENAI_API_KEY` in the gateway KV, removes those provider values from the gateway environment, and discovers models from all five provider entries.

The gateway records every discovered provider model and excludes endpoint-only and safety-only model identifiers from the general-chat pool. Before any cheapest-ranking step, TEPP's bootstrap ACL admits only discovery rows whose provider-reported prompt and completion token-price components are both present and exactly `0.0`. Paid, partially priced, and fully unpriced production rows are excluded. The existing contextual-orchestrator top-N selector may rank only that explicitly free subset, and at most three admitted agents are enabled.

If model discovery succeeds but no general-chat candidate has complete explicit zero-cost evidence, bootstrap fails closed. It does not interpret missing pricing as free and does not fall back to a paid route. The secret-free discovery report records the number of explicitly free candidates and the number of non-free/unknown candidates excluded from the chat pool.

This ACL is required because TEPP protected main currently checksum-pins a contextual-orchestrator revision that predates upstream's richer native `orchestrator/free` discovery classification. TEPP may delegate the same decision to a later native contract only after the replacement contextual-orchestrator archive has a reproducible digest and the exact dependency change is reviewed. Removing checksum validation or guessing a replacement digest is not an alternative.

OpenCode calls only `http://127.0.0.1:18000/v1` with a separately generated loopback bearer token and receives no provider credential. Liveness and authenticated model-list checks must pass before the proposal agent runs.

The workflow remains fail-closed when any provider credential is absent, the PR or issue queue is unreadable, an open PR or issue exists, the publication App is unavailable, no general-chat route is discovered, or no discovered general-chat route has explicit zero-cost evidence. The verifier and publisher receive neither provider nor model credentials.

## Alternatives considered

1. **Direct NVIDIA NIM configuration** — rejected because it bypasses contextual-orchestrator discovery and routing.
2. **Rank every discovered route by generic cheapest cost** — rejected because cheapest is not a free-tier proof and the pinned selector's unpriced-row behavior is unsuitable for a fail-closed zero-cost workflow.
3. **Treat absent price metadata as zero** — rejected because unknown cost is not evidence of free service.
4. **Silently advance contextual-orchestrator to current upstream** — rejected because the workflow requires an exact checksum-pinned dependency and the replacement archive digest has not been independently established in this repair.
5. **One CLI registration process per provider** — rejected because the default in-memory KV is process-local and would silently lose earlier registrations.
6. **Persistent shared credential database in the workflow** — rejected because it expands infrastructure and retention scope for an ephemeral proposal path.
7. **One in-process gateway bootstrap with explicit-zero admission, pinned source, and loopback auth** — accepted as the smallest auditable repair compatible with the current dependency pin.

## Consequences

- All configured provider discovery paths remain auditable without exposing provider keys to OpenCode.
- Provider discovery may succeed while the proposal run is refused because no candidate has complete zero-cost evidence. This availability loss is intentional; it is preferable to silent spend.
- Bytez-style or other price-unknown rows do not enter the hourly pool merely because a generic cost selector would rank them as zero.
- The three-candidate selection is cost-bounded routing, not a scientific quality claim. TEPP deterministic evidence and human review remain authoritative.
- Provider terms, retention, region, and confidentiality must still be reviewed for every admitted provider before the schedule is enabled.
- The workflow is headless; no Figma artifact is applicable to this decision.

## Verification

The contract tests assert the hourly schedule, all five credential names, immutable gateway source digest, loopback-only OpenCode configuration, health and model-list probes, provider-key removal from OpenCode, no Copilot token, and absence of merge/release commands.

The free-admission regression tests prove that a paid route and a fully unpriced route never reach the ranking selector when a free route is available, partial pricing is not accepted as free, and a discovery result with no explicitly free general-chat route fails closed. These tests exercise the TEPP bootstrap boundary rather than changing contextual-orchestrator's canonical provider-routing implementation.

## Rollback

Disable the schedule or revert the repair through a reviewed PR. Removing any required provider secret causes a stable no-op. Rollback does not touch review-agent credentials, protected-branch rules, or scientific runtime contracts. Do not restore the pre-repair paid/unknown fallback behavior as an incident shortcut.
