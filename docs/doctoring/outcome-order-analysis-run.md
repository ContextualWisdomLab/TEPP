# Input-process-outcome analysis-run composition

**Active slice:** ADR 0070 / `outcome_order_v1`
**Protected-main status:** not implemented-main

`outcome_order` already refuses reverse or contemporaneous event-time
rank on `input_to` and `process_to`, and refuses to treat `outcome_of`
as a state transition. This slice binds `OutcomeKind`,
`refuse_reverse_ipo_order`, and `refuse_outcome_of_as_transition` to a
cutoff-safe analysis-run profile so operators can request a digest-bound
identity artifact.

The artifact inference status is
`input_process_forward_outcome_of_is_not_transition`.
`kind_recovery_rate` stays library-side. This is not membership-target,
not location-membership, not membership-posterior ICC, not copied-text,
not copy-identity, not citation-edge, not GPU, not MCMC, and not topic
birth/split/merge.
