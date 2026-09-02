# Subevent-containment analysis-run composition

**Active slice:** ADR 0074 / `subevent_containment_v1`
**Protected-main status:** not implemented-main

`subevent_containment` already refuses a half-open child interval that
starts before or ends after its parent. This slice binds `EventInterval`
and `refuse_escaped_subevent` to a cutoff-safe analysis-run profile so
operators can request a digest-bound identity artifact.

The artifact inference status is
`subevent_interval_cannot_escape_parent_interval`.
`containment_recovery_rate` and `identity_recovery_rate` stay
library-side. This is subevent-versus-parent containment, not
episode-membership-window containment. This is not inferred-status, not
relation-absence, not outcome-order, not membership-target, not
location-membership, not membership-posterior ICC, not copied-text, not
copy-identity, not citation-edge, not GPU, not MCMC, and not topic
birth/split/merge.
