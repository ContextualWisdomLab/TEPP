# Analysis-run idempotent persistence

PostgreSQL documents `ON CONFLICT` as the native alternative to unique-key
errors and guarantees an atomic insert-or-conflict outcome under concurrency.
TEPP uses `DO NOTHING` on the `(tenant_record_id, idempotency_key)` unique
constraint and verifies the canonical payload after the insert attempt; unlike
`DO UPDATE`, this path does not update the conflicting row (PostgreSQL Global
Development Group, 2026b). State events separately use
`SELECT ... FOR UPDATE` on the owning request row before reading the latest
event, so competing workers serialize on the durable run identity rather than
a process-local mutex (PostgreSQL Global Development Group, 2026a).

This storage rule grants no scientific authority. The API and analysis engine
remain responsible for request/result binding, cutoff eligibility, and artifact
validation.

## References

PostgreSQL Global Development Group. (2026a). *Explicit locking*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/18/explicit-locking.html

PostgreSQL Global Development Group. (2026b). *INSERT*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/18/sql-insert.html
