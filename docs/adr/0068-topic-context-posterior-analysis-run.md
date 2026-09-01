# ADR 0068 — Posterior topic-context producer as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-09-01
**Supersedes:** None; complements ADR 0022 (cutoff-safe analysis-run execution) and ADR 0024 (posterior topic-context producer contract).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already validates digest-bound posterior topic-context
artifacts inside `analysis_engine::TopicContextPosteriorArtifact`. The
producer contract keeps full-rank logistic-normal coordinates, refuses
collapsed missing draws, and labels the claim boundary
`posterior_topic_coordinates_not_importance`. Operators still cannot
request that validator as a cutoff-safe analysis-run output.

Independent TDT link-criterion fitting, location-membership refusals,
copied-text residue refusals, provenance-is-not-transition refusals, and
composed fitted-lineage remain different profiles. Full Bayesian sampling,
GPU, and invented topic birth/split/merge remain later GAP-004 work and
are not this slice. ADR 0064 through ADR 0067 are already taken by live
sibling PRs.

## Decision

Add the `topic_context_posterior_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes an already-constructed `TopicContextPosteriorArtifact`;
- requires the request snapshot, knowledge cutoff, and accepted run
  identity to match the offered artifact;
- invokes the existing producer `sha256`/validate path without
  reimplementing TRSL-TM fitting;
- emits a digest-bound terminal result under
  `tepp.topic_context_posterior.v1` with inference status
  `posterior_topic_coordinates_not_importance`;
- refuses reuse of `lineage_criterion_v1`, `case_deletion_refit_v1`,
  `composed_fitted_lineage_v1`, `fitted_candidate_k_v1`,
  `trsl_topic_lineage_v1`, and `method_effects_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends,
  infer topic importance, or emit invented birth/split/merge events.
  Lineage events remain producer-supplied.

This is posterior topic coordinates, not importance and not a sampler.

## Alternatives considered

1. Bind another refusal or lineage-criterion profile — rejected because
   those binds are already live as separate analysis-run profiles.
2. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main as executors.
3. Collapse missing draws into a point estimate — rejected because the
   producer contract already fails closed on incomplete draw sets.
4. Bind the existing producer validator to ADR 0022's analysis-run
   profile — accepted.

## Consequences

Operators can request cutoff-safe posterior topic-context validation as a
digest-bound terminal result. The artifact does not claim topic
importance, Bayesian sampling, GPU parity, or invented birth/split/merge.
Snapshot/profile/cutoff mismatch and producer-contract refusal fail
closed.

## Verification

The PR includes Rust integration tests for successful digest-bound
coordinates, incomplete draw refusal, run-identity mismatch,
snapshot/profile/cutoff mismatch including reuse of live sibling
profiles. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `topic_context_posterior_v1` profile. No persisted
schema migration is introduced. Supersede only with an ADR that keeps
posterior coordinates distinct from importance, sampling, and invented
lineage events.
