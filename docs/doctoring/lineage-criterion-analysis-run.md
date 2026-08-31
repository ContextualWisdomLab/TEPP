# Independent TDT link-criterion analysis-run composition

**Active slice:** ADR 0063 / `lineage_criterion_v1`
**Protected-main status:** not implemented-main

`analysis_engine` already fits independently observed TDT link-criterion
Jeffreys posteriors through `fit_lineage_criterion_posteriors`. Event-time
draws remain producer evidence. This slice binds that runner to a
cutoff-safe analysis-run profile so an operator can request a digest-bound
terminal result.

The executor does not infer a date from record order and does not promote
CHRONOS predictions to observed facts. Raw posteriors stay with the
scientific fitter. It is not a Bayesian sampler, not GPU execution, and not
topic birth/split/merge.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
