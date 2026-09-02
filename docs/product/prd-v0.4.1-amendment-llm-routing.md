# TEPP PRD v0.4.1 Amendment — Released contextual-orchestrator routing boundary

**Status:** active-PR product-authority amendment; not implemented-main  
**Amends:** `docs/product/prd-v0.4-approved.md` §12 LLM responsibilities  
**Decision date:** 2026-09-02  
**Owning implementation vehicle:** PR #480  

## Problem

The approved v0.4 baseline still authorized live model tests with a provider-specific secret. TEPP also carried an hourly bootstrap that could rank provider discovery rows itself. Both behaviors put provider routing and credential policy in the consumer repository even though `contextual-orchestrator` is the canonical CWL owner of provider discovery/routing and `orchestrator/free` policy.

A second constraint is material: on 2026-09-02 contextual-orchestrator protected `main@8839081659df587b19642be17b9114f9dee8b666` has no GitHub release. A mutable main commit, open PR head, or checksum-pinned source snapshot cannot be promoted to production dependency authority merely because its source is reviewable.

## Product requirement

All TEPP semantic LLM work—semantic unitization, interpretation, verification, judging, label/explanation proposal, and model-backed automation—uses a **released, versioned contextual-orchestrator API/client/schema** through a TEPP ACL.

TEPP owns the semantic task, minimum evidence bundle, access/tool policy, reasoning/verification policy, scientific-risk policy, provenance requirements, and abstention semantics. `contextual-orchestrator` owns provider-key auto-discovery, provider/model/group routing, request-family adaptation, free/paid policy, fallback, streaming/tool-call lifecycle, and provider execution.

Model-backed GitHub Actions request `orchestrator/free` using the gateway credential only. TEPP does not select a provider, concrete model, provider group, or paid fallback and does not receive provider API keys. If the released orchestrator cannot provide a required capability, the consumer fails closed until the canonical owner releases that capability.

LLM output never performs numerical estimation, scientific acceptance, or authoritative candidate activation.

## Admission and release constraint

The existing explicit-zero TEPP bootstrap regression remains defense-in-depth evidence against accidental paid/unknown routing, but it does not make an unreleased orchestrator snapshot a valid production dependency.

Deployable semantic execution requires all of the following:

1. a compatible immutable contextual-orchestrator release;
2. released contract/schema/client identity and reproducible artifact digest/provenance;
3. TEPP ACL compatibility fixtures and exact dependency review;
4. model-backed Actions using only `orchestrator/free` plus gateway credential;
5. exact-current TEPP CI/security/review evidence after the dependency bump.

Until those conditions hold, semantic live execution is fail-closed rather than silently reverting to direct provider access.

## Alternatives considered

- **Keep direct provider secrets in TEPP.** Rejected because it duplicates provider authority, leaks provider policy into a consumer, and makes key/routing changes branch-local.
- **Pin a mutable or unreleased contextual-orchestrator source commit.** Rejected as production authority because a source checksum proves identity, not release governance, supportability, or immutable published contract provenance.
- **Let TEPP rank provider models while using contextual-orchestrator only as transport.** Rejected because provider/model/group/free-policy routing belongs to the canonical owner.
- **Consume a released contextual-orchestrator contract through an ACL and fail closed when absent.** Selected because ownership, provenance, rollback, and consumer compatibility remain explicit.

## Risks and effects

A released-contract requirement can temporarily make live semantic features unavailable when the owner has not published a compatible release. That availability loss is intentional and preferable to hidden spend, mutable dependencies, or provider-specific execution outside the owner boundary.

Routing receipts may still record the provider/model actually selected by contextual-orchestrator for reproducibility. Those values are observed provenance, not TEPP routing inputs.

Long-running reasoning, streaming, and tool-call work must not be terminated solely by a short elapsed-time default. User cancellation, provider termination, and explicit administrative timeout remain distinct typed outcomes.

## Acceptance evidence

PR #480 must keep the canonical product/technical/architecture/LLM documents free of provider-key execution authority and must enforce the released contextual-orchestrator boundary with deterministic documentation fitness tests. Protected-main promotion additionally requires the immutable owner release and consumer-adoption evidence above; an active PR cannot claim that release exists.
