# ADR 0015 — Autonomous development, review, and merge authority separation

**Decision status:** Accepted  
**Implementation maturity:** accepted-target  
**Date:** 2026-08-12  
**Supersedes:** None; narrows and clarifies the automation clauses previously mixed into ADR 0006.

## Context

TEPP uses automated development and review workflows, including OpenCode/model-backed work. A model process that receives repository-write authority, reviewer credentials, or merge authority in the same trust boundary can convert untrusted model output into source mutation or policy bypass. The product-development loop must also continue safely while GitHub checks or central reviewers are slow, rather than weakening gates or treating waiting as completion.

## Decision

TEPP separates autonomous development into distinct authority classes:

1. **Inventory/planning authority** may read repository state and select one bounded task.
2. **Model proposal authority** may use `NVIDIA_NIM_API_KEY` to generate a proposed patch or structured change, but receives no repository-write, reviewer, release, or merge credential.
3. **Deterministic verification authority** runs tests, policy, security, documentation, and allowed-path checks without model credentials and emits immutable verification evidence.
4. **Publication authority** may publish only a verified bounded proposal to a branch/PR using short-lived least-privilege repository authority; it does not execute model-generated commands as trusted code merely because they were proposed.
5. **Independent review authority** uses the existing dedicated reviewer identities/credentials and is never repurposed as a development publisher.
6. **Merge/release authority** remains GitHub repository policy plus exact-head required gates and qualifying approvals. No LLM or autonomous development agent can self-approve or override them.

`COPILOT_GITHUB_TOKEN` is prohibited as a model/development credential. Existing review-agent credential names and key chains are not renamed, copied, or reused. GitHub Actions autonomous development uses immutably pinned OpenCode/runtime dependencies and least-privilege permissions.

A slow central OpenCode/Noema/CodeRabbit/check lane blocks only that exact merge/review action. The work-conserving loop may advance non-conflicting TEPP work but cannot manufacture success, perturb a clean head solely to retrigger a reviewer, or bypass branch protection.

## Alternatives considered

1. **Give the model job repository write permission** — rejected because model prompt/input becomes part of a write-authority boundary.
2. **Use one bot identity for development, review, and merge** — rejected because independence and auditability collapse.
3. **Separate proposal, deterministic verification, publication, independent review, and merge authority** — accepted.

## Consequences

- model compromise or prompt injection does not automatically grant source-write authority;
- reviewer evidence remains independent from developer evidence;
- every autonomous mutation has an auditable proposal, verifier, publisher, exact source head, and resulting PR identity;
- autonomous loops can remain productive around external latency without weakening technical or governance gates;
- one-shot/self-modifying/encoded-patch writer workflows are not an acceptable normal publication mechanism.

## Failure and recovery

Missing inventory, stale source identity, verifier failure, changed branch head, unavailable required credential, allowed-path violation, or mismatched proposal/verification digest fails closed. If publication cannot use the exact verified parent, it is retried only after refetch/reverification rather than force-pushed. If a model/provider is unavailable, deterministic/repository work continues where possible and model-dependent work becomes deferred/abstained.

## Security, privacy, and governance impact

Secrets are materialized only in the job that needs them and never exposed to repository content, model prompts, untrusted documents, ordinary logs, or unrelated jobs. Provider payload minimization follows ADR 0009. Review/merge authority separation is a governance control and cannot be relaxed merely to reduce latency.

## Compatibility and migration

Repository-local automation should call central `.github` reusable workflows when those are the authoritative control plane, pinned to immutable revisions where supported. TEPP may remain usable without central automation; loss of the control-plane integration does not change TEPP's scientific/runtime contracts.

## Verification

Required tests and evidence cover permission scopes, absence of repository-write credentials from model jobs, digest binding between proposal and verification, allowed-path policy, stale-head refusal, branch CAS/no-force publication, secret non-leakage, reviewer-identity separation, inability to self-approve, exact-head merge gating, and bounded behavior when review/check providers are unavailable or delayed.

## Rollback and supersession

If an automation path is unsafe or unverifiable, disable that writer path and fall back to human or deterministic publication without changing product scientific behavior. Supersede only with an architecture that preserves at least equivalent separation between model proposal, deterministic verification, publication, independent review, and merge/release authority.
