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
10. LLM live tests use `NVIDIA_NIM_API_KEY`. `COPILOT_GITHUB_TOKEN` is prohibited. Existing independent review-agent credentials must not be repurposed.
11. LLM orchestration allocates test-time computation between direct routing and deeper multi-agent workflows. Workflow depth, decomposition, access lists, recursion, role-specific reasoning effort, and ablations are recorded.
12. Database object names contain at least two words and use `snake_case` by default. CamelCase or PascalCase is permitted only where language conventions require it.
13. Every scientific or standards claim is traced to an authoritative primary source and cited in APA 7th style in `docs/research/`.
14. Changes that alter latent-variable meaning, temporal semantics, event ontology, multilingual invariance, or estimator targets require an ADR and PRD version change.

## Repository architecture

Use modular MSA boundaries. Each service or crate must work independently and through stable contracts when imported by CWL organization repositories, `naruon`, or `contextual-orchestrator`. Avoid hidden global state and repository-specific coupling.

## Pull-request loop

For every open PR:

1. inspect unresolved reviews and exact-head checks;
2. reproduce each actionable defect with a failing test;
3. implement the smallest scientifically and architecturally valid fix;
4. rerun focused and complete verification;
5. update ADRs, architecture, references, CHANGELOG, and manifests;
6. merge only after current-head required checks and independent approvals pass;
7. re-enumerate the queue and continue.

When the queue reaches zero, select one buyer-visible product gap, implement one bounded vertical slice, open a PR, and resume the same loop. Never bypass branch protection or claim queued checks have passed.

## Release contract

A release requires a clean PR queue, exact-head CI/security evidence, reproducible artifacts, SBOM and provenance, validated migrations, updated `CHANGELOG.md`, version consistency, rollback instructions, and no unresolved scientific or security blocker.
