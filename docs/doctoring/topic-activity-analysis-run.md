# Topic activity/dormancy/reactivation analysis-run composition

**Active slice:** ADR 0051 / `topic_activity_v1`
**Protected-main status:** not implemented-main

`topic_lineage` already keeps a global P0 topic identity across activity,
dormancy, and reactivation. This slice binds that contract to a cutoff-safe
analysis-run profile so an operator can request a digest-bound terminal
result.

The executor refuses reminted reactivation identities and illegal activity
transitions. It is not a Bayesian sampler, not GPU execution, not topic
birth/split/merge, and not the fitted `trsl_topic_lineage_v1` sequence-edge
profile.

Exact-head Checks and two independent approvals are required before any
implemented-main claim.
