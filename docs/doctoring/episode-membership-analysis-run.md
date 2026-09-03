# Episode-membership analysis-run composition

**Active slice:** ADR 0072 / `episode_membership_v1`
**Protected-main status:** not implemented-main

`episode_membership` already refuses a membership window that starts
before or ends after its episode. This slice binds `EventWindow` and
`refuse_membership_outside_episode` to a cutoff-safe analysis-run
profile so operators can request a digest-bound identity artifact.

The artifact inference status is
`membership_window_cannot_escape_episode_interval`.
`identity_recovery_rate` stays library-side. This is membership-window
containment, not subevent-versus-parent containment. This is not
relation-absence, not outcome-order, not membership-target, not
location-membership, not membership-posterior ICC, not copied-text, not
copy-identity, not citation-edge, not GPU, not MCMC, and not topic
birth/split/merge.
