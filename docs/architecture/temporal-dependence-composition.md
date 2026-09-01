# Temporal dependence composition boundary

**Status:** Accepted target; the upstream Published Language is not yet a released production dependency.
**Architecture authority:** ADR 0011.
**Research authority:** [`docs/research/temporal-dependence-models.md`](../research/temporal-dependence-models.md).

This document defines the anti-corruption boundary between reusable static psychometric model specification in `ContextualWisdomLab/fast-mlsirm` and TEPP-owned temporal/event composition. It does not add a numerical estimator and does not make an upstream research candidate a supported TEPP model.

## Ownership

`fast-mlsirm` owns reusable response-family, dimensional, generalized-mixed, and dependence-aware psychometric specification and numerical kernels. Its Published Language must be a released, versioned, immutable candidate manifest whose structural identity covers the exact base response formulation, parameter blocks, dimensional structure, generalized-mixed structure, and dependence structure.

TEPP owns only the temporal/event composition placed around that published candidate:

- event or valid time, assertion time, document time, system time, available time, and knowledge cutoff;
- leakage-safe historical eligibility;
- measurement occasion as a method/rater facet distinct from substantive event time;
- irregular observation intervals;
- time-varying covariates, random effects, cross-classification, and multiple membership;
- longitudinal invariance and drift;
- latent-state evolution and transition equations;
- temporal alignment of dependence geometry;
- event ontology and temporal graph constraints;
- temporal known-truth recovery and rolling-origin validation.

The first clock role is **event or valid time**, matching the approved six-clock contract. A state may be represented by an event instant or a validity interval, but TEPP does not mint a seventh independent clock by storing `event_time` and `valid_time` as unrelated analysis-time authorities.

`contextual-orchestrator` owns every LLM provider call, routing decision, verifier/adjudicator workflow, credential, and model-call provenance. TEPP never calls a model provider directly.

## Published-language intake

The TEPP ACL consumes a released/versioned upstream candidate contract rather than branching on model-family names. At minimum the contract must carry:

```text
candidate_id
contract_version
contract_digest
base_response_family
base_formulation_id
response_scale
base_parameter_blocks
dimensional_formulation
dimension_count
generalized_mixed_formulation
fixed_effect_blocks
random_effect_blocks
membership_formulation
dependence_kind
dependence_formulation_id
dependence_parameter_blocks
capability_status
estimator_evidence_reference
identification_evidence_reference
recovery_evidence_reference
primary_citation_references
```

The ACL rejects missing structural identity, unknown contract versions, digest mismatch, and a request that silently substitutes a local-independent candidate for a dependence-aware request.

TEPP does not duplicate the upstream `ResponseKernel`, `GeneralizedMixedStructure`, LSIRM, MLSIRM, or DLSJM implementation. When reusable static arithmetic currently exists locally in TEPP, its migration path is parity and recovery against the fast-mlsirm owner, followed by replacement with a released versioned adapter and removal of the duplicate production source.

## Generic temporal compiler

Temporal expansion is a composition over the upstream candidate identity, not a switch over names such as `rasch`, `2plm`, `mirt`, `ggum`, `lsirm`, or `dlsjm`.

For each upstream candidate admitted by the ACL, TEPP materializes one temporal-candidate specification with an identity derived from the upstream candidate plus the complete temporal contract. New compatible base families therefore inherit temporal composition automatically when fast-mlsirm publishes them; TEPP does not add a family-specific temporal wrapper.

A temporal candidate records at least:

```text
temporal_candidate_id
upstream_candidate_id
upstream_contract_version
upstream_contract_digest
temporal_formulation_id
clock_role_contract
event_or_valid_time_semantics
occasion_facet_semantics
state_equation_id
temporal_identification_rules
alignment_rules
time_varying_covariates
time_varying_random_effects
time_varying_membership_contract
irregular_interval_contract
estimator_owner
estimator_id
temporal_recovery_contract
capability_status
primary_citation_references
extension_citation_references
```

The status is exactly one of `supported`, `research_candidate`, or `unsupported`. Auto-expansion is never auto-activation.

`supported` requires an explicit combined generative/state equation, temporal identification, an implemented estimator owned by the correct repository, candidate-scoped recovery, and exact citations. A structurally representable but novel coupling is `research_candidate`. An incoherent coupling is `unsupported` with a machine-readable reason. Unknown combinations never fall back to a simpler model.

## Base-family identity is preserved

Temporal composition preserves exact base-family semantics and parameter meaning.

- Rasch remains Rasch and is not renamed generic 1PL.
- 2PLM, justified 3PLM, formulation-qualified 4PLM and 5PLM retain their discrimination, asymptote, guessing, slipping, and asymmetry semantics.
- Confirmatory and exploratory MIRT retain their factor/loading contract; exploratory structure remains a hypothesis until confirmatory and recovery evidence supports longitudinal use.
- Ideal-point/GGUM response processes remain distinct from dominance models. Dependence and time are orthogonal operators and do not convert a dominance model to ideal-point response or vice versa.
- Testlet, rater/facet, nested, crossed, cross-classified, and multiple-membership structure remains explicit. A latent-space dependence layer cannot hide a known hierarchy, rater, method, item-family, or omitted covariate.

## LSIRM and MLSIRM temporal composition

LSIRM represents residual person-item interaction through person and item positions in an interaction space and a distance-related interaction effect. TEPP temporal composition preserves the base-model parameters plus the upstream person/item interaction positions, distances, and interaction-strength parameters while adding explicitly identified temporal state evolution.

MLSIRM is the multidimensional-main-effect latent-space extension described by Kang and Jeon (2025). Multilevel, cross-classified, and multiple-membership operators are separate generalized-mixed dimensions of the specification; the acronym is not redefined as “multilevel LSIRM.”

