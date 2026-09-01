# Temporal dependence model research authority

This document is the canonical primary-research register for the LSIRM, MLSIRM, and DLSJM dependence families referenced by TEPP's temporal-composition architecture. It establishes what the cited studies support and, equally importantly, what they do not establish. Repository promotion policy is labeled separately and does not masquerade as a result of the cited papers.

## Scope

TEPP does not treat a published static dependence family as evidence for every possible response-family × generalized-mixed × dependence × temporal-state combination. The named studies support their stated model families. A novel temporal coupling remains `research_candidate` until the combined generative/state equation, identification and longitudinal comparability, Rust estimator, data-support conditions, and known-truth recovery are explicit and verified. That promotion status is **TEPP policy under ADR 0014 and ADR 0011**, not a claim that the three cited papers prescribe TEPP's release process.

The reusable static/generalized-mixed/dependence specification and numerical kernel owner is `ContextualWisdomLab/fast-mlsirm`. TEPP owns temporal/event composition, including event-or-valid time, assertion time, document time, system time, available time, knowledge cutoff, irregular intervals, time-varying membership/covariates, longitudinal comparability, and temporal recovery. Repository ownership is an architectural decision under ADR 0011, not a conclusion of the psychometric papers.

## LSIRM — primary evidence and extension boundary

Jeon et al. (2021) introduce a latent-space item response model in which residual person–item interactions are represented through person and item positions and their latent-space relationship. For TEPP this supports the existence and interpretation of a residual person–item interaction geometry. It does not establish that known testlets, raters, item families, hierarchy, cross-classification, multiple membership, or omitted covariates should be absorbed by that geometry (Jeon et al., 2021).

The cited study is static and does not establish a TEPP temporal state equation. Accordingly, requiring an explicit temporal state model, cross-occasion identification/alignment, uncertainty propagation, and temporal known-truth recovery before a new longitudinal LSIRM is promoted is **TEPP scientific-promotion policy** governed by ADR 0014. These are prerequisites for TEPP to make a longitudinal claim; they are not presented as requirements printed by Jeon et al. (2021).

## MLSIRM — primary evidence and extension boundary

Kang and Jeon (2025) develop a multidimensional latent-space item-response formulation and discuss conditional dependence in that multidimensional setting. In TEPP terminology, MLSIRM refers to that multidimensional-main-effect latent-space extension; the acronym is not redefined to mean “multilevel LSIRM” (Kang & Jeon, 2025). Multilevel, cross-classified, and multiple-membership structure is represented separately in TEPP's generalized-mixed specification by architecture policy.

Preserving the exact base response formulation, multidimensional loading/trait structure, and person/item interaction geometry follows the need not to change the model being attributed to Kang and Jeon (2025). Requiring a declared temporal identification/alignment procedure, longitudinal invariance evidence, and recovery before interpreting cross-time trajectories is **TEPP extension and claim-promotion policy** under ADR 0014; the 2025 paper is not cited as a longitudinal MLSIRM validation study.

## DLSJM — primary evidence and extension boundary

Jin and Jeon (2019) provide the canonical baseline for the doubly latent-space joint model of local item dependence and local person dependence. DLSJM keeps the item-dependence and person-dependence spaces distinct; it is not an alias for LSIRM person–item interaction geometry (Jin & Jeon, 2019).

TEPP therefore preserves separate item-space and person-space parameter/state identities when it creates a temporal DLSJM research candidate. The cited DLSJM is not itself evidence for a longitudinal DLSJM state process. Requiring explicit cross-occasion coordinate alignment and, when clusters are interpreted, label alignment before comparing maps is **TEPP temporal-extension policy** designed to prevent raw non-identified coordinate maps from being treated as longitudinal evidence. A primary longitudinal DLSJM source must be added here before those extension-specific procedures are attributed to published research.

## Scientific evidence carried from the primary families

The following requirements concern fidelity to the cited static families rather than TEPP release governance:

- an LSIRM attribution must retain the residual person–item latent-space interaction structure supported by Jeon et al. (2021);
- an MLSIRM attribution must retain the multidimensional latent-space response formulation supported by Kang and Jeon (2025);
- a DLSJM attribution must keep local item-dependence and local person-dependence spaces distinct as in Jin and Jeon (2019);
- novel response-family, generalized-mixed, or temporal couplings must be labeled as extensions rather than described as if they appeared in those papers.

## TEPP promotion and recovery policy

The following is repository policy governed by ADR 0014 (scientific claim promotion), ADR 0011 (service/model ownership), ADR 0002 (temporal semantics), and the approved PRD. It is intentionally stronger and broader than what any one cited dependence paper establishes.

For an LSIRM/MLSIRM/DLSJM **temporal** candidate to be classified `supported`, TEPP requires:

- exact base response and generalized-mixed formulation;
- exact dependence formulation and parameter blocks;
- explicit temporal generative/state equation and six-clock role semantics;
- identified longitudinal comparison procedure, including map alignment when the representation is only identifiable up to transformations;
- implemented estimator owned by the canonical repository;
- leakage-safe recovery that separates event-or-valid time from available time and enforces the analysis knowledge cutoff;
- model-appropriate known-truth recovery with RMSE, bias, interval/credible-interval coverage, convergence, and uncertainty reporting for the parameters actually claimed;
- irregular-gap, delayed/retrospective-record, missing-occasion, and changing-membership cases when the temporal formulation permits them;
- language/source drift tests when multilingual/source invariance is part of the claim;
- CPU `f64` reference evidence and CPU/GPU parity only when an accelerator implementation is claimed; a skipped or unexecuted GPU path is not evidence;
- Monte Carlo uncertainty for simulation summaries rather than an arbitrary replication pass percentage;
- primary-source traceability for established model components and explicit `research_candidate` labeling for novel couplings.

These items are acceptance evidence for a TEPP claim. They should not be cited as findings of Jeon et al. (2021), Jin and Jeon (2019), or Kang and Jeon (2025) unless a specific item is explicitly supported by the relevant paper. Auto-expansion only materializes a candidate contract; it does not satisfy TEPP's promotion conditions.

## APA 7 references

Jeon, M., Jin, I. H., Schweinberger, M., & Baugh, S. (2021). Mapping unobserved item–respondent interactions: A latent space item response model with interaction map. *Psychometrika, 86*(2), 378–403. https://doi.org/10.1007/s11336-021-09762-5

Jin, I. H., & Jeon, M. (2019). A doubly latent space joint model for local item and person dependence in the analysis of item response data. *Psychometrika, 84*(1), 236–260. https://doi.org/10.1007/s11336-018-9630-0

Kang, I., & Jeon, M. (2025). Multidimensional latent space item response models: A note on the relativity of conditional dependence. *Psychometrika, 90*(2), 799–826. https://doi.org/10.1017/psy.2025.5
