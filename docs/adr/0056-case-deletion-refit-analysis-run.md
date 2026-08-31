# ADR 0056 — Exhaustive case-deletion refit as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0012 (producer-owned case-deletion influence) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already runs the same scientific fitter on the complete corpus
and on every actual `D \ {i}` corpus inside
`analysis_engine::fit_exhaustive_case_deletion`. Operators still cannot
request that runner as a digest-bound analysis-run output. Fitted
candidate-`K` selection, Pareto-front selection, composed fitted-lineage,
and topic activity remain different profiles. Full Bayesian sampling, GPU,
and topic birth/split/merge remain later GAP-004 work and are not this
slice.

Reweighting, a fixed posterior, or a diagonal approximation must not
replace an actual deleted-data fit.

## Decision

Add the `case_deletion_refit_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-constructed `CaseDeletionDocument` values, a seed-domain
  base, and an existing `CaseDeletionRefitter`;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `fit_exhaustive_case_deletion` without reimplementing leave-one-out
  fitting;
- emits a canonical SHA-256-digested `tepp.case_deletion_refit.v1` artifact
  with document count, deletion-refit count, independent seed-domain count,
  the full-fit seed domain, and inference status
  `exhaustive_actual_deletion_not_reweighting_approx`;
- keeps raw posteriors with the scientific fitter rather than copying them
  onto the operator artifact;
- refuses reuse of `composed_fitted_lineage_v1`, `fitted_candidate_k_v1`,
  `pareto_candidate_k_v1`, and `trsl_topic_lineage_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends, or
  emit topic birth/split/merge.

This is exhaustive actual deletion, not reweighting and not a posterior
sampler.

## Alternatives considered

1. Bind another fitted candidate-`K` or composed-lineage profile — rejected
   because those binds are already live as separate analysis-run profiles.
2. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main.
3. Copy raw posteriors onto the operator artifact — rejected because the
   fitter owns posterior meaning and the analysis-run contract stays
   identity-free and bounded.
4. Bind the existing exhaustive runner to ADR 0022's analysis-run profile —
   accepted.

## Consequences

Operators can request cutoff-safe exhaustive actual case-deletion as a
digest-bound terminal result. The artifact does not claim reweighting,
influence diagnostics, Bayesian sampling, GPU parity, or topic
birth/split/merge. Snapshot/profile/cutoff mismatch, invalid corpora, and
fitter refusal fail closed.

## Verification

The PR includes Rust unit and integration tests for successful exhaustive
counts, invalid corpora, fitter refusal, snapshot/profile/cutoff mismatch
including reuse of live sibling profiles, and artifact tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `case_deletion_refit_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps actual
deleted-data fits distinct from reweighting, fixed posteriors, and
Bayesian sampling.
