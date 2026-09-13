# Hourly Contextual Orchestrator OpenCode Development — Evidence Doctoring

## Claim boundary

This workflow can propose one bounded pull request only after both the pull-request and issue queues are empty. It does not claim autonomous semantic correctness, production safety, customer acceptance, scientific validity, or permission to merge, release, deploy, or approve. Deterministic tests, schemas, security scanners, independent review, human judgment, and exact-head rules remain separate controls.

The current PR is also not evidence that live semantic automation is deployable. On the 2026-09-02 owner sweep, contextual-orchestrator protected `main` was `6d60c756b6481c59bd8fee95996315279bd708d5` and the repository had no GitHub release. TEPP therefore fails closed until an immutable compatible owner release exists and is adopted.

## Source-supported facts and project decisions

| Topic | Source-supported fact | TEPP decision |
|---|---|---|
| GitHub jobs | Jobs execute with explicitly scoped permissions and hosted job time limits | Proposal, verifier, and publisher remain separate jobs; the 55-minute proposal-job limit is an administrative budget, not a model reasoning timeout |
| GitHub App authentication | A workflow can mint an installation token with explicit repository permissions | Publication token is minted only in the publisher after immutable proposal validation |
| Artifact handoff | Artifact upload/download actions expose numeric IDs and digests | Patch ID, digest, base, size, count, and modes are checked before verification and publication |
| GitHub releases | Release records identify tag, draft/prerelease state, and immutable-release state where supported | Model-backed execution requires the configured contextual-orchestrator tag to resolve to a non-draft, non-prerelease release with `.immutable == true` |
| OpenCode | OpenCode can call an OpenAI-compatible endpoint and operate under bounded permissions | One checksum-pinned OpenCode binary calls only the configured HTTPS contextual-orchestrator gateway and requests `orchestrator/free` |
| contextual-orchestrator | The CWL owner is responsible for provider discovery, provider/model/group routing, free/paid admission, fallback, lifecycle, and provider execution | TEPP no longer bootstraps provider routing locally; it supplies only the semantic task/evidence policy and gateway credential |
| SSDF | NIST SP 800-218 recommends protected build environments, review, provenance, and vulnerability response | Fresh verification, immutable artifacts, least privilege, and ordinary PR governance remain mandatory |
| AI risk | ISO/IEC 23894:2023 and ISO/IEC 42001:2023 require contextual risk treatment and controlled change | Stable refusals, owner boundaries, bounded proposals, traceable decisions, and rollback are documented |
| Test-time compute | Fugu, Conductor, and TRINITY distinguish routing from deeper role-based workflows | Runtime experiments preserve routing-versus-conduct distinctions and record role/topology/reasoning ablations without replacing deterministic scientific gates |

## RED → causal repair trace

Issue #479 first exposed a cost-admission defect: consumer-side cheapest ranking could admit paid or unpriced provider rows. The predecessor regression constrained local admission to explicit zero-valued pricing. Review of the same current feature then found that this “fix” still violated the canonical owner boundary because TEPP continued to discover/rank/select provider models and boot a mutable owner source snapshot.

The stronger repair is architectural rather than another price predicate:

1. retire `scripts/run_contextual_orchestrator.py` from TEPP production source;
2. require `CONTEXTUAL_ORCHESTRATOR_RELEASE` and query the owner release by tag;
3. require a non-draft, non-prerelease release with `.immutable == true`;
4. require an HTTPS `CONTEXTUAL_ORCHESTRATOR_BASE_URL` and gateway credential;
5. verify liveness and authenticated `/v1/models` over HTTPS and require `orchestrator/free`;
6. configure OpenCode only against the released gateway route;
7. remove consumer-side provider keys, provider discovery, price ranking, mutable owner source pins, plaintext loopback bearer transport, and the elapsed-time OpenCode kill;
8. preserve the GitHub job time limit as an explicit administrative budget;
9. correct publisher base revalidation to compare the live default-head SHA with `EXPECTED_BASE` rather than an unbound shell variable.

