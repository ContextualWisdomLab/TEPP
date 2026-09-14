# Rubin loading projection activation contract

Status: branch-local scientific design and implementation evidence for issue #505, including the source-snapshot authority repair tracked by #511. This document does not authorize inferential activation and does not promote Draft #504 to scientific/product claim authority.

## Problem

`rubin_loading_uncertainty_v1` can correctly compute the bounded descriptive quantities `Qbar`, `Ubar`, `B`, and `T` for finite complete-data indicator draws. The arithmetic does not identify how those draws were generated, whether the draw generator is compatible with the estimand/analysis procedure, or which claim-specific Validation Evidence supports an inferential projection.

Rubin-style variance validity is therefore not inferred from the combining formula alone. Rubin (1996) ties multiple-imputation inference to the imputation procedure, while Meng (1994) and Xie and Meng (2017) show that imputer/analyst uncongeniality can change variance and coverage behavior. The supported unit of activation in TEPP is a specific generator/analysis/evidence pairing, not an arbitrary finite draw matrix.

## Owner boundary

- `psychometric_core` owns reusable `Qbar/Ubar/B/T` and robust loading-point arithmetic.
- `analysis_engine` owns request admission, snapshot/cutoff composition, Validation Evidence binding, and projection policy.
- Validation Evidence supplies claim-specific recovery/coverage evidence for one declared design envelope and one immutable source snapshot. It does not become estimator code.
- LLM output is not numerical, scientific-acceptance, or activation authority.
- No external generator source is copied into TEPP. A generator is consumed only through an immutable versioned contract identity plus evidence provenance.

## Implemented receipt

