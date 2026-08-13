# Live SQL transport contracts (persistence follow-on)

## Scope

Extends Task 8 / ADR 0013 with:

1. `SqlSession` transport trait and recording offline implementation;
2. migration SQL statement splitting and ordered batch application;
3. parameterized document/audit SQL rendering for bitemporal tables;
4. `LiveDocumentRepository` over any `SqlSession`;
5. fail-closed `DATABASE_URL` configuration gate for `SQLx` pool wiring;
6. optional `live-sqlx` feature compiling a real `SQLx`/`PgPool` open/execute driver behind validated URL and pool options;
7. exact-head live PostgreSQL CI (`live-postgres` job) that opens the pool, applies foundation+RLS migrations, exercises document insert/revise/as-of/audit SQL, and proves tenant isolation under `tepp_app_runtime` when `TEPP_LIVE_POSTGRES=1`.

Offline/`RecordingSqlSession` backends keep deterministic default CI free of a database process; `live-sqlx` fails closed without a reachable server. Tenant RLS migration `0002` (FORCE policies + `tepp_app_runtime` + session GUC) and live isolation proof are included. Concurrent document-write stress (atomic revise + SQLSTATE mapping) is on the active PR; remaining physical ERD constraints and backup/restore remain follow-ons.

## Authority

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

ISO/IEC. (2011). *ISO/IEC 9075-2:2011 Information technology — Database languages — SQL — Part 2: Foundation (SQL/Foundation)*. International Organization for Standardization.

PostgreSQL Global Development Group. (2024). *PostgreSQL 16.9 documentation*. https://www.postgresql.org/docs/16/

## Verification

- unit tests for URL validation, empty batches, statement splitting, recording sessions, migration apply, insert/revise/audit SQL, and digest fail-closed paths;
- live integration (`tests/live_postgres.rs`) gated by `TEPP_LIVE_POSTGRES=1` plus validated `DATABASE_URL`, required in the `live-postgres` CI service job;
- workspace line/branch coverage must remain complete (`sqlx_live.rs` remains ignored for authored LLVM coverage until broader live success-path instrumentation lands).
