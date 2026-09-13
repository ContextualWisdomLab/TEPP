# Non-lexical modality analysis-run composition

**Active slice:** ADR 0061 / `modality_source_v1`
**Decision status:** Proposed
**Protected-main status:** not implemented-main

`modality_source` already refuses to treat non-lexical modality as unique
latent content or as stopword deletion. This slice binds those refusals
to a digest-bound analysis-run profile while preserving historical replay.

Each `ModalitySourceDocument` carries immutable `snapshot_id` and
`AvailableTime` provenance. Cross-snapshot evidence fails before aggregation.
Same-snapshot evidence that became available after `knowledge_cutoff` is
excluded before duplicate and domain admission, so later rows cannot change an
earlier result; duplicate identities that were actually visible at the cutoff
still fail closed. Raw input is bounded by `MAX_EVIDENCE_UNITS` before identity
allocation. Request and executor cutoffs compare parsed instants rather than RFC
3339 spelling.

The artifact inference status is
`non_lexical_modality_is_not_unique_content_not_stopword_deletion` while the
terminal provider validation state is separately `validated`.
`identity_recovery_rate` stays library-side. This is not a prompt-source
census, not style-source, not copy-identity, not a simulation
method-effect census, not GPU, not MCMC, and not topic birth/split/merge.

Current RED-to-repair lineage begins with `0a08fcab054903e907ccbef5ff8694218159b52a`
for equivalent-cutoff/validation-state regressions and
`edb5d3351265d100caf260e6d8ddf207c39e3003` for explicit provenance,
historical replay, cross-snapshot and raw-census contracts. Repairs are
`5a9dac8cdc05665ee5b1e78c9c69c9b18b429d33` for instant binding/provider
validation state and `e5c1555a6d2c7f10defa0471f45d358b96954c34` for provenance-aware cutoff
admission. Dependency ownership was corrected in `ab733e9ffd2111a6c1c63aa5fc981678a5682638`.
Hosted current-head checks remain authoritative; these commit IDs are repair
traceability, not transferred acceptance evidence.
