# Rubin loading projection activation contract

Status: branch-local scientific design evidence for issue #505. This document does not authorize inferential activation and does not promote Draft #504 to scientific/product claim authority.

## Problem

`rubin_loading_uncertainty_v1` can correctly compute the bounded descriptive quantities `Qbar`, `Ubar`, `B`, and `T` for finite complete-data indicator draws. The arithmetic does not identify how those draws were generated, whether the draw generator is compatible with the estimand/analysis procedure, or which claim-specific Validation Evidence supports an inferential projection.

Rubin-style variance validity is therefore not inferred from the combining formula alone. Rubin (1996) ties multiple-imputation inference to the imputation procedure, while Meng (1994) and Xie and Meng (2017) show that imputer/analyst uncongeniality can change variance and coverage behavior. The supported unit of activation in TEPP must be a specific generator/analysis/evidence pairing, not an arbitrary finite draw matrix.

## Owner boundary

- `psychometric_core` continues to own reusable `Qbar/Ubar/B/T` and robust loading-point arithmetic.
- `analysis_engine` owns request admission, snapshot/cutoff composition, Validation Evidence binding, and projection policy.
- Validation Evidence supplies claim-specific recovery/coverage evidence for one declared design envelope. It does not become estimator code.
- LLM output is not numerical, scientific-acceptance, or activation authority.
- No external generator source is copied into TEPP. A generator is consumed only through an immutable versioned contract identity plus evidence provenance.

## Proposed receipt

A future positive path should consume a bounded immutable receipt equivalent to the following logical schema. Field names are normative for the implementation design; this document is not yet a public wire-version commitment.

