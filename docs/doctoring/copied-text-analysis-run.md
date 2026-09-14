# Copied-text analysis-run composition

**Active slice:** ADR 0065 / `copied_text_v1`
**Implementation maturity:** active-PR (not implemented-main)

`copied_text` already refuses to treat copied-text residue as unique latent content or as stopword deletion. This slice binds those refusals to a digest-bound Analysis Run profile without copying the domain vocabulary.

Every input row now carries immutable `snapshot_id` provenance and explicit `AvailableTime`. The executor compares request and execution cutoffs as parsed temporal instants, rejects cross-snapshot rows, and excludes same-snapshot rows that were unavailable at the requested cutoff before duplicate or domain admission. A future row that reuses an otherwise visible identity therefore cannot change an earlier historical replay; duplicate identities among evidence actually visible at the cutoff still fail closed.

The raw input population is bounded by `MAX_EVIDENCE_UNITS` before duplicate-tracking allocation, and `document_count` is the cutoff-admitted identity count rather than the raw slice length. The artifact retains inference status `copied_text_is_not_unique_content_not_stopword_deletion`, while the terminal provider validation status is separately `validated`.

The 256 KiB wire limit remains an untrusted-input admission boundary. Valid artifact identifiers, strict timestamps, and census counts are bounded, including worst-case JSON identifier escaping, so canonical output stays below that limit without a second post-validation egress check.

`identity_recovery_rate` stays library-side. This is not template-copy identity, citation-edge provenance, corpus-background, modality-source, prompt-source, style-source, simulation method-effect estimation, GPU, MCMC, or topic birth/split/merge.
