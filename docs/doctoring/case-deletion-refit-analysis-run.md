# Exhaustive case-deletion analysis-run composition

**Active slice:** ADR 0056 / `case_deletion_refit_v1`
**Protected-main status:** not implemented-main

`analysis_engine` already fits the complete corpus and every actual
`D \ {i}` corpus through `fit_exhaustive_case_deletion`. This slice binds
that runner to a cutoff-safe analysis-run profile so an operator can
request a digest-bound terminal result.

The executor refuses reweighting, a fixed posterior, and a diagonal
approximation as substitutes for an actual deleted-data fit. Raw posteriors
stay with the scientific fitter. It is not a Bayesian sampler, not GPU
execution, and not topic birth/split/merge.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