The old explicit-zero test remains in Git history and PR review lineage as reproducer evidence. Keeping its provider-selection production helper would preserve the wrong owner, so current fitness instead asserts that the helper is absent and the workflow can only reach the released `orchestrator/free` boundary.

## Trust-boundary rationale

The proposal runner receives read-only repository authority and the gateway credential. It has no publication token. The verifier receives neither gateway nor publication credentials and re-runs the release-quality gate against the immutable patch. The publisher applies the same patch without executing it and receives a narrow Maintainer App token only after metadata parsing and artifact checks.

Gateway transport is HTTPS-only. Loopback binding alone was rejected as a confidentiality control because a bearer token over plaintext transport remains cleartext regardless of remote reachability. TEPP does not implement its own TLS/proxy for this lane because doing so would retain a second gateway lifecycle; the released contextual-orchestrator endpoint owns serving/transport while TEPP validates its HTTPS consumer boundary.

Model execution has no fixed elapsed-time kill command. The proposal job's 55-minute GitHub limit is an explicit administrative resource boundary. User cancellation, owner/provider termination, model reasoning completion, stream completion, and tool-call completion remain semantically distinct.

## Orchestration research application

**Fugu.** Fugu motivates query-adaptive choice between direct model use and deeper coordinated execution. It does not authorize consumer-side provider selection.

**Conductor.** Conductor motivates task decomposition, worker assignment, access-list control, and topology experiments. TEPP records those as orchestration-policy variables passed through the released owner boundary.

**TRINITY.** TRINITY motivates specialized thinker/worker/verifier/synthesis roles and reasoning-level ablation. It does not replace evidence admission, numerical estimation, or scientific acceptance.

These sources motivate evaluation hypotheses; they do not prove deeper orchestration improves TEPP. Comparable-budget ablations, evidence support, failure/abstention behavior, and human review remain required.

## APA 7th references

GitHub. (n.d.). *Security hardening for GitHub Actions*. GitHub Docs. Retrieved August 6, 2026, from https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions

GitHub. (n.d.). *Making authenticated API requests with a GitHub App in a GitHub Actions workflow*. GitHub Docs. Retrieved August 6, 2026, from https://docs.github.com/en/apps/creating-github-apps/writing-code-for-a-github-app/making-authenticated-api-requests-with-a-github-app-in-a-github-actions-workflow

International Organization for Standardization, & International Electrotechnical Commission. (2023a). *Information technology—Artificial intelligence—Guidance on risk management* (ISO/IEC Standard No. 23894:2023). https://www.iso.org/standard/77304.html

International Organization for Standardization, & International Electrotechnical Commission. (2023b). *Information technology—Artificial intelligence—Management system* (ISO/IEC Standard No. 42001:2023). https://www.iso.org/standard/42001

National Institute of Standards and Technology. (2022). *Secure Software Development Framework (SSDF) version 1.1: Recommendations for mitigating the risk of software vulnerabilities* (NIST SP 800-218). https://doi.org/10.6028/NIST.SP.800-218

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2025). Learning to orchestrate agents in natural language with the Conductor. *arXiv*. https://arxiv.org/abs/2512.04388

OpenCode. (2026, August 4). *OpenCode 1.18.13* [Computer software release]. GitHub. https://github.com/anomalyco/opencode/releases/tag/v1.18.13

Sakana AI. (2026, June 22). *Sakana Fugu: One model to command them all*. https://sakana.ai/fugu-release/

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2025). TRINITY: An evolved LLM coordinator. *arXiv*. https://arxiv.org/abs/2512.04695

## Residual risk

A release record and HTTPS route prove a stronger ownership/transport boundary but not semantic correctness, provider commercial terms, or scientific validity. The verifier executes untrusted proposed code on an ephemeral hosted runner with ordinary egress. Artifact digests establish identity, not correctness. None of these controls authorizes TEPP to bypass an absent owner release, exact-head review, required workflows, or scientific acceptance gates.
