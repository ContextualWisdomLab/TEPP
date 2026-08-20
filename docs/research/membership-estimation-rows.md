# Multiple-membership estimation rows

## Scope

This note doctors the `membership_core` estimator-input slice that prevents atomistic fallacy:

1. `estimation_rows_at` emits one row per active membership at an event time;
2. recovered membership weights are scored with computed RMSE against the known total;
3. `refuse_atomistic_collapse` denies a row set shorter than the known group multiplicity.

No database migration is allocated. Full multilevel/ESEM estimators remain accepted-target.

## Authoritative sources

Diez Roux, A. V. (2002). A glossary for multilevel analysis. *Journal of Epidemiology & Community Health, 56*(8), 588–594. https://doi.org/10.1136/jech.56.8.588

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124. https://doi.org/10.1177/1471082X0100100202

## Application

Diez Roux (2002) defines the atomistic fallacy as inferring group-level process from individuals treated as independent. Browne, Goldstein, and Rasbash (2001) require explicit multiple-membership weights when a unit belongs to several classifications. TEPP therefore keeps every active membership as an estimation row and refuses to collapse a known multiplicity of three (author, department, project) into one independent document row (Diez Roux, 2002; Browne et al., 2001).

## Verification

- one document with three active memberships emits three rows;
- summed weights recover the known total `2.0` with RMSE below `1e-15`;
- a one-row subset against multiplicity `3` returns `AtomisticCollapseRefused`;
- empty networks and empty row sets fail closed.
