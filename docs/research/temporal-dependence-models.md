# Temporal dependence model research authority

This document is the canonical primary-research register for the LSIRM, MLSIRM, and DLSJM dependence families referenced by TEPP's temporal-composition architecture. It establishes what the cited studies support and, equally importantly, what they do not establish.

## Scope

TEPP does not treat a published static dependence family as evidence for every possible response-family × generalized-mixed × dependence × temporal-state combination. The named studies support their stated model families. A novel temporal coupling remains `research_candidate` until the combined generative/state equation, identification and longitudinal alignment, Rust estimator, data-support conditions, and known-truth recovery are explicit and verified.

The reusable static/generalized-mixed/dependence specification and numerical kernel owner is `ContextualWisdomLab/fast-mlsirm`. TEPP owns temporal/event composition, including event-or-valid time, assertion time, document time, system time, available time, knowledge cutoff, irregular intervals, time-varying membership/covariates, longitudinal alignment, and temporal recovery.

## LSIRM

Jeon et al. (2021) introduce a latent-space item response model in which residual person–item interactions are represented through person and item positions and their latent-space relationship. For TEPP this supports the existence and interpretation of a residual person–item interaction geometry. It does not establish that known testlets, raters, item families, hierarchy, cross-classification, multiple membership, or omitted covariates should be absorbed by that geometry.

A temporal LSIRM extension is therefore not automatically `supported`. Longitudinal use additionally requires an explicit state model for the interaction geometry, identification across occasions, translation/rotation/reflection alignment, uncertainty propagation, and known-truth recovery.

## MLSIRM

Kang and Jeon (2025) develop a multidimensional latent-space item-response formulation. In TEPP terminology, MLSIRM refers to that multidimensional-main-effect latent-space extension; the acronym is not redefined to mean “multilevel LSIRM.” Multilevel, cross-classified, and multiple-membership structure is represented separately in the generalized-mixed specification.

Temporal MLSIRM candidates must preserve the exact base response formulation, multidimensional loading/trait structure, person/item interaction geometry, and the temporal identification required to compare states over time. Exploratory loading or geometry hypotheses do not become production longitudinal scoring structure without confirmatory/invariance and recovery evidence.

## DLSJM

Jin and Jeon (2019) provide the canonical baseline for the doubly latent-space joint model of local item dependence and local person dependence. DLSJM keeps the item-dependence and person-dependence spaces distinct; it is not an alias for LSIRM person–item interaction geometry.

TEPP temporal DLSJM research candidates must therefore keep separate item-space and person-space states, distances, clusters, and uncertainty. Comparison across occasions requires explicit translation/rotation/reflection alignment for each space and cluster-label alignment where clusters are interpreted longitudinally. Raw maps from separate occasions are not longitudinal evidence.

## Promotion and recovery rule

For any LSIRM/MLSIRM/DLSJM temporal candidate, `supported` requires all of the following:

- exact base response and generalized-mixed formulation;
- exact dependence formulation and parameter blocks;
- explicit temporal generative/state equation and clock-role semantics;
- identification and longitudinal map-alignment rules;
- implemented estimator owned by the canonical repository;
- leakage-safe event-time/available-time recovery design;
- model-appropriate RMSE, bias, interval coverage, convergence and uncertainty evidence;
- recovery under irregular gaps, delayed/retrospective records, missing occasions, changing memberships, and applicable language/source drift;
- primary-source traceability for the established components and explicit extension labeling for novel couplings.

Auto-expansion only materializes a candidate contract. It does not satisfy these promotion conditions.

## APA 7 references

Jeon, M., Jin, I. H., Schweinberger, M., & Baugh, S. (2021). Mapping unobserved item–respondent interactions: A latent space item response model with interaction map. *Psychometrika, 86*(2), 378–403. https://doi.org/10.1007/s11336-021-09762-5

Jin, I. H., & Jeon, M. (2019). A doubly latent space joint model for local item and person dependence in the analysis of item response data. *Psychometrika, 84*(1), 236–260. https://doi.org/10.1007/s11336-018-9630-0

Kang, I., & Jeon, M. (2025). Multidimensional latent space item response models: A note on the relativity of conditional dependence. *Psychometrika, 90*(2), 799–826. https://doi.org/10.1017/psy.2025.5
