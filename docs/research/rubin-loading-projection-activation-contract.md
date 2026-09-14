# Rubin loading projection activation contract

Status: branch-local scientific design and implementation evidence for issue #505, including source-snapshot authority repairs #511/#512, exact runtime-design work on the current #506 head, and concrete draw-payload authority #515. This document does not authorize inferential activation and does not promote Draft #504 to scientific/product claim authority.

## Problem

`rubin_loading_uncertainty_v1` can correctly compute the bounded descriptive quantities `Qbar`, `Ubar`, `B`, and `T` for finite complete-data indicator draws. The arithmetic does not identify how those draws were generated, whether the draw generator is compatible with the estimand/analysis procedure, or which claim-specific Validation Evidence supports an inferential projection.

Rubin-style variance validity is therefore not inferred from the combining formula alone. Rubin (1996) ties multiple-imputation inference to the imputation procedure, while Meng (1994) and Xie and Meng (2017) show that imputer/analyst uncongeniality can change variance and coverage behavior. The supported unit of activation in TEPP is a specific generator/analysis/evidence pairing applied to a specific immutable source population and concrete draw payload, not an arbitrary finite matrix that merely has an approved shape.

## Owner boundary

- `psychometric_core` owns reusable `Qbar/Ubar/B/T` and robust loading-point arithmetic.
- `analysis_engine` owns request admission, snapshot/cutoff composition, Validation Evidence binding, concrete draw-payload binding, and projection policy.
- Validation Evidence supplies claim-specific recovery/coverage evidence for one declared design envelope and one immutable source snapshot. It does not become estimator code.
- LLM output is not numerical, scientific-acceptance, or activation authority.
- No external generator source is copied into TEPP. A generator is consumed only through an immutable versioned contract identity plus evidence provenance.
- A draw-payload SHA-256 is a content commitment, not generator-execution attestation. It prevents receipt reuse against different values after issuance. If the trusted generator boundary cannot issue or attest the digest-bound payload, execution attestation remains a separate owner gap rather than an inferred property of the digest.

## Implemented receipt

Draft #506 exposes `RubinProjectionActivationReceiptV1` as a bounded immutable canonical-JSON value object with these fields:

```text
schema_version
generator_contract_id
generator_contract_version
analysis_contract_id
analysis_contract_version
validation_evidence_id
validation_evidence_sha256
validation_evidence_available_at
source_snapshot_id
source_snapshot_sha256
complete_data_draws_sha256
knowledge_cutoff
design_envelope_id
```

The public draw-bound boundary wraps the internal class-level activation receipt. The internal class-level decision is not re-exported as the product boundary; public callers receive the draw-bound receipt and draw-bound decision. The receipt is limited to 16 KiB before parsing. Identifiers use the Analysis Run identifier bound. Validation Evidence SHA-256, source-snapshot SHA-256, and complete-data-draw SHA-256 are exactly 64 lowercase hexadecimal characters. `validation_evidence_available_at` and `knowledge_cutoff` are stored canonically after typed parsing. Receipt SHA-256 is computed from the validated canonical JSON, so all three content commitments participate in the public receipt digest.

The receipt schema is still `tepp.rubin_projection_activation_receipt.v1`. This is not a mutation of a released wire contract: the receipt remains Draft, has never shipped from protected `main`, and TEPP has no immutable release containing it. The concrete draw commitment is therefore part of the eventual v1 definition rather than a post-release compatibility change.

Because this receipt type is specifically the Rubin loading projection authority, its validation boundary requires `analysis_contract_id == rubin_loading_uncertainty` and the exact current `RUBIN_LOADING_MODEL_CONTRACT_VERSION`. A future approved-pairing registry can select generator/evidence/design combinations, but it cannot redefine which analysis semantics the receipt authorizes.

`source_snapshot_id` must equal the Analysis Run snapshot and must itself be immutable authority. Activation-specific validation rejects branch, pull-request, issue, repository, and latest-release locator shapes for the snapshot exactly as it does for generator/analysis/evidence/design authority. A caller cannot make `main` or `refs/heads/main` scientific provenance merely by supplying the same alias as `expected_snapshot_id`.

