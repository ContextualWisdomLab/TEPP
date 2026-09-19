# Longitudinal CWC analysis-run bind

**Review date:** 2026-09-19
**Active slice:** GAP-006 / issue #169 remaining operator-visible composition
**Scientific acceptance owner:** #501
**Evidence-identity integrity owner:** #592
**Historical-artifact leakage owner:** #593
**Cutoff-before-snapshot provenance owner:** #595
**Finite-binary64 permutation owner:** #596 / released fast-mlsirm numerical contract
**Lockfile integrity owner:** #597
**Artifact structural-validity owner:** #599
**Admitted-evidence commitment owner:** #600

Protected main owns Enders and Tofighi (2007) CWC within/between/contextual OLS in `psychometric_core`. This branch only composes that owner into `analysis_engine`; it does not implement a second estimator, DSEM, RI-CLPM, persistence, or causal identification.

## Repaired application contract

The predecessor profile was not fully historical or self-consistent. It compared request/executor cutoffs as RFC 3339 text, trusted a run-level snapshot label while individual rows lacked snapshot provenance, accepted artifact counts outside the executable population envelope, accepted a finite but inconsistent contextual effect, discarded the causal-refusal provider result, reused the scientific inference label as terminal provider validation state, dropped the opaque evidence identity needed to prevent cutoff-visible replay/pseudo-replication, emitted `excluded_after_cutoff_count` so later-only corpus existence altered an earlier artifact digest, checked row snapshot provenance before availability admission so a future-only row from another snapshot could fail an otherwise identical historical replay, accepted an all-singleton success-artifact count shape that the executor cannot produce, and after admission dropped the exact evidence identity/numeric provenance from the digest-bound artifact.

Current branch behavior is stricter:

- every `LongitudinalClusterScore` carries an opaque immutable `evidence_id`, source `snapshot_id`, and `AvailableTime`;
- `evidence_id` and `snapshot_id` use the existing bounded analysis identifier contract;
- raw input cardinality is bounded before filtering;
- rows with `AvailableTime > knowledge_cutoff` are excluded from the historical evidence population before snapshot, identity, scientific-domain, or admitted-payload commitment;
- cross-snapshot input fails closed when the row is cutoff-visible;
- repeated identity among cutoff-visible evidence fails closed with `AnalysisEngineError::DuplicateEvidence` before CWC composition;
- future-unavailable rows, including rows reusing a visible identity or carrying another snapshot identity, do not change the historical artifact or terminal result;
- public artifact counts are cutoff-visible `row_count` and `cluster_count`; no future-only exclusion counter is emitted;
- numerically equal rows with distinct evidence identities remain distinct evidence; predictor/outcome/cluster/time tuples are never treated as identity;
- `admitted_evidence_sha256` commits to the exact cutoff-visible population using a versioned domain tag, explicit count and length delimiters, opaque evidence/snapshot IDs, cluster key, exact predictor/outcome binary64 bits, and canonical availability time;
- only the provenance commitment is canonicalized by immutable evidence ID, so benign source enumeration does not change it; estimator inputs remain untouched and #596 is not hidden by sorting;
- standalone artifact import rejects malformed or non-lowercase admitted-evidence digests;
- a successful artifact requires `cluster_count < row_count`, rejecting the impossible all-singleton count shape without claiming this structural rule proves full-rank design;
- equivalent legal RFC 3339 spellings bind by `KnowledgeCutoff::instant()`;
- `contextual_effect` must equal the exact `between_slope - within_slope` value produced by the owner contract;
- the exact `CausalUnderidentified` refusal is required, while unexpected success or another provider error fails closed;
- terminal `validation_status` is `validated`; the artifact separately carries `composed_cwc_slopes_not_causal`.

The snapshot, identity, and admitted-payload rules are historical-population rules, not raw-corpus side channels. Replaying one cutoff-visible source row can change stacked within-cluster weighting, cluster means, within/between slopes, contextual effect, and `row_count`; allowing it would turn transport replay into a scientific weighting rule. A cutoff-visible row from another snapshot similarly violates provenance. Conversely, validating, counting, or hashing later-unavailable rows would leak future corpus state into an earlier replay. The new commitment is source-text-free content identity; it does not attest which upstream mapping implementation produced those coordinates.

RED / repair lineage on this branch:

