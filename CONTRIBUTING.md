# Contributing to TEPP

## Before starting

Read `AGENTS.md`, `ARCHITECTURE.md`, the approved PRD, relevant ADRs, and research references. A change is not ready merely because it compiles.

## Change workflow

1. Choose one bounded, buyer-visible or scientifically necessary vertical slice.
2. Create an isolated branch or worktree.
3. Write the failing unit, property, integration, simulation, or recovery test first.
4. Confirm that the test fails for the intended reason.
5. Implement the smallest valid change.
6. Run focused tests, then complete verification.
7. Update documentation, APA 7th references, ADRs, model manifests, schemas, migrations, and `CHANGELOG.md`.
8. Open a pull request with source, assumptions, numerical tolerances, risks, rollback, and verification evidence.
9. Address every actionable review thread and rerun checks on the resulting head.
10. Merge only when required current-head checks and independent approvals pass.

## Required verification

As applicable, a PR must provide fresh evidence for:

- Rust formatting, linting with warnings denied, compilation, and complete tests;
- production line and branch coverage at 100%;
- complete public API and safety-contract docstrings;
- deterministic seeds and reproducibility manifest;
- true-parameter RMSE, bias, interval coverage, convergence, and failure-rate studies;
- temporal leakage and partial-order invariants;
- multilevel/multiple-membership recovery and atomistic-fallacy safeguards;
- multilingual alignment and measurement-invariance checks;
- CPU `f64` versus parallel CPU/GPU numerical parity;
- GPU execution without skipped tests and measured peak VRAM/fallback behavior;
- database migrations, constraints, rollback, and two-word object naming;
- dependency licenses, advisories, action pins, SBOM, provenance, and package/install smoke tests;
- accessibility, exact-value tables, no-JavaScript/print exports, and visual regression where relevant.

Monte Carlo acceptance thresholds must include Monte Carlo uncertainty rather than comparing a finite observed rate directly with its nominal target.

## Research and documentation

Use primary papers, international standards, official specifications, and official library documentation. Record APA 7th references in `docs/research/standards-and-literature.md` and link methodological claims to the exact section, equation, or requirement they support. Clearly distinguish replicated published methods, adaptations, and novel TEPP methods.

## LLM development

- Treat all model output as untrusted structured input.
- Preserve exact source spans and evidence identifiers.
- Route every semantic LLM operation and model-backed GitHub Actions workflow through a released, versioned `contextual-orchestrator` contract. GitHub Actions use only `orchestrator/free` through the contextual-orchestrator gateway credential.
- Do not select or hard-code a provider, model, provider group, or paid fallback in TEPP, and do not expose provider API keys to TEPP workflows. If a released orchestrator contract cannot provide the required capability, fail closed and repair the canonical owner before adopting the change here.
- Never use or introduce `COPILOT_GITHUB_TOKEN`, and never repurpose independent review-agent credentials as execution credentials.
- Record the contextual-orchestrator release/contract identity, route, prompt hash, reasoning effort, workflow depth, tools/access list, seed where supported, latency, token usage, and cost. Record provider/model identity only when the released orchestrator returns it as execution provenance; it is evidence, not TEPP routing authority.
- Include direct-routing versus orchestrated and reasoning-effort ablations where scientifically relevant. LLM output never replaces numerical estimation or scientific acceptance.

## Database naming

Database objects use at least two words and `snake_case` by default, such as `document_record`, `event_instance`, `topic_definition`, and `audit_event`. Single-word object names are rejected.
