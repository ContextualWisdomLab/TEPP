# ADR 0053 — Pareto candidate-`K` selection as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (candidate-`K` / Pareto gates) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already admits a unique `K` from a Pareto-filtered statistical
front inside `model_selection::select_candidate_k` and scores selected-`K`
RMSE against known truth. Operators still cannot request that gate as a
digest-bound analysis-run output. Schwarz fitted candidate-`K` selection is a
different profile. Joint Gauss-Newton Laplace draws are a different profile.
Topic activity/dormancy is a different profile. Full Bayesian sampling, GPU,
and topic birth/split/merge remain later GAP-004 work and are not this slice.

An LLM vote must not define the numerical optimum.

## Decision

Add the `pareto_candidate_k_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-constructed `ModelCandidate` values plus selected-`K`
  replications and known-truth `K`;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `select_candidate_k` and `selected_k_root_mean_square_error`
  without reimplementing Pareto dominance or RMSE;
- refuses LLM-vote-only authority and empty candidate sets;
- emits a canonical SHA-256-digested `tepp.pareto_candidate_k.v1` artifact
  with selected `K`, candidate/statistical counts, truth `K`, RMSE, and
  inference status `pareto_statistical_front_not_fitted_schwarz_sampler`;
- does not invent a Bayesian sampler, persist rows, select GPU backends, or
  emit topic-lineage edges.

This is Pareto-front statistical selection, not Schwarz fitted selection, not
joint Laplace plausible-value draws, and not a posterior sampler.

## Alternatives considered

1. Bind another Schwarz `select_fitted_candidate_k` profile — rejected
   because that bind is already live as a separate analysis-run profile.
2. Bind joint Gauss-Newton Laplace draws — rejected because that bind is
   already live as a separate analysis-run profile.
3. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main.
4. Bind topic activity/dormancy — already live as a separate analysis-run
   profile.
5. Bind the existing Pareto `select_candidate_k` gate to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe Pareto candidate-`K` as a digest-bound
terminal result. The artifact does not claim Schwarz fitted selection,
joint Laplace draws, Bayesian sampling, GPU parity, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, empty sets, and
LLM-only authority fail closed.

## Verification

The PR includes Rust unit and integration tests for smaller-`K` ties, higher
held-out likelihood, LLM-vote non-authority, empty sets, positive RMSE,
snapshot/profile/cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `pareto_candidate_k_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps Pareto
statistical selection distinct from LLM votes, Schwarz fitted selection,
joint Laplace draws, and Bayesian sampling.