- `6fd006e6d582c0a79cca7c007f7db4e8540409d1` → `ec2bc21c71d2601e5b81073459fd6347080b97df`: cutoff identity, artifact consistency, count bounds, provider/domain separation, causal-refusal enforcement;
- `99dbf5ea2a3876575f3f52557e36d9036a05cf4` → `5efa234b7fe9fd5cfef0998ee473b7ccc9887b3f` → `90eb364334175e1b0f3924eaf278f2bc5c26efa1`: snapshot/availability provenance and fixture migration;
- `e66a90f3c6be7c04ecc9a310baf955b16dbd6f76` introduces #592 evidence identity and visible-duplicate/equal-value contracts but initially over-constrains a future-unavailable duplicate;
- `138be1fb8ad3ae47a18d7e05645d2ca2830cd341` adds production evidence identity and the first duplicate check;
- `66cffe0c88f2c8e54881f6018a776c62daaf788c` / `418222f232dfc1eb8d6fa27d841e2302148ccf55` migrate fixtures;
- `bb8cb21ac40fc9f1a2d86612e8cbaaac5ccecb67` adds the leakage-safe supplemental #592 RED;
- `6b07a4eafa3c009e36299aeca6e87a5b45201d56` moves duplicate admission inside the cutoff-visible branch;
- `35a8f082b87c0f4e9fdd6e41f6f5fee4cd3f601b` adds #593's distinct future-row historical-invariance RED and makes the future-only artifact leakage explicit;
- `28509b74a7949fcfc8b40f479b7cbabdc76b89d1` removes `excluded_after_cutoff_count` from the public artifact and internal result projection while keeping the raw cardinality guard;
- `f1230ef23c51865aee6143d3cc7b1e4cb0db3240` / `4befe868eaf0d2384358c8fffc59432c96f1874b` migrate execution/review regressions to the no-future-census artifact;
- `98daaa73422f65fd8be9dfdb9e0cd0df50192617` currentizes ADR 0033 on evidence identity and historical replay;
- `0ea8f6573d8519762e3ef9038ec4c4892c8e3a8b` is the #595 public RED: cutoff-visible foreign-snapshot evidence still fails closed, while a future-unavailable foreign-snapshot row must leave artifact and terminal result unchanged;
- `07360220cf8e7018107b271dc8e6e2c2f49b0c58` is the #595 causal repair: availability admission now precedes snapshot provenance, without weakening visible snapshot refusal or the raw cardinality ceiling;
- `2a05f4d38f5136d32ab0170249dd3c47cf95949e` is the #596 public consumer RED proving pathological finite-binary64 row order can alter the result under the current numerical owner; no local summation repair is authorized;
- `5a5a2201b341bdc8c62cc23f0a2f62fc4dbe9c36` is retained as the #597 lockfile-integrity RED, and `456f0a30f4dcaae10bc01bf902e83f75154d3dfb` restores inherited registry checksums while preserving only the intended `psychometric_core` dependency delta;
- `91377deed6fec2907ba34a33dd8bc122f58f3aa5` → `5e61f7df1d8fc16c93ce75af05e92e2046988c94` → `c40d306327309dee20d988b5d6699e0c3e706925`: #599 proves and minimally repairs the impossible all-singleton success-artifact shape;
- `da1a846901f54f8500a30ae4a4988289da170916` is the #600 public RED: changing one cutoff-visible opaque evidence identity while keeping coordinates, counts, and slopes equal must change the digest-bound result;
- `625a3a4562dbdb3527e3377eabb9231fc374f1b9` adds the versioned domain-separated admitted-evidence commitment and standalone digest validation;
- `676173fac5568f6147d560233aca5c1b9203da8d` migrates the public artifact fixture, `d2159307ec8e289fb5c5850c09c05bd668c6429e` proves benign permutation stability for the new commitment, and `988679869a8241ec5a7f932ee9fd1bf3df77dd42` keeps the digest helper slice-bounded without changing behavior;
- ADR 0033 remains `Proposed`, not protected-main `Accepted` authority.

## Scientific evidence boundary

Existing known-truth CWC tests are useful regression evidence, but one noiseless profile fixture does not establish commercial recovery. Issue #501 requires repeated true-parameter recovery with RMSE, bias, Monte Carlo uncertainty, explicit attempted/recovered/failed denominators, cluster-size and signal/noise variation, unequal follow-up/time-varying availability, and leakage-safe rolling-origin evaluation.

#592, #593, #595, #597, #599 and #600 are integrity prerequisites, not substitutes for #501. #596 is a separate unresolved numerical invariant whose repair belongs to the released fast-mlsirm finite-binary64 owner. Do not promote this profile to scientific acceptance or release readiness from deterministic fixtures alone. LLM judgments are not numerical acceptance evidence.

## Merge boundary

The PR remains Draft until the valid #592/#593/#595/#596/#597/#599/#600 deltas survive the #416 fold, exact-head Rust/documentation/security/coverage gates, review-thread resolution, qualifying current-head independent approval, shared documentation consolidation, and #501 or equivalent checked-in scientific evidence converge on the surviving head. Predecessor checks or reviews do not transfer after a head change.
