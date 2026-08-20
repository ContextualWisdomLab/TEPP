# Concurrent document-write stress (doctoring)

## Scope

This increment proves the ADR 0013 concurrent-write verification item for `document_record` without allocating migration `0007` while `0006` remains in flight.

`LiveDocumentRepository::revise` now submits one `DO` block that:

1. updates the unique open version (`system_to IS NULL`);
2. requires `GET DIAGNOSTICS ROW_COUNT = 1`;
3. inserts the successor revision in the same implicit transaction; and
4. raises `serialization_failure` when another session already closed the open row.

Live `SQLx` maps PostgreSQL `unique_violation` to `DuplicateDocumentRecord` and `serialization_failure` / `deadlock_detected` / `exclusion_violation` to `ConcurrentWriteConflict`. Distinct-identity inserts remain compatible. Append-only `source_artifact` mutations still fail closed under concurrent sessions via migration `0004`.

The increment does not add a partial unique index on the open row, persist event-level membership, or implement backup/restore.

## Authority

Berenson, H., Bernstein, P., Gray, J., Melton, J., O'Neil, E., & O'Neil, P. (1995). A critique of ANSI SQL isolation levels. *ACM SIGMOD Record, 24*(2), 1–10. https://doi.org/10.1145/568271.223785

Jensen, C. S., & Snodgrass, R. T. (1999). Temporal data management. *IEEE Transactions on Knowledge and Data Engineering, 11*(1), 36–44. https://doi.org/10.1109/69.755613

PostgreSQL Global Development Group. (2024). *Transaction isolation*. In *PostgreSQL 16 documentation*. https://www.postgresql.org/docs/16/transaction-iso.html

## Verification

- unit/contract tests cover atomic revise SQL text, digest refusal, and every classified SQLSTATE including the unmapped default;
- live PostgreSQL CI (`TEPP_LIVE_POSTGRES=1`) races four sessions on the same first insert and the same revision, asserts exactly one winner, preserves the closed first version, accepts independent inserts, and rejects concurrent append-only `source_artifact` updates.
