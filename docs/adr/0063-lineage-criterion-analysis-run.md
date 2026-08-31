# ADR 0063 — Independent TDT link-criterion fitting as an analysis-run output profile

**Decision status:** Accepted
**Implementation maturity:** active-PR — composed on this branch; not implemented-main
**Date:** 2026-08-31
**Supersedes:** None; complements ADR 0023 (lineage-criterion anchor) and ADR 0022 (cutoff-safe analysis-run execution).
**Figma File ID:** N/A — this increment changes a Rust service crate and has no user-interface surface.
**Storybook inventory:** N/A — no reusable web object or interaction changed.

## Context

Protected main already fits independently observed TDT link-criterion
Jeffreys posteriors inside
`analysis_engine::fit_lineage_criterion_posteriors`. Event-time draws remain
producer evidence from the temporal model; the fitter does not infer a date
from record order or promote CHRONOS predictions to observed facts.
Operators still cannot request that runner as a digest-bound analysis-run
output.

Method-effect labels, template-copy refusals, house-voice refusals,
prompt-boilerplate refusals, non-lexical modality refusals, exhaustive
case-deletion, composed fitted-lineage, Pareto candidate-`K`, and topic
activity remain different profiles. Full Bayesian sampling, GPU, and topic
birth/split/merge remain later GAP-004 work and are not this slice. ADR
0058 through ADR 0061 are already taken by live sibling PRs.

## Decision

Add the `lineage_criterion_v1` analysis-run output profile to
`analysis_engine`. The executor:

- consumes already-constructed `LineageCriterionObservation` values and a
  common draw count;
- requires the request snapshot and knowledge cutoff to match the offered
  construction;
- invokes `fit_lineage_criterion_posteriors` without reimplementing Jeffreys
  fitting;
- emits a canonical SHA-256-digested `tepp.lineage_criterion.v1` artifact
  with pair count, draw count, and inference status
  `independent_tdt_criterion_not_date_from_record_order`;
- keeps raw posteriors and pair identities with the scientific fitter rather
  than copying them onto the operator artifact;
- refuses reuse of `case_deletion_refit_v1`, `composed_fitted_lineage_v1`,
  `fitted_candidate_k_v1`, `pareto_candidate_k_v1`, `trsl_topic_lineage_v1`,
  and `method_effects_v1` as this profile;
- does not invent a Bayesian sampler, persist rows, select GPU backends,
  infer dates from record order, or emit topic birth/split/merge.

This is independent TDT link-criterion fitting, not a date inference and
not a posterior sampler.

## Alternatives considered

1. Bind another method-effect, case-deletion, or composed-lineage profile —
   rejected because those binds are already live as separate analysis-run
   profiles.
2. Invent a Bayesian sampler or topic birth/split/merge engine — rejected
   because those functions do not exist on protected main.
3. Copy raw posteriors or pair identities onto the operator artifact —
   rejected because the fitter owns posterior meaning and the analysis-run
   contract stays identity-free and bounded.
4. Bind the existing independent-criterion runner to ADR 0022's
   analysis-run profile — accepted.

## Consequences

Operators can request cutoff-safe independent TDT link-criterion fitting as
a digest-bound terminal result. The artifact does not claim date inference
from record order, CHRONOS promotion, Bayesian sampling, GPU parity, or
topic birth/split/merge. Snapshot/profile/cutoff mismatch, invalid
observations, and fitter refusal fail closed.

## Verification

The PR includes Rust unit and integration tests for successful pair/draw
counts, empty or invalid observations, criterion refusal, snapshot/profile/
cutoff mismatch including reuse of live sibling profiles, and artifact
tampering. Run:

```text
cargo fmt --all -- --check
cargo test -p analysis_engine
cargo clippy -p analysis_engine --all-targets -- -D warnings
python3 scripts/validate_documentation.py
```

## Rollback and supersession

Rollback removes the `lineage_criterion_v1` profile. No persisted schema
migration is introduced. Supersede only with an ADR that keeps independent
TDT link-criterion fitting distinct from date inference, CHRONOS
promotion, and Bayesian sampling.
