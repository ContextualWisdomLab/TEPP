# Corpus-background analysis-run composition

**Active slice:** ADR 0062 / `corpus_background_v1`
**Decision status:** Proposed
**Protected-main status:** not implemented-main

`corpus_background` already refuses to treat corpus-level background wording as
unique latent content or as stopword deletion. This Draft binds those refusals
to a digest-bound Analysis Run profile without taking ownership of the domain
vocabulary.

The historical census is leakage-safe only when source provenance is explicit.
`CorpusBackgroundDocument` therefore requires both immutable `snapshot_id` and
`AvailableTime`. Documents from another snapshot fail closed before aggregation;
same-snapshot rows unavailable at the cutoff are excluded before duplicate
identity and domain admission. Request and executor cutoffs are compared as
parsed `KnowledgeCutoff` instants, not text. Duplicate identities among rows
actually visible at the cutoff still fail closed. `document_count` is the
admitted census count, while raw input remains bounded by
`MAX_EVIDENCE_UNITS`.

RED `dabe8a60cf5603b0be6ee7860d26bea0d1cb7e49` exposed textual cutoff comparison
and validation-status conflation; repair
`34336a883c70634cb1641cad38bc8873fab10bc9` uses instant equality and keeps
terminal `validation_status = "validated"` separate from the artifact inference
claim. RED `c9ca5d4a0b217b9ae4c28a065e7c4c8be00f7a35` requires availability
provenance and historical replay invariance when a future row reuses a visible
identity; repair `cf7da47260f3d731a15ae09be8d0f343db8a529d` applies cutoff admission
before identity/domain admission. RED `348f063bcb7add29a5c77fd16bc6e9ca9c86aab7`
then exposes missing document-level snapshot provenance; repair
`66221cdefc71a761cbf9e048082cb0f8ca06622f` binds each document to its source
snapshot and rejects cross-snapshot input. Existing fixtures were migrated to
both provenance dimensions, and `102768cbcbc6d94748b061bffb342f29a82a0553`
adds the validation-branch contracts requested by review.

The artifact inference status remains
`corpus_background_is_not_unique_content_not_stopword_deletion`.
`identity_recovery_rate` stays library-side. This is not a modality-source
census, not prompt-source, not style-source, not copy-identity, not a simulation
method-effect census, not GPU, not MCMC, and not topic birth/split/merge.

Branch evidence is not protected-main acceptance. The profile remains a Draft
fold candidate for the surviving Analysis Run landing vehicle and must reacquire
exact-head checks and qualifying review after any consolidation.