Snapshot identity is not a content commitment. The receipt therefore also binds `source_snapshot_sha256`, and runtime activation requires an independently supplied expected source-snapshot SHA-256. Positive eligibility requires exact equality among the runtime digest, receipt digest, and owner-controlled approved pairing digest. Reusing the same logical snapshot ID for different bytes is a provenance failure.

Likewise, an approved design-envelope label plus `observation_count`/`draw_count` is not the identity of the actual matrix. The receipt therefore binds `complete_data_draws_sha256`, and the public activation decision receives an independently supplied runtime draw-payload digest. A mismatch or malformed runtime digest is rejected before class-level generator/analysis/evidence approval is evaluated. This closes payload substitution in which two matrices share the same snapshot, cutoff, dimensions, indicator, and approved generator class but differ in values.

The activation receipt does not replace row-level `AvailableTime`; both the source observations and the Validation Evidence itself must have been available at or before the run's `KnowledgeCutoff`.

The receipt's `validation_evidence_available_at`, `source_snapshot_id`, and `source_snapshot_sha256` are transport metadata, not self-authenticating authority. Positive class-level eligibility requires the owner-controlled approved pairing to carry the canonical availability instant and the exact immutable source snapshot identity/digest covered by that Validation Evidence ID/digest. The decision rejects a receipt that attempts to backdate the evidence clock, reuse the same approved evidence package against another snapshot, or reuse the same snapshot name with different content.

## Activation decision

The public `decide_rubin_projection_activation` is a fail-closed composition over the draw-bound receipt, expected snapshot identity/digest, independently supplied expected complete-data-draw digest, cutoff, actual observation/draw counts, and indicator kind. The production approval registry is not caller-supplied and is intentionally empty on this Draft branch.

The decision order is:

1. no receipt -> `DescriptiveOnly`;
2. require a canonical runtime `complete_data_draws_sha256` and exact equality with the receipt draw-payload digest; a mismatch rejects before class-level approval;
3. validate the bounded class-level receipt contract, require the canonical Rubin analysis ID/version, and require immutable locator-safe authority fields, including the source snapshot identity and canonical source snapshot SHA-256;
4. require the expected Analysis Run snapshot to be an immutable locator-safe identifier, require a canonical expected source snapshot SHA-256, and require exact receipt/runtime snapshot identity and digest equality;
5. compare receipt/run knowledge cutoffs by instant, not RFC 3339 spelling;
6. parse the owner-controlled approved Validation Evidence availability, require canonical form, and require the receipt availability to equal that authoritative instant;
7. require the authoritative Validation Evidence availability to be at or before the Analysis Run cutoff;
8. require the owner-controlled approved source snapshot identity to be immutable and locator-safe, require its source snapshot SHA-256 to be canonical, and require exact identity/digest equality with the receipt/runtime snapshot;
9. require an exact immutable generator contract ID/version;
10. require the exact Rubin analysis contract ID/version again at registry pairing rather than allowing registry metadata to alter receipt semantics;
11. require exact Validation Evidence identity and SHA-256;
12. require the actual runtime observation count and draw count to be exact owner-approved design points; a design-envelope label never interpolates between points;
13. require exact design envelope and indicator-kind coverage;
14. only an exact approved class-level pairing reached through the draw-payload gate can return `Eligible`; otherwise return `Rejected`.

A positive artifact state is not emitted by #506. The current production registry contains no approved pairing, so a valid candidate receipt cannot self-authorize. A bare string such as `validated_rubin_inference` remains insufficient.

## Required refusal matrix

