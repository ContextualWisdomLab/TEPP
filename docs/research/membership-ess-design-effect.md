# Membership ESS and design effect

## Scope

Adds Kish effective sample size and design-effect helpers for weighted multiple membership, plus group-normalized ESS for co-partitioned groups. These pure CPU `f64` functions feed multilevel estimators and split-weight diagnostics without collapsing multiple membership into a single hierarchy.

## Authority

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Browne, W. J., Goldstein, H., & Rasbash, J. (2001). Multiple membership multiple classification (MMMC) models. *Statistical Modelling, 1*(2), 103–124. https://doi.org/10.1177/1471082X0100100202

## Verification

- unit oracle tests for equal weights, single-positive weights, invalid inputs, and group-normalized ESS;
- workspace coverage gates remain complete.
