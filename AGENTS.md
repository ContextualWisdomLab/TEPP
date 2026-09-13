# AGENTS.md

## Mission

TEPP is the Temporal Event Psychometrics Platform: a multilingual, temporal, relational measurement system that combines evidence-grounded language processing, event ontology, topic measurement, TDT/CHRONOS-style event reasoning, and longitudinal psychometrics.

## Non-negotiable engineering contracts

1. Production psychometric and mathematical arithmetic is implemented in Rust.
2. Every estimator has a CPU `f64` reference path. Parallel CPU and GPU paths must demonstrate numerical parity against it.
3. GPU execution is VRAM-budgeted, streamed, and able to fall back safely to CPU. OOM is an expected state, not an unhandled exception.
4. Temporal modeling distinguishes event/valid time, assertion time, document time, system time, availability time, and knowledge cutoff. No analysis may use evidence whose availability time exceeds its cutoff.
5. Forward state-transition and input-process-outcome edges never move backward in event time. Citation, revision, translation, and retrospective-reporting edges may point to the past but never become reverse state transitions.
6. Models must support multilevel, cross-classified, and multiple-membership structures. Documents may simultaneously belong to authors, departments, customers, partners, competitors, projects, opportunity pools, templates, languages, and event episodes.
7. Multilingual measurement uses one shared latent semantic space. Language-specific morphology and lexical emissions may vary, but equivalent meanings must be aligned and tested for measurement invariance.
8. Production line and branch coverage are 100%. All public modules, traits, structs, enums, functions, methods, error variants, configuration fields, and safety contracts have complete docstrings.
9. Scientific acceptance requires realistic synthetic truth: parameter recovery, RMSE, bias, interval coverage, temporal ordering, graph recovery, invariance, and CPU/GPU parity. Skipped or ignored GPU tests are not evidence.
10. Every semantic LLM operation and every model-backed GitHub Actions workflow goes through a released, versioned `contextual-orchestrator` contract. Actions use the `orchestrator/free` route through the gateway credential only; TEPP must not select a provider/model/group, declare a paid fallback, call providers directly, or consume provider API keys such as NVIDIA NIM, OpenRouter, OpenAI, or Bytez credentials. If the released orchestrator contract cannot supply the required capability, fail closed and repair the canonical owner before consumer adoption. `COPILOT_GITHUB_TOKEN` is prohibited. Independent review-agent credentials must not be repurposed as execution credentials.
11. LLM orchestration allocates test-time computation between direct routing and deeper multi-agent workflows. Workflow depth, decomposition, access lists, recursion, role-specific reasoning effort, verification/adjudication, and comparable-budget ablations are recorded. LLM output never replaces deterministic/statistical scientific authority. Model timeout defaults must not terminate reasoning/stream/tool-call work merely because elapsed time is long; user cancellation, provider termination, and explicit administrative limits remain distinct outcomes.
12. Database object names contain at least two words and use `snake_case` by default. CamelCase or PascalCase is permitted only where language conventions require it.
13. Every scientific or standards claim is traced to an authoritative primary source and cited in APA 7th style in `docs/research/`.
14. Changes that alter latent-variable meaning, temporal semantics, event ontology, multilingual invariance, estimator targets, privacy authority, or service authority require an ADR and a PRD version change when the approved product/measurement target changes.
15. Do not blanket-mask PII when doing so destroys valid authorship, temporal, longitudinal, event, entity-role, or multiple-membership measurement. Use purpose-bound authorization, opaque analytical identifiers, separately protected identity mapping, encryption, selective disclosure, retention/deletion, and auditable privileged access.
16. Design toward CSAP and SOC 2 evidence readiness and align AI governance with current published ISO/NIST guidance where applicable, but never claim certification, attestation, conformance, or legal sufficiency without external evidence.
17. Preserve standalone operation and modular MSA composition. `naruon`, `contextual-orchestrator`, and other CWL services integrate only through released/versioned APIs or immutable artifacts plus explicit ACLs; no direct cross-service application-table access or mutable sibling-head dependency is permitted.
18. Documents, external metadata, serialized payloads, model checkpoints, and LLM outputs are untrusted until their owning boundary validates identity, provenance, size/depth, authorization, and scientific semantics.
19. Figma/Product Design becomes authoritative only for a stable product interaction contract; UI design never overrides the PRD, data model, numerical/scientific contract, or protected-main implementation truth.

## Repository architecture

Use modular MSA boundaries. Each service or crate must work independently and through stable contracts when imported by CWL organization repositories, `naruon`, or `contextual-orchestrator`. Avoid hidden global state and repository-specific coupling. Read `DOCUMENTATION.md` for the canonical product/technical/security/privacy/API/operability/traceability graph.

## Pull-request and autonomous execution loop

For every open PR:

1. inspect unresolved reviews and exact-head checks;
2. reproduce each actionable defect with a failing test where applicable;
3. implement the smallest scientifically and architecturally valid fix;
4. rerun focused and complete verification;
5. update ADRs, architecture, references, CHANGELOG, manifests, and canonical documentation when affected;
6. merge only after current-head required checks and qualifying independent approvals pass;
7. re-enumerate the queue and continue.

A merge, review request, documentation update, queued check, blocked PR, or one completed product slice is an intermediate state while another safe action exists. Waiting on one branch blocks only that branch. After every mutation, merge, or defer decision, rebuild the executable queue. When the PR/issue queue reaches zero, select the highest-impact bounded buyer-visible, scientific-validity, security, privacy, operability, or ecosystem gap, implement it, open/review/merge its PR, then continue. Never bypass branch protection or claim queued/stale evidence has passed.

## Release contract

A release requires a clean integration state, exact protected-head CI/security evidence, scientific/recovery acceptance appropriate to changed capabilities, reproducible artifacts, SBOM and provenance, validated migrations/rollback/recovery where present, updated `CHANGELOG.md`, version consistency, accessibility/operability evidence for user-facing components, and no unresolved scientific, privacy, security, or supply-chain blocker.
