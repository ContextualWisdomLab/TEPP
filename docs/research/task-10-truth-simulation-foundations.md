# Task 10 — Realistic temporal/event truth simulation foundations

## Scope

`tepp_simulation` generates deterministic known-truth corpora that:

1. separate latent event occurrence from document creation and availability;
2. attach multilevel memberships and method-effect variants (revision, translation, template-copy);
3. inject controlled missingness and relation false-negative/false-positive noise;
4. emit a digest-bound truth manifest for recovery studies.

## Authoritative sources

Morris, T. P., White, I. R., & Crowther, M. J. (2019). Using simulation studies to evaluate statistical methods. *Statistics in Medicine, 38*(11), 2074–2102. https://doi.org/10.1002/sim.8086

Kish, L. (1965). *Survey sampling*. John Wiley & Sons.

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Verification

Seeded determinism, temporal order, membership multiplicity, parent linkage, and digest integrity are fail-closed. Workspace line and branch coverage gates must remain complete.
