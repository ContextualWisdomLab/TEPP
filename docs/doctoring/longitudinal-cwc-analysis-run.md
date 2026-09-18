# Longitudinal CWC analysis-run bind

**Review date:** 2026-09-19
**Active slice:** GAP-006 / issue #169 remaining operator-visible composition
**Scientific acceptance owner:** #501
**Evidence-identity integrity owner:** #592
**Historical-artifact leakage owner:** #593

Protected main owns Enders and Tofighi (2007) CWC within/between/contextual OLS in `psychometric_core`. This branch only composes that owner into `analysis_engine`; it does not implement a second estimator, DSEM, RI-CLPM, persistence, or causal identification.

## Repaired application contract

The predecessor profile was not fully historical or self-consistent. It compared request/executor cutoffs as RFC 3339 text, trusted a run-level snapshot label while individual rows lacked snapshot provenance, accepted artifact counts outside the executable population envelope, accepted a finite but inconsistent contextual effect, discarded the causal-refusal provider result, reused the scientific inference label as terminal provider validation state, dropped the opaque evidence identity needed to prevent cutoff-visible replay/pseudo-replication, and emitted `excluded_after_cutoff_count`, which allowed later-only corpus existence to alter an earlier artifact digest.

Current branch behavior is stricter:

- every `LongitudinalClusterScore` carries an opaque immutable `evidence_id`, source `snapshot_id`, and `AvailableTime`;
- `evidence_id` and `snapshot_id` use the existing bounded analysis identifier contract;
- raw input cardinality is bounded before filtering;
- cross-snapshot input fails closed as a provenance violation;
- rows with `AvailableTime > knowledge_cutoff` are excluded from the historical evidence population before identity admission;
- repeated identity among cutoff-visible evidence fails closed with `AnalysisEngineError::DuplicateEvidence` before CWC composition;
- future-unavailable rows, including rows reusing a visible identity, do not change the historical artifact or terminal result;
- public artifact counts are cutoff-visible `row_count` and `cluster_count`; no future-only exclusion counter is emitted;
- numerically equal rows with distinct evidence identities remain distinct evidence; predictor/outcome/cluster/time tuples are never treated as identity;
- equivalent legal RFC 3339 spellings bind by `KnowledgeCutoff::instant()`;
- `contextual_effect` must equal the exact `between_slope - within_slope` value produced by the owner contract;
- the exact `CausalUnderidentified` refusal is required, while unexpected success or another provider error fails closed;
- terminal `validation_status` is `validated`; the artifact separately carries `composed_cwc_slopes_not_causal`.

The identity rule is not tuple deduplication. Replaying one cutoff-visible source row can change stacked within-cluster weighting, cluster means, within/between slopes, contextual effect, and `row_count`; allowing it would turn transport replay into a scientific weighting rule. Conversely, counting or identity-admitting later-unavailable rows would leak future corpus state into an earlier replay.

RED / repair lineage on this branch:

- `6fd006e6d582c0a79cca7c007f7db4e8540409d1` → `ec2bc21c71d2601e5b81073459fd6347080b97df`: cutoff identity, artifact consistency, count bounds, provider/domain separation, causal-refusal enforcement;
- `99dbf5ea2a3876575f3f52557e36d903d6a05cf4` → `5efa234b7fe9fd5cfef0998ee473b7ccc9887b3f` → `90eb364334175e1b0f3924eaf278f2bc5c26efa1`: snapshot/availability provenance and fixture migration;
- `e66a90f3c6be7c04ecc9a310baf955b16dbd6f76` introduces #592 evidence identity and visible-duplicate/equal-value contracts but initially over-constrains a future-unavailable duplicate;
- `138be1fb8ad3ae47a18d7e05645d2ca2830cd341` adds production evidence identity and the first duplicate check;
- `66cffe0c88f2c8e54881f6018a776c62daaf788c` / `418222f232dfc1eb8d6fa27d841e2302148ccf55` migrate fixtures;
- `bb8cb21ac40fc9f1a2d86612e8cbaaac5ccecb67` adds the leakage-safe supplemental #592 RED;
- `6b07a4eafa3c009e36299aeca6e87a5b45201d56` moves duplicate admission inside the cutoff-visible branch;
- `35a8f082b87c0f4e9fdd6e41f6f5fee4cd3f601b` adds #593's distinct future-row historical-invariance RED and makes the future-only artifact leakage explicit;
- `28509b74a7949fcfc8b40f479b7cbabdc76b89d1` removes `excluded_after_cutoff_count` from the public artifact and internal result projection while keeping the raw cardinality guard;
- `f1230ef23c51865aee6143d3cc7b1e4cb0db3240` / `4befe868eaf0d2384358c8fffc59432c96f1874b` migrate execution/review regressions to the no-future-census artifact;
- `98daaa73422f65fd8be9dfdb9e0cd0df50192617` currentizes ADR 0033 on evidence identity and historical replay;
- ADR 0033 remains `Proposed`, not protected-main `Accepted` authority.

## Scientific evidence boundary

Existing known-truth CWC tests are useful regression evidence, but one noiseless profile fixture does not establish commercial recovery. Issue #501 requires repeated true-parameter recovery with RMSE, bias, Monte Carlo uncertainty, explicit attempted/recovered/failed denominators, cluster-size and signal/noise variation, unequal follow-up/time-varying availability, and leakage-safe rolling-origin evaluation.

#592 and #593 are prerequisites for an uncontaminated historical recovery population, not substitutes for #501. Do not promote this profile to scientific acceptance or release readiness from deterministic fixtures alone. LLM judgments are not numerical acceptance evidence.

## Merge boundary

The PR remains Draft until #592/#593 survive the #416 fold, exact-head Rust/documentation/security/coverage gates, review-thread resolution, qualifying current-head independent approval, shared documentation consolidation, and #501 or equivalent checked-in scientific evidence converge on the surviving head. Predecessor checks or reviews do not transfer after a head change.
