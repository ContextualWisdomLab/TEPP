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

The one-shot worker uses a session-level advisory lock while computation is in
flight. PostgreSQL documents that session advisory locks persist until explicit
release or session end, which supplies crash cleanup without inventing a lease
table for this single-attempt slice (PostgreSQL Global Development Group,
2026a). The database lock is coordination only: canonical input SHA-256,
request cutoff, Git commit, and dependency-lock identities remain separate
scientific reproducibility gates.

This storage rule grants no scientific authority. The API and analysis engine
remain responsible for request/result binding, cutoff eligibility, and artifact
validation.

## References

PostgreSQL Global Development Group. (2026a). *Explicit locking*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/18/explicit-locking.html

PostgreSQL Global Development Group. (2026b). *INSERT*. PostgreSQL 18 documentation. https://www.postgresql.org/docs/18/sql-insert.html
