# Multilevel and multiple-membership measurement (atomistic fallacy prevention)

## Claim boundary

TEPP documents and events may simultaneously belong to authors, departments, customers, partners, competitors, projects, opportunity pools, templates, languages, and event episodes. Treating documents as independent atoms produces atomistic fallacy, inflates effective sample size, and can leak related units across validation splits (ADR 0003; AGENTS.md §6).

## Implemented foundation

- `membership_core` encodes time-varying weighted multiple membership with contextual roles and event-time validity.
- `corpus_split` (active PR) must co-partition relation-connected groups and honor knowledge cutoffs.
- `relation_graph` (active PR) separates forward transitions from provenance edges that may point backward.

## Estimator target (future)

Production multilevel estimators (cross-classified / multiple-membership ESEM/DSEM) remain accepted-target in `psychometric_core`. Recovery studies must use realistic synthetic truth with known multilevel structure and report RMSE, bias, and coverage via `validation_core`.

## Authority sources

Raudenbush, S. W., & Bryk, A. S. (2002). *Hierarchical linear models: Applications and data analysis methods* (2nd ed.). Sage.

Snijders, T. A. B. (2011). Statistical models for social networks. *Annual Review of Sociology, 37*, 131–153. https://doi.org/10.1146/annurev.soc.012809.102709

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124. https://doi.org/10.1177/1471082X0100100202
