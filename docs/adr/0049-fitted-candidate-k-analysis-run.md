# ADR 0049 — Fitted candidate-`K` selection as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (TRSL-TM / candidate-`K` gates) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already fits each candidate `K` with the CPU `f64` TRSL-TM
reference and scores Schwarz's (1978) `ℓ − (p ln N)/2` inside
`model_selection::select_fitted_candidate_k`. Operators still cannot request
that selection as a digest-bound analysis-run output. Recovery of a single
fixed-`K` topic-lineage artifact is a different profile. Full Bayesian
sampling, GPU, method effects, and topic birth/split/merge remain later
GAP-004 work and are not this slice.

An LLM vote must not define the numerical optimum. Lexical TF-IDF/BM25,
stopword deletion, and LLM labels remain forbidden inferential coordinates.

## Decision

Add the `fitted_candidate_k_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-validated `ReferenceTopicInput` plus
  `FittedCandidateKConfig`;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- invokes `select_fitted_candidate_k` without reimplementing Schwarz scoring
  or the reference fit;
- refuses lexical methods and refuses LLM-vote-only authority;
- emits a canonical SHA-256-digested `tepp.fitted_candidate_k.v1` artifact
  with selected `K`, candidate count, evidence count, and inference status
  `fitted_schwarz_candidate_k_not_bayesian_sampler`;
- does not invent a Bayesian sampler, persist rows, select GPU backends, or
  emit topic-lineage edges.

This is statistical candidate-`K` selection, not a posterior sampler and not
the `trsl_topic_lineage_v1` fixed-`K` lineage profile.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind candidate-`K`
   selection to an analysis run.
2. Bind GAP-013 interpreter/verifier — rejected because that is a different
   orchestrator/LLM port, not model-selection authority.
3. Put candidate-`K` selection into `tepp_api` — rejected because transport
   contracts and scientific composition would become one service boundary.
4. Bind the existing `model_selection` fitted candidate-`K` selector to
   ADR 0022's analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe fitted candidate-`K` as a digest-bound
terminal result. The artifact does not claim Bayesian sampling, GPU parity,
or topic birth/split/merge. Snapshot/profile/cutoff mismatch, lexical
methods, failed fits, and LLM-only authority fail closed.

## Verification

The PR includes Rust unit and integration tests for true-`K` recovery on a
separated two-topic corpus, LLM-vote non-authority, lexical refusal, failed
fits, snapshot/profile/cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `fitted_candidate_k_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps Schwarz
fitted selection distinct from LLM votes, lexical weights, and Bayesian
sampling.