| Case | Required result |
| --- | --- |
| no activation receipt | descriptive-only |
| malformed receipt/evidence digest/snapshot digest/draw-payload digest/time | fail closed |
| runtime complete-data-draw digest differs from receipt digest | fail closed before class-level approval |
| syntactically valid but noncanonical Rubin analysis ID/version | fail closed at receipt admission |
| unknown generator ID/version | fail closed |
| known generator with wrong analysis contract/version | fail closed |
| correct pairing with unknown Validation Evidence ID | fail closed |
| correct evidence ID with digest mismatch | fail closed |
| receipt backdates or changes owner-controlled Validation Evidence availability | fail closed |
| approved-registry Validation Evidence availability is noncanonical | fail closed |
| authoritative evidence availability is after the run cutoff | fail closed |
| receipt snapshot identity different from the Analysis Run snapshot | fail closed |
| same snapshot identity but receipt/runtime source snapshot SHA-256 differs | fail closed |
| approved Validation Evidence snapshot identity or SHA-256 different from the receipt/runtime snapshot | fail closed |
| approved-registry source snapshot SHA-256 is malformed/noncanonical | fail closed |
| approved-registry snapshot is a mutable branch/PR/issue/repository/latest locator | fail closed |
| receipt or expected snapshot is a mutable branch/PR/issue/repository/latest locator | fail closed |
| mutable branch/PR identity presented as approval | fail closed because it has no approved immutable pairing |
| actual runtime observation/draw counts are not exact approved points | fail closed |
| design envelope or indicator-kind mismatch | fail closed |
| exact draw digest + exact approved pairing + exact evidence digest + authoritative cutoff-safe availability + exact immutable evidence snapshot identity/SHA-256 | eligible in the composed pure decision algorithm |

The class-level executable unit contract includes a test-only approved pairing to prove its positive branch without inserting production approval data. The draw-authority boundary separately proves that an exact runtime digest passes to the class-level decision while a substituted or malformed digest is rejected first. Test fixtures are private and cannot populate the production registry.

"Eligible" is narrower than release-ready. ADR 0014 still requires implementation authority, claim-specific scientific evidence, exact-head quality/security evidence, and qualifying review before a protected-main product claim.

## Current production registry

There is intentionally no approved production Rubin generator/analysis pairing on this branch. Draft #504 is candidate repeated-sampling evidence for its declared synthetic Gaussian generator and rolling-origin design. Its branch-local results must not be inserted into a production allow-list while its exact head lacks terminal required checks and qualifying independent current-head approval.

Any future production pairing must bind the authoritative Validation Evidence availability and the immutable evidence source snapshot identity **and canonical source snapshot SHA-256** alongside the immutable evidence ID/digest. A caller-provided receipt cannot introduce, override, backdate, retarget, or re-content any owner-controlled provenance field. The registry also cannot substitute another analysis ID/version for the canonical Rubin loading analysis contract baked into the receipt type. Separately, each concrete activation receipt must bind the exact complete-data draw payload used by that run.

## Design envelope for the current candidate evidence

The #504 evidence is scoped to the current single-level `rubin_loading_uncertainty_v1` profile for its declared Gaussian simulation design, tested observation/draw settings, explicit attempted/recovered/failed denominators, bias/RMSE and Monte Carlo uncertainty, a scoped large-sample normal coverage diagnostic, and leakage-safe rolling-origin replay. It does not establish universal congeniality, multilevel/cross-classified/multiple-membership validity, arbitrary imputation-model validity, or Mislevy person-level plausible-value inference.

A future evidence package can widen the envelope only by adding relevant known-truth recovery/coverage evidence and receiving a new immutable identity/digest. Updating a mutable PR body or replacing an evidence file under the same identity is not admissible widening. Exact observation/draw count points are part of the evidence envelope; an envelope label does not authorize untested intermediate or extrapolated dimensions.

## Leakage, replay, and payload-substitution invariants

For a fixed immutable snapshot and cutoff, later-available source rows cannot change the historical result. The same rule applies to activation authority: Validation Evidence unavailable at the historical cutoff cannot retroactively authorize the older run. A later evidence package may authorize a later run under a later cutoff, but the earlier artifact remains descriptive-only unless a new artifact is produced under the corresponding authority and product policy.

