# Relation-absence analysis-run composition

**Active slice:** ADR 0071 / `relation_absence_v1`
**Protected-main status:** not implemented-main

`relation_absence` already refuses to treat an unobserved pair as
evidence of no relationship. This slice binds `ObservationStatus` and
`refuse_absence_as_negative` to a cutoff-safe analysis-run profile so
operators can request a digest-bound identity artifact.

The artifact inference status is
`unobserved_is_not_negative_observed_inferred_are_presence`.
`status_recovery_rate` stays library-side. `no_relationship` is not a
wire status. This is not outcome-order, not membership-target, not
location-membership, not membership-posterior ICC, not copied-text, not
copy-identity, not citation-edge, not GPU, not MCMC, and not topic
birth/split/merge.
