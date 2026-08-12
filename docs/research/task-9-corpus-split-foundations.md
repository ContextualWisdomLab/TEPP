# Task 9 — Leakage-safe corpus split foundations

## Scope

Task 9 delivers storage-independent split contracts:

1. knowledge-cutoff snapshots that reject `available_time > knowledge_cutoff`;
2. relation-connected groups over revision, translation, copied-variant, and same-episode links;
3. fail-closed leakage checks that reject partitions separating group members;
4. rolling-origin windows over strictly increasing cutoffs;
5. Kish effective sample size and group-normalized weights for duplicate-aware estimation.

## Authoritative sources

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Tashman, L. J. (2000). Out-of-sample tests of forecasting accuracy: An analysis and review. *International Journal of Forecasting, 16*(4), 437–450. https://doi.org/10.1016/S0169-2070(00)00065-0

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Verification

Unit and integration contracts cover cutoff exclusion, co-partition of linked variants, rolling-origin order, ESS, and group-normalized weights. Workspace line and branch coverage gates must remain complete.
