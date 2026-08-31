# House-voice style analysis-run composition

**Active slice:** ADR 0059 / `style_source_v1`
**Protected-main status:** not implemented-main

`style_source` already refuses to treat house-voice style residue as
unique latent content or as stopword deletion. This slice binds those
refusals to a cutoff-safe analysis-run profile so operators can request
a digest-bound identity artifact.

The artifact inference status is
`style_residue_is_not_unique_content_not_stopword_deletion`.
`identity_recovery_rate` stays library-side. This is not a copy-identity
census, not a simulation method-effect census, not GPU, not MCMC, and
not topic birth/split/merge.