The availability clock is owner-controlled approval metadata, not a caller assertion. Matching an approved Validation Evidence ID and digest while supplying an earlier receipt timestamp is insufficient and fails closed. Equivalent RFC 3339 spellings of the same instant compare equal only after typed parsing; persisted receipt and registry timestamps remain canonical. Cross-snapshot evidence is a provenance violation, not a censorable future row.

Snapshot provenance is owner-controlled authority, not presentation text. Mutable refs such as `main`, `refs/heads/main`, PR numbers, repository tree URLs, or latest aliases cannot identify the source population of an authoritative projection even if they currently resolve to the intended commit. A stable logical snapshot ID is still insufficient by itself: historical replay must also bind the exact source bytes with a canonical SHA-256. The receipt/runtime identity and digest must match the identity and digest recorded by the approved Validation Evidence pairing.

Concrete draw content is a third content boundary. The same snapshot and the same matrix dimensions can carry different values. An activation receipt issued for one canonical complete-data-draw payload cannot be replayed against another payload with the same observation count and draw count. `complete_data_draws_sha256` therefore participates in receipt SHA-256 and is independently rechecked against runtime input before approval. This prevents substitution; it does not certify how the matching bytes were generated.

## Evidence identity and mutability

A Validation Evidence digest is content identity, not a review badge. The receipt binds the evidence artifact containing the scientific design, attempted/recovered/failed population, recovery/coverage summaries, Monte Carlo uncertainty, implementation/source identity, and applicable design envelope. Git branch names, PR numbers, check URLs, comments, or latest-release aliases are mutable locators and cannot substitute for the digest-bound evidence identity.

Content identity, availability provenance, source-population identity, source-population content identity, and concrete draw-payload identity are separate invariants. The approved pairing therefore binds the exact evidence digest, authoritative availability instant, immutable source snapshot ID, and canonical source snapshot SHA-256; the per-run receipt separately binds the exact draw payload. Reusing an evidence digest while changing only a caller-supplied timestamp, snapshot name, snapshot content, or draw content cannot change historical eligibility.

If an approved evidence artifact or source snapshot is superseded, the replacement receives a new identity/digest as applicable. Existing historical receipts continue to name the evidence, source bytes, and draw bytes under which they were evaluated; they are not silently rewritten to the newest package.

## Remaining implementation sequence

1. Keep #506's negative projection policy, bounded activation receipt, canonical Rubin analysis identity, immutable snapshot identity/digest validation, exact runtime design-point validation, public-wire refusal, owner-controlled evidence-availability matching, owner-controlled evidence snapshot identity/digest matching, and #515 draw-payload binding intact.
2. Keep the production approved-pairing registry empty while #504 is Draft or lacks exact-head scientific/review gates.
3. Establish the canonical serialization/hashing owner for the actual complete-data draw payload at the execution boundary. The current activation API requires an independent digest, but a digest alone is not generator-execution attestation.
4. Once candidate evidence is independently accepted, publish its immutable Validation Evidence identity/digest, authoritative `AvailableTime`, immutable source snapshot ID, and canonical source snapshot SHA-256, then add only that exact pairing through the owner path.
5. Bind any future positive artifact projection state to the full activation-receipt digest and reacquire exact-head tests, authored line/branch coverage, documentation, security, CodeQL, and independent review.
6. Fold the complete source/test/evidence delta into the surviving Analysis Run vehicle by ordinary non-force conflict resolution; predecessor checks and approvals do not transfer.

## Primary evidence

- Meng, X.-L. (1994). Multiple-imputation inferences with uncongenial sources of input. *Statistical Science, 9*(4), 538–558. https://doi.org/10.1214/ss/1177010269
- Rubin, D. B. (1987). *Multiple Imputation for Nonresponse in Surveys*. Wiley. https://doi.org/10.1002/9780470316696
- Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908
- Xie, X., & Meng, X.-L. (2017). Dissecting multiple imputation from a multi-phase inference perspective: What happens when God's, imputer's and analyst's models are uncongenial? *Statistica Sinica, 27*(4), 1485–1594. https://doi.org/10.5705/ss.2014.067
