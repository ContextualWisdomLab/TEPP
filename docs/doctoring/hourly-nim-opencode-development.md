# Hourly Contextual Orchestrator OpenCode Development — Evidence Doctoring

## Claim boundary

This workflow can propose one bounded pull request only after both the pull-request
and issue queues are empty. It does not claim autonomous
semantic correctness, production safety, customer acceptance, scientific
validity, or permission to merge, release, deploy, or approve. Deterministic
tests, schemas, security scanners, CodeRabbit/OpenCode review, human judgment,
and exact-head branch rules remain independent controls.

## Source-supported facts and project decisions

| Topic | Source-supported fact | TEPP decision |
|---|---|---|
| GitHub jobs | Jobs run on separate runner instances and receive scoped tokens | Model, verifier, and publisher are separate jobs |
| GitHub App authentication | A workflow can mint an installation token with explicit repository permissions | Publication token is minted only after non-executing validation |
| Artifact handoff | Artifact upload/download actions expose immutable IDs and digests | Patch ID, digest, base, size, count, and modes are checked twice |
| OpenCode | OpenCode is a programmable coding agent with provider configuration | One checksum-pinned binary calls only the loopback gateway |
| contextual-orchestrator | The pinned gateway discovers configured provider models, filters general-chat eligibility, and routes through its KV credential seam | All five provider keys are registered at bootstrap; endpoint-only models stay out of chat routing; OpenCode receives only a loopback token |
| Provider APIs | OpenAI-compatible and provider-specific model-list APIs expose discoverable models | Discovery is attempted for OpenAI, OpenRouter, both NVIDIA NIM keys, and Bytez before selection |
| SSDF | NIST SP 800-218 recommends protected build environments, review, provenance, and vulnerability response | Fresh verification and ordinary PR governance remain mandatory |
| AI risk | ISO/IEC 23894:2023 and 42001:2023 require contextual risk treatment and controlled change | Stable no-op reasons, bounded proposals, traceable decisions, and rollback are documented |
| Test-time compute | Fugu, Conductor, and TRINITY distinguish routing from deeper role-based workflows | Runtime LLM increments must preserve route/conduct and access-list controls |

The GitHub, OpenCode, NVIDIA, NIST, ISO, and paper references below follow APA 7
conventions as closely as the source type permits.

## Upstream version evidence

The official OpenCode GitHub release API identified OpenCode 1.18.13 as the
current upstream release on 2026-08-06. The workflow deliberately pins OpenCode 1.17.13 because its Linux x64 archive SHA-256
`157afa289d1a8d9372de0ce19ac726119b937a1f6b201808d46f06e4e59bb348`
had already been independently reviewed in the CWL Noema workflow. The newer
archive is not adopted until its exact asset digest is independently captured,
reviewed, committed, and exercised. “Latest” is not allowed to mean
“unverified.”

This is a supply-chain project decision, not a claim that OpenCode 1.17.13 is
functionally superior. The scheduled agent cannot auto-update itself.

## Orchestration research application

**Fugu.** Fugu frames orchestration as selecting between direct model use and
deeper coordinated execution. TEPP already exposes explicit
`auto|route|conduct` organization modes; the autonomous prompt requires future
LLM changes to preserve this distinction.

**Conductor.** Conductor generates natural-language subtasks, worker assignment,
and access lists. The prompt therefore requires explicit workflow stages,
dependencies, least-privilege evidence access, and bounded recursive depth.

**TRINITY.** TRINITY emphasizes specialized thinker, worker, verifier, and
synthesis roles. The prompt requires role-specific reasoning effort and
reasoning-level ablation rather than one undifferentiated maximum-effort call.

These papers motivate evaluation hypotheses. They do not prove that deeper
orchestration always improves TEPP reports. Deterministic grounding,
latency-insensitive quality tests, provider-reported usage where available, and
human review must compare forced routing and conducted cells.

## APA 7th references

GitHub. (n.d.). *Security hardening for GitHub Actions*. GitHub Docs. Retrieved
August 6, 2026, from
https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions

GitHub. (n.d.). *Making authenticated API requests with a GitHub App in a GitHub
Actions workflow*. GitHub Docs. Retrieved August 6, 2026, from
https://docs.github.com/en/apps/creating-github-apps/writing-code-for-a-github-app/making-authenticated-api-requests-with-a-github-app-in-a-github-actions-workflow

ContextualWisdomLab. (2026). *contextual-orchestrator* (commit
e226e1197bdfc890c9d8e5b9b648c78857d7e465) [Computer software]. GitHub.
https://github.com/ContextualWisdomLab/contextual-orchestrator

International Organization for Standardization, & International Electrotechnical
Commission. (2023a). *Information technology—Artificial intelligence—Guidance
on risk management* (ISO/IEC Standard No. 23894:2023).
https://www.iso.org/standard/77304.html

International Organization for Standardization, & International Electrotechnical
Commission. (2023b). *Information technology—Artificial
intelligence—Management system* (ISO/IEC Standard No. 42001:2023).
https://www.iso.org/standard/42001

National Institute of Standards and Technology. (2022). *Secure Software
Development Framework (SSDF) version 1.1: Recommendations for mitigating the
risk of software vulnerabilities* (NIST SP 800-218).
https://doi.org/10.6028/NIST.SP.800-218

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2025).
Learning to orchestrate agents in natural language with the Conductor. *arXiv*.
https://arxiv.org/abs/2512.04388

NVIDIA. (n.d.). *NVIDIA NIM APIs*. NVIDIA API Catalog. Retrieved August 6, 2026,
from https://build.nvidia.com/

OpenCode. (2026, August 4). *OpenCode 1.18.13* [Computer software release].
GitHub. https://github.com/anomalyco/opencode/releases/tag/v1.18.13

Sakana AI. (2026, June 22). *Sakana Fugu: One model to command them all*.
https://sakana.ai/fugu-release/

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2025).
TRINITY: An evolved LLM coordinator. *arXiv*.
https://arxiv.org/abs/2512.04695

## Tooling limitations recorded

Consensus search was requested for this increment but did not return a usable
paper record. Context7 reported its monthly quota exhausted. Primary paper
records, official documentation, and the existing Contextual Orchestrator
doctoring were therefore used directly. This limitation does not weaken the
deterministic workflow tests and must be revisited in a later literature update.

## Residual risk

Provider keys exist inside the ephemeral gateway process during bootstrap,
proposed code is executed by an uncredentialed verifier with ordinary
hosted-runner egress, and artifact digests do not establish semantic safety. A
later broker can narrow provider egress; a later central reusable workflow can
remove duplicated repository policy. Neither future improvement may combine
model execution with publication authority.
