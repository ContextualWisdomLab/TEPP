# ADR 0017 — Hourly contextual-orchestrator gateway for autonomous proposals

**Decision status:** Accepted
**Implementation maturity:** active-PR — workflow wiring and bootstrap contract are implemented on this PR and require exact-head Checks and protected-main integration
**Date:** 2026-08-20
**Supersedes:** None; this ADR clarifies the hourly proposal credential route without changing scientific or reviewer authority
**Related ADRs:** ADR 0010 (LLM test-time compute), ADR 0011 (modular MSA), ADR 0015 (autonomous development authority)

## Context

The hourly product-development workflow previously configured OpenCode directly
against one NVIDIA NIM endpoint. That did not exercise the repository's
contextual-orchestrator integration or its model-discovery contract, and it put a
provider key in the model-agent process. The scheduler must remain a proposal
producer: it may not merge, release, deploy, approve, or alter reviewer-agent
credentials.

## Decision

The hourly proposal runner starts a pinned, ephemeral
`ContextualWisdomLab/contextual-orchestrator` loopback gateway. At bootstrap it
registers `BYTEZ_API_KEY`, `NVIDIA_NIM_API_KEY`, `NVIDIA_NIM_API_KEY_SUB`,
`OPENROUTER_API_KEY`, and `OPENAI_API_KEY` in the gateway KV, removes those
provider values from the gateway environment, and discovers models from all five
provider entries. The gateway records every discovered provider model, excludes
endpoint-only and safety-only model identifiers from the chat pool, and enables
the three lowest-cost general-chat candidates using contextual-orchestrator's
existing price-book selector. This prevents an embedding or image model from
being sent to an ordinary chat endpoint while preserving full discovery
evidence.

OpenCode calls only `http://127.0.0.1:18000/v1` with a separately generated
loopback bearer token. It receives no provider credential. Liveness and
authenticated model-list checks must pass before the proposal agent runs.

The workflow remains fail-closed when any provider credential is absent, the
PR or issue queue is unreadable, an open PR or issue exists, or the publication
App is unavailable.
The verifier and publisher receive neither provider nor model credentials.

## Alternatives considered

1. **Direct NVIDIA NIM configuration** — rejected because it bypasses the
   contextual-orchestrator discovery and routing boundary.
2. **One CLI registration process per provider** — rejected because the default
   in-memory KV is process-local and would silently lose earlier registrations.
3. **Persistent shared credential database in the workflow** — rejected for this
   ephemeral proposal path because it expands infrastructure and retention scope.
4. **One in-process gateway bootstrap with pinned source and loopback auth** —
   accepted as the smallest auditable boundary.

## Consequences

- All configured provider discovery paths are exercised and auditable without
  exposing provider keys to OpenCode.
- Discovery can fail for an individual provider while another provider supplies
  a candidate; zero discovered candidates fail the proposal job.
- The three-candidate selection is cost-oriented, not a scientific quality
  claim. TEPP deterministic evidence and human review remain authoritative.
- Provider terms, retention, region, and confidentiality must be reviewed for
  every configured provider before the schedule is enabled.
- The workflow is not a Figma or visual interaction surface; no Figma file is
  applicable to this headless scheduler decision.
- Figma File ID: N/A — this headless scheduler introduces no visual interaction
  contract or design artifact.

## Verification

The PR contract tests assert the hourly schedule, all five credential names,
immutable gateway source digest, loopback-only OpenCode configuration, health and
model-list probes, provider-key removal from OpenCode, no Copilot token, and
absence of merge/release commands. The bootstrap import and credential transfer
self-checks verify that the provider keys enter the KV and leave the process
environment before serving.

## Rollback

Disable the schedule or revert this PR. Removing any required provider secret
causes a stable no-op. Rollback does not touch review-agent credentials,
protected-branch rules, or scientific runtime contracts.
