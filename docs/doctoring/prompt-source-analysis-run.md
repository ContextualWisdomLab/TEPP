# Prompt-boilerplate analysis-run composition

**Active slice:** ADR 0060 / `prompt_source_v1`
**Decision status:** Proposed
**Protected-main status:** not implemented-main

`prompt_source` already refuses to treat prompt boilerplate as unique latent
content or as stopword deletion. This slice binds those refusals to a
digest-bound analysis-run profile while preserving historical replay.

Each `PromptSourceDocument` carries immutable `snapshot_id` and `AvailableTime`
provenance. Cross-snapshot evidence fails before aggregation. Same-snapshot
evidence that became available after `knowledge_cutoff` is excluded before
duplicate and domain admission, so later rows cannot change an earlier result;
duplicate identities actually visible at the cutoff still fail closed. Raw input
is bounded by `MAX_EVIDENCE_UNITS` before identity allocation. Request and
executor cutoffs compare parsed instants rather than RFC 3339 spelling.

The artifact inference status is
`prompt_boilerplate_is_not_unique_content_not_stopword_deletion` while terminal
provider validation state is separately `validated`. `identity_recovery_rate`
stays library-side. This is not a style-source census, not copy-identity, not a
simulation method-effect census, not GPU, not MCMC, and not topic
birth/split/merge.

RED `62bff7787e0d0e26fcfa20060a50cf345c37b7a3` records the explicit provenance,
equivalent-cutoff, historical-replay, cross-snapshot, raw-bound, and provider
validation contracts. Repair `2e65b53adf09ddacd6647e036fcac0008dda098c` implements those application
boundaries; `4c45f1359c310937692e754e70a22a07d10c480a` assigns cutoff eligibility to a
production dependency. Hosted current-head checks remain authoritative; these
commit IDs are traceability, not transferred acceptance evidence.
