# Model-run artifact chain persistence (doctoring)

## Scope

Append-only `corpus_split_manifest`, `model_run`, and `model_artifact` rows
bind a scientific run to a reproducibility manifest (and optionally a
relation-aware split) under fail-closed digest and label validation (ADR 0013;
logical ERD `MODEL_RUN` / `MODEL_ARTIFACT`). This slice records identity and
SQL insert/lookup contracts only; topic/posterior payload tables and full
physical ERD constraints remain follow-on work.

## Authority

Peng, R. D. (2011). Reproducible research in computational science. *Science,
334*(6060), 1226–1227. https://doi.org/10.1126/science.1213847

National Academies of Sciences, Engineering, and Medicine. (2019).
*Reproducibility and replicability in science*. The National Academies Press.
https://doi.org/10.17226/25303

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE
Transactions on Knowledge and Data Engineering, 11*(1), 36–44.
https://doi.org/10.1109/69.755613

## Verification

- unit tests for digest/label fail-closed validation and SQL rendering;
- live repository recording-session coverage for insert and lookup statements;
- live PostgreSQL CI exercises split → run → artifact insert/lookup under
  foundation, RLS, and migration `0003` when `TEPP_LIVE_POSTGRES=1`.
