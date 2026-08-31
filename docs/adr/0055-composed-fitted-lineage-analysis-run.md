# ADR 0055 — Fitted candidate-`K` composed with topic lineage as an analysis-run profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (fitted candidate-`K` and the CPU `f64` reference) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already fits each candidate `K` through
`model_selection::select_fitted_candidate_k` and already emits digest-bound
topic-lineage artifacts through `execute_topic_lineage_run`. Operators still
cannot request those two existing functions as one cutoff-safe analysis-run
profile. Standalone fitted candidate-`K` selection, Pareto-front selection,
joint Laplace draws, and topic activity remain different profiles. Full
Bayesian sampling, GPU, and topic birth/split/merge remain later GAP-004
work and are not this slice.

An LLM vote must not define the numerical optimum.

## Decision

Add the `composed_fitted_lineage_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-constructed `ReferenceTopicInput` plus
  `FittedCandidateKConfig`, method name, and optional LLM votes;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `select_fitted_candidate_k` then `execute_topic_lineage_run` at the
  selected `K` without reimplementing Schwarz scoring or lineage edges;
- emits a canonical SHA-256-digested `tepp.composed_fitted_lineage.v1`
  artifact with selected `K`, candidate/evidence counts, lineage topic/edge
  counts, the inner lineage digest, and inference status
  `fitted_k_composed_lineage_not_bayesian_sampler`;
- refuses reuse of `fitted_candidate_k_v1`, `pareto_candidate_k_v1`,
  `trsl_topic_lineage_v1`, and `joint_posterior_draws_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends, or
  emit topic birth/split/merge.

This is end-to-end composition of existing selection and lineage functions,
not a second Schwarz-only bind and not a posterior sampler.

## Alternatives considered

1. Bind another standalone `select_fitted_candidate_k` profile — rejected
   because that bind is already live as a separate analysis-run profile.
2. Bind another standalone topic-lineage profile — rejected because that
   executor is already on protected main.
3. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main.
4. Compose fitted selection with the existing topic-lineage executor under
   ADR 0022 — accepted.

## Consequences

Operators can request cutoff-safe fitted-`K` selection followed by a
production lineage fit as one digest-bound terminal result. The artifact
does not claim Schwarz-only selection, Pareto-front selection, Bayesian
sampling, GPU parity, or topic birth/split/merge. Snapshot/profile/cutoff
mismatch, lexical methods, and failed selection fail closed.

## Verification

The PR includes Rust unit and integration tests for successful composition,
lexical-method refusal, snapshot/profile/cutoff mismatch including reuse of
live sibling profiles, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `composed_fitted_lineage_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
fitted selection distinct from LLM votes, Pareto-front selection, and
Bayesian sampling, and keeps lineage association distinct from causation.
