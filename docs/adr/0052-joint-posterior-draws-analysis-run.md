# ADR 0052 — Joint posterior Laplace draws as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (TRSL-TM joint precision / plausible values) and ADR 0022 (cutoff-safe analysis-run execution). Does not reuse ADR 0049 (fitted candidate-`K`), ADR 0050 (interpreter/verifier), or ADR 0051 (topic activity/dormancy).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already materializes deterministic joint Gaussian Laplace
plausible values from an identified Gauss-Newton precision via
`JointCoordinatePrecision::draw_joint_gaussian` (Philox4x32-10, Box-Muller,
Cholesky). Operators still cannot request that draw set as a digest-bound
analysis-run output. Fitted candidate-`K` (#404 / ADR 0049) is Schwarz
selection and explicitly not a sampler. Fixed-`K` `trsl_topic_lineage_v1`
emits predecessor/successor edges, not draws. The
`tepp.topic_context_posterior.v1` producer contract remains DTO-only.

GPU kernels, method effects, MCMC, and topic birth/split/merge remain later
GAP-004 work and are not this slice.

## Decision

Add the `joint_posterior_draws_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-validated `ReferenceTopicInput` plus
  `ReferenceTopicModelConfig` and caller-owned topic identities;
- requires the request snapshot and knowledge cutoff to match the offered
  input construction;
- fits the CPU `f64` TRSL-TM reference, builds the joint Laplace precision,
  and draws through `draw_joint_gaussian` without reimplementing Philox,
  Box-Muller, or Cholesky;
- emits a canonical SHA-256-digested `tepp.joint_posterior_draws.v1` artifact
  with `draw_set_id`, algorithm version, seed, draw/document/topic counts, and
  inference status `joint_gaussian_laplace_plausible_values_not_mcmc`;
- does not persist draw coordinates on the operator artifact (the draw-set
  digest already binds them), invent MCMC, select GPU backends, score
  candidate `K`, or emit topic birth/split/merge events.

This is Laplace plausible-value materialization, not MCMC and not Schwarz
candidate-`K` selection.

## Alternatives considered

1. Restore another Driver p.16 standardised matrix — rejected because those
   recoveries are already a live micro-PR family and do not bind posterior
   draws to an analysis run.
2. Duplicate fitted candidate-`K` (#404) — rejected because that profile
   explicitly refuses to be a sampler.
3. Emit the full `tepp.topic_context_posterior.v1` producer contract —
   rejected because that DTO also requires membership, activity, and lineage
   events this slice does not fit.
4. Bind the existing joint Laplace draw generator to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe joint Laplace draws as a digest-bound
terminal result. The artifact does not claim MCMC, GPU parity, method
effects, or topic birth/split/merge. Snapshot/profile/cutoff mismatch, failed
fits, and zero/oversized draw counts fail closed.

## Verification

The PR includes Rust unit and integration tests for digest-bound draws on a
separated two-topic corpus, zero-draw refusal, non-convergence, snapshot /
profile / cutoff mismatch, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `joint_posterior_draws_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps Laplace
plausible values distinct from MCMC, Schwarz candidate-`K`, and GPU kernels.
