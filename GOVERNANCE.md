# TEPP Governance

## Decision hierarchy

1. User and organization policy.
2. Approved PRD and versioned acceptance criteria.
3. Architecture decision records.
4. `AGENTS.md`, `ARCHITECTURE.md`, `SECURITY.md`, and contribution contracts.
5. Implementation plans and issue-level specifications.
6. Code and tests.

When code or tests conflict with a higher-level contract, the implementation is corrected rather than silently redefining the contract.

## Required decisions

An ADR and PRD version change are required for changes to:

- the meaning of latent topics, factors, states, events, or scores;
- temporal clocks, interval logic, leakage policy, or forward-transition semantics;
- multilingual shared-space or measurement-invariance assumptions;
- reflective, formative, network, ESEM, DSEM, or causal interpretations;
- model-selection objectives or LLM judge authority;
- production numerical precision, backend parity, or VRAM fallback guarantees;
- tenant, privacy, audit, or evidence-provenance boundaries.

## Pull-request governance

No author may self-certify a protected merge gate. The current head must satisfy required checks and independent approval. A prior head’s success, local-only result, queued job, or automation assertion is not a substitute.

Duplicate or superseded PRs are closed with an explicit reason. Valid blocked PRs remain active, receive bounded fixes, and use auto-merge only when repository policy permits. Waiting for reviews or checks does not halt useful work on independent, non-conflicting tasks.

## Scientific review

Every estimator or scoring method identifies:

- construct and estimand;
- data-generating assumptions;
- hierarchy, multiple membership, and time structure;
- identification and invariance conditions;
- uncertainty propagation;
- primary-source traceability;
- simulation design and acceptance region;
- known failure modes and prohibited interpretations.

Novel methods are labeled as novel and compared with strong baselines; they are not presented as established because an LLM produced a plausible explanation.

## Release governance

A release is approved only with a clean or explicitly waived PR queue, exact-head checks, version and CHANGELOG alignment, migration and rollback evidence, reproducible packages, SBOM and provenance, security review, validated language and population profiles, and documented operating limits.
