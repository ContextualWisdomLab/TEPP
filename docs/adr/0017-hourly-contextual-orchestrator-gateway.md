# ADR 0017 — Hourly contextual-orchestrator gateway for autonomous proposals

**Decision status:** Accepted
**Implementation maturity:** active-PR — released-owner workflow wiring requires exact-head Checks and protected-main integration
**Date:** 2026-08-20
**Last clarified:** 2026-09-02
**Supersedes:** the earlier implementation interpretation that bootstrapped provider routing inside TEPP; Git history preserves that repair lineage
**Related ADRs:** ADR 0010 (LLM test-time compute), ADR 0011 (modular MSA), ADR 0015 (autonomous development authority)

## Context

The hourly product-development workflow originally configured OpenCode against a provider-specific endpoint. A first repair moved the provider keys into a locally started contextual-orchestrator gateway and added explicit-zero price filtering before local cheapest-model ranking. That repair established useful RED evidence for the hidden-spend defect, but it left two architecture violations in the consumer repository:

1. TEPP still discovered, admitted, ranked, and selected provider models even though contextual-orchestrator owns provider/model/group routing and `orchestrator/free` policy.
2. The workflow consumed a checksum-pinned source commit rather than a released, versioned owner contract. A source checksum proves identity but does not establish immutable release authority, supportability, or contract provenance.

The loopback implementation also sent its bearer token over plaintext HTTP and imposed a 900-second elapsed-time kill on OpenCode. Neither behavior belongs in the released gateway contract.

## Decision

The hourly proposal workflow consumes only a **released, versioned contextual-orchestrator contract**. Repository variable `CONTEXTUAL_ORCHESTRATOR_RELEASE` names the required owner release. Before semantic execution, the workflow queries the contextual-orchestrator GitHub release by tag and requires the matching release to be neither draft nor prerelease and to report `.immutable == true`.

`CONTEXTUAL_ORCHESTRATOR_BASE_URL` is an HTTPS gateway base URL. The proposal runner uses only `CONTEXTUAL_ORCHESTRATOR_GATEWAY_TOKEN` to check gateway liveness and authenticated model discovery. The route catalog must expose `orchestrator/free`. OpenCode then requests `contextual-orchestrator/orchestrator/free` through the same released gateway. TEPP does not receive provider credentials and does not discover, rank, select, or fall back among providers/models/groups.

If the owner release is absent, mutable, draft/prerelease, the gateway URL is not HTTPS, the gateway credential is unavailable, liveness/model-list checks fail, or `orchestrator/free` is absent, the proposal run fails closed. A mutable owner main commit, open PR head, source archive, or checksum-pinned snapshot is never substituted for the released contract.

The previous TEPP-owned provider bootstrap is retired. Its explicit-zero tests remain useful historical RED evidence for issue #479, but the stronger invariant is now ownership: free/paid/provider admission lives in contextual-orchestrator and the TEPP workflow may only request `orchestrator/free`.

OpenCode has no elapsed-time-only model timeout in this workflow. GitHub's proposal job retains a 55-minute explicit administrative job budget; administrative workflow exhaustion, user cancellation, provider termination, and model/stream/tool-call completion are distinct outcomes.

The proposal/verifier/publisher separation remains unchanged. The proposal runner receives read-only repository authority plus the gateway credential; the verifier receives neither model nor publication credentials; the publisher mints the dedicated Maintainer App token only after immutable patch verification and never executes proposed code.

## Alternatives considered

1. **Direct provider configuration in TEPP** — rejected because it duplicates owner routing and credential policy.
2. **TEPP-side explicit-zero admission plus cheapest ranking** — rejected as the production design. It closes hidden spend only if consumer-side provider metadata is complete and still leaves routing authority in the wrong bounded context.
3. **Treat absent price metadata as zero** — rejected because unknown cost is not evidence of free service.
4. **Pin a contextual-orchestrator source commit/archive** — rejected as production authority because checksum identity is not a released contract.
5. **Start a loopback HTTP gateway with a local bearer token** — rejected because it creates consumer-owned gateway lifecycle/routing and cleartext bearer transport even on loopback.
6. **Consume immutable released `orchestrator/free` through an HTTPS gateway credential** — selected because provider routing, free policy, lifecycle, and credential discovery remain at the canonical owner while TEPP retains only semantic-task policy and consumer conformance.

## Consequences

- TEPP can be unavailable for live semantic automation while contextual-orchestrator has no compatible immutable release. That fail-closed availability loss is intentional.
- Provider/model identities may still appear in orchestrator receipts as observed provenance; they are not TEPP routing inputs.
- A release tag alone is insufficient. TEPP also requires immutable release metadata, HTTPS gateway availability, route presence, and exact consumer-side review/Checks after adoption.
- Provider retention, region, confidentiality, and commercial obligations remain owner/operator concerns and must be reviewed before enabling the gateway.
- The workflow is headless; no Figma artifact is applicable to this decision.

## Verification

Repository contract tests require:

- `CONTEXTUAL_ORCHESTRATOR_RELEASE`, `CONTEXTUAL_ORCHESTRATOR_BASE_URL`, and the gateway credential;
- GitHub release-tag lookup with `.immutable == true`;
- HTTPS-only gateway probes and `orchestrator/free` presence;
- no provider-key names, mutable contextual-orchestrator source pins, local provider-selection helper, loopback HTTP route, or elapsed-time OpenCode kill in the hourly workflow;
- no consumer-side provider-routing bootstrap source;
- queue/base revalidation, immutable proposal artifact identity, and exact publication authority separation.

The historical explicit-zero admission regression is preserved in Git history and PR review lineage as the reproducer for issue #479. It is not retained as current production routing code because doing so would keep a second owner for provider/free policy.

## Rollback

Disable the schedule or revert the repair through a reviewed PR. Removing the released-contract tag, HTTPS gateway URL, gateway credential, or publication App configuration leaves the workflow fail-closed. Rollback does not touch review-agent credentials, protected-branch rules, or scientific runtime contracts. Do not restore direct provider access, local provider ranking, plaintext loopback bearer transport, or a mutable owner snapshot as an incident shortcut.