Draft #506 now implements `RubinProjectionActivationReceiptV1` as a bounded immutable canonical-JSON value object with these fields:

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
knowledge_cutoff
design_envelope_id
```

The receipt is limited to 16 KiB before parsing. Identifiers use the Analysis Run identifier bound. Validation Evidence SHA-256 is exactly 64 lowercase hexadecimal characters. `validation_evidence_available_at` and `knowledge_cutoff` are stored canonically after typed parsing. Receipt SHA-256 is computed from the validated canonical JSON.

Because this receipt type is specifically the Rubin loading projection authority, its public validation boundary also requires `analysis_contract_id == rubin_loading_uncertainty` and the exact current `RUBIN_LOADING_MODEL_CONTRACT_VERSION`. A future approved-pairing registry can select generator/evidence/design combinations, but it cannot redefine which analysis semantics the receipt authorizes.

`source_snapshot_id` must equal the Analysis Run snapshot and must itself be immutable authority. Activation-specific validation rejects branch, pull-request, issue, repository, and latest-release locator shapes for the snapshot exactly as it does for generator/analysis/evidence/design authority. A caller cannot make `main` or `refs/heads/main` scientific provenance merely by supplying the same alias as `expected_snapshot_id`.

The activation receipt does not replace row-level `AvailableTime`; both the source observations and the Validation Evidence itself must have been available at or before the run's `KnowledgeCutoff`.

The receipt's `validation_evidence_available_at` and `source_snapshot_id` are transport metadata, not self-authenticating authority. Positive eligibility requires the owner-controlled approved pairing to carry both the canonical availability instant and the immutable source snapshot covered by that exact Validation Evidence ID/digest. The decision rejects a receipt that attempts to backdate the evidence clock or reuse the same approved evidence package against another snapshot.

## Activation decision

`decide_rubin_projection_activation` is a pure fail-closed decision over the receipt, expected snapshot/cutoff, and indicator kind. The production approval registry is not caller-supplied and is intentionally empty on this Draft branch.

The decision order is:

1. no receipt -> `DescriptiveOnly`;
2. validate the bounded receipt contract, require the canonical Rubin analysis ID/version, and require immutable locator-safe authority fields, including the source snapshot;
3. require the expected Analysis Run snapshot to be an immutable locator-safe identifier and require exact receipt/run snapshot identity;
4. compare receipt/run knowledge cutoffs by instant, not RFC 3339 spelling;
5. parse the owner-controlled approved Validation Evidence availability, require canonical form, and require the receipt availability to equal that authoritative instant;
6. require the authoritative Validation Evidence availability to be at or before the Analysis Run cutoff;
7. require the owner-controlled approved source snapshot itself to be an immutable locator-safe identifier and require exact receipt/authority snapshot identity;
8. require an exact immutable generator contract ID/version;
9. require the exact Rubin analysis contract ID/version again at registry pairing rather than allowing registry metadata to alter receipt semantics;
10. require exact Validation Evidence identity and SHA-256;
11. require exact design envelope and indicator-kind coverage;
12. only an exact approved pairing can return `Eligible`; otherwise return `Rejected`.

A positive artifact state is not emitted by #506. The current production registry contains no approved pairing, so a valid candidate receipt cannot self-authorize. A bare string such as `validated_rubin_inference` remains insufficient.

## Required refusal matrix

| Case | Required result |
| --- | --- |
| no activation receipt | descriptive-only |
| malformed receipt/digest/time | fail closed |
| syntactically valid but noncanonical Rubin analysis ID/version | fail closed at receipt admission |
| unknown generator ID/version | fail closed |
| known generator with wrong analysis contract/version | fail closed |
| correct pairing with unknown Validation Evidence ID | fail closed |
| correct evidence ID with digest mismatch | fail closed |
| receipt backdates or changes owner-controlled Validation Evidence availability | fail closed |
| approved-registry Validation Evidence availability is noncanonical | fail closed |
| authoritative evidence availability is after the run cutoff | fail closed |
| receipt snapshot different from the Analysis Run snapshot | fail closed |
| approved Validation Evidence snapshot different from the receipt/runtime snapshot | fail closed |
| approved-registry snapshot is a mutable branch/PR/issue/repository/latest locator | fail closed |
| receipt or expected snapshot is a mutable branch/PR/issue/repository/latest locator | fail closed |
| mutable branch/PR identity presented as approval | fail closed because it has no approved immutable pairing |
| design envelope or indicator-kind mismatch | fail closed |
| exact approved pairing + exact evidence digest + authoritative cutoff-safe availability + exact immutable evidence snapshot | eligible in the pure decision algorithm |

The executable unit contract includes a test-only approved pairing to prove the positive branch without inserting any production approval data. Test fixtures are private to the decision module and cannot populate the production registry.

"Eligible" is narrower than release-ready. ADR 0014 still requires implementation authority, claim-specific scientific evidence, exact-head quality/security evidence, and qualifying review before a protected-main product claim.

## Current production registry

There is intentionally no approved production Rubin generator/analysis pairing on this branch. Draft #504 is candidate repeated-sampling evidence for its declared synthetic Gaussian generator and rolling-origin design. Its branch-local results must not be inserted into a production allow-list while its exact head lacks terminal required checks and qualifying independent current-head approval.

Any future production pairing must bind the authoritative Validation Evidence availability and the immutable evidence source snapshot alongside the immutable evidence ID/digest. A caller-provided receipt cannot introduce, override, backdate, or retarget either owner-controlled provenance field. The registry also cannot substitute another analysis ID/version for the canonical Rubin loading analysis contract baked into the receipt type.

## Design envelope for the current candidate evidence

The #504 evidence is scoped to the current single-level `rubin_loading_uncertainty_v1` profile for its declared Gaussian simulation design, tested observation/draw settings, explicit attempted/recovered/failed denominators, bias/RMSE and Monte Carlo uncertainty, a scoped large-sample normal coverage diagnostic, and leakage-safe rolling-origin replay. It does not establish universal congeniality, multilevel/cross-classified/multiple-membership validity, arbitrary imputation-model validity, or Mislevy person-level plausible-value inference.

A future evidence package can widen the envelope only by adding relevant known-truth recovery/coverage evidence and receiving a new immutable identity/digest. Updating a mutable PR body or replacing an evidence file under the same identity is not admissible widening.

## Leakage and replay invariants

For a fixed immutable snapshot and cutoff, later-available source rows cannot change the historical result. The same rule applies to activation authority: Validation Evidence unavailable at the historical cutoff cannot retroactively authorize the older run. A later evidence package may authorize a later run under a later cutoff, but the earlier artifact remains descriptive-only unless a new artifact is produced under the corresponding authority and product policy.

The availability clock is owner-controlled approval metadata, not a caller assertion. Matching an approved Validation Evidence ID and digest while supplying an earlier receipt timestamp is insufficient and fails closed. Equivalent RFC 3339 spellings of the same instant compare equal only after typed parsing; persisted receipt and registry timestamps remain canonical. Cross-snapshot evidence is a provenance violation, not a censorable future row.

Snapshot identity is also owner-controlled authority, not presentation text. Mutable refs such as `main`, `refs/heads/main`, PR numbers, repository tree URLs, or latest aliases cannot identify the source population of an authoritative projection even if they currently resolve to the intended commit. Historical replay must name one stable snapshot identity, and the receipt/run snapshot must match the snapshot recorded by the approved Validation Evidence pairing.

## Evidence identity and mutability

A Validation Evidence digest is content identity, not a review badge. The receipt binds the evidence artifact containing the scientific design, attempted/recovered/failed population, recovery/coverage summaries, Monte Carlo uncertainty, implementation/source identity, and applicable design envelope. Git branch names, PR numbers, check URLs, comments, or latest-release aliases are mutable locators and cannot substitute for the digest-bound evidence identity.

Content identity, availability provenance, and source-population identity are separate invariants. The approved pairing therefore binds the exact evidence digest, the authoritative availability instant, and the immutable source snapshot. Reusing the digest while changing only a caller-supplied timestamp or snapshot cannot change historical eligibility.

If an approved evidence artifact is superseded, the replacement receives a new identity/digest. Existing historical receipts continue to name the evidence under which they were evaluated; they are not silently rewritten to the newest package.

## Remaining implementation sequence

1. Keep #506's negative projection policy, bounded activation receipt, canonical Rubin analysis identity, immutable snapshot/authority validation, public-wire refusal, owner-controlled evidence-availability matching, and owner-controlled evidence-snapshot matching intact.
2. Keep the production approved-pairing registry empty while #504 is Draft or lacks exact-head scientific/review gates.
3. Once candidate evidence is independently accepted, publish its immutable Validation Evidence identity/digest, authoritative `AvailableTime`, and immutable source snapshot, then add only that exact pairing through the owner path.
4. Bind any future positive artifact projection state to the activation-receipt digest and reacquire exact-head tests, authored line/branch coverage, documentation, security, CodeQL, and independent review.
5. Fold the complete source/test/evidence delta into the surviving Analysis Run vehicle by ordinary non-force conflict resolution; predecessor checks and approvals do not transfer.

## Primary evidence

- Meng, X.-L. (1994). Multiple-imputation inferences with uncongenial sources of input. *Statistical Science, 9*(4), 538–558. https://doi.org/10.1214/ss/1177010269
- Rubin, D. B. (1987). *Multiple Imputation for Nonresponse in Surveys*. Wiley. https://doi.org/10.1002/9780470316696
- Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908
- Xie, X., & Meng, X.-L. (2017). Dissecting multiple imputation from a multi-phase inference perspective: What happens when God's, imputer's and analyst's models are uncongenial? *Statistica Sinica, 27*(4), 1485–1594. https://doi.org/10.5705/ss.2014.067
