# Analysis-run idempotent persistence

PostgreSQL documents `ON CONFLICT` as the native alternative to unique-key
errors and guarantees an atomic insert-or-conflict outcome under concurrency.
TEPP therefore anchors a submission on the tenant/idempotency unique constraint
and verifies the canonical payload after the insert attempt. State events lock
the owning request row before reading the latest event, so competing workers
serialize on the durable run identity rather than a process-local mutex.

This storage rule grants no scientific authority. The API and analysis engine
remain responsible for request/result binding, cutoff eligibility, and artifact
validation.

## References

PostgreSQL Global Development Group. (2026). *INSERT*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/current/sql-insert.html

PostgreSQL Global Development Group. (2026). *Explicit locking*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/current/explicit-locking.html
