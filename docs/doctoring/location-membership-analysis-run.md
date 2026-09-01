# Location-membership analysis-run composition

**Active slice:** ADR 0066 / `location_membership_v1`
**Protected-main status:** not implemented-main

`location_membership` already refuses to treat location membership as
permanent entity identity or as a language channel. This slice binds
those refusals to a cutoff-safe analysis-run profile so operators can
request a digest-bound location-membership refusal artifact. Every input
document carries a validated availability clock; post-cutoff evidence and
corpora above the shared 100,000-document execution bound fail closed before
counting. The terminal summary reports the artifact's five census statistics.

The artifact inference status is
`location_is_not_entity_identity_not_language_channel`.
`identity_recovery_rate` stays library-side. This is not
membership-posterior ICC, not copied-text, not citation-edge, not
corpus-background, not GPU, not MCMC, and not topic birth/split/merge.
