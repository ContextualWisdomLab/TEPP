# Corpus-background analysis-run composition

**Active slice:** ADR 0062 / `corpus_background_v1`
**Protected-main status:** not implemented-main

`corpus_background` already refuses to treat corpus-level background
wording as unique latent content or as stopword deletion. This slice
binds those refusals to a cutoff-safe analysis-run profile so operators
can request a digest-bound identity artifact.

The artifact inference status is
`corpus_background_is_not_unique_content_not_stopword_deletion`.
`identity_recovery_rate` stays library-side. This is not a modality-source
census, not prompt-source, not style-source, not copy-identity, not a
simulation method-effect census, not GPU, not MCMC, and not topic
birth/split/merge.