A temporal LSIRM/MLSIRM candidate must define how interaction geometry evolves and how successive maps are identified. Raw coordinates from two occasions cannot be compared before the declared translation/rotation/reflection alignment. If scale or orientation is not identified across time, the candidate remains `research_candidate` regardless of apparently smooth trajectories.

The canonical primary-research scope and extension limits are maintained in [`docs/research/temporal-dependence-models.md`](../research/temporal-dependence-models.md).

## DLSJM temporal composition

DLSJM follows Jin and Jeon (2019) as the baseline formulation for joint local item dependence and local person dependence. It is not an LSIRM alias.

The upstream candidate must preserve distinct parameter blocks for the item-dependence space and person-dependence space. TEPP then composes distinct time-indexed state processes over those spaces. Temporal DLSJM must specify, separately for each space:

- state/evolution equation;
- translation, rotation, and reflection alignment;
- scale/identification constraints;
- cluster-label alignment when clustering is interpreted longitudinally;
- uncertainty for positions, distances, clusters, and transition parameters.

No result may compare raw item or person maps across occasions without alignment. A novel response-family × generalized-mixed × DLSJM × temporal-state coupling remains an explicitly named extension and a `research_candidate` until its combined likelihood/state equation, identification, estimator, and recovery are established.

## Generalized mixed, multilevel, cross-classified, and multiple membership

Generalized-mixed structure composes orthogonally with dependence and time when the full formulation is scientifically coherent.

One observation may belong simultaneously to multiple organizations, projects, teams, sources, languages, item families, judges, raters, templates, or event episodes. Cross-classification is not multiple membership; both remain explicit in the candidate identity.

Multiple-membership weights are auditable and time-valid. They are either observed/normalized under the declared design or estimated by an explicit model. TEPP does not invent equal weights as a fallback. Membership changes are state input with event-or-valid/available-time provenance; future membership cannot enter a historical cutoff.

## Explanatory and exploratory candidates

Explanatory covariates preserve their knowledge cutoff and join the combined model rather than being attached in an unvalidated post-hoc regression when joint estimation is required.

Exploratory factors/loadings and latent-space geometry are hypotheses. Auto-expansion may materialize temporal exploratory LSIRM/MLSIRM/DLSJM candidates, but they cannot become longitudinal production scoring structures without confirmatory identification, invariance, and recovery evidence.

## Local-dependence diagnostic order

The diagnostic boundary is explicit:

1. Model known factors, testlets, item families, raters/methods, hierarchy, cross-classification, multiple membership, and justified covariates first.
2. Residual person-item interaction may motivate LSIRM/MLSIRM.
3. Joint residual local-item and local-person dependence may motivate DLSJM when its relational representation matches the substantive question.
4. Dependence geometry cannot be used to absorb known design structure or omitted explanatory variables.

LLM item-generation lineage remains evidence from LineageWeave/contextual-orchestrator and is never numerical dependence evidence by itself.

## Temporal recovery contract

Every expanded temporal dependence candidate receives a generated recovery specification covering all parameters that exist in its composed identity. As applicable this includes:

- latent states and trajectories;
- fixed and random effects and covariance;
- time-valid membership weights;
- factors and loadings;
- lower/upper asymptotes and asymmetry parameters;
- ideal-point locations;
- LSIRM/MLSIRM person and item positions, distances, and interaction strength;
- DLSJM item-space and person-space positions/distances/clusters separately;
- temporal transition/dynamic parameters;
- posterior/interval uncertainty and coverage.

Known-truth simulation separates event-or-valid time from available time. The suite includes irregular gaps, delayed reports, retrospective documents, missing occasions, changing memberships, and language/source drift. Evaluation uses leakage-safe rolling origins. Monte Carlo uncertainty is reported for simulation summaries; arbitrary pass percentages or rule-of-thumb thresholds are not scientific promotion criteria.

A recovery artifact is evidence. A separate Scientific Claim Promotion Decision applies the preregistered method-specific acceptance contract from ADR 0014. A transport success, mergeable PR, LLM judgment, or generic RMSE threshold cannot promote a candidate.

## DDD context map

```text
fast-mlsirm Model Specification / Numerical Core
        Published Language: released versioned candidate manifest
                         |
                         v
              TEPP anti-corruption layer
                         |
                         v
TEPP Temporal/Event Composition + Longitudinal Validation
                         |
          +--------------+---------------+
          |                              |
          v                              v
 Analysis Run application          Claim Promotion
 adapters/persistence              separate authority

contextual-orchestrator --ACL--> Interpretation only
LineageWeave ------------ACL--> Evidence/lineage input only
```

Dependency direction is one-way. TEPP may depend on a released versioned fast-mlsirm contract. fast-mlsirm must not import TEPP temporal ontology. Neither repository accesses the other repository's database.

## Current implementation status

The upstream generalized-mixed/dependence candidate compiler is a contract-in-progress until it reaches the canonical fast-mlsirm protected branch and is released/versioned. TEPP must re-read its live identity each execution rather than treating a remembered PR number or SHA as authority.

Until that contract is released, TEPP develops only ACL schema/tests and temporal composition semantics that do not duplicate the upstream compiler or numerical kernel. Any local reusable static psychometric implementation discovered during that work is a migration candidate, not a second canonical source.

## Research basis

The canonical research discussion and APA 7 references for LSIRM, MLSIRM, and DLSJM are in [`docs/research/temporal-dependence-models.md`](../research/temporal-dependence-models.md). This architecture document intentionally does not maintain a second independent citation authority.