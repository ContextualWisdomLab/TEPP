# Inferred-status analysis-run composition

**Active slice:** ADR 0073 / `inferred_status_v1`
**Protected-main status:** not implemented-main

`inferred_status` already refuses to treat an inferred relation as
observed evidence or as a state transition. This slice binds
`EvidenceStatus`, `refuse_inferred_as_observed`, and
`refuse_inferred_as_transition` to a cutoff-safe analysis-run profile so
operators can request a digest-bound identity artifact.

The artifact inference status is
`inferred_is_not_observed_and_not_transition`.
`identity_recovery_rate` stays library-side. `unobserved` and
`no_relationship` are not wire statuses here. This is not
relation-absence, not episode-membership, not outcome-order, not
membership-target, not location-membership, not membership-posterior
ICC, not copied-text, not copy-identity, not citation-edge, not
subevent containment, not GPU, not MCMC, and not topic birth/split/merge.
