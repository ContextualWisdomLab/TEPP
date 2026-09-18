# Longitudinal CWC analysis-run bind

**Review date:** 2026-09-19
**Active slice:** GAP-006 / issue #169 remaining operator-visible composition
**Scientific acceptance owner:** #501
**Evidence-identity integrity owner:** #592

Protected main owns Enders and Tofighi (2007) CWC within/between/contextual OLS in `psychometric_core`. This branch only composes that owner into `analysis_engine`; it does not implement a second estimator, DSEM, RI-CLPM, persistence, or causal identification.

## Repaired application contract

The predecessor profile was not fully historical or self-consistent. It compared request/executor cutoffs as RFC 3339 text, trusted a run-level snapshot label while individual rows lacked snapshot provenance, accepted artifact counts outside the executable population envelope, accepted a finite but inconsistent contextual effect, discarded the causal-refusal provider result, reused the scientific inference label as terminal provider validation state, and dropped the opaque evidence identity that the protected-main analysis corpus uses to prevent replay/pseudo-replication.

Current branch behavior is stricter:

- every `LongitudinalClusterScore` carries an opaque immutable `evidence_id`, source `snapshot_id`, and `AvailableTime`;
- `evidence_id` and `snapshot_id` use the existing bounded analysis identifier contract;
- repeated evidence identity fails closed with `AnalysisEngineError::DuplicateEvidence` before cutoff filtering or scientific composition, including a duplicate whose second appearance is unavailable at the historical cutoff;
- numerically equal rows with distinct evidence identities remain distinct evidence; predictor/outcome/cluster/time tuples are never treated as identity;
- cross-snapshot rows fail closed before scientific composition;
- same-snapshot rows unavailable at the cutoff are censored before the CWC owner is called;
- equivalent legal RFC 3339 spellings bind by `KnowledgeCutoff::instant()`;
- raw execution population and completed artifact row/exclusion counts share the `MAX_EVIDENCE_UNITS` envelope;
- `contextual_effect` must equal the exact `between_slope - within_slope` value produced by the owner contract;
- the exact `CausalUnderidentified` refusal is required, while unexpected success or another provider error fails closed;
- terminal `validation_status` is `validated`; the artifact separately carries `composed_cwc_slopes_not_causal`.

The evidence-identity rule is not deduplication for convenience. Replaying one row can change stacked within-cluster weighting, cluster means, within/between slopes, contextual effect, and `row_count`; allowing it would turn transport replay into a scientific weighting rule.

RED / repair lineage on this branch:

- `6fd006e6d582c0a79cca7c007f7db4e8540409d1` adds failing review regressions for equivalent cutoffs, contextual-effect tampering and impossible artifact counts;
- `ec2bc21c71d2601e5b81073459fd6347080b97df` repairs those temporal/artifact/provider-status contracts and makes the causal-refusal result enforceable;
- `99dbf5ea2a3876575f3f52557e36d903d6a05cf4` adds the row-level immutable snapshot-provenance RED;
- `5efa234b7fe9fd5cfef0998ee473b7ccc9887b3f` binds snapshot provenance in production admission;
- `90eb364334175e1b0f3924eaf278f2bc5c26efa1` migrates the existing integration fixtures to the explicit provenance contract;
- `e66a90f3c6be7c04ecc9a310baf955b16dbd6f76` adds the public #592 RED for explicit evidence identity, pre-cutoff and post-cutoff duplicate refusal, equal-value/distinct-identity admission, and identifier validation;
- `138be1fb8ad3ae47a18d7e05645d2ca2830cd341` adds production evidence identity and fail-closed duplicate detection without touching the `psychometric_core` arithmetic;
- `66cffe0c88f2c8e54881f6018a776c62daaf788c` and `418222f232dfc1eb8d6fa27d841e2302148ccf55` migrate the existing integration and review fixtures to the explicit identity contract;
- ADR 0033 is `Proposed`, not protected-main `Accepted` authority.

## Scientific evidence boundary

Existing known-truth CWC tests are useful regression evidence, but one noiseless profile fixture does not establish commercial recovery. Issue #501 requires repeated true-parameter recovery with RMSE, bias, Monte Carlo uncertainty, explicit attempted/recovered/failed denominators, cluster-size and signal/noise variation, unequal follow-up/time-varying availability, and leakage-safe temporal evaluation where availability changes over time.

#592 is a prerequisite integrity repair for that evidence population, not a substitute for #501. Do not promote this profile to scientific acceptance or release readiness from deterministic fixtures alone. LLM judgments are not numerical acceptance evidence.

## Merge boundary

The PR remains Draft until exact-head Rust/documentation/security/coverage gates, review-thread resolution, qualifying current-head independent approval, shared documentation consolidation, and #501 or equivalent checked-in scientific evidence converge on the surviving head. Predecessor checks or reviews do not transfer after a head change.