```text
RubinProjectionActivationReceiptV1
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

The receipt must be canonical-JSON serializable and digest-bound. Identifiers use the Analysis Run identifier bound. SHA-256 values use the repository's lowercase canonical digest representation. `validation_evidence_available_at` and `knowledge_cutoff` are parsed typed clocks and compared by instant, not RFC 3339 text.

`source_snapshot_id` must equal the Analysis Run snapshot. The activation receipt does not replace row-level `AvailableTime`; both the source observations and the Validation Evidence itself must have been available at or before the run's `KnowledgeCutoff`.

## Activation decision

The production decision is fail closed and ordered. Expensive/scientific work is not used to repair invalid authority metadata.

1. Validate the bounded receipt wire representation and canonical timestamps.
2. Require exact Analysis Run snapshot and knowledge-cutoff binding.
3. Require `validation_evidence_available_at <= knowledge_cutoff` by instant.
4. Require an exact immutable generator contract ID and version.
5. Require the exact `rubin_loading_uncertainty_v1` analysis contract/version rather than a family-name match.
6. Require exact Validation Evidence identity and SHA-256 for that generator/analysis pairing.
7. Require the declared design envelope to cover the active indicator kind and scientific conditions being projected. An envelope is not widened by similarity, a newer mutable branch, or an LLM judgment.
8. Only after all preceding checks pass may an artifact carry a claim-specific positive projection state. Otherwise the current `descriptive_only_unbound_draw_generation_provenance` state remains authoritative.

The positive projection state should carry or digest-bind the activation receipt identity. A bare string such as `validated_rubin_inference` without the receipt is insufficient.

## Required refusal matrix

The executable contract must distinguish the following cases without changing Rubin arithmetic:

| Case | Required result |
| --- | --- |
| no activation receipt | descriptive-only |
| malformed receipt/digest/time | fail closed |
| unknown generator ID/version | fail closed |
| known generator with wrong analysis contract/version | fail closed |
| correct pairing with unknown Validation Evidence ID | fail closed |
| correct evidence ID with digest mismatch | fail closed |
| evidence available only after the run cutoff | fail closed |
| receipt snapshot different from the Analysis Run snapshot | fail closed |
| mutable branch/PR identity presented as approval | fail closed |
| design envelope mismatch | fail closed |
| exact approved pairing + exact evidence digest + cutoff-safe provenance | eligible for the claim-specific positive projection state |

"Eligible" is deliberately narrower than release-ready. ADR 0014 still requires implementation authority, claim-specific scientific evidence, exact-head quality/security evidence, and qualifying review before a protected-main product claim.

## Current production registry

There is intentionally no approved production Rubin generator/analysis pairing on this branch. Draft #504 is candidate repeated-sampling evidence for its declared synthetic Gaussian generator and rolling-origin design. Its branch-local results must not be inserted into a production allow-list while its exact head lacks terminal required checks and qualifying independent current-head approval.

A test-only fixture may exercise the positive decision algorithm with an explicitly local fake registry, but production code must keep the approved registry empty until the evidence package has independently crossed ADR 0014 claim-promotion gates. Test fixtures must not share constants with production approval data in a way that can silently promote them.

## Design envelope for the current candidate evidence

The #504 evidence is intentionally scoped. It covers the current single-level `rubin_loading_uncertainty_v1` profile for its declared Gaussian simulation design, the tested observation/draw settings, explicit attempted/recovered/failed denominators, bias/RMSE and Monte Carlo uncertainty, a scoped large-sample normal coverage diagnostic, and leakage-safe rolling-origin replay. It does not establish universal congeniality, multilevel/cross-classified/multiple-membership validity, arbitrary imputation-model validity, or Mislevy person-level plausible-value inference.

A future evidence package can widen the envelope only by adding the relevant known-truth recovery/coverage evidence and receiving a new immutable identity/digest. Updating a mutable PR body or replacing an evidence file under the same identity is not an admissible widening mechanism.

## Leakage and replay invariants

For a fixed snapshot and cutoff, later-available source rows cannot change the historical result. The same rule applies to activation authority: Validation Evidence that was not available at the historical cutoff cannot retroactively authorize the older run. A later evidence package may authorize a later run under a later cutoff, but the earlier artifact remains descriptive-only unless a new artifact is produced under the corresponding authority and product policy.

Equivalent RFC 3339 spellings of the same instant are equivalent after typed parsing. Cross-snapshot evidence is a provenance violation, not a censorable future row.

## Evidence identity and mutability

A Validation Evidence digest is content identity, not a review badge. The receipt must bind the evidence artifact that contains the scientific design, attempted/recovered/failed population, recovery/coverage summaries, Monte Carlo uncertainty, implementation/source identity, and applicable design envelope. Git branch names, PR numbers, check URLs, comments, or latest-release aliases are mutable locators and cannot substitute for the digest-bound evidence identity.

If the approved evidence artifact is superseded, the replacement receives a new identity/digest. Existing historical receipts continue to name the evidence under which they were evaluated; they are not silently rewritten to the newest package.

## Implementation sequence

1. Keep #506's negative projection policy and public-wire refusal intact.
2. Add a bounded activation-receipt value object and a pure fail-closed decision function with missing/unknown/mismatch/stale/approved tests.
3. Keep the production approved-pairing registry empty while #504 is Draft or lacks exact-head scientific/review gates.
4. Once the candidate evidence is independently accepted, publish its immutable Validation Evidence identity/digest and add that exact pairing through the owner path.
5. Bind the positive artifact projection state to the receipt digest and reacquire exact-head tests, authored line/branch coverage, documentation, security, CodeQL, and independent review.
6. Fold the complete source/test/evidence delta into the surviving Analysis Run vehicle by ordinary non-force conflict resolution; predecessor checks and approvals do not transfer.

## Primary evidence

- Meng, X.-L. (1994). Multiple-imputation inferences with uncongenial sources of input. *Statistical Science, 9*(4), 538–558. https://doi.org/10.1214/ss/1177010269
- Rubin, D. B. (1987). *Multiple Imputation for Nonresponse in Surveys*. Wiley. https://doi.org/10.1002/9780470316696
- Rubin, D. B. (1996). Multiple imputation after 18+ years. *Journal of the American Statistical Association, 91*(434), 473–489. https://doi.org/10.1080/01621459.1996.10476908
- Xie, X., & Meng, X.-L. (2017). Dissecting multiple imputation from a multi-phase inference perspective: What happens when God's, imputer's and analyst's models are uncongenial? *Statistica Sinica, 27*(4), 1485–1594. https://doi.org/10.5705/ss.2014.067
