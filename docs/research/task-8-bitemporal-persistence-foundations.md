# Task 8 — Bitemporal persistence foundations

## Scope

Task 8 delivers storage-contract foundations for TEPP persistence under ADR 0013 without requiring a live PostgreSQL instance in CI:

1. multi-word `snake_case` migration SQL for the foundation tables;
2. fail-closed migration contract validation (tenant, temporal, naming);
3. knowledge-cutoff eligibility (`available_time <= knowledge_cutoff`);
4. in-memory bitemporal document versions with `as_known_at` / `as_valid_at` replay and append-only audit identity.

Live SQLx repositories, RLS, concurrent write stress, backup/restore, and relation-aware split manifests remain accepted-target follow-ons behind the same contracts.

## Authoritative sources

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

Snodgrass, R. T. (Ed.). (1995). *The TSQL2 temporal query language*. Springer. https://doi.org/10.1007/978-1-4615-2289-8

ISO/IEC. (2011). *ISO/IEC 9075-2:2011 Information technology — Database languages — SQL — Part 2: Foundation (SQL/Foundation)* (with temporal extensions lineage in later SQL:2011 packages). International Organization for Standardization.

## Verification

- unit tests for naming, tenant/temporal contracts, cutoff eligibility, revision supersession, audit immutability, and nested-SQL body parsing;
- workspace line and branch coverage gates must remain complete for production modules.
