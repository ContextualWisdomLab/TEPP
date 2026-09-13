# Template-copy identity analysis-run composition

**Active slice:** ADR 0058 / `copy_identity_v1`
**Protected-main status:** not implemented-main

`copy_identity` already refuses to treat a template copy as the source
document identity or as a state transition. This slice binds those
refusals to a cutoff-safe analysis-run profile so operators can request
a digest-bound identity artifact.

The artifact inference status is
`template_copy_is_not_source_identity_not_transition`.
`identity_recovery_rate` stays library-side. This is not a simulation
method-effect census, not GPU, not MCMC, and not topic birth/split/merge.
