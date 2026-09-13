# House-voice style analysis-run composition

**Active slice:** ADR 0059 / `style_source_v1`
**Decision status:** Proposed
**Protected-main status:** not implemented-main

`style_source` already refuses to treat house-voice style residue as unique
latent content or as stopword deletion. This slice binds those refusals to a
digest-bound analysis-run profile while preserving historical replay.

Each `StyleSourceDocument` carries immutable `snapshot_id` and `AvailableTime`
provenance. Cross-snapshot evidence fails before aggregation. Same-snapshot
evidence that became available after `knowledge_cutoff` is excluded before
duplicate and domain admission, so later rows cannot change an earlier result;
duplicate identities actually visible at the cutoff still fail closed. Raw input
is bounded by `MAX_EVIDENCE_UNITS` before identity allocation. Request and
executor cutoffs compare parsed instants rather than RFC 3339 spelling.

The artifact inference status is
`style_residue_is_not_unique_content_not_stopword_deletion` while terminal
provider validation state is separately `validated`. `identity_recovery_rate`
stays library-side. This is not a copy-identity census, not a simulation
method-effect census, not GPU, not MCMC, and not topic birth/split/merge.

RED `260bcaeb1c50058d5f8bf02b1c8a95cf3444cecc` records the explicit provenance,
equivalent-cutoff, historical-replay, cross-snapshot, raw-bound, and provider
validation contracts. Repair `63087f6512c86a91c47a362f4941728d24a5feaa` implements those application
boundaries and extends artifact invariant tests to the second refusal count and
count overflow; `92cc73cb25341f120b508018fdf38108d20e00ef` assigns cutoff eligibility to a
production dependency. Hosted current-head checks remain authoritative; these
commit IDs are traceability, not transferred acceptance evidence.
