# Reproducibility manifest persistence (doctoring)

## Scope

Append-only `reproducibility_manifest` rows record knowledge cutoff, evidence
digest, code commit, and dependency-lock digest under fail-closed validation
(ADR 0013). Immutable `model_run` linkage and split-manifest artifact chaining
remain follow-on work; this slice only proves insert/lookup SQL contracts.
SQL contracts live in `persistence_postgres` and remain storage-independent of
HTTP export envelopes in `tepp_api`.

## Authority

Peng, R. D. (2011). Reproducible research in computational science. *Science, 334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

National Academies of Sciences, Engineering, and Medicine. (2019). *Reproducibility and replicability in science*. The National Academies Press. https://doi.org/10.17226/25303

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

## Verification

- unit tests for digest/commit fail-closed validation and SQL rendering;
- live repository recording-session coverage for insert and lookup statements;
- live PostgreSQL CI exercises insert/lookup under foundation migrations when `TEPP_LIVE_POSTGRES=1`.
